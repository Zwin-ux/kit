//! `kit add`, `kit remove`, `kit list`: install kits into agents, record
//! them in `kit.lock`, undo them exactly. See `docs/dev/DESIGN-KITS.md` §1, §4.

use super::catalog::{self, Kit};
use super::lock::{ApprovedChecks, Entry, Lock, LockedHook};
use super::plan::{self, Action, Applied, tilde};
use super::writers::{self, Agent, Options, Resolved, Scope};
use crate::cli::{AddArgs, ListKitsArgs, RemoveArgs};
use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::io::{BufRead, IsTerminal, Write as _};
use std::path::{Path, PathBuf};

pub fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .context("cannot find your home directory (HOME is not set)")
}

/// `--global`, or the git repo around the current directory.
pub fn scope(global: bool) -> Result<Scope> {
    if global {
        return Ok(Scope::Global { home: home_dir()? });
    }
    match repo_root(Path::new(".")) {
        Some(root) => Ok(Scope::Repo(root)),
        None => bail!(
            "not in a git repo. Use --global to install for all your projects, or cd into a repo"
        ),
    }
}

/// A repo root for display: `~/code/shop`.
pub fn display_root(root: &Path) -> String {
    let home = home_dir().ok();
    match home.as_deref().and_then(|h| root.strip_prefix(h).ok()) {
        Some(rest) => format!("~/{}", rest.display()),
        None => root.display().to_string(),
    }
}

pub fn repo_root(dir: &Path) -> Option<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

/// Agents named with `--agent`, else the ones `kit setup` saved, else
/// every agent installed here.
fn agents(chosen: &[Agent]) -> Result<Vec<Agent>> {
    let saved = super::config::Config::load()?.map(|c| c.agents());
    let mut agents: Vec<Agent> = match saved {
        _ if !chosen.is_empty() => chosen.to_vec(),
        Some(saved) if !saved.is_empty() => saved,
        _ => Agent::ALL.into_iter().filter(|a| a.installed()).collect(),
    };
    agents.sort();
    agents.dedup();
    if agents.is_empty() {
        bail!(
            "no coding agent found. Install one, or name one with --agent:\n  \
             Claude Code   npm i -g @anthropic-ai/claude-code\n  \
             Codex         npm i -g @openai/codex"
        );
    }
    Ok(agents)
}

/// How hooks call back into Kit: `kit` if that is this binary, else its path.
fn hook_program() -> String {
    let me = std::env::current_exe().ok();
    let canon = |p: &Path| std::fs::canonicalize(p).ok();
    if let (Some(me), Some(on_path)) = (&me, plan::find_program("kit"))
        && canon(me).is_some()
        && canon(me) == canon(&on_path)
    {
        return "kit".into();
    }
    match me {
        Some(p) => format!("\"{}\"", p.display()),
        None => "kit".into(),
    }
}

// ---- kit add --------------------------------------------------------------

/// Kits to install: each requested kit and its bases, bases first.
struct Chosen {
    kits: Vec<Kit>,
    /// Names asked for on the command line, with what was typed.
    requested: Vec<(String, String)>,
    /// (base, kit that extends it).
    edges: Vec<(String, String)>,
}

fn choose(specs: &[String]) -> Result<Chosen> {
    let mut chosen = Chosen {
        kits: Vec::new(),
        requested: Vec::new(),
        edges: Vec::new(),
    };
    for spec in specs {
        let chain = catalog::resolve(spec)?;
        let top = chain
            .last()
            .expect("resolve returns the kit")
            .name()
            .to_string();
        chosen.requested.push((top.clone(), spec.clone()));
        for kit in chain {
            if kit.name() != top {
                chosen.edges.push((kit.name().to_string(), top.clone()));
            }
            if !chosen.kits.iter().any(|k| k.name() == kit.name()) {
                chosen.kits.push(kit);
            }
        }
    }
    Ok(chosen)
}

/// The plan, split into what to do and what is already there.
struct Prepared {
    /// (kit, action) to apply, in order.
    todo: Vec<(String, Action)>,
    /// (kit, record) already installed by another kit, now shared.
    shared: Vec<(String, Applied)>,
    /// (kit, record) an older version of the kit installed that the new
    /// one no longer has: undone after the new version is in place.
    stale: Vec<(String, Applied)>,
    /// Skill folders changed by hand that this upgrade would replace.
    edited: Vec<String>,
}

