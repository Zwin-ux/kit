//! `kit land <id>` — bring a proven run's changes into the repo.
//!
//! A PASS receipt ends in a kept worktree and a `diff.patch`. Land turns
//! that into a branch (default) or into edits in the working tree
//! (`--apply`). It never switches the user's branch, and the default mode
//! never touches their working tree: the commit is built with plumbing.

mod git;
mod message;

use crate::engine::{infer, paths, store, worktree};
use anyhow::{Context, Result, bail};
use kit_core::{Receipt, RunState};
use std::path::{Path, PathBuf};

/// Parsed `kit land` arguments.
#[derive(Debug)]
struct LandArgs {
    id: String,
    branch: Option<String>,
    apply: bool,
    force: bool,
    json: bool,
}

fn parse(args: &[String]) -> Result<LandArgs> {
    let mut out = LandArgs {
        id: String::new(),
        branch: None,
        apply: false,
        force: false,
        json: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--branch" | "-b" => {
                i += 1;
                let name = args.get(i).context("--branch needs a name")?;
                out.branch = Some(name.clone());
            }
            "--apply" => out.apply = true,
            "--force" | "-f" => out.force = true,
            "--json" => out.json = true,
            other if !other.starts_with('-') && out.id.is_empty() => out.id = other.into(),
            other => bail!("unknown kit land flag: {other}. Run `kit --help`"),
        }
        i += 1;
    }
    if out.id.is_empty() {
        bail!("missing run id. Use `kit land <id>`; `kit receipt list` shows the ids");
    }
    if out.apply && out.branch.is_some() {
        bail!("use --branch or --apply, not both");
    }
    Ok(out)
}

/// What a land did, for the text and JSON printers.
pub(crate) struct Landed {
    pub mode: &'static str,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub base: Option<String>,
    pub base_source: &'static str,
    pub three_way: bool,
    pub already: bool,
    pub files: Vec<String>,
}

pub fn cmd_land(args: &[String]) -> Result<()> {
    let a = parse(args)?;
    let dir = store::resolve_run_dir(&a.id)?;
    let receipt =
        store::read_receipt(&a.id)?.with_context(|| format!("no receipt for `{}`", a.id))?;
    let id = receipt.id.0.clone();
    let mut warnings = Vec::new();
    check_proof(&receipt, a.force, &mut warnings)?;

    let patch = read_patch(&dir, &receipt)?;
    if patch.trim().is_empty() {
        bail!("run {id} changed no files. There is nothing to land");
    }
    let repo = receipt.spec.repo.clone();
    if !repo.join(".git").exists() {
        bail!(
            "the run's repo {} is not a git checkout. Put the repo back at that path, then run again",
            repo.display()
        );
    }

    let landed = if a.apply {
        git::land_apply(&repo, &id, &patch, a.force)?
    } else {
        let branch = a.branch.clone().unwrap_or_else(|| default_branch(&id));
        let base = find_base(&repo, &dir, &id, &mut warnings);
        let msg = message::commit_message(&receipt, a.force && !is_proven(&receipt));
        git::land_branch(&repo, &id, &branch, base, &patch, &msg, &mut warnings)?
    };
    let worktree_removed = cleanup_worktree(&repo, &dir, &id, &patch, &mut warnings);
    print(
        &receipt,
        &repo,
        &landed,
        a.force,
        worktree_removed,
        a.json,
        warnings,
    )
}

/// `kit/<first 12 chars of the id>`, the same prefix `kit land` accepts.
fn default_branch(id: &str) -> String {
    format!("kit/{}", id.chars().take(12).collect::<String>())
}

fn is_proven(r: &Receipt) -> bool {
    r.state == RunState::Pass && r.gate.as_ref().is_some_and(|g| !infer::is_vacuous(g))
}

/// Refuse a run the gate did not prove, unless `--force`.
fn check_proof(r: &Receipt, force: bool, warnings: &mut Vec<String>) -> Result<()> {
    let id = &r.id;
    let state = format!("{:?}", r.state).to_ascii_lowercase();
    let problem =
        if r.state == RunState::Unconfigured || (r.state == RunState::Pass && !is_proven(r)) {
            format!("run {id} has no gate checks (UNCONFIGURED), so nothing proved it")
        } else if r.state != RunState::Pass {
            format!("run {id} is {state}, not pass. Kit lands only runs that pass the gate")
        } else {
            return Ok(());
        };
    if !force {
        bail!(
            "{problem}. Fix: run `kit init`, then `kit run` again. Or use --force to land it anyway"
        );
    }
    let note = format!("FORCED: {problem}. Landed with --force");
    eprintln!("kit: {note}");
    warnings.push(note);
    Ok(())
}

