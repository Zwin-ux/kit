//! Kit command line — product entry for the Control Room and headless runs.
//!
//! Default surface is the ratatui Control Room (PRD §4.2). `kit run` is the M1
//! headless path: worktree → dry-run stream → gate → receipt.

mod cli;
mod engine;
mod init;
mod land;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use cli::{Cli, Command, ReceiptAction};
use engine::{RunOptions, execute, spawn_production};
use kit_core::{AgentKind, Bounds, RunDelta, RunId, RunState};
use kit_tui::{EngineCommand, LaunchConfig, run_configured};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        // --help and --version print and exit 0; usage errors exit 2.
        Err(err) if !json || !err.use_stderr() => err.exit(),
        Err(err) => {
            let text = err.render().to_string();
            let first = text.lines().next().unwrap_or_default();
            fail(&args, true, first.trim_start_matches("error: "));
        }
    };
    if let Err(err) = dispatch(cli).await {
        fail(&args, json, &format!("{err:#}"));
    }
}

/// Print a could-not-run error (one envelope under --json) and exit 2.
fn fail(args: &[String], json: bool, message: &str) -> ! {
    if json {
        let envelope = json_envelope(
            &cli::command_name(args),
            false,
            serde_json::Value::Null,
            Some(message.to_string()),
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&envelope).unwrap_or_default()
        );
    } else {
        eprintln!("kit: {message}");
    }
    std::process::exit(2);
}

async fn dispatch(cli: Cli) -> Result<()> {
    let json = cli.global.json;
    if let Some(dir) = &cli.global.dir {
        // Like `git -C`: every command below resolves "." against DIR.
        std::env::set_current_dir(dir)
            .with_context(|| format!("cannot use -C {}: no such folder", dir.display()))?;
    }
    if cli.demo && cli.command.is_some() {
        anyhow::bail!("--demo opens the Control Room. Use `kit --demo` on its own");
    }
    match cli.command {
        None => launch_tui(cli.demo).await,
        Some(Command::Run(args)) => cmd_run(args, json).await,
        Some(Command::Init(args)) => init::cmd_init(args, json).await,
        Some(Command::Land(args)) => land::cmd_land(args, json),
        Some(Command::Doctor) => {
            print_doctor(env!("CARGO_PKG_VERSION"), json);
            Ok(())
        }
        Some(Command::Receipt(args)) => match args.action {
            None => cmd_receipt_list(args.list.limit, json),
            Some(ReceiptAction::List(list)) => cmd_receipt_list(list.limit, json),
            Some(ReceiptAction::Show(show)) => cmd_receipt_show(&show.id, show.output, json),
        },
        Some(Command::Completions { shell }) => {
            clap_complete::generate(shell, &mut Cli::command(), "kit", &mut std::io::stdout());
            Ok(())
        }
    }
}

async fn launch_tui(demo: bool) -> Result<()> {
    use std::io::IsTerminal;
    // Without a terminal the TUI would draw into a pipe and wait for keys
    // that never come.
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        anyhow::bail!(
            "the Control Room needs an interactive terminal. In scripts, use `kit run \"…\" --json`"
        );
    }
    let (delta_tx, delta_rx) = mpsc::channel::<(RunId, RunDelta)>(256);
    let (cmd_tx, cmd_rx) = mpsc::channel::<EngineCommand>(64);

    // Engine supervisor: registry + max-8 concurrency (see engine::supervisor).
    spawn_production(cmd_rx, delta_tx);

    run_configured(
        LaunchConfig {
            demo,
            engine_tx: Some(cmd_tx),
            probe_agents: true,
        },
        delta_rx,
    )
    .await
}

