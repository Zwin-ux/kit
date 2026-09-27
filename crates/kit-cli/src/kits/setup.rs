//! `kit setup`: the first-run questions (which agents, which focus, where),
//! then the same plan and confirm as `kit add`. Screens 1–6 of
//! `docs/dev/DESIGN-KITS.md` §1. Every answer is also a flag, so setup
//! runs without a terminal too.

use super::catalog;
use super::config::Config;
use super::install::{self, Outcome, Request, home_dir, repo_root};
use super::picker::{self, Item, Picker, Tone};
use super::writers::{Agent, Scope};
use crate::cli::SetupArgs;
use anyhow::{Result, bail};
use kit_core::AgentKind;
use kit_tui::ansi::Paint;
use std::io::{IsTerminal, Write};
use std::path::Path;

fn kind(agent: Agent) -> AgentKind {
    match agent {
        Agent::Claude => AgentKind::Claude,
        Agent::Codex => AgentKind::Codex,
        Agent::Grok => AgentKind::Grok,
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
    let paint = Paint::stdout();
    if !json {
        banner(&paint, tty);
        if tty {
            print!("{}", paint.muted("Looking for agents on this machine…"));
            let _ = std::io::stdout().flush();
        } else {
            println!("Looking for agents on this machine…");
        }
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
            ("not installed".to_string(), "")
        };
        rows.push((agent, version, login));
        if st.installed {
            found.push(agent);
        }
    }
    let asking_agents = args.agent.is_empty() && !found.is_empty();
    if tty && !json {
        // The picker shows what was found; clear the "Looking…" line.
        print!("\r\x1b[K");
    }
    if !json && !asking_agents {
        if tty {
            println!();
        }
        // Versions differ in length ("2.1.283", "codex-cli 0.155.0"): pad
        // them so the login column lines up.
        let width = rows
            .iter()
            .filter(|(_, _, l)| !l.is_empty())
            .map(|(_, v, _)| v.chars().count())
            .max()
            .unwrap_or(0);
        for (agent, version, login) in &rows {
            let line = format!(
                "  {}{:12}  {version:width$}   {login}",
                paint.mark(kind(*agent)),
                agent.title()
            );
            println!("{}", line.trim_end());
        }
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
        let items: Vec<Item> = rows
            .iter()
            .map(|(agent, version, login)| {
                let item = Item::new(agent.title()).agent(kind(*agent));
                if found.contains(agent) {
                    let tone = match *login {
                        "logged in" => Tone::Good,
                        "not logged in" => Tone::Warn,
                        _ => Tone::Muted,
                    };
                    item.detail(version.clone()).note(*login, tone)
                } else {
                    item.detail("not installed")
                        .note("Kit can still write its files", Tone::Muted)
                }
            })
            .collect();
        let defaults: Vec<usize> = Agent::ALL
            .iter()
            .enumerate()
            .filter(|(_, a)| preselect.contains(a))
            .map(|(i, _)| i)
            .collect();
        let picker =
            Picker::new("Which agents should Kit set up?", "Agents", items, true).select(&defaults);
        let Some(picked) = picker::run(picker)? else {
            println!("Nothing was changed.");
            return Ok(());
        };
        picked.iter().map(|&i| Agent::ALL[i]).collect()
    } else {
        args.agent.clone()
    };

    // Screen 3: focus.
    let kits: Vec<String> = if args.kit.is_empty() {
        let Some(picked) = pick_kits(
            "What should your agents focus on?",
            "Focus",
            &saved.kits,
            &[],
        )?
        else {
            println!("Nothing was changed.");
            return Ok(());
        };
        picked
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
        let items = vec![
            Item::new("All my projects").detail("your agents use it everywhere"),
            Item::new("This repo only")
                .detail(format!(
                    "{} · commit it to share with your team",
                    short_path(&install::display_root(root))
                ))
                .answer(format!("This repo only ({})", repo_name(root))),
        ];
        let start = usize::from(saved.scope.as_deref() == Some("repo"));
        let picker = Picker::new("Where should Kit install it?", "Install for", items, false)
            .select(&[start]);
        match picker::run(picker)? {
            None => {
                println!("Nothing was changed.");
                return Ok(());
            }
            Some(p) if p == [1] => Scope::Repo(root.clone()),
            Some(_) => Scope::Global { home: home_dir()? },
        }
    } else {
        Scope::Global { home: home_dir()? }
    };
    if tty && !json {
        println!();
    }

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
        "{}",
        paint.muted(&format!(
            "Saved your choices to {}.",
            super::plan::tilde(&super::config::path())
        ))
    );
    let not_now: Vec<&String> = saved.kits.iter().filter(|k| !kits.contains(k)).collect();
    for k in not_now {
        println!(
            "{}{k}  {}",
            paint.muted(&format!("{:10}", "still")),
            paint.muted(&format!("installed; kit remove {k} removes it"))
        );
    }
    if let Some(example) = kits
        .iter()
        .filter_map(|k| catalog::find(k).ok())
        .find_map(|k| k.manifest.kit.example)
    {
        // The same label column as `kit add`'s check/undo lines.
        let step = |label: &str, cmd: &str, why: &str| {
            let why = if why.is_empty() {
                String::new()
            } else {
                format!("   {}", paint.muted(why))
            };
            println!(
                "{}{}{why}",
                paint.muted(&format!("{label:10}")),
                paint.bold(cmd)
            );
        };
        println!();
        let mut label = "try";
        if let Some(first) = agents.iter().find(|a| found.contains(a)) {
            step(label, &format!("{} \"{example}\"", first.id()), "");
            label = "then";
        }
        match &scope {
            Scope::Repo(root) if !root.join("kit.toml").exists() => {
                step(label, "kit init", "writes the checks that prove a run");
                step(
                    "then",
                    &format!("kit run \"{example}\""),
                    "runs it in its own worktree",
                );
            }
            _ => step(
                label,
                &format!("kit run \"{example}\""),
                "runs it in its own worktree, proven by your checks",
            ),
        }
    }
    if tty && paint.enabled() {
        // The fox signs off after the last step, with what bare `kit` does
        // from now on. Terminals only.
        sign_off(&paint);
    } else if tty {
        println!();
        println!("From now on, kit opens the Control Room, where you watch your runs.");
    }
    Ok(())
}

