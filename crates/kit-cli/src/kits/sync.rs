//! `kit sync`: make this machine match a `kit.lock`. Installs what is
//! missing, at the pins the lock records, and refuses content that differs
//! from the hashes it recorded. Never removes anything.
//! See `docs/dev/DESIGN-MARKETPLACE.md`.

use super::install::{self, Expect, Request, scope};
use super::lock::{self, Entry, Lock};
use super::plan::{Applied, tilde};
use super::writers::{Agent, Scope};
use crate::cli::SyncArgs;
use anyhow::{Context, Result, bail};
use std::path::Path;

/// Is this recorded change still on disk as recorded?
pub fn present(a: &Applied) -> bool {
    let json = |file: &Path| -> Option<serde_json::Value> {
        serde_json::from_str(&std::fs::read_to_string(file).ok()?).ok()
    };
    match a {
        Applied::Skill { .. } => super::plan::drifted(a).is_ok_and(|d| d.is_none()),
        Applied::Rules { file, kit, .. } => {
            std::fs::read_to_string(file).is_ok_and(|t| t.contains(&format!("<!-- kit:{kit} ")))
        }
        Applied::McpJson { file, name, .. } => {
            json(file).is_some_and(|d| d["mcpServers"].get(name).is_some())
        }
        Applied::McpToml { file, name, .. } => std::fs::read_to_string(file)
            .ok()
            .and_then(|t| t.parse::<toml_edit::DocumentMut>().ok())
            .is_some_and(|d| d.get("mcp_servers").and_then(|s| s.get(name)).is_some()),
        Applied::HookJson {
            file, event, entry, ..
        } => json(file).is_some_and(|d| {
            d["hooks"][event.as_str()]
                .as_array()
                .is_some_and(|l| l.contains(entry))
        }),
        // An agent's own CLI owns this (`claude mcp add`); `kit doctor` checks it.
        Applied::ClaudeMcp { .. } => true,
    }
}

/// One line for a missing change.
fn describe(a: &Applied) -> String {
    match a {
        Applied::Skill { dir, .. } => format!("skill {}", tilde(dir)),
        Applied::Rules { file, kit, .. } => format!("{kit} block in {}", tilde(file)),
        Applied::McpJson { file, name, .. } | Applied::McpToml { file, name, .. } => {
            format!("MCP server {name} in {}", tilde(file))
        }
        Applied::HookJson { file, .. } => format!("hook in {}", tilde(file)),
        Applied::ClaudeMcp { name, .. } => format!("Claude Code MCP server {name}"),
    }
}

/// Code-running changes: local MCP servers and hooks.
fn runs_code(a: &Applied) -> bool {
    matches!(
        a,
        Applied::McpJson { .. }
            | Applied::McpToml { .. }
            | Applied::ClaudeMcp { .. }
            | Applied::HookJson { .. }
    )
}

/// What the lock holds the install to.
fn expect(source: &Lock) -> Expect {
    let mut e = Expect::default();
    for entry in &source.kits {
        e.versions.insert(entry.name.clone(), entry.version.clone());
        for a in &entry.applied {
            if let Applied::Skill { dir, hash } = a
                && let Some(name) = dir.file_name()
            {
                e.skills.insert(
                    (entry.name.clone(), name.to_string_lossy().into_owned()),
                    hash.clone(),
                );
            }
        }
    }
    e
}

fn load_from(file: &Path) -> Result<Lock> {
    let raw =
        std::fs::read_to_string(file).with_context(|| format!("cannot read {}", file.display()))?;
    let lock: Lock = serde_json::from_str(&raw)
        .with_context(|| format!("{} is not a kit.lock", file.display()))?;
    if lock.schema != lock::SCHEMA {
        bail!(
            "{} was written by a newer kit (schema {}). Update kit",
            file.display(),
            lock.schema
        );
    }
    Ok(lock)
}

/// The kit spec an entry of a proposal installs from. A folder kit in a
/// repo's kit.lock is named relative to the repo (`./kits/mine`).
fn spec_of(e: &Entry, root: Option<&Path>) -> Result<String> {
    let s = e.pin.as_deref().unwrap_or(&e.source);
    if let Some(rel) = s.strip_prefix("./") {
        // No `..` and no link on the way, down to KIT.toml itself.
        let folder = root.and_then(|r| {
            inside(r, &Path::new(rel).join("KIT.toml"))?;
            inside(r, Path::new(rel))
        });
        let Some(folder) = folder else {
            bail!(
                "kit.lock names {} at {s}, which is not a folder in this repo",
                e.name
            );
        };
        return Ok(folder.display().to_string());
    }
    if let Some(folder) = s.strip_prefix("folder ") {
        bail!(
            "kit.lock names {} from a folder outside the repo ({folder}), which is on someone else's machine. Add it by path: kit add <folder>",
            e.name
        );
    }
    Ok(s.to_string())
}