async fn cmd_run(args: cli::RunArgs, json: bool) -> Result<()> {
    let task = args.task().to_string();
    if task.trim().is_empty() {
        anyhow::bail!(
            "the task is empty. Say what the agent should do: kit run \"add a test for X\""
        );
    }
    let repo = ".".to_string();
    let allow_vacuous = args.allow_vacuous;
    // None = live with the chosen agent. --dry-run runs no agent.
    let dry_run = args.dry_run.then_some(true);
    let agent = args.agent.map(AgentKind::from);
    let kind = agent.unwrap_or(AgentKind::Codex);
    // Stderr only: under --json, stdout holds one envelope.
    if let Some(hint) = init_hint(Path::new(&repo)) {
        eprintln!("kit: {hint}");
    }
    let opts = RunOptions {
        repo,
        agent: kind,
        task,
        dry_run,
        bounds: Bounds::default(),
    };

    // Stream agent output, state changes and silence notices to stderr while the
    // run is live. Stdout stays the result: exactly one envelope under --json.
    let (delta_tx, delta_rx) = mpsc::channel::<(RunId, RunDelta)>(256);
    let echo = tokio::spawn(echo_deltas(delta_rx, kind, QUIET_NOTICE, |line| {
        eprint!("{line}")
    }));
    let result = execute(opts, None, Some(delta_tx)).await;
    let _ = echo.await;
    let result = result?;

    let gate_vacuous = result
        .gate
        .as_ref()
        .map(engine::infer::is_vacuous)
        .unwrap_or(false);
    // Dry-run has no proof claim (CEO stamp); live vacuous fails unless allowed.
    let dry = dry_run == Some(true);

    let exit_nonzero = matches!(
        result.state,
        RunState::Pass if gate_vacuous && !allow_vacuous && !dry
    ) || matches!(result.state, RunState::Fail)
        || !matches!(result.state, RunState::Pass | RunState::Fail);

    if json {
        let data = serde_json::json!({
            "id": result.id.0,
            "state": format!("{:?}", result.state).to_ascii_lowercase(),
            "receiptDir": result.receipt_dir,
            "worktreeRemoved": result.worktree_removed,
            "gatePassed": result.gate.as_ref().map(|g| g.passed),
            "gateVacuous": gate_vacuous,
        });
        let ok = !exit_nonzero;
        let envelope = json_envelope("run", ok, data, None);
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!("run {}", result.id);
        println!("  state     {}", state_label(result.state));
        println!("  receipt   {}", result.receipt_dir.display());
        if let Some(wt) = &result.worktree {
            println!(
                "  worktree  {} (kept: it has the run's changes)",
                wt.display()
            );
        } else if result.worktree_removed {
            println!("  worktree  removed (clean)");
        }
        if let Some(g) = &result.gate {
            let label = if gate_vacuous {
                "UNCONFIGURED"
            } else if g.passed {
                "PASS"
            } else {
                "FAIL"
            };
            println!("  gate      {label}");
        }
        if let Some(step) = land_hint(
            result.state,
            gate_vacuous,
            &result.receipt_dir,
            &result.id.0,
        ) {
            println!();
            println!("{step}");
        }
    }

    let code = match result.state {
        RunState::Pass if gate_vacuous && !allow_vacuous && !dry => 1,
        RunState::Pass => 0,
        RunState::Fail => 1,
        _ => 2,
    };
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

/// Silence longer than this gets a stderr notice, so a stalled agent is visible.
const QUIET_NOTICE: Duration = Duration::from_secs(30);

/// Echo run deltas through `emit` until the engine drops its sender.
///
/// Session B stalled with no output at all: every delta was discarded, so a
/// working agent and a hung one looked the same. Output chunks and state
/// changes are echoed as they arrive; each `quiet` stretch adds a notice.
async fn echo_deltas(
    mut rx: mpsc::Receiver<(RunId, RunDelta)>,
    agent: AgentKind,
    quiet: Duration,
    mut emit: impl FnMut(&str),
) {
    let mut state = RunState::Queued;
    let mut silent = Duration::ZERO;
    loop {
        match tokio::time::timeout(quiet, rx.recv()).await {
            Ok(Some((_, delta))) => {
                silent = Duration::ZERO;
                if let RunDelta::State(next) = &delta {
                    state = *next;
                }
                if let Some(line) = delta_line(&delta) {
                    emit(&line);
                }
            }
            Ok(None) => return,
            Err(_) => {
                silent += quiet;
                emit(&format!(
                    "kit: still {} ({agent}), no output for {}s — Ctrl-C to abort\n",
                    state_label(state),
                    silent.as_secs()
                ));
            }
        }
    }
}

/// Stderr text for one delta; `None` for deltas the final result reports.
fn delta_line(delta: &RunDelta) -> Option<String> {
    match delta {
        RunDelta::Output(chunk) if chunk.ends_with('\n') => Some(chunk.clone()),
        RunDelta::Output(chunk) => Some(format!("{chunk}\n")),
        RunDelta::State(state) => Some(format!("kit: state {}\n", state_label(*state))),
        RunDelta::Worktree(_) | RunDelta::Gate(_) => None,
    }
}

/// The next step after a proven run with changes: `Next: kit land <id>`.
fn land_hint(state: RunState, vacuous: bool, receipt_dir: &Path, id: &str) -> Option<String> {
    (state == RunState::Pass && !vacuous && receipt_dir.join("diff.patch").is_file())
        .then(|| format!("Next: kit land {id}"))
}

/// One line that points to `kit init` when `repo` is a folder with no kit.toml.
fn init_hint(repo: &Path) -> Option<String> {
    (repo.is_dir() && !repo.join("kit.toml").exists())
        .then(|| "no kit.toml in this repo. Run `kit init` to write a gate.".to_string())
}

fn state_label(state: RunState) -> String {
    format!("{state:?}").to_ascii_lowercase()
}

/// CEO stamp P4 — thin JSON envelope (`schemaVersion: 1`, camelCase).
fn json_envelope(
    command: &str,
    ok: bool,
    data: serde_json::Value,
    error: Option<String>,
) -> serde_json::Value {
    envelope(command, ok, data, error, Vec::new())
}

/// [`json_envelope`] with warnings.
pub(crate) fn envelope(
    command: &str,
    ok: bool,
    data: serde_json::Value,
    error: Option<String>,
    warnings: Vec<String>,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": 1,
        "command": command,
        "ok": ok,
        "data": data,
        "error": error,
        "warnings": warnings,
    })
}

