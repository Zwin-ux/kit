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
        Applied::Command { .. } => true,
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
        Applied::Command { undo } => undo.join(" "),
    }
}

/// Code-running changes: local MCP servers and hooks.
fn runs_code(a: &Applied) -> bool {
    matches!(
        a,
        Applied::McpJson { .. }
            | Applied::McpToml { .. }
            | Applied::Command { .. }
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

pub fn cmd_sync(args: SyncArgs, json: bool) -> Result<()> {
    let target_scope = scope(args.global)?;
    let flag = if matches!(target_scope, Scope::Global { .. }) {
        " --global"
    } else {
        ""
    };
    let source = match &args.from {
        Some(file) => load_from(file)?,
        None => Lock::load(&target_scope)?,
    };
    let requested: Vec<&Entry> = source.kits.iter().filter(|e| e.requested).collect();
    if requested.is_empty() {
        let lock_file = args
            .from
            .clone()
            .unwrap_or_else(|| lock::path(&target_scope));
        bail!(
            "{} pins no kits. Install one with kit add <kit>{flag}, or name another lock with --from",
            tilde(&lock_file)
        );
    }

    // What the target already has, minus anything no longer on disk.
    let mut target = Lock::load(&target_scope)?;
    // A skill edited by hand is kept (and its record, so nothing rewrites
    // it) unless --force; only kits in the source lock are looked at.
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
    let total: usize = source.kits.iter().map(|e| e.applied.len()).sum();
    let names: Vec<String> = requested
        .iter()
        .map(|e| format!("{} {}", e.name, e.version))
        .collect();

    if args.from.is_none() && missing.is_empty() {
        if json {
            let data = serde_json::json!({ "inSync": true, "kits": names, "missing": [] });
            let env = crate::envelope("sync", true, data, None, vec![]);
            println!("{}", serde_json::to_string_pretty(&env)?);
        } else {
            if edited.is_empty() {
                println!(
                    "In sync: kit.lock pins {} and everything is in place.",
                    plural(requested.len(), "kit")
                );
            } else {
                println!(
                    "In sync, apart from hand edits: kit.lock pins {}.",
                    plural(requested.len(), "kit")
                );
                kept_note(&edited);
            }
        }
        return Ok(());
    }
    if args.check {
        let lines: Vec<String> = missing.iter().map(describe).collect();
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
        println!(
            "Not in sync with kit.lock: {} of {total} missing",
            missing.len()
        );
        for l in &lines {
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
        bail!("kit.lock names no agent Kit knows. Name one with --agent");
    }
    let kits: Vec<String> = requested
        .iter()
        .map(|e| e.pin.clone().unwrap_or_else(|| e.source.clone()))
        .collect();
    // Code was left out at install if the lock records none; keep it out.
    let no_code = args.no_code || !source.kits.iter().flat_map(|e| &e.applied).any(runs_code);

    if !json {
        match &args.from {
            Some(f) => println!("Installing what {} pins: {}", tilde(f), names.join(", ")),
            None => {
                let first: Vec<String> = missing.iter().take(3).map(describe).collect();
                let more = missing.len().saturating_sub(first.len());
                let more = if more > 0 {
                    format!(", {more} more")
                } else {
                    String::new()
                };
                println!(
                    "Missing   {} of {total}: {}{more}",
                    missing.len(),
                    first.join(", ")
                );
            }
        }
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
            args.from.as_deref().map_or("kit.lock".into(), tilde)
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
        };
        assert!(!present(&rules));
        std::fs::write(&md, "<!-- kit:demo 0.1.0 -->\nx\n<!-- /kit:demo -->\n").unwrap();
        assert!(present(&rules));

        let mcp = dir.join(".mcp.json");
        let a = Applied::McpJson {
            file: mcp.clone(),
            name: "docs".into(),
            created: true,
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
}
