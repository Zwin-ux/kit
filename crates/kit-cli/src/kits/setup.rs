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
        println!("Kit sets your coding agents up for a job, and proves what they do.\n");
        println!("Looking for agents on this machine…");
    }
    let mut found = Vec::new();
    for agent in Agent::ALL {
        let st = kit_agents::adapter(kind(agent)).probe().await;
        if !json {
            let state = match (st.installed, st.authenticated) {
                (false, _) => "not found".to_string(),
                (true, auth) => format!(
                    "{}{}",
                    st.version.as_deref().unwrap_or("installed"),
                    if auth { "" } else { "   not logged in" }
                ),
            };
            println!("  {:12}  {state}", agent.title());
        }
        if st.installed {
            found.push(agent);
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
        let picked = answered(
            inquire::MultiSelect::new("Which agents should Kit set up?", labels.clone())
                .with_default(&defaults)
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
        // Job kits first; Essentials (the base of the others) last.
        let mut order: Vec<&catalog::Kit> =
            all.iter().filter(|k| k.name() != "essentials").collect();
        order.extend(all.iter().filter(|k| k.name() == "essentials"));
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
        let defaults: Vec<usize> = order
            .iter()
            .enumerate()
            .filter(|(_, k)| saved.kits.iter().any(|s| s == k.name()))
            .map(|(i, _)| i)
            .collect();
        let picked = answered(
            inquire::MultiSelect::new("What should your agents focus on?", labels)
                .with_default(&defaults)
                .with_page_size(8)
                .with_help_message("space toggle · enter confirm · see one with kit show <kit>")
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
            "This repo only   {} (commit it to share with your team)",
            install::display_root(root)
        );
        let options = vec![
            "All my projects   your agents use it everywhere".to_string(),
            repo_label,
        ];
        let start = usize::from(saved.scope.as_deref() == Some("repo"));
        let picked = answered(
            inquire::Select::new("Install for", options)
                .with_starting_cursor(start)
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
        println!("  kit run \"{example}\"   (in its own worktree, proven by your checks)");
    }
    Ok(())
}

/// Bare `kit` with no saved setup, in a terminal: run setup first.
pub fn first_run() -> bool {
    std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
        && !super::config::path().exists()
}
