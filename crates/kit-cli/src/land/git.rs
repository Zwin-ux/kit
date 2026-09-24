//! Git work for `kit land`. Branch mode builds the commit in a temporary
//! index (`read-tree`, `apply --cached`, `write-tree`, `commit-tree`,
//! `update-ref`): no checkout, so the user's branch and files stay as they are.

use super::Landed;
use crate::engine::worktree::git;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Scratch dir for the temporary index, patch and message. Removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new(id: &str) -> Result<Self> {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "kit-land-{id}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        Ok(Self(dir))
    }

    fn write(&self, name: &str, text: &str) -> Result<PathBuf> {
        let path = self.0.join(name);
        std::fs::write(&path, text).with_context(|| format!("write {}", path.display()))?;
        Ok(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Run `cmd`; stdout trimmed on success, else stderr as the error.
fn run(cmd: &mut Command) -> Result<String> {
    let out = cmd.output().context("run git")?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

pub fn has_commit(repo: &Path, sha: &str) -> bool {
    run(git(repo).args(["cat-file", "-e", &format!("{sha}^{{commit}}")])).is_ok()
}

/// Paths the patch touches (`git apply --numstat`).
fn patch_files(repo: &Path, patch: &Path) -> Result<Vec<String>> {
    let out = run(git(repo).args(["apply", "--numstat"]).arg(patch))?;
    Ok(out
        .lines()
        .filter_map(|l| l.splitn(3, '\t').nth(2).map(str::to_owned))
        .collect())
}

/// Newest commit on a branch whose message holds `trailer`.
fn find_landed(repo: &Path, trailer: &str, scope: &str) -> Option<String> {
    let grep = format!("--grep={trailer}");
    let sha =
        run(git(repo).args(["log", "-F", &grep, "-n", "1", "--format=%H", scope, "--"])).ok()?;
    (!sha.is_empty()).then_some(sha)
}

pub fn land_branch(
    repo: &Path,
    id: &str,
    branch: &str,
    base: Option<(String, &'static str)>,
    patch: &str,
    message: &str,
    warnings: &mut Vec<String>,
) -> Result<Landed> {
    if run(git(repo).args(["check-ref-format", "--branch", branch])).is_err() {
        bail!("`{branch}` is not a valid branch name. Use --branch NAME with a valid name");
    }
    let trailer = format!("Kit-Receipt: {id}");
    let full_ref = format!("refs/heads/{branch}");
    let exists = run(git(repo).args(["rev-parse", "--verify", "--quiet", &full_ref])).is_ok();
    let landed = if exists {
        match find_landed(repo, &trailer, &full_ref) {
            Some(c) => Some((c, branch.to_owned())),
            None => bail!(
                "branch {branch} exists and does not hold run {id}. Use --branch NAME to pick another name"
            ),
        }
    } else {
        find_landed(repo, &trailer, "--branches").map(|c| {
            let on = run(git(repo).args(["branch", "--format=%(refname:short)", "--contains", &c]))
                .ok()
                .and_then(|s| s.lines().next().map(str::to_owned))
                .unwrap_or_else(|| branch.to_owned());
            (c, on)
        })
    };
    if let Some((commit, on)) = landed {
        return Ok(Landed {
            mode: "branch",
            branch: Some(on),
            commit: Some(commit),
            base: None,
            base_source: "none",
            three_way: false,
            already: true,
            files: Vec::new(),
        });
    }

    let (base_sha, base_source, three_way) = match base {
        Some((sha, source)) => (sha, source, false),
        None => {
            let head = run(git(repo).args(["rev-parse", "--verify", "HEAD^{commit}"]))
                .context("the repo has no HEAD commit. Make a first commit, then run again")?;
            (head, "head", true)
        }
    };
    let scratch = Scratch::new(id)?;
    let patch_path = scratch.write("diff.patch", patch)?;
    let msg_path = scratch.write("message.txt", message)?;
    let index = scratch.0.join("index");
    let files = patch_files(repo, &patch_path)?;
    let with_index = |args: &[&str]| {
        let mut c = git(repo);
        c.env("GIT_INDEX_FILE", &index).args(args);
        c
    };

    run(&mut with_index(&["read-tree", &base_sha])).context("read the base commit")?;
    let mut apply = with_index(&["apply", "--cached", "--binary"]);
    if three_way {
        apply.arg("--3way");
    }
    if let Err(err) = run(apply.arg(&patch_path)) {
        bail!(
            "the diff does not apply to {}: {err:#}. Update the repo so it has the run's base commit, or use --apply on a clean tree and fix the conflicts by hand",
            &base_sha[..base_sha.len().min(10)]
        );
    }
    let tree = run(&mut with_index(&["write-tree"])).context("write the tree")?;
    let commit = run(
        git(repo)
            .args(["commit-tree", &tree, "-p", &base_sha, "-F"])
            .arg(&msg_path),
    )
    .map_err(|err| {
        anyhow::anyhow!(
            "git cannot make the commit: {err:#}. Set user.name and user.email in git config, then run again"
        )
    })?;
    let reflog = format!("kit land {id}");
    run(git(repo).args(["update-ref", "-m", &reflog, &full_ref, &commit, ""]))
        .with_context(|| format!("create branch {branch}"))?;
    if three_way {
        warnings.push(format!(
            "branch {branch} starts at HEAD, not at the run's base"
        ));
    }
    Ok(Landed {
        mode: "branch",
        branch: Some(branch.to_owned()),
        commit: Some(commit),
        base: Some(base_sha),
        base_source,
        three_way,
        already: false,
        files,
    })
}

pub fn land_apply(repo: &Path, id: &str, patch: &str, force: bool) -> Result<Landed> {
    let scratch = Scratch::new(id)?;
    let patch_path = scratch.write("diff.patch", patch)?;
    let files = patch_files(repo, &patch_path)?;
    let mut landed = Landed {
        mode: "apply",
        branch: None,
        commit: None,
        base: None,
        base_source: "none",
        three_way: false,
        already: false,
        files,
    };
    // Applied before: the reverse patch applies. Checked before the dirty
    // test, because a first --apply leaves the tree dirty.
    let reverse = git(repo)
        .args(["apply", "--check", "--reverse", "--binary"])
        .arg(&patch_path)
        .output()
        .is_ok_and(|o| o.status.success());
    if reverse {
        landed.already = true;
        return Ok(landed);
    }
    let status = run(git(repo).args(["status", "--porcelain"]))?;
    if !status.is_empty() && !force {
        bail!(
            "{} has uncommitted changes. Commit or stash them, then run again. Or use --force",
            repo.display()
        );
    }
    if let Err(err) = run(git(repo).args(["apply", "--binary"]).arg(&patch_path)) {
        bail!(
            "the diff does not apply to your working tree: {err:#}. Use `kit land {id}` to put it on a new branch instead"
        );
    }
    Ok(landed)
}