fn prepare(
    resolved: &[Resolved<'_>],
    agents: &[Agent],
    scope: &Scope,
    opts: Options,
    lock: &Lock,
) -> Result<Prepared> {
    let hook = hook_program();
    let mut p = Prepared {
        todo: Vec::new(),
        shared: Vec::new(),
        stale: Vec::new(),
        edited: Vec::new(),
    };
    for r in resolved {
        let name = r.kit.name().to_string();
        let actions = writers::plan(r, agents, scope, opts, &hook);
        for action in &actions {
            let key = action.key();
            // Two kits in one install that want the same thing differently.
            if let Some((other, first)) = p.todo.iter().find(|(k, a)| *k != name && a.key() == key)
                && !plan::same_content(first, action)
            {
                bail!(
                    "{other} and {name} both set {}, differently. Install one of them",
                    action
                        .describe()
                        .split_whitespace()
                        .skip(1)
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
            let Some(done) = lock.applied().find(|a| a.key() == key) else {
                p.todo.push((name.clone(), action.clone()));
                continue;
            };
            if plan::in_place(action, done) {
                p.shared.push((name.clone(), done.clone()));
                continue;
            }
            let owners: Vec<&str> = lock
                .kits
                .iter()
                .filter(|e| e.applied.iter().any(|a| a.key() == key))
                .map(|e| e.name.as_str())
                .collect();
            match owners.iter().find(|o| **o != name) {
                // Another kit put it there, with different content.
                Some(owner) => bail!(
                    "{} is installed by {owner} with different content than {name} wants. Remove {owner} first",
                    action
                        .describe()
                        .split_whitespace()
                        .skip(1)
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                // An older version of this kit: replace it, unless the
                // user changed what Kit installed; that needs --force.
                None => {
                    if let Applied::Skill { dir, .. } = done
                        && dir.exists()
                        && let Some(msg) = plan::drifted(done)?
                    {
                        p.edited.push(msg);
                    }
                    let mut action = action.clone();
                    // The older version's gate commands come out unless this
                    // version or another installed kit still wants them.
                    if let (
                        Action::GateToml {
                            file,
                            previous,
                            shared,
                            ..
                        },
                        Applied::GateToml { added, .. },
                    ) = (&mut action, done)
                    {
                        previous.clone_from(added);
                        *shared = lock
                            .kits
                            .iter()
                            .filter(|e| e.name != name)
                            .flat_map(|e| &e.applied)
                            .filter_map(|a| match a {
                                Applied::GateToml {
                                    file: f, wanted, ..
                                } if f == file => Some(wanted.iter().cloned()),
                                _ => None,
                            })
                            .flatten()
                            .collect();
                    }
                    p.todo.push((name.clone(), action));
                }
            }
        }
        // What an older version installed that this one no longer has. Only
        // when the version changed and every agent it was installed for is
        // in this install, so `kit add x -a codex` never drops Claude's files.
        if let Some(e) = lock.get(&name)
            && e.version != r.kit.manifest.kit.version
            && e.agents.iter().all(|a| agents.iter().any(|x| x.id() == a))
        {
            let keys: BTreeSet<String> = actions.iter().map(Action::key).collect();
            for old in &e.applied {
                let used_elsewhere = lock
                    .kits
                    .iter()
                    .any(|o| o.name != name && o.applied.iter().any(|a| a.key() == old.key()));
                if !keys.contains(&old.key()) && !used_elsewhere {
                    p.stale.push((name.clone(), old.clone()));
                }
            }
        }
    }
    Ok(p)
}

/// In a repo, every file Kit writes must land inside it. A repo can make
/// `.claude` or `CLAUDE.md` a link to elsewhere; Kit refuses to follow it,
/// unless it is a link to another file in the same repo.
fn confine(p: &Prepared, scope: &Scope) -> Result<()> {
    let Scope::Repo(root) = scope else {
        return Ok(());
    };
    for (_, a) in &p.todo {
        if let Some(path) = a.path()
            && !plan::inside(path, root)
        {
            bail!(
                "{} is a link, or leads outside this repo. Kit will not write through it",
                tilde(path)
            );
        }
    }
    Ok(())
}

/// One `kit add`, from the command line or from `kit setup`.
pub struct Request {
    pub kits: Vec<String>,
    pub agents: Vec<Agent>,
    pub scope: Scope,
    pub no_code: bool,
    pub yes: bool,
    pub print: bool,
    pub force: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Installed,
    AlreadyInstalled,
    Cancelled,
    Printed,
}

pub fn cmd_add(args: AddArgs, json: bool) -> Result<()> {
    let req = Request {
        scope: scope(args.global)?,
        agents: agents(&args.agent)?,
        kits: args.kits,
        no_code: args.no_code,
        yes: args.yes,
        print: args.print,
        force: args.force,
    };
    add(&req, json).map(|_| ())
}

pub fn add(req: &Request, json: bool) -> Result<Outcome> {
    let (scope, agents) = (&req.scope, req.agents.as_slice());
    plan::follow_links_within(scope.root());
    super::lock::check_not_linked(scope)?;
    let chosen = choose(&req.kits)?;
    let mut lock = Lock::load(scope)?;

    if !json
        && chosen
            .kits
            .iter()
            .any(|k| k.manifest.skill.iter().any(|s| s.source.is_some()))
    {
        eprintln!("Fetching  pinned skills (cached after the first time)");
    }
    let resolved: Vec<Resolved<'_>> = chosen
        .kits
        .iter()
        .map(Resolved::load)
        .collect::<Result<_>>()?;

    let mut opts = Options {
        no_code: req.no_code,
    };
    let mut prepared = prepare(&resolved, agents, scope, opts, &lock)?;
    confine(&prepared, scope)?;
    let text = render_plan(&chosen, agents, scope, &prepared, opts);

    if req.print || json && !req.yes {
        finish_print(&chosen, agents, scope, &prepared, opts, &text, json)?;
        return Ok(Outcome::Printed);
    }
    let work = !prepared.stale.is_empty()
        || prepared
            .todo
            .iter()
            .any(|(_, a)| !matches!(a, Action::Skip { .. }));
    if !json {
        print!("{text}");
    }
    if let Some(first) = prepared.edited.first()
        && !req.force
    {
        bail!(
            "{first} since Kit installed it, and the upgrade would replace your edit. \
             Copy your changes somewhere safe, then run again with --force"
        );
    }
    if !work {
        record(
            &mut lock,
            &chosen,
            agents,
            opts,
            Vec::new(),
            &prepared.shared,
        );
        lock.save(scope)?;
        if !json {
            println!("Already installed. Nothing to do.");
        } else {
            print_json("add", &chosen, agents, scope, &prepared, opts, true)?;
        }
        return Ok(Outcome::AlreadyInstalled);
    }

    if !req.yes {
        let code = prepared.todo.iter().any(|(_, a)| a.runs_code());
        match ask(code)? {
            Answer::Yes => {}
            Answer::No => {
                println!("Nothing was changed.");
                return Ok(Outcome::Cancelled);
            }
            Answer::NoCode => {
                println!("Leaving out MCP servers, hooks and gate checks. Skills and rules only.");
                opts.no_code = true;
                prepared = prepare(&resolved, agents, scope, opts, &lock)?;
                confine(&prepared, scope)?;
            }
        }
    }

    let actions: Vec<&Action> = prepared
        .todo
        .iter()
        .map(|(_, a)| a)
        .filter(|a| !matches!(a, Action::Skip { .. }))
        .collect();
    let owners: Vec<String> = prepared
        .todo
        .iter()
        .filter(|(_, a)| !matches!(a, Action::Skip { .. }))
        .map(|(k, _)| k.clone())
        .collect();
    let owned: Vec<Action> = actions.into_iter().cloned().collect();
    let ours: std::collections::HashSet<String> = lock.applied().map(Applied::key).collect();
    let applied = plan::apply_all(&owned, req.force, &ours)?;
    let pairs: Vec<(String, Applied)> = owners
        .into_iter()
        .zip(applied)
        .filter_map(|(k, a)| Some((k, a?)))
        .collect();
    let mut left = Vec::new();
    for (_, old) in prepared.stale.iter().rev() {
        match plan::undo(old, req.force) {
            Ok(Some(msg)) => left.push(msg),
            Ok(None) => {}
            Err(err) => left.push(format!("{err:#}")),
        }
    }
    for (kit, old) in &prepared.stale {
        if let Some(e) = lock.get_mut(kit) {
            e.applied.retain(|a| a.key() != old.key());
        }
    }
    record(&mut lock, &chosen, agents, opts, pairs, &prepared.shared);
    lock.save(scope)?;
    for msg in &left {
        eprintln!("kept      {msg}");
    }

    if json {
        print_json("add", &chosen, agents, scope, &prepared, opts, true)?;
        return Ok(Outcome::Installed);
    }
    let names: Vec<&str> = chosen.requested.iter().map(|(n, _)| n.as_str()).collect();
    let titles: Vec<&str> = chosen
        .kits
        .iter()
        .filter(|k| names.contains(&k.name()))
        .map(|k| k.manifest.kit.title.as_str())
        .collect();
    let who: Vec<&str> = agents.iter().map(|a| a.title()).collect();
    println!(
        "Done. {} {} {} in {}.",
        super::setup::and_list(&who),
        if who.len() == 1 { "has" } else { "have" },
        super::setup::and_list(&titles),
        scope.label()
    );
    match super::lock::shared_path(scope) {
        Some(shared) => println!(
            "Recorded in {}; {} is a copy to commit for your team.",
            tilde(&super::lock::path(scope)),
            tilde(&shared)
        ),
        None => println!("Recorded in {}.", tilde(&super::lock::path(scope))),
    }
    let flag = if matches!(scope, Scope::Global { .. }) {
        " --global"
    } else {
        ""
    };
    println!("check     kit list{flag}");
    println!("undo      kit remove {}{flag}", names.join(" "));
    Ok(Outcome::Installed)
}

/// Write what was installed into the lock.
fn record(
    lock: &mut Lock,
    chosen: &Chosen,
    agents: &[Agent],
    opts: Options,
    applied: Vec<(String, Applied)>,
    shared: &[(String, Applied)],
) {
    for kit in &chosen.kits {
        let name = kit.name().to_string();
        if lock.get(&name).is_none() {
            lock.kits.push(Entry {
                name: name.clone(),
                version: String::new(),
                source: name.clone(),
                requested: false,
                required_by: Vec::new(),
                agents: Vec::new(),
                hooks: Vec::new(),
                checks: ApprovedChecks::default(),
                applied: Vec::new(),
            });
        }
        let entry = lock.get_mut(&name).expect("just inserted");
        entry.version.clone_from(&kit.manifest.kit.version);
        if let Some((_, spec)) = chosen.requested.iter().find(|(n, _)| *n == name) {
            entry.requested = true;
            entry.source.clone_from(spec);
        }
        for (base, top) in &chosen.edges {
            if *base == name && !entry.required_by.contains(top) {
                entry.required_by.push(top.clone());
            }
        }
        for a in agents {
            if !entry.agents.iter().any(|x| x == a.id()) {
                entry.agents.push(a.id().to_string());
            }
        }
        if !opts.no_code {
            entry.hooks = kit
                .manifest
                .hook
                .iter()
                .map(|h| LockedHook {
                    glob: h.glob.clone(),
                    run: h.run.clone(),
                    builtin: h.builtin,
                })
                .collect();
        }
        // What doctor may run later: fixed now, from what the user approved.
        let check = &kit.manifest.check;
        entry.checks = ApprovedChecks {
            mcp: check
                .mcp_starts
                .iter()
                .filter_map(|n| Some((n.clone(), kit.manifest.mcp.get(n)?.clone())))
                .filter(|(_, s)| !(opts.no_code && s.runs_code()))
                .collect(),
            commands: if opts.no_code {
                Vec::new()
            } else {
                check.commands.clone()
            },
        };
        let mine = applied
            .iter()
            .chain(shared)
            .filter(|(k, _)| *k == name)
            .map(|(_, a)| a.clone());
        for a in mine {
            let key = a.key();
            match entry.applied.iter_mut().find(|x| x.key() == key) {
                Some(slot) => *slot = plan::merge(slot, a),
                None => entry.applied.push(a),
            }
        }
    }
}

enum Answer {
    Yes,
    No,
    NoCode,
}

fn ask(code: bool) -> Result<Answer> {
    if !std::io::stdin().is_terminal() {
        bail!(
            "kit add needs your yes before it writes anything. Run it in a terminal, or add --yes (and --no-code to skip anything that runs code)"
        );
    }
    let prompt = if code {
        "Continue?  [y] install  [n] cancel  [s] skills and rules only (no code): "
    } else {
        "Continue?  [y] install  [n] cancel: "
    };
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    Ok(match line.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => Answer::Yes,
        "s" if code => Answer::NoCode,
        _ => Answer::No,
    })
}

/// Exactly what an action will run: the MCP server's command line, for
/// the hook each of the kit's hook commands with the files it runs on, and
/// for the gate each command every `kit run` in the repo will run.
fn runs(chosen: &Chosen, kit: &str, a: &Action) -> Vec<String> {
    match a {
        Action::GateToml { file, commands, .. } => inferred_checks(file)
            .into_iter()
            .map(|(label, c)| format!("{c}   (inferred {label} check, still runs)"))
            .chain(
                commands
                    .iter()
                    .map(|c| format!("{c}   (in the gate of every kit run)")),
            )
            .collect(),
        Action::HookJson { .. } => chosen
            .kits
            .iter()
            .filter(|k| k.name() == kit)
            .flat_map(|k| &k.manifest.hook)
            .map(|h| h.describe())
            .collect(),
        _ => a.command().into_iter().collect(),
    }
}

/// The checks Kit infers for the repo that holds `kit_toml`, when that file
/// names no format, typecheck or test check of its own. A kit's gate
/// commands run after these; they never replace them.
fn inferred_checks(kit_toml: &Path) -> Vec<(String, String)> {
    let Some(root) = kit_toml.parent() else {
        return Vec::new();
    };
    let gate = match std::fs::read_to_string(kit_toml) {
        Ok(text) => match toml::from_str::<kit_core::KitConfig>(&text) {
            Ok(cfg) => cfg.gate,
            Err(_) => return Vec::new(),
        },
        Err(_) => kit_core::GateConfig::default(),
    };
    crate::engine::infer::with_inferred(&gate, root)
        .map(|g| {
            g.checks()
                .into_iter()
                .filter(|(label, _)| *label != "extra")
                .map(|(l, c)| (l.to_string(), c.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Plan notes under a gate change: commands an upgrade takes out, and
/// programs this machine does not have (every run would fail its gate).
fn gate_notes(kit: &str, a: &Action) -> Vec<String> {
    let Action::GateToml {
        commands,
        previous,
        shared,
        ..
    } = a
    else {
        return Vec::new();
    };
    let mut out: Vec<String> = previous
        .iter()
        .filter(|c| !commands.contains(c) && !shared.contains(c))
        .map(|c| format!("drop  {c}   (no longer in {kit})"))
        .collect();
    let gate = kit_core::GateConfig {
        extra: commands.clone(),
        ..kit_core::GateConfig::default()
    };
    for program in crate::engine::infer::missing_programs(&gate) {
        out.push(format!(
            "note  {program} is not installed here; until it is, every kit run in this repo fails its gate"
        ));
    }
    out
}

/// `[check]` commands `kit doctor` will run later, per kit (none with no code).
fn doctor_commands(chosen: &Chosen, p: &Prepared, opts: Options) -> Vec<(String, String)> {
    if opts.no_code {
        return Vec::new();
    }
    chosen
        .kits
        .iter()
        .filter(|k| p.todo.iter().any(|(n, _)| n == k.name()))
        .flat_map(|k| {
            k.manifest
                .check
                .commands
                .iter()
                .map(|c| (k.name().to_string(), c.clone()))
        })
        .collect()
}

fn render_plan(
    chosen: &Chosen,
    agents: &[Agent],
    scope: &Scope,
    p: &Prepared,
    opts: Options,
) -> String {
    let mut s = String::new();
    let who: Vec<&str> = agents.iter().map(|a| a.title()).collect();
    for (name, _) in &chosen.requested {
        let kit = chosen
            .kits
            .iter()
            .find(|k| k.name() == name)
            .expect("requested kit is chosen");
        let meta = &kit.manifest.kit;
        let extends = if meta.extends.is_empty() {
            String::new()
        } else {
            format!("  (extends {})", meta.extends.join(", "))
        };
        let _ = writeln!(
            s,
            "{} {}{extends}  →  {}, {}",
            meta.title,
            meta.version,
            super::setup::and_list(&who),
            scope.label()
        );
        let _ = writeln!(s, "{}", kit.level.label());
    }
    let _ = writeln!(s);

    // Skills, grouped by folder; details once, then "same N skills".
    let mut dirs: Vec<PathBuf> = Vec::new();
    for (_, a) in &p.todo {
        if let Action::Skill { dir, .. } = a
            && let Some(parent) = dir.parent()
            && !dirs.iter().any(|d| d == parent)
        {
            dirs.push(parent.to_path_buf());
        }
    }
    for (i, parent) in dirs.iter().enumerate() {
        let here: Vec<(&String, &Action)> = p
            .todo
            .iter()
            .filter(|(_, a)| matches!(a, Action::Skill { dir, .. } if dir.parent() == Some(parent)))
            .map(|(k, a)| (k, a))
            .collect();
        let _ = writeln!(s, "skills    {:<3} {}/", here.len(), tilde(parent));
        if i > 0 {
            continue;
        }
        let width = here
            .iter()
            .filter_map(|(_, a)| match a {
                Action::Skill { payload, .. } => Some(payload.name.len()),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        let rows: Vec<(&str, String, &str)> = here
            .iter()
            .filter_map(|(kit, a)| {
                let Action::Skill { payload, .. } = a else {
                    return None;
                };
                let sref = chosen
                    .kits
                    .iter()
                    .find(|k| k.name() == kit.as_str())
                    .and_then(|k| k.manifest.skill.iter().find(|s| s.name == payload.name));
                let origin = sref
                    .and_then(|s| s.origin())
                    .unwrap_or_else(|| format!("{kit} (in the kit)"));
                let licence = sref
                    .and_then(|s| s.licence.as_deref())
                    .unwrap_or("no licence");
                Some((payload.name.as_str(), origin, licence))
            })
            .collect();
        let owidth = rows.iter().map(|(_, o, _)| o.len()).max().unwrap_or(0);
        for (name, origin, licence) in rows {
            let _ = writeln!(s, "            {name:width$}  {origin:owidth$}  {licence}");
        }
    }
    let mut said = Vec::new();
    for (kit, a) in &p.todo {
        match a {
            Action::Skill { .. } => {}
            // One agent skipped for several kits is said once.
            Action::Skip { .. } if said.contains(&a.describe()) => {}
            Action::Skip { .. } => {
                let _ = writeln!(s, "{}", a.describe());
                said.push(a.describe());
            }
            Action::Rules { text, .. } => {
                let _ = writeln!(s, "{}  + {} lines", a.describe(), text.lines().count());
            }
            _ if a.runs_code() => {
                let _ = writeln!(s, "{}   RUNS CODE", a.describe());
                for cmd in runs(chosen, kit, a) {
                    let _ = writeln!(s, "            runs  {cmd}");
                }
                for note in gate_notes(kit, a) {
                    let _ = writeln!(s, "            {note}");
                }
                // When a hook runs goes on its own line, so neither wraps.
                if matches!(a, Action::HookJson { .. }) {
                    for h in chosen
                        .kits
                        .iter()
                        .filter(|k| k.name() == kit)
                        .flat_map(|k| &k.manifest.hook)
                    {
                        let when = match &h.glob {
                            Some(g) => format!("after each edit of {g}"),
                            None => "after each edit".into(),
                        };
                        let _ = writeln!(s, "            when  {when}");
                    }
                }
            }
            _ => {
                let _ = writeln!(s, "{}", a.describe());
            }
        }
    }
    for (kit, old) in &p.stale {
        match plan::drifted(old).ok().flatten() {
            Some(msg) if old.path().is_some_and(|p| p.exists()) => {
                let _ = writeln!(
                    s,
                    "keep      {msg}; no longer in {kit}, left in place (--force removes it)"
                );
            }
            _ => {
                let _ = writeln!(s, "remove    {}   (no longer in {kit})", old.describe());
            }
        }
    }
    for msg in &p.edited {
        let _ = writeln!(
            s,
            "edited    {msg}; the upgrade replaces it (needs --force)"
        );
    }
    if !p.shared.is_empty() {
        let _ = writeln!(
            s,
            "already   {} in place (installed before, or by another kit)",
            p.shared.len()
        );
    }
    let checks = doctor_commands(chosen, p, opts);
    for (_, cmd) in &checks {
        let _ = writeln!(s, "check     kit doctor runs `{cmd}`   RUNS CODE");
    }
    let code = p.todo.iter().filter(|(_, a)| a.runs_code()).count() + checks.len();
    let _ = writeln!(s);
    if code > 0 {
        let _ = writeln!(
            s,
            "Runs code on your machine: {code} (MCP servers, hooks and checks, each shown above)."
        );
        let _ = writeln!(s);
    }
    s
}

fn finish_print(
    chosen: &Chosen,
    agents: &[Agent],
    scope: &Scope,
    p: &Prepared,
    opts: Options,
    text: &str,
    json: bool,
) -> Result<()> {
    if json {
        return print_json("add", chosen, agents, scope, p, opts, false);
    }
    print!("{text}");
    println!("Nothing was written (--print).");
    Ok(())
}

fn print_json(
    command: &str,
    chosen: &Chosen,
    agents: &[Agent],
    scope: &Scope,
    p: &Prepared,
    opts: Options,
    applied: bool,
) -> Result<()> {
    let actions: Vec<_> = p
        .todo
        .iter()
        .map(|(kit, a)| {
            serde_json::json!({
                "kit": kit, "change": a.describe(), "key": a.key(),
                "runsCode": a.runs_code(), "runs": runs(chosen, kit, a),
                "skipped": matches!(a, Action::Skip { .. }),
            })
        })
        .collect();
    let data = serde_json::json!({
        "kits": chosen.requested.iter().map(|(n, _)| n).collect::<Vec<_>>(),
        "installs": chosen.kits.iter().map(Kit::name).collect::<Vec<_>>(),
        "agents": agents.iter().map(|a| a.id()).collect::<Vec<_>>(),
        "scope": scope_json(scope),
        "actions": actions,
        "doctorChecks": doctor_commands(chosen, p, opts)
            .into_iter()
            .map(|(kit, run)| serde_json::json!({ "kit": kit, "runs": run }))
            .collect::<Vec<_>>(),
        "shared": p.shared.len(),
        "edited": p.edited,
        "applied": applied,
    });
    let warnings = if applied {
        vec![]
    } else {
        vec!["nothing was written; add --yes to install".to_string()]
    };
    let env = crate::envelope(command, true, data, None, warnings);
    println!("{}", serde_json::to_string_pretty(&env)?);
    Ok(())
}

fn scope_json(scope: &Scope) -> serde_json::Value {
    match scope {
        Scope::Global { .. } => serde_json::json!("global"),
        Scope::Repo(root) => serde_json::json!(root.display().to_string()),
    }
}

// ---- kit remove -------------------------------------------------------------

pub fn cmd_remove(args: RemoveArgs, json: bool) -> Result<()> {
    let scope = scope(args.global)?;
    plan::follow_links_within(scope.root());
    // Before undoing anything: a linked lock would refuse the save after
    // the files were already changed, leaving a stale record.
    super::lock::check_not_linked(&scope)?;
    let mut lock = Lock::load(&scope)?;
    let flag = if args.global { " --global" } else { "" };
    for name in &args.kits {
        match lock.get(name) {
            None => bail!(
                "{name} is not installed in {}. See kit list{flag}",
                scope.label()
            ),
            Some(e) if !e.requested => bail!(
                "{name} was installed as part of {}. Remove that instead",
                e.required_by.join(", ")
            ),
            Some(_) => {}
        }
    }

    // A base another installed kit extends stays until that kit goes; it
    // only stops being one the user asked for.
    let mut still_needed = Vec::new();
    for name in &args.kits {
        let e = lock.get(name).expect("checked above");
        let users: Vec<String> = e
            .required_by
            .iter()
            .filter(|r| !args.kits.contains(r))
            .cloned()
            .collect();
        if !users.is_empty() {
            still_needed.push((name.clone(), users));
        }
    }

    // The kits named, plus bases nothing else needs any more.
    let mut going: BTreeSet<String> = args
        .kits
        .iter()
        .filter(|n| !still_needed.iter().any(|(s, _)| s == *n))
        .cloned()
        .collect();
    loop {
        let more: Vec<String> = lock
            .kits
            .iter()
            .filter(|e| !going.contains(&e.name) && !e.requested && !e.required_by.is_empty())
            .filter(|e| e.required_by.iter().all(|r| going.contains(r)))
            .map(|e| e.name.clone())
            .collect();
        if more.is_empty() {
            break;
        }
        going.extend(more);
    }

    let staying: Vec<&Entry> = lock
        .kits
        .iter()
        .filter(|e| !going.contains(&e.name))
        .collect();
    let mut undo: Vec<Applied> = Vec::new();
    for e in lock.kits.iter().filter(|e| going.contains(&e.name)) {
        for a in &e.applied {
            let key = a.key();
            let kept = staying
                .iter()
                .any(|s| s.applied.iter().any(|x| x.key() == key));
            if !kept && !undo.iter().any(|u| u.key() == key) {
                undo.push(a.clone());
            }
        }
    }

    // A gate command another installed kit also wants stays, and becomes
    // that kit's to remove.
    // (kit, command, whether Kit created the file)
    let mut handed: Vec<(String, String, bool)> = Vec::new();
    for a in &mut undo {
        if let Applied::GateToml {
            file,
            added,
            created,
            ..
        } = a
        {
            let created = *created;
            added.retain(|cmd| {
                let heir = staying.iter().find(|s| {
                    s.applied.iter().any(|x| {
                        matches!(x, Applied::GateToml { file: f, wanted, .. }
                            if f == file && wanted.contains(cmd))
                    })
                });
                match heir {
                    Some(h) => {
                        handed.push((h.name.clone(), cmd.clone(), created));
                        false
                    }
                    None => true,
                }
            });
        }
    }

    let summary = removal_summary(&undo);
    if !json {
        for (name, users) in &still_needed {
            println!(
                "{name} stays: {} extends it. It goes when {} is removed.",
                users.join(", "),
                if users.len() == 1 {
                    "that kit"
                } else {
                    "they are"
                }
            );
        }
    }
    if going.is_empty() {
        for (name, _) in &still_needed {
            if let Some(e) = lock.get_mut(name) {
                e.requested = false;
            }
        }
        lock.save(&scope)?;
        if json {
            let data = serde_json::json!({
                "removed": [], "scope": scope_json(&scope), "changes": 0,
                "stays": still_needed.iter().map(|(n, u)| serde_json::json!({"kit": n, "extendedBy": u})).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&crate::envelope("remove", true, data, None, vec![]))?
            );
        }
        return Ok(());
    }
    if !json {
        let names: Vec<&str> = going.iter().map(String::as_str).collect();
        println!(
            "Removes {} from {}: {summary}.",
            names.join(", "),
            scope.label()
        );
    }
    if !args.yes {
        if !std::io::stdin().is_terminal() || json {
            bail!("kit remove needs your yes. Run it in a terminal, or add --yes");
        }
        print!("Continue? [y/N] ");
        std::io::stdout().flush()?;
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)?;
        if !matches!(line.trim(), "y" | "Y" | "yes") {
            println!("Nothing was changed.");
            return Ok(());
        }
    }

    let mut kept = Vec::new();
    for a in undo.iter().rev() {
        if let (Scope::Repo(root), Some(path)) = (&scope, a.path())
            && !plan::inside(path, root)
        {
            kept.push(format!(
                "{} is now a link, or leads outside this repo; left alone",
                tilde(path)
            ));
            continue;
        }
        if let Some(msg) = plan::undo(a, args.force)? {
            kept.push(msg);
        }
    }
    lock.kits.retain(|e| !going.contains(&e.name));
    for (kit, cmd, was_created) in &handed {
        if let Some(e) = lock.get_mut(kit) {
            for a in &mut e.applied {
                if let Applied::GateToml {
                    added,
                    wanted,
                    created,
                    ..
                } = a
                    && wanted.contains(cmd)
                {
                    if !added.contains(cmd) {
                        added.push(cmd.clone());
                    }
                    *created |= *was_created;
                }
            }
        }
    }
    for (name, _) in &still_needed {
        if let Some(e) = lock.get_mut(name) {
            e.requested = false;
        }
    }
    for e in &mut lock.kits {
        e.required_by.retain(|r| !going.contains(r));
    }
    lock.save(&scope)?;

    if json {
        let data = serde_json::json!({
            "removed": going, "scope": scope_json(&scope), "changes": undo.len(), "leftInPlace": kept,
        });
        let env = crate::envelope("remove", true, data, None, kept);
        println!("{}", serde_json::to_string_pretty(&env)?);
        return Ok(());
    }
    for msg in &kept {
        println!("kept      {msg}");
    }
    println!("Done.");
    Ok(())
}

fn removal_summary(undo: &[Applied]) -> String {
    let count = |f: fn(&Applied) -> bool| undo.iter().filter(|a| f(a)).count();
    let parts = [
        (
            count(|a| matches!(a, Applied::Skill { .. })),
            "skill",
            "skills",
        ),
        (
            count(|a| matches!(a, Applied::Rules { .. })),
            "rules block",
            "rules blocks",
        ),
        (
            count(|a| {
                matches!(
                    a,
                    Applied::McpJson { .. } | Applied::McpToml { .. } | Applied::ClaudeMcp { .. }
                )
            }),
            "MCP server",
            "MCP servers",
        ),
        (
            count(|a| matches!(a, Applied::HookJson { .. })),
            "hook",
            "hooks",
        ),
        (
            count(|a| matches!(a, Applied::GateToml { added, .. } if !added.is_empty())),
            "gate entry",
            "gate entries",
        ),
    ];
    let words: Vec<String> = parts
        .iter()
        .filter(|(n, ..)| *n > 0)
        .map(|(n, one, many)| format!("{n} {}", if *n == 1 { one } else { many }))
        .collect();
    if words.is_empty() {
        "nothing on disk".into()
    } else {
        words.join(", ")
    }
}

// ---- kit list ---------------------------------------------------------------

pub fn cmd_list(args: ListKitsArgs, json: bool) -> Result<()> {
    let mut scopes = vec![Scope::Global { home: home_dir()? }];
    if !args.global
        && let Some(root) = repo_root(Path::new("."))
    {
        scopes.push(Scope::Repo(root));
    }
    let mut out = Vec::new();
    for scope in &scopes {
        let lock = Lock::load(scope)?;
        let mut rows = Vec::new();
        for e in &lock.kits {
            let mut drift = Vec::new();
            for a in &e.applied {
                if let Some(d) = plan::drifted(a)? {
                    drift.push(d);
                }
            }
            rows.push((e.clone(), drift));
        }
        let proposed = proposed_here(scope, &lock);
        out.push((scope.clone(), rows, proposed));
    }

    if json {
        let data: Vec<_> = out
            .iter()
            .map(|(scope, rows, proposed)| {
                let kits: Vec<_> = rows
                    .iter()
                    .map(|(e, drift)| {
                        serde_json::json!({
                            "name": e.name, "version": e.version, "agents": e.agents,
                            "requested": e.requested, "requiredBy": e.required_by,
                            "skills": skill_names(e).len(), "drift": drift,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "scope": scope_json(scope), "kits": kits, "notInstalledHere": proposed,
                })
            })
            .collect();
        let env = crate::envelope(
            "list",
            true,
            serde_json::json!({ "scopes": data }),
            None,
            vec![],
        );
        println!("{}", serde_json::to_string_pretty(&env)?);
        return Ok(());
    }

    for (_, _, proposed) in &out {
        if !proposed.is_empty() {
            println!(
                "This repo's kit.lock lists {}, not installed on this machine.",
                proposed.join(", ")
            );
            println!("next      kit add <kit>   (shows the plan before anything runs)");
            println!();
        }
    }
    if out.iter().all(|(_, rows, _)| rows.is_empty()) {
        println!("No kits installed.");
        println!("next      kit show");
        return Ok(());
    }
    for (scope, rows, _) in &out {
        if rows.is_empty() {
            continue;
        }
        println!("{}  ({})", scope.label(), tilde(&super::lock::path(scope)));
        let width = rows.iter().map(|(e, _)| e.name.len()).max().unwrap_or(0);
        for (e, drift) in rows {
            let mut parts = vec![format!("{} skills", skill_names(e).len())];
            let mcp = e
                .applied
                .iter()
                .filter(|a| {
                    matches!(
                        a,
                        Applied::McpJson { .. }
                            | Applied::McpToml { .. }
                            | Applied::ClaudeMcp { .. }
                    )
                })
                .count();
            if mcp > 0 {
                parts.push(format!("{mcp} mcp"));
            }
            if !e.hooks.is_empty() {
                parts.push(format!("{} hook", e.hooks.len()));
            }
            let status = match drift.len() {
                0 => "ok".to_string(),
                1 => drift[0].clone(),
                n => format!("{n} changes by hand: {}", drift[0]),
            };
            let part_of = if e.requested {
                String::new()
            } else {
                format!("  (part of {})", e.required_by.join(", "))
            };
            println!(
                "  {:width$}   {:7}  {:14}  {:28}  {status}{part_of}",
                e.name,
                e.version,
                e.agents.join(", "),
                parts.join(" · ")
            );
        }
    }
    Ok(())
}

/// Kit names in the repo's shared kit.lock that this machine has not
/// installed. Names only: nothing else in that file is trusted or used.
fn proposed_here(scope: &Scope, lock: &Lock) -> Vec<String> {
    let Some(file) = super::lock::shared_path(scope) else {
        return Vec::new();
    };
    let Ok(raw) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Vec::new();
    };
    v["kits"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|k| k["requested"].as_bool().unwrap_or(true))
        .filter_map(|k| k["name"].as_str())
        .filter(|n| super::manifest::is_slug(n) && lock.get(n).is_none())
        .map(str::to_string)
        .collect()
}

fn skill_names(e: &Entry) -> BTreeSet<String> {
    e.applied
        .iter()
        .filter_map(|a| match a {
            Applied::Skill { dir, .. } => dir.file_name().map(|n| n.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}
