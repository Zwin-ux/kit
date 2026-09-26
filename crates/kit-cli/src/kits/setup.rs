//! `kit setup`: the first-run questions (which agents, which focus, where),
//! then the same plan and confirm as `kit add`. Screens 1–6 of
//! `docs/dev/DESIGN-KITS.md` §1. Every answer is also a flag, so setup
//! runs without a terminal too.

use super::catalog;
use super::config::Config;
use super::install::{self, Outcome, Request, home_dir, repo_root};
use super::writers::{Agent, Scope};
use crate::cli::SetupArgs;
use anyhow::{Result, bail};
use inquire::error::InquireError;
use kit_core::AgentKind;
use std::io::IsTerminal;
use std::path::Path;

fn kind(agent: Agent) -> AgentKind {
    match agent {
        Agent::Claude => AgentKind::Claude,
        Agent::Codex => AgentKind::Codex,
        Agent::Grok => AgentKind::Grok,
    }
}

/// Ctrl-C or Esc on any screen: say so, change nothing.
fn answered<T>(r: Result<T, InquireError>) -> Result<Option<T>> {
    match r {
        Ok(v) => Ok(Some(v)),
        Err(InquireError::OperationCanceled | InquireError::OperationInterrupted) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub async fn cmd_setup(args: SetupArgs, json: bool) -> Result<()> {
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal() && !json;
    let saved = Config::load()?.unwrap_or_default();
    let in_repo = repo_root(Path::new("."));
    let needs_questions =
        args.agent.is_empty() || args.kit.is_empty() || !args.global && !args.this_repo;
    if needs_questions && !tty {
        bail!(
            "kit setup asks questions and needs a terminal. Without one, answer with flags:\n  \
             kit setup --agent claude --kit frontend-design --global --yes"
        );
    }
    if !args.yes && !json && !std::io::stdin().is_terminal() {
        bail!(
            "kit setup asks before it writes anything, and needs a terminal for that. \
             To run it unattended, add --yes (and --no-code to skip anything that runs code), \
             or use `kit add <kit> --yes`"
        );
    }

    // Screen 1: welcome and detection.
    if !json {
        println!("Kit sets your coding agents up for one job, then proves what they do.\n");
        println!("Looking for agents on this machine…");
    }
    let mut found = Vec::new();
    let mut rows = Vec::new();
    for agent in Agent::ALL {
        let st = kit_agents::adapter(kind(agent)).probe().await;
        let (version, login) = if st.installed {
            // The same three words as `kit doctor`.
            let unchecked = st
                .remedy
                .as_deref()
                .is_some_and(|r| r.starts_with("login not checked"));
            let login = match (st.authenticated, unchecked) {
                (true, false) => "logged in",
                (true, true) => "login not checked",
                (false, _) => "not logged in",
            };
            (short_version(st.version.as_deref()).to_string(), login)
        } else {
            ("not found".to_string(), "")
        };
        rows.push((agent, version, login));
        if st.installed {
            found.push(agent);
        }
    }
    if !json {
        // Versions differ in length ("2.1.283", "codex-cli 0.155.0"): pad
        // them so the login column lines up.
        let width = rows
            .iter()
            .filter(|(_, _, l)| !l.is_empty())
            .map(|(_, v, _)| v.chars().count())
            .max()
            .unwrap_or(0);
        for (agent, version, login) in &rows {
            let line = format!("  {:12}  {version:width$}   {login}", agent.title());
            println!("{}", line.trim_end());
        }
    }
    if !json {
        println!();
    }

    // Screen 2: agents.
    let agents: Vec<Agent> = if args.agent.is_empty() {
        if found.is_empty() {
            println!("No coding agent found. Install one, then run kit setup again:");
            println!("  Claude Code   npm i -g @anthropic-ai/claude-code");
            println!("  Codex         npm i -g @openai/codex");
            println!("Or set up an agent you will install later: kit setup --agent codex");
            return Ok(());
        }
        let preselect: Vec<Agent> = if saved.agents().is_empty() {
            found.clone()
        } else {
            saved.agents()
        };
        let labels: Vec<String> = Agent::ALL
            .iter()
            .map(|a| {
                if found.contains(a) {
                    a.title().to_string()
                } else {
                    format!(
                        "{:12} not installed (Kit can still write its files)",
                        a.title()
                    )
                }
            })
            .collect();
        let defaults: Vec<usize> = Agent::ALL
            .iter()
            .enumerate()
            .filter(|(_, a)| preselect.contains(a))
            .map(|(i, _)| i)
            .collect();
        let names = |opts: &[inquire::list_option::ListOption<&String>]| {
            let titles: Vec<&str> = opts.iter().map(|o| Agent::ALL[o.index].title()).collect();
            and_list(&titles)
        };
        let picked = answered(
            inquire::MultiSelect::new("Which agents should Kit set up?", labels.clone())
                .with_default(&defaults)
                .with_formatter(&names)
                .with_help_message("↑↓ move · space toggle · enter confirm · esc cancel")
                .with_validator(|l: &[inquire::list_option::ListOption<&String>]| {
                    Ok(if l.is_empty() {
                        inquire::validator::Validation::Invalid("Pick at least one".into())
                    } else {
                        inquire::validator::Validation::Valid
                    })
                })
                .raw_prompt(),
        )?;
        let Some(picked) = picked else {
            println!("Nothing was changed.");
            return Ok(());
        };
        picked.iter().map(|o| Agent::ALL[o.index]).collect()
    } else {
        args.agent.clone()
    };

    // Screen 3: focus.
    let kits: Vec<String> = if args.kit.is_empty() {
        let all = catalog::bundled()?;
        // Job kits first, in the order people most often start with;
        // Essentials (the base of the others) last.
        const FIRST: &[&str] = &[
            "frontend-design",
            "fullstack-design",
            "backend-engineer",
            "llm-engineer",
        ];
        let rank = |k: &&catalog::Kit| match k.name() {
            "essentials" => FIRST.len() + 1,
            name => FIRST.iter().position(|f| *f == name).unwrap_or(FIRST.len()),
        };
        let mut order: Vec<&catalog::Kit> = all.iter().collect();
        order.sort_by_key(|k| (rank(k), k.name().to_string()));
        let width = order
            .iter()
            .map(|k| k.manifest.kit.title.len())
            .max()
            .unwrap_or(0);
        let labels: Vec<String> = order
            .iter()
            .map(|k| {
                format!(
                    "{:width$}   {}",
                    k.manifest.kit.title, k.manifest.kit.description
                )
            })
            .collect();
        // What was picked last time, or the first kit on a first run.
        let mut defaults: Vec<usize> = order
            .iter()
            .enumerate()
            .filter(|(_, k)| saved.kits.iter().any(|s| s == k.name()))
            .map(|(i, _)| i)
            .collect();
        if defaults.is_empty() {
            defaults.push(0);
        }
        let titles: Vec<&str> = order
            .iter()
            .map(|k| k.manifest.kit.title.as_str())
            .collect();
        let names = |opts: &[inquire::list_option::ListOption<&String>]| {
            let picked: Vec<&str> = opts.iter().map(|o| titles[o.index]).collect();
            and_list(&picked)
        };
        let picked = answered(
            inquire::MultiSelect::new("What should your agents focus on?", labels)
                .with_default(&defaults)
                .with_formatter(&names)
                .with_page_size(8)
                .with_help_message("↑↓ move · space toggle · enter confirm · esc cancel · kit show <kit> for details")
                .with_validator(|l: &[inquire::list_option::ListOption<&String>]| {
                    Ok(if l.is_empty() {
                        inquire::validator::Validation::Invalid("Pick at least one".into())
                    } else {
                        inquire::validator::Validation::Valid
                    })
                })
                .raw_prompt(),
        )?;
        let Some(picked) = picked else {
            println!("Nothing was changed.");
            return Ok(());
        };
        picked
            .iter()
            .map(|o| order[o.index].name().to_string())
            .collect()
    } else {
        args.kit.clone()
    };

    // Screen 4: scope.
    let scope = if args.global {
        Scope::Global { home: home_dir()? }
    } else if args.this_repo {
        match &in_repo {
            Some(root) => Scope::Repo(root.clone()),
            None => bail!("--this-repo needs a git repo. cd into one, or use --global"),
        }
    } else if let Some(root) = &in_repo {
        let repo_label = format!(
            "This repo only   {}   commit it to share with your team",
            short_path(&install::display_root(root))
        );
        let options = vec![
            "All my projects   your agents use it everywhere".to_string(),
            repo_label,
        ];
        let start = usize::from(saved.scope.as_deref() == Some("repo"));
        let picked = answered(
            inquire::Select::new("Install for", options)
                .with_starting_cursor(start)
                .with_help_message("↑↓ move · enter confirm · esc cancel")
                .with_formatter(&|o| {
                    if o.index == 0 {
                        "All my projects".into()
                    } else {
                        "This repo only".into()
                    }
                })
                .raw_prompt(),
        )?;
        match picked {
            None => {
                println!("Nothing was changed.");
                return Ok(());
            }
            Some(o) if o.index == 1 => Scope::Repo(root.clone()),
            Some(_) => Scope::Global { home: home_dir()? },
        }
    } else {
        Scope::Global { home: home_dir()? }
    };

    // Screens 5 and 6: the plan, confirm, install.
    let req = Request {
        kits: kits.clone(),
        agents: agents.clone(),
        scope: scope.clone(),
        no_code: args.no_code,
        yes: args.yes,
        print: false,
        force: args.force,
    };
    let outcome = install::add(&req, json)?;
    if matches!(outcome, Outcome::Cancelled | Outcome::Printed) {
        return Ok(());
    }
    let config = Config {
        agents: agents.iter().map(|a| a.id().to_string()).collect(),
        kits: kits.clone(),
        scope: Some(match scope {
            Scope::Global { .. } => "global".into(),
            Scope::Repo(_) => "repo".into(),
        }),
    };
    config.save()?;
    if json {
        return Ok(());
    }
    println!(
        "Saved your choices to {}.",
        super::plan::tilde(&super::config::path())
    );
    let not_now: Vec<&String> = saved.kits.iter().filter(|k| !kits.contains(k)).collect();
    for k in not_now {
        println!("still installed   {k} (kit remove {k} to remove it)");
    }
    if let Some(example) = kits
        .iter()
        .filter_map(|k| catalog::find(k).ok())
        .find_map(|k| k.manifest.kit.example)
    {
        println!();
        println!("Try it:");
        if let Some(first) = agents.iter().find(|a| found.contains(a)) {
            println!("  {} \"{example}\"", first.id());
        }
        match &scope {
            Scope::Repo(root) if !root.join("kit.toml").exists() => {
                println!("  kit init   (the checks that prove a run; kit run needs them)");
                println!("  kit run \"{example}\"   (in its own worktree)");
            }
            _ => println!("  kit run \"{example}\"   (in its own worktree, proven by your checks)"),
        }
    }
    if tty {
        // The fox, resting, signs off after the last step. Terminals only.
        println!();
        for line in kit_tui::fox::lines(0) {
            println!("  {}", line.trim_end());
        }
    }
    Ok(())
}

/// Bare `kit` with no saved setup, in a terminal: run setup first. Someone
/// who already added a kit or ran `kit run` is past that, so they get the
/// Control Room.
pub fn first_run() -> bool {
    std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
        && !super::config::path().exists()
        && !used_before()
}

fn used_before() -> bool {
    let has_entries =
        |dir: std::path::PathBuf| std::fs::read_dir(dir).is_ok_and(|mut d| d.next().is_some());
    has_entries(crate::engine::paths::runs_dir())
        || has_entries(crate::engine::paths::kit_home().join("repos"))
        || crate::engine::paths::kit_home().join("kit.lock").exists()
}

/// `2.1.283 (Claude Code)` → `2.1.283`: the name is already on the line.
fn short_version(v: Option<&str>) -> &str {
    match v {
        Some(v) => v.split(" (").next().unwrap_or(v).trim(),
        None => "installed",
    }
}

/// "A", "A and B", "A, B and C".
/// A path short enough for one option line: the last two folders after
/// `…/` when it is long, so it never breaks mid-word.
fn short_path(path: &str) -> String {
    const MAX: usize = 40;
    if path.chars().count() <= MAX {
        return path.to_string();
    }
    let sep = if path.contains('\\') && !path.contains('/') {
        '\\'
    } else {
        '/'
    };
    let parts: Vec<&str> = path.split(sep).filter(|p| !p.is_empty()).collect();
    let tail = parts[parts.len().saturating_sub(2)..].join(&sep.to_string());
    format!("…{sep}{tail}")
}

pub fn and_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::short_path;

    #[test]
    fn long_paths_keep_whole_folder_names() {
        assert_eq!(short_path("~/code/kit"), "~/code/kit");
        let long = "/home/someone/work/clients/acme/projects/storefront-web";
        assert_eq!(short_path(long), "…/projects/storefront-web");
        let win = r"C:\Users\someone\work\clients\acme\projects\storefront";
        assert_eq!(short_path(win), r"…\projects\storefront");
    }
}