/// `kit receipt [list]` — runs under `~/.kit/runs/`, newest first.
fn cmd_receipt_list(limit: usize, json: bool) -> Result<()> {
    let rows = engine::store::list_receipts(limit)?;
    if json {
        let items: Vec<serde_json::Value> = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "state": r.state,
                    "agent": r.agent,
                    "repo": r.repo,
                    "task": r.task,
                    "gatePassed": r.gate_passed,
                    "dir": r.dir,
                })
            })
            .collect();
        let data = serde_json::json!({
            "kitHome": engine::paths::kit_home(),
            "runsDir": engine::paths::runs_dir(),
            "count": items.len(),
            "receipts": items,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&json_envelope("receipt.list", true, data, None))?
        );
    } else if rows.is_empty() {
        println!(
            "no receipts under {} — run `kit run --dry-run --task smoke` first",
            engine::paths::runs_dir().display()
        );
    } else {
        println!(
            "{:<28} {:<8} {:<8} {:<12} TASK",
            "ID", "STATE", "AGENT", "REPO"
        );
        for r in &rows {
            let short = if r.id.len() > 26 {
                format!("{}…", &r.id[..25])
            } else {
                r.id.clone()
            };
            println!(
                "{:<28} {:<8} {:<8} {:<12} {}",
                short, r.state, r.agent, r.repo, r.task
            );
        }
        println!();
        println!(
            "{} receipt(s) in {}  ·  kit receipt show <id>",
            rows.len(),
            engine::paths::runs_dir().display()
        );
    }
    Ok(())
}