/// Ask which kits, from the bundled catalogue: job kits first, Essentials
/// last, with `selected` (or the first kit) ticked. `None` when cancelled.
pub fn pick_kits(
    question: &str,
    summary: &str,
    selected: &[String],
    installed: &[String],
) -> Result<Option<Vec<String>>> {
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
    let items: Vec<Item> = order
        .iter()
        .map(|k| {
            let d = &k.manifest.kit.description;
            Item::new(&k.manifest.kit.title).detail(if installed.iter().any(|i| i == k.name()) {
                format!("installed · {d}")
            } else {
                d.clone()
            })
        })
        .collect();
    let mut defaults: Vec<usize> = order
        .iter()
        .enumerate()
        .filter(|(_, k)| selected.iter().any(|s| s == k.name()))
        .map(|(i, _)| i)
        .collect();
    if defaults.is_empty() {
        // The first kit not yet installed.
        let first = order
            .iter()
            .position(|k| !installed.iter().any(|i| i == k.name()))
            .unwrap_or(0);
        defaults.push(first);
    }
    let picker = Picker::new(question, summary, items, true)
        .select(&defaults)
        .hint("kit show <kit> for details");
    Ok(picker::run(picker)?.map(|picked| {
        picked
            .iter()
            .map(|&i| order[i].name().to_string())
            .collect()
    }))
}

/// The fox head beside the one thing setup has not said yet.
fn sign_off(paint: &Paint) {
    let beside = [
        String::new(),
        String::new(),
        paint.bold("You're set."),
        format!(
            "From now on, {} opens the Control Room,",
            paint.accent("kit")
        ),
        "where you watch your runs.".to_string(),
    ];
    println!();
    for (i, head) in paint.fox_head().iter().enumerate() {
        let pad = 16usize.saturating_sub(kit_tui::ansi::visible_len(head));
        let text = beside.get(i).map_or("", String::as_str);
        let line = format!("  {head}{}   {text}", " ".repeat(pad));
        println!("{}", line.trim_end());
    }
}

/// The first lines of setup. In a terminal wide enough, the fox head with
/// the name and the promise beside it; otherwise two lines of text.
fn banner(paint: &Paint, tty: bool) {
    let name = format!(
        "{}  {}",
        paint.title("kit"),
        paint.muted(env!("CARGO_PKG_VERSION"))
    );
    let wide = crossterm::terminal::size().is_ok_and(|(w, _)| w >= 66);
    if !(tty && paint.enabled() && wide) {
        println!("{name}");
        println!("Kit sets your coding agents up for one job, then proves what they do.\n");
        return;
    }
    let beside = [
        String::new(),
        name,
        "Kit sets your coding agents up for one job,".to_string(),
        "then proves what they do.".to_string(),
    ];
    println!();
    for (i, head) in paint.fox_head().iter().enumerate() {
        let pad = 16usize.saturating_sub(kit_tui::ansi::visible_len(head));
        let text = beside.get(i).map_or("", String::as_str);
        let line = format!("  {head}{}   {text}", " ".repeat(pad));
        println!("{}", line.trim_end());
    }
    println!();
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

/// The repo's folder name, as `kit add` names it (`this repo (shop)`).
fn repo_name(root: &Path) -> String {
    root.file_name().map_or_else(
        || root.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
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