/// `diff.patch` as written by the run; the receipt's `diff` if it is missing.
fn read_patch(dir: &Path, r: &Receipt) -> Result<String> {
    let path = dir.join("diff.patch");
    if path.is_file() {
        return std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()));
    }
    Ok(r.diff.clone())
}

/// Base commit: `base.txt`, else the kept worktree's HEAD, else `None`
/// (land then applies with --3way onto the repo HEAD).
fn find_base(
    repo: &Path,
    dir: &Path,
    id: &str,
    warnings: &mut Vec<String>,
) -> Option<(String, &'static str)> {
    let candidate = store::read_base(dir).map(|b| (b, "recorded")).or_else(|| {
        let wt = paths::worktrees_dir().join(id);
        worktree::head_commit(&wt).ok().map(|b| (b, "worktree"))
    });
    match candidate {
        Some((sha, source)) if git::has_commit(repo, &sha) => Some((sha, source)),
        Some((sha, _)) => {
            warnings.push(format!(
                "base commit {sha} is not in {}. Kit applied the diff onto HEAD with --3way",
                repo.display()
            ));
            None
        }
        None => {
            warnings.push(
                "the receipt has no base commit. Kit applied the diff onto HEAD with --3way".into(),
            );
            None
        }
    }
}

/// Remove the kept run worktree when it holds exactly the landed diff.
fn cleanup_worktree(
    repo: &Path,
    dir: &Path,
    id: &str,
    patch: &str,
    warnings: &mut Vec<String>,
) -> bool {
    let wt: PathBuf = paths::worktrees_dir().join(id);
    if !wt.is_dir() {
        return false;
    }
    let base = store::read_base(dir).or_else(|| worktree::head_commit(&wt).ok());
    let same = base
        .and_then(|b| worktree::worktree_diff(&wt, &b).ok())
        .is_some_and(|now| now == patch);
    if same {
        worktree::remove_worktree(repo, &wt);
        return true;
    }
    warnings.push(format!(
        "worktree {} has changes that are not in the receipt. Kit kept it",
        wt.display()
    ));
    false
}

fn print(
    r: &Receipt,
    repo: &Path,
    l: &Landed,
    forced: bool,
    worktree_removed: bool,
    json: bool,
    warnings: Vec<String>,
) -> Result<()> {
    let next = next_step(l);
    if json {
        let data = serde_json::json!({
            "id": r.id.0,
            "mode": l.mode,
            "repo": repo,
            "branch": l.branch,
            "commit": l.commit,
            "base": l.base,
            "baseSource": l.base_source,
            "threeWay": l.three_way,
            "alreadyLanded": l.already,
            "forced": forced,
            "files": l.files,
            "worktreeRemoved": worktree_removed,
            "next": next,
        });
        let env = crate::envelope("land", true, data, None, warnings);
        println!("{}", serde_json::to_string_pretty(&env)?);
        return Ok(());
    }
    message::print_text(r, repo, l, worktree_removed, &next);
    Ok(())
}

fn next_step(l: &Landed) -> String {
    match (&l.branch, l.mode) {
        (Some(b), _) => format!("git merge {b}"),
        _ => "git diff, then git commit".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn parses_flags_and_refuses_bad_mixes() {
        let a = parse(&argv("01ABC --branch feat/x --force --json")).unwrap();
        assert_eq!(a.id, "01ABC");
        assert_eq!(a.branch.as_deref(), Some("feat/x"));
        assert!(a.force && a.json && !a.apply);
        assert!(
            parse(&argv(""))
                .unwrap_err()
                .to_string()
                .contains("kit receipt list")
        );
        let both = parse(&argv("01 --apply --branch x"))
            .unwrap_err()
            .to_string();
        assert!(both.contains("not both"), "{both}");
        assert_eq!(
            default_branch("01M06A2PXBBH43ZFF3GJ9VQW94"),
            "kit/01M06A2PXBBH"
        );
    }
}