/// `kit receipt show <id>` — one run's proof.
fn cmd_receipt_show(id: &str, show_output: bool, json: bool) -> Result<()> {
    let Some(receipt) = engine::store::read_receipt(id)? else {
        anyhow::bail!("receipt not found for `{id}`");
    };
    let dir = engine::store::resolve_run_dir(id)?;
    if json {
        let mut data = serde_json::to_value(&receipt)?;
        if let Some(obj) = data.as_object_mut() {
            obj.insert(
                "dir".into(),
                serde_json::Value::String(dir.display().to_string()),
            );
        }
        if show_output {
            let tail = engine::store::read_output_tail(id, 64 * 1024)?;
            if let Some(obj) = data.as_object_mut() {
                obj.insert("outputTail".into(), serde_json::Value::String(tail));
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&json_envelope("receipt.show", true, data, None))?
        );
    } else {
        println!("receipt {}", receipt.id);
        println!("  dir       {}", dir.display());
        println!("  state     {}", state_label(receipt.state));
        println!("  agent     {}", receipt.spec.agent.label());
        println!("  repo      {}", receipt.spec.repo.display());
        println!(
            "  task      {}",
            receipt.spec.task.lines().next().unwrap_or("")
        );
        if let Some(g) = &receipt.gate {
            // Same labels as `kit run`: zero checks proves nothing.
            let label = if engine::infer::is_vacuous(g) {
                "UNCONFIGURED"
            } else if g.passed {
                "PASS"
            } else {
                "FAIL"
            };
            println!("  gate      {label}  ({} checks)", g.checks.len());
            for c in &g.checks {
                println!(
                    "            {:?}  {}  {}",
                    c.status,
                    c.label,
                    c.summary.as_deref().unwrap_or("")
                );
            }
        } else {
            println!("  gate      (none)");
        }
        if !receipt.diff.is_empty() {
            println!(
                "  diff      {} bytes (see {}/diff.patch)",
                receipt.diff.len(),
                dir.display()
            );
        }
        if show_output {
            let tail = engine::store::read_output_tail(id, 8 * 1024)?;
            println!();
            println!("--- output.log (tail) ---");
            print!("{tail}");
            if !tail.ends_with('\n') {
                println!();
            }
        } else {
            println!();
            println!("  tip  kit receipt show {} --output", receipt.id);
        }
        let vacuous = receipt.gate.as_ref().is_none_or(engine::infer::is_vacuous);
        if let Some(step) = land_hint(receipt.state, vacuous, &dir, &receipt.id.0) {
            println!("{step}");
        }
    }
    Ok(())
}

/// Names that collide with this binary on PATH (npm 0.1 ships `kit` + `kit.cmd`).
#[cfg(windows)]
const KIT_PATH_NAMES: &[&str] = &["kit.exe", "kit.cmd", "kit"];
#[cfg(not(windows))]
const KIT_PATH_NAMES: &[&str] = &["kit"];

fn same_kit_file(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    if let (Ok(ca), Ok(cb)) = (std::fs::canonicalize(a), std::fs::canonicalize(b))
        && ca == cb
    {
        return true;
    }
    same_file_identity(a, b)
}

/// True when `a` and `b` are the same inode / NTFS file index (hardlinks).
fn same_file_identity(a: &Path, b: &Path) -> bool {
    match (file_identity(a), file_identity(b)) {
        (Some(ia), Some(ib)) => ia == ib,
        _ => {
            #[cfg(windows)]
            {
                a.as_os_str().eq_ignore_ascii_case(b.as_os_str())
            }
            #[cfg(not(windows))]
            {
                false
            }
        }
    }
}

#[cfg(unix)]
fn file_identity(path: &Path) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.dev(), meta.ino()))
}

/// Volume serial + file index via `GetFileInformationByHandle` (stable).
#[cfg(windows)]
fn file_identity(path: &Path) -> Option<(u32, u64)> {
    use std::fs::File;
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    struct FileTime {
        dw_low_date_time: u32,
        dw_high_date_time: u32,
    }
    #[repr(C)]
    struct ByHandleFileInformation {
        dw_file_attributes: u32,
        ft_creation_time: FileTime,
        ft_last_access_time: FileTime,
        ft_last_write_time: FileTime,
        dw_volume_serial_number: u32,
        n_file_size_high: u32,
        n_file_size_low: u32,
        n_number_of_links: u32,
        n_file_index_high: u32,
        n_file_index_low: u32,
    }

    unsafe extern "system" {
        fn GetFileInformationByHandle(
            handle: *mut std::ffi::c_void,
            info: *mut ByHandleFileInformation,
        ) -> i32;
    }

    let file = File::open(path).ok()?;
    let mut info = unsafe { std::mem::zeroed::<ByHandleFileInformation>() };
    let ok = unsafe { GetFileInformationByHandle(file.as_raw_handle().cast(), &mut info) };
    if ok == 0 {
        return None;
    }
    let index = (u64::from(info.n_file_index_high) << 32) | u64::from(info.n_file_index_low);
    Some((info.dw_volume_serial_number, index))
}