/// `path` from a proposal, under `root`: relative, no `..`, and no link on
/// the way (a planted link must not make the check read outside the repo).
fn inside(root: &Path, path: &Path) -> Option<std::path::PathBuf> {
    let plain = path.components().next().is_some()
        && path
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)));
    if !plain {
        return None;
    }
    let mut at = root.to_path_buf();
    for c in path.components() {
        at.push(c);
        if std::fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
            return None;
        }
    }
    Some(at)
}

/// `kit sync --check` in a repo this machine has no Kit record for (a
/// fresh CI runner): the repo's kit.lock against the files committed with
/// it and against each kit's own `KIT.toml`. Runs nothing and writes
/// nothing in the repo; a `github:` kit is fetched at its pin into Kit's
/// cache under ~/.kit, as any install would. Exit 1 on any drift.
fn check_committed(
    source: &Lock,
    requested: &[&Entry],
    label: &str,
    root: &Path,
    json: bool,
) -> Result<()> {
    let mut gone: Vec<String> = Vec::new();
    let mut skills = std::collections::BTreeSet::new();
    for a in source.applied() {
        let mut here = a.clone();
        if let Some(path) = here.path_mut() {
            let Some(full) = inside(root, path) else {
                bail!(
                    "{label} names {}, outside this repo. Kit will not read it",
                    path.display()
                );
            };
            *path = full;
        }
        if let Applied::Skill { dir, .. } = a
            && let Some(name) = dir.file_name()
        {
            skills.insert(name.to_string_lossy().into_owned());
        }
        if !present(&here) {
            gone.push(match (a, &here) {
                (Applied::Skill { dir, .. }, Applied::Skill { dir: full, .. }) if full.is_dir() => {
                    format!("skill {} differs from {label}", dir.display())
                }
                _ => describe(a),
            });
        }
    }
    for e in &source.kits {
        let kit = super::catalog::find(&spec_of(e, Some(root))?)?;
        let m = &kit.manifest;
        if m.kit.version != e.version {
            gone.push(format!(
                "kit {}: {label} pins {}, the kit is {}",
                e.name, e.version, m.kit.version
            ));
        }
        for s in m.skill.iter().filter(|s| !skills.contains(&s.name)) {
            gone.push(format!("skill {} of {} is not in {label}", s.name, e.name));
        }
    }
    let names: Vec<String> = requested
        .iter()
        .map(|e| format!("{} {}", e.name, e.version))
        .collect();
    let against = format!("{label} and the files in this repo (this machine has no Kit record)");
    if json {
        let data = serde_json::json!({
            "inSync": gone.is_empty(), "kits": names, "missing": gone, "checkedAgainst": against,
        });
        let err =
            (!gone.is_empty()).then(|| format!("{} out of sync", plural(gone.len(), "change")));
        let env = crate::envelope("sync", gone.is_empty(), data, err, vec![]);
        println!("{}", serde_json::to_string_pretty(&env)?);
    } else if gone.is_empty() {
        println!(
            "In sync: {label} pins {} and every file it lists is in place.",
            plural(requested.len(), "kit")
        );
        println!("checked   {against}");
    } else {
        println!(
            "Not in sync with {label}: {} missing or changed",
            gone.len()
        );
        for l in &gone {
            println!("  {l}");
        }
        println!("checked   {against}");
        println!("Fix it: kit sync, then commit what it writes");
    }
    if !gone.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

pub fn cmd_sync(args: SyncArgs, json: bool) -> Result<()> {
    let target_scope = scope(args.global)?;
    let flag = if matches!(target_scope, Scope::Global { .. }) {
        " --global"
    } else {
        ""
    };
    // The proposal: what to install. A repo's kit.lock (or a --from file)
    // is only a proposal; it goes through the same plan and yes as kit add,
    // and nothing in it (hooks, checks, applied paths) enters Kit's record.
    // Kit's own record for this scope is what is installed here.
    let proposal_file = match (&args.from, &target_scope) {
        (Some(file), _) => Some(file.clone()),
        (None, Scope::Repo(_)) => lock::shared_path(&target_scope),
        (None, Scope::Global { .. }) => None,
    };
    let source = match &proposal_file {
        Some(file) if file.is_file() => load_from(file)?,
        Some(file) => bail!(
            "no kit.lock at {}. Install kits with kit add <kit>{flag}, or name one with --from",
            tilde(file)
        ),
        None => Lock::load(&target_scope)?,
    };
    let label = proposal_file.as_deref().map_or_else(
        || "kit.lock".to_string(),
        |f| {
            if args.from.is_some() {
                tilde(f)
            } else {
                "kit.lock".into()
            }
        },
    );
    let requested: Vec<&Entry> = source.kits.iter().filter(|e| e.requested).collect();
    if requested.is_empty() {
        bail!(
            "{label} pins no kits. Install one with kit add <kit>{flag}, or name another lock with --from"
        );
    }
    let root = match &target_scope {
        Scope::Repo(r) => Some(r.clone()),
        Scope::Global { .. } => super::install::repo_root(Path::new(".")),
    };

    // A fresh CI runner has no record of this repo: check the committed
    // files against the proposal instead of reporting everything missing.
    if args.check
        && let Scope::Repo(root) = &target_scope
        && !lock::path(&target_scope).exists()
    {
        return check_committed(&source, &requested, &label, root, json);
    }

    let mut target = Lock::load(&target_scope)?;
    // Kits proposed but not installed here (or at another version).
    let mut missing_kits: Vec<String> = Vec::new();
    for e in &requested {
        match target.get(&e.name) {
            Some(t) if t.version == e.version => {}
            Some(t) => missing_kits.push(format!(
                "{} {} (installed here: {})",
                e.name, e.version, t.version
            )),
            None => missing_kits.push(format!("kit {} {}", e.name, e.version)),
        }
    }
    // Pieces Kit's record says are installed but are gone from disk. A
    // skill edited by hand is kept (and its record, so nothing rewrites it)
    // unless --force.
    let mut missing: Vec<Applied> = Vec::new();
    let mut edited: Vec<String> = Vec::new();
    for e in &mut target.kits {
        if !source.kits.iter().any(|s| s.name == e.name) {
            continue;
        }
        e.applied.retain(|a| {
            if present(a) {
                return true;
            }
            let changed = matches!(a, Applied::Skill { dir, .. } if dir.is_dir());
            if changed && !args.force {
                if let Applied::Skill { dir, .. } = a {
                    edited.push(tilde(dir));
                }
                return true;
            }
            missing.push(a.clone());
            false
        });
    }
    let kept_note = |edited: &[String]| {
        for d in edited {
            println!(
                "kept      {d} was changed by hand (kit sync --force puts back the pinned copy)"
            );
        }
    };
    let gone: Vec<String> = missing_kits
        .iter()
        .cloned()
        .chain(missing.iter().map(describe))
        .collect();
    let names: Vec<String> = requested
        .iter()
        .map(|e| format!("{} {}", e.name, e.version))
        .collect();

    if gone.is_empty() {
        if json {
            let data = serde_json::json!({ "inSync": true, "kits": names, "missing": [] });
            let env = crate::envelope("sync", true, data, None, vec![]);
            println!("{}", serde_json::to_string_pretty(&env)?);
        } else {
            if edited.is_empty() {
                println!(
                    "In sync: {label} pins {} and everything is in place.",
                    plural(requested.len(), "kit"),
                );
            } else {
                println!(
                    "In sync, apart from hand edits: {label} pins {}.",
                    plural(requested.len(), "kit")
                );
                kept_note(&edited);
            }
        }
        return Ok(());
    }
    if args.check {
        let lines = &gone;
        if json {
            let data = serde_json::json!({ "inSync": false, "kits": names, "missing": lines });
            let env = crate::envelope(
                "sync",
                false,
                data,
                Some(format!("{} missing", plural(lines.len(), "change"))),
                vec![],
            );
            println!("{}", serde_json::to_string_pretty(&env)?);
            std::process::exit(1);
        }
        // Out of sync is an answer, not a failure to run: exit 1, like
        // `cargo fmt --check`. Could-not-run errors stay exit 2.
        println!("Not in sync with {label}: {} missing", gone.len());
        for l in lines {
            println!("  {l}");
        }
        kept_note(&edited);
        println!("Fix it: kit sync{flag}");
        std::process::exit(1);
    }

    let agents: Vec<Agent> = if args.agent.is_empty() {
        let mut v: Vec<Agent> = requested
            .iter()
            .flat_map(|e| &e.agents)
            .filter_map(|id| Agent::ALL.into_iter().find(|a| a.id() == id))
            .collect();
        v.sort();
        v.dedup();
        v
    } else {
        args.agent.clone()
    };
    if agents.is_empty() {
        bail!("{label} names no agent Kit knows. Name one with --agent");
    }
    let kits: Vec<String> = requested
        .iter()
        .map(|e| spec_of(e, root.as_deref()))
        .collect::<Result<_>>()?;
    // Code was left out at install if the lock records none; keep it out.
    let no_code = args.no_code || !source.kits.iter().flat_map(|e| &e.applied).any(runs_code);

    if !json {
        let first: Vec<&str> = gone.iter().take(3).map(String::as_str).collect();
        let more = gone.len().saturating_sub(first.len());
        let more = if more > 0 {
            format!(", {more} more")
        } else {
            String::new()
        };
        println!("Missing   {}: {}{more}", gone.len(), first.join(", "));
        println!();
    }
    let req = Request {
        kits,
        agents,
        scope: target_scope.clone(),
        no_code,
        yes: args.yes,
        print: args.print,
        force: args.force,
    };
    if !json {
        kept_note(&edited);
    }
    let outcome = install::add_to(&req, json, target, &expect(&source))?;
    if !json && outcome == install::Outcome::Installed {
        println!(
            "{} matches {}.",
            match target_scope {
                Scope::Global { .. } => "This machine",
                Scope::Repo(_) => "This repo",
            },
            label
        );
    }
    Ok(())
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presence_follows_the_files() {
        let dir = std::env::temp_dir().join(format!("kit-sync-present-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let md = dir.join("CLAUDE.md");
        let rules = Applied::Rules {
            file: md.clone(),
            kit: "demo".into(),
            created: true,
            original: None,
        };
        assert!(!present(&rules));
        std::fs::write(&md, "<!-- kit:demo 0.1.0 -->\nx\n<!-- /kit:demo -->\n").unwrap();
        assert!(present(&rules));

        let mcp = dir.join(".mcp.json");
        let a = Applied::McpJson {
            file: mcp.clone(),
            name: "docs".into(),
            created: true,
            previous: None,
            original: None,
        };
        std::fs::write(&mcp, r#"{"mcpServers":{"other":{}}}"#).unwrap();
        assert!(!present(&a));
        std::fs::write(&mcp, r#"{"mcpServers":{"docs":{"url":"https://x"}}}"#).unwrap();
        assert!(present(&a));

        let toml = dir.join("config.toml");
        let a = Applied::McpToml {
            file: toml.clone(),
            name: "docs".into(),
            created: true,
            previous: None,
            original: None,
        };
        std::fs::write(&toml, "[mcp_servers.docs]\nurl = \"https://x\"\n").unwrap();
        assert!(present(&a));

        let settings = dir.join("settings.json");
        let entry = serde_json::json!({"matcher": "Edit"});
        let a = Applied::HookJson {
            file: settings.clone(),
            event: "PostToolUse".into(),
            entry: entry.clone(),
            created: true,
            original: None,
        };
        std::fs::write(&settings, r#"{"hooks":{"PostToolUse":[]}}"#).unwrap();
        assert!(!present(&a));
        std::fs::write(
            &settings,
            format!(r#"{{"hooks":{{"PostToolUse":[{entry}]}}}}"#),
        )
        .unwrap();
        assert!(present(&a));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_kit_is_never_reached_through_a_link() {
        let base = std::env::temp_dir().join(format!("kit-sync-link-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (repo, outside) = (base.join("repo"), base.join("outside"));
        std::fs::create_dir_all(repo.join("kits/mine")).unwrap();
        std::fs::create_dir_all(outside.join("mine")).unwrap();
        std::fs::write(repo.join("kits/mine/KIT.toml"), "").unwrap();
        std::fs::write(outside.join("mine/KIT.toml"), "").unwrap();
        let entry = |source: &str| -> Entry {
            serde_json::from_value(serde_json::json!({
                "name": "mine", "version": "0.1.0", "source": source,
                "requested": true, "agents": ["claude"], "applied": [],
            }))
            .unwrap()
        };
        let ok = spec_of(&entry("./kits/mine"), Some(&repo)).unwrap();
        assert_eq!(Path::new(&ok), repo.join("kits/mine"));
        assert!(spec_of(&entry("./../outside/mine"), Some(&repo)).is_err());

        std::os::unix::fs::symlink(&outside, repo.join("linked")).unwrap();
        assert!(spec_of(&entry("./linked/mine"), Some(&repo)).is_err());
        std::fs::remove_file(repo.join("kits/mine/KIT.toml")).unwrap();
        std::os::unix::fs::symlink(
            outside.join("mine/KIT.toml"),
            repo.join("kits/mine/KIT.toml"),
        )
        .unwrap();
        assert!(
            spec_of(&entry("./kits/mine"), Some(&repo)).is_err(),
            "KIT.toml itself may not be a link"
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}