#[cfg(not(any(windows, unix)))]
fn file_identity(_path: &Path) -> Option<(u64, u64)> {
    None
}

/// Other `kit` / `kit.exe` / `kit.cmd` files on `path_env` that are not `current`.
fn other_kits_on_path(current: &Path, path_env: &OsStr) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for dir in std::env::split_paths(path_env) {
        for name in KIT_PATH_NAMES {
            let candidate = dir.join(name);
            if !candidate.is_file() {
                continue;
            }
            if same_kit_file(&candidate, current) {
                continue;
            }
            let key = std::fs::canonicalize(&candidate).unwrap_or_else(|_| candidate.clone());
            if !seen.insert(key) {
                continue;
            }
            out.push(candidate);
        }
    }
    out
}

/// What a `kit` found on PATH runs, read from its npm shim or symlink target.
#[derive(Debug, PartialEq, Eq)]
enum PathKit {
    /// npm `@mzwin/kit` 1.x: the launcher for this same binary.
    Launcher,
    /// npm `@mzwin/kit` 0.1.x: the old Node app.
    Node01,
    Other,
}

fn classify_path_kit(path: &Path) -> PathKit {
    let mut text = std::fs::canonicalize(path)
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    // npm shims are a few hundred bytes; never read a binary.
    if std::fs::metadata(path).is_ok_and(|m| m.len() <= 64 * 1024)
        && let Ok(bytes) = std::fs::read(path)
    {
        text.push('\n');
        text.push_str(&String::from_utf8_lossy(&bytes));
    }
    classify_shim_text(&text)
}

fn classify_shim_text(text: &str) -> PathKit {
    let text = text.replace('\\', "/");
    if text.contains("@mzwin/kit/bin/kit.js") {
        PathKit::Launcher
    } else if text.contains("@mzwin/kit/dist/bin.js") {
        PathKit::Node01
    } else {
        PathKit::Other
    }
}

fn print_doctor(version: &str, json: bool) {
    let kit_home = engine::paths::kit_home();
    let skills = kit_agents::skills::resolve_skills_dir(std::path::Path::new("."));
    let statuses = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(kit_agents::probe_all())
    });

    let binary_path = std::env::current_exe().ok();
    let collisions = match (&binary_path, std::env::var_os("PATH")) {
        (Some(current), Some(path_env)) => other_kits_on_path(current, &path_env),
        (None, Some(path_env)) => other_kits_on_path(Path::new(""), &path_env),
        _ => Vec::new(),
    };
    // The npm 1.x launcher runs this same binary: not a collision.
    let collisions: Vec<(PathBuf, PathKit)> = collisions
        .into_iter()
        .map(|p| {
            let kind = classify_path_kit(&p);
            (p, kind)
        })
        .filter(|(_, kind)| *kind != PathKit::Launcher)
        .collect();
    let install = match std::env::var("KIT_LAUNCHER").as_deref() {
        Ok("npm") => "npm",
        _ => "binary",
    };
    let binary_display = binary_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(unknown)".into());
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let kit_toml = Some(cwd.join("kit.toml")).filter(|p| p.is_file());
    let path_collisions: Vec<String> = collisions
        .iter()
        .map(|(p, _)| p.display().to_string())
        .collect();

    if json {
        let agents: Vec<serde_json::Value> = statuses
            .iter()
            .map(|st| {
                serde_json::json!({
                    "agent": st.kind.label(),
                    "installed": st.installed,
                    "ready": st.is_ready(),
                    "version": st.version,
                    "remedy": st.remedy,
                })
            })
            .collect();
        let data = serde_json::json!({
            "version": version,
            "binary": "ok",
            "binaryPath": binary_display,
            "install": install,
            "pathCollisions": path_collisions,
            "controlRoom": "ok",
            "gateEngine": "ok",
            "runEngine": "ok",
            "kitHome": kit_home,
            "skillsPack": skills.as_ref().map(|p| p.display().to_string()),
            "agents": agents,
            "kitToml": kit_toml.as_ref().map(|p| p.display().to_string()),
        });
        let envelope = json_envelope("doctor", true, data, None);
        println!(
            "{}",
            serde_json::to_string_pretty(&envelope).unwrap_or_default()
        );
        return;
    }

    println!("kit doctor {version}");
    println!();
    println!("status:");
    println!("  binary          ok (rust)");
    println!("  binary path     {binary_display}");
    println!("  installed via   {install}");
    println!("  control room    ok (kit-tui)");
    println!("  gate engine     ok (kit-gate)");
    println!("  run engine      ok (worktree + adapters + receipt)");
    println!("  kit home        {}", kit_home.display());
    if let Some(s) = skills {
        println!("  skills pack     {}", s.display());
    } else {
        println!("  skills pack     missing (.agents/skills)");
    }
    match &kit_toml {
        Some(p) => println!("  kit.toml        {}", p.display()),
        None if cwd.join(".git").exists() => {
            println!("  kit.toml        missing. Run `kit init` to write a gate")
        }
        None => {}
    }
    if !collisions.is_empty() {
        println!();
        println!("warning:");
        for (p, kind) in &collisions {
            match kind {
                PathKit::Node01 => {
                    println!("  kit 0.1 (Node) is on PATH: {}", p.display());
                    println!("    → npm install -g @mzwin/kit@alpha");
                }
                _ => println!("  another kit is on PATH: {}", p.display()),
            }
        }
    }
    println!();
    println!("agents:");
    for st in statuses {
        let flag = if st.is_ready() {
            "ready"
        } else if st.installed {
            "not ready"
        } else {
            "missing"
        };
        let ver = st.version.as_deref().unwrap_or("-");
        println!("  {:8}  {flag:9}  {ver}", st.kind.label());
        if let Some(r) = st.remedy {
            println!("            → {r}");
        }
    }
    println!();
    println!("try:");
    if kit_toml.is_none() {
        println!("  kit init");
    }
    println!("  kit --demo");
    println!("  kit run --dry-run --task \"smoke\" --json");
    println!("  kit run --agent codex --task \"…\"");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Last line of the real npm 0.1.3 `kit.cmd` shim (Windows, 2026-09-23).
    const NODE01_CMD: &str = r#"endLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & set PATHEXT=%PATHEXT:;.JS;=;% & "%_prog%"  "%dp0%\node_modules\@mzwin\kit\dist\bin.js" %*"#;

    #[test]
    fn npm_shims_are_told_apart() {
        assert_eq!(classify_shim_text(NODE01_CMD), PathKit::Node01);
        let launcher_cmd = NODE01_CMD.replace(r"dist\bin.js", r"bin\kit.js");
        assert_eq!(classify_shim_text(&launcher_cmd), PathKit::Launcher);
        // Unix: the global bin is a symlink; canonicalize names the target.
        assert_eq!(
            classify_shim_text("/usr/lib/node_modules/@mzwin/kit/bin/kit.js"),
            PathKit::Launcher
        );
        assert_eq!(classify_shim_text("/usr/local/bin/kit"), PathKit::Other);
    }

    fn scratch(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "kit-doctor-path-{}-{}-{}",
            label,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    fn join_path(dirs: &[PathBuf]) -> OsString {
        std::env::join_paths(dirs).expect("join PATH")
    }

    /// Only a proven PASS with a diff points to `kit land`.
    #[test]
    fn land_hint_only_for_proven_runs_with_a_diff() {
        let dir = scratch("landhint");
        assert_eq!(land_hint(RunState::Pass, false, &dir, "01X"), None);
        fs::write(dir.join("diff.patch"), "+x\n").unwrap();
        assert_eq!(
            land_hint(RunState::Pass, false, &dir, "01X").as_deref(),
            Some("Next: kit land 01X")
        );
        assert_eq!(land_hint(RunState::Pass, true, &dir, "01X"), None);
        assert_eq!(land_hint(RunState::Fail, false, &dir, "01X"), None);
    }

    fn write_kit(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, b"kit-test-stub\n").expect("stub kit");
        path
    }

    #[test]
    fn other_kits_skips_current_and_same_file() {
        let dir = scratch("self");
        let name = if cfg!(windows) { "kit.exe" } else { "kit" };
        let current = write_kit(&dir, name);
        let hits = other_kits_on_path(&current, &join_path(&[dir]));
        assert!(hits.is_empty(), "current exe is not a collision: {hits:?}");
    }

    #[test]
    fn other_kits_reports_foreign_kit_cmd() {
        let mine = scratch("mine");
        let other = scratch("other");
        let current_name = if cfg!(windows) { "kit.exe" } else { "kit" };
        let other_name = if cfg!(windows) { "kit.cmd" } else { "kit" };
        let current = write_kit(&mine, current_name);
        let collision = write_kit(&other, other_name);
        let hits = other_kits_on_path(&current, &join_path(&[mine, other]));
        assert_eq!(hits, vec![collision]);
    }

    #[test]
    fn other_kits_empty_path_is_clean() {
        let hits = other_kits_on_path(Path::new("/no/such/kit"), OsStr::new(""));
        assert!(hits.is_empty());
    }

    #[test]
    fn other_kits_skips_hardlink_of_current() {
        let dir = scratch("hardlink");
        let other_dir = scratch("hardlink-other");
        let name = if cfg!(windows) { "kit.exe" } else { "kit" };
        let current = write_kit(&dir, name);
        let alias = other_dir.join(name);
        std::fs::hard_link(&current, &alias).expect("hardlink");
        let hits = other_kits_on_path(&current, &join_path(&[dir, other_dir]));
        assert!(
            hits.is_empty(),
            "hardlink of current is the same file: {hits:?}"
        );
    }

    #[test]
    fn delta_line_echoes_output_and_state_only() {
        assert_eq!(
            delta_line(&RunDelta::Output("kit: spawning grok\n".into())).as_deref(),
            Some("kit: spawning grok\n")
        );
        assert_eq!(
            delta_line(&RunDelta::Output("text: done".into())).as_deref(),
            Some("text: done\n")
        );
        assert_eq!(
            delta_line(&RunDelta::State(RunState::Gating)).as_deref(),
            Some("kit: state gating\n")
        );
        assert_eq!(delta_line(&RunDelta::Worktree(PathBuf::from("wt"))), None);
    }

    /// Session B: a live run is never silent. Deltas stream, silence is
    /// reported, and the echo ends once the engine drops its sender.
    #[tokio::test]
    async fn echo_deltas_streams_flags_silence_and_ends_with_sender() {
        let (tx, rx) = mpsc::channel(8);
        let id = RunId::default();
        tx.send((id.clone(), RunDelta::State(RunState::Running)))
            .await
            .unwrap();
        tx.send((id, RunDelta::Output("text: working\n".into())))
            .await
            .unwrap();
        let mut lines = Vec::new();
        let echo = echo_deltas(rx, AgentKind::Grok, Duration::from_millis(20), |line| {
            lines.push(line.to_string())
        });
        let hold_then_drop = async move {
            tokio::time::sleep(Duration::from_millis(90)).await;
            drop(tx);
        };
        tokio::join!(echo, hold_then_drop);
        assert_eq!(lines[..2], ["kit: state running\n", "text: working\n"]);
        assert!(
            lines[2..]
                .iter()
                .any(|l| l.starts_with("kit: still running (grok), no output for")),
            "silence must be reported: {lines:?}"
        );
    }
}
