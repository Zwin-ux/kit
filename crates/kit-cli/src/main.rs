//! Kit command line — product entry for the Control Room and headless runs.
//!
//! Default surface is the ratatui Control Room (PRD §4.2). `kit run` is the M1
//! headless path: worktree → dry-run stream → gate → receipt.

mod cli;
mod engine;
mod init;
mod kits;
mod land;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use cli::{Cli, Command, ReceiptAction};
use engine::{RunOptions, execute_headless, spawn_production};
use kit_core::{AgentKind, Bounds, RunDelta, RunId, RunState};
use kit_tui::{EngineCommand, LaunchConfig, run_configured};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
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
        None if !cli.demo && kits::setup::first_run() => {
            kits::setup::cmd_setup(cli::SetupArgs::default(), json).await
        }
        None => launch_tui(cli.demo).await,
        Some(Command::Setup(args)) => kits::setup::cmd_setup(args, json).await,
        Some(Command::Show { kit }) => kits::cmd_show(kit.as_deref(), json),
        Some(Command::Add(args)) => kits::install::cmd_add(args, json),
        Some(Command::Remove(args)) => kits::install::cmd_remove(args, json),
        Some(Command::List(args)) => kits::install::cmd_list(args, json),
        Some(Command::Search(args)) => kits::search::cmd_search(&args, json),
        Some(Command::Sync(args)) => kits::sync::cmd_sync(args, json),
        Some(Command::New(args)) => kits::new::cmd_new(&args, json),
        Some(Command::Hook {
            event: cli::HookCommand::AfterEdit { kit, scope },
        }) => std::process::exit(kits::hook::after_edit(&kit, scope)?),
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
            runs_dir: Some(engine::paths::runs_dir()),
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
    // Not a git repo: nothing can run, so say it once and write no receipt.
    let root = engine::worktree::resolve_repo(&repo)?;
    let allow_vacuous = args.allow_vacuous;
    // None = live with the chosen agent. --dry-run runs no agent.
    let dry_run = args.dry_run.then_some(true);
    let agent = args.agent.map(AgentKind::from);
    let kind = match agent {
        Some(kind) => kind,
        // A dry run starts no agent, so there is nothing to probe.
        None if args.dry_run => cli::AgentArg::PREFERENCE[0],
        None => {
            let mut statuses = Vec::new();
            for kind in cli::AgentArg::PREFERENCE {
                let status = kit_agents::adapter(kind).probe().await;
                let ready = status.is_ready();
                statuses.push(status);
                if ready {
                    break;
                }
            }
            let kind = pick_agent(&statuses)?;
            eprintln!("kit: using {kind}, the first ready agent. Choose with --agent");
            kind
        }
    };
    // Stderr only: under --json, stdout holds one envelope.
    if let Some(hint) = init_hint(&root) {
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
    let outcome = execute_headless(opts, Some(delta_tx)).await;
    let _ = echo.await;
    // A run that failed still has its receipt; the error rides along.
    // The error already went out in the stream as `kit: run failed: …`.
    let (result, run_error) = outcome?;

    let gate_vacuous = result
        .gate
        .as_ref()
        .map(engine::infer::is_vacuous)
        .unwrap_or(false);
    // Dry-run has no proof claim (CEO stamp); live vacuous fails unless allowed.
    let dry = dry_run == Some(true);

    let code = match result.state {
        RunState::Pass => 0,
        RunState::Unconfigured if allow_vacuous || dry => 0,
        RunState::Fail | RunState::Unconfigured => 1,
        _ => 2,
    };
    let exit_nonzero = code != 0;

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
        let envelope = json_envelope("run", ok, data, run_error);
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        let has_diff = result.receipt_dir.join("diff.patch").is_file();
        let summary = RunSummary {
            state: result.state,
            gate: result.gate.as_ref(),
            vacuous: gate_vacuous,
            dry,
            id: &result.id.0,
            receipt_dir: &result.receipt_dir,
            worktree: result.worktree.as_deref(),
            has_diff,
        };
        for line in summary.lines() {
            println!("{line}");
        }
    }

    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

/// The first ready agent in preference order, or an error naming the fix.
fn pick_agent(statuses: &[kit_agents::AgentStatus]) -> Result<AgentKind> {
    if let Some(ready) = statuses.iter().find(|s| s.is_ready()) {
        return Ok(ready.kind);
    }
    if let Some(unready) = statuses.iter().find(|s| s.installed) {
        let fix = unready
            .remedy
            .as_deref()
            .unwrap_or("run it once by hand to finish its setup");
        anyhow::bail!(
            "{} is installed but not ready ({fix}). Fix that and run again, or see `kit doctor`",
            unready.kind
        );
    }
    anyhow::bail!(
        "no coding agent found (looked for claude, codex, grok, ollama). Install Claude Code or Codex, then check with `kit doctor`. To try the pipeline without one: kit run --dry-run \"…\""
    )
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
        RunDelta::State(RunState::Running) => Some("kit: agent running in its worktree\n".into()),
        RunDelta::State(RunState::Gating) => {
            Some("kit: agent done. Running the checks in kit.toml\n".into())
        }
        // Queued is instant; the end state is the verdict line on stdout.
        RunDelta::State(_) => None,
        RunDelta::Worktree(_) | RunDelta::Gate(_) => None,
    }
}

/// Short form of a run id for people: 12 characters, the same prefix
/// `kit land` names its branch with. The first 10 characters of a ULID are
/// its timestamp, so fewer would collide for runs dispatched together.
fn short_id(id: &str) -> &str {
    id.get(..12).unwrap_or(id)
}

/// What `kit run` prints when a run ends: verdict, checks, where, next step.
struct RunSummary<'a> {
    state: RunState,
    gate: Option<&'a kit_core::GateOutcome>,
    vacuous: bool,
    dry: bool,
    id: &'a str,
    receipt_dir: &'a Path,
    worktree: Option<&'a Path>,
    has_diff: bool,
}

impl RunSummary<'_> {
    fn verdict(&self) -> &'static str {
        match self.state {
            RunState::Pass | RunState::Unconfigured if self.dry => "DRY RUN",
            RunState::Unconfigured => "UNCONFIGURED",
            RunState::Pass if self.vacuous => "UNCONFIGURED",
            RunState::Pass => "PASS",
            RunState::Fail => "FAIL",
            RunState::Killed => "KILLED",
            _ => "ERROR",
        }
    }

    /// The one command to run next.
    fn next(&self) -> String {
        let id = short_id(self.id);
        match self.state {
            RunState::Pass | RunState::Unconfigured if self.dry => {
                "drop --dry-run to run an agent. A dry run proves nothing".into()
            }
            RunState::Unconfigured => "kit init, so the next run has checks to pass".into(),
            RunState::Pass if self.vacuous => "kit init, so the next run has checks to pass".into(),
            RunState::Pass if self.has_diff => format!("kit land {id}"),
            RunState::Pass => "nothing to land: the agent changed no files".into(),
            _ => format!("kit receipt show {id} --output"),
        }
    }

    fn lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut head = self.verdict().to_string();
        if let Some(g) = self.gate.filter(|_| !self.vacuous) {
            head.push(' ');
            for c in &g.checks {
                let mark = match c.status {
                    kit_core::CheckStatus::Pass => "✓",
                    kit_core::CheckStatus::Skipped => "-",
                    kit_core::CheckStatus::Fail | kit_core::CheckStatus::TimedOut => "✗",
                };
                head.push_str(&format!(" {} {mark}", c.label));
            }
        }
        out.push(head);
        if let Some(g) = self.gate {
            if let Some(c) = g.first_failure() {
                let why = match c.status {
                    kit_core::CheckStatus::TimedOut => "timed out".to_string(),
                    _ => c
                        .summary
                        .as_deref()
                        .and_then(|s| s.lines().next())
                        .unwrap_or("failed")
                        .to_string(),
                };
                out.push(format!("  {}: {why}", c.label));
            }
            if let Some(first) = g.scope_violations.first() {
                out.push(format!(
                    "  scope: {} file(s) outside [gate.scope], first {first}",
                    g.scope_violations.len()
                ));
            }
        }
        out.push(String::new());
        out.push(format!(
            "run       {}  (receipt {})",
            short_id(self.id),
            kits::plan::tilde(self.receipt_dir)
        ));
        if let Some(wt) = self.worktree {
            out.push(format!(
                "worktree  {} (kept: it has the run's changes)",
                kits::plan::tilde(wt)
            ));
        }
        out.push(format!("next      {}", self.next()));
        out
    }
}

/// How long a run took, from its receipt: `850ms`, `12.3s`, `4m 05s`, `1h 02m`.
fn run_duration(start: Option<SystemTime>, end: Option<SystemTime>) -> Option<String> {
    let d = end?.duration_since(start?).ok()?;
    let secs = d.as_secs();
    Some(if secs == 0 {
        format!("{}ms", d.as_millis())
    } else if secs < 60 {
        format!("{:.1}s", d.as_secs_f64())
    } else if secs < 3600 {
        format!("{}m {:02}s", secs / 60, secs % 60)
    } else {
        format!("{}h {:02}m", secs / 3600, secs % 3600 / 60)
    })
}

/// `next  kit land <id>` for a proven run with changes (receipt show).
fn land_hint(state: RunState, vacuous: bool, receipt_dir: &Path, id: &str) -> Option<String> {
    (state == RunState::Pass && !vacuous && receipt_dir.join("diff.patch").is_file())
        .then(|| format!("next      kit land {}", short_id(id)))
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
            "no runs yet under {}. Start one with kit run \"describe a task\"",
            engine::paths::runs_dir().display()
        );
    } else {
        println!(
            "{:<28} {:<12} {:<8} {:<12} TASK",
            "ID", "STATE", "AGENT", "REPO"
        );
        for r in &rows {
            let short = if r.id.len() > 26 {
                format!("{}…", &r.id[..25])
            } else {
                r.id.clone()
            };
            println!(
                "{:<28} {:<12} {:<8} {:<12} {}",
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
        println!("  dir       {}", kits::plan::tilde(&dir));
        println!("  state     {}", state_label(receipt.state));
        if let Some(took) = run_duration(receipt.started_at, receipt.ended_at) {
            println!("  took      {took}");
        }
        println!("  agent     {}", receipt.spec.agent.label());
        println!("  repo      {}", kits::plan::tilde(&receipt.spec.repo));
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
            let n = g.checks.len();
            println!(
                "  gate      {label}  ({n} check{})",
                if n == 1 { "" } else { "s" }
            );
            let width = g.checks.iter().map(|c| c.label.len()).max().unwrap_or(0);
            for c in &g.checks {
                let status = format!("{:?}", c.status).to_ascii_uppercase();
                let row = format!("            {status:4}  {:width$}  {}", c.label, c.command);
                println!("{}", row.trim_end());
                if let Some(first) = c.summary.as_deref().filter(|s| !s.is_empty()) {
                    println!("                  {first}");
                }
            }
        } else {
            println!("  gate      (none)");
        }
        if !receipt.diff.is_empty() {
            println!(
                "  diff      {} bytes (see {})",
                receipt.diff.len(),
                kits::plan::tilde(&dir.join("diff.patch"))
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
            println!("tip       kit receipt show {} --output", receipt.id);
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
    let kits = kits::doctor::check_installed().unwrap_or_else(|e| {
        eprintln!("kit: cannot check installed kits: {e:#}");
        Vec::new()
    });
    let kits_ok = kits.iter().all(kits::doctor::KitReport::ok);

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
            "agents": agents,
            "kitToml": kit_toml.as_ref().map(|p| p.display().to_string()),
            "kits": kits::doctor::to_json(&kits),
        });
        let envelope = json_envelope(
            "doctor",
            kits_ok,
            data,
            (!kits_ok).then(|| "an installed kit failed a check".to_string()),
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&envelope).unwrap_or_default()
        );
        if !kits_ok {
            std::process::exit(1);
        }
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
    println!("  kit home        {}", kits::plan::tilde(&kit_home));
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
                    println!("    → npm install -g @mzwin/kit");
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
    kits::doctor::print(&kits);
    println!();
    println!("try:");
    if kits.is_empty() {
        println!("  kit setup");
    }
    if kit_toml.is_none() {
        println!("  kit init");
    }
    println!("  kit run \"describe a task\"");
    println!("  kit --demo");
    if !kits_ok {
        std::process::exit(1);
    }
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
    fn run_duration_reads_like_the_gate_log() {
        let t = SystemTime::UNIX_EPOCH;
        let took = |ms: u64| run_duration(Some(t), Some(t + Duration::from_millis(ms)));
        assert_eq!(took(850).as_deref(), Some("850ms"));
        assert_eq!(took(12_300).as_deref(), Some("12.3s"));
        assert_eq!(took(245_000).as_deref(), Some("4m 05s"));
        assert_eq!(took(3_720_000).as_deref(), Some("1h 02m"));
        assert_eq!(run_duration(Some(t), None), None);
        // A clock that went backwards says nothing rather than something wrong.
        assert_eq!(
            run_duration(Some(t + Duration::from_secs(5)), Some(t)),
            None
        );
    }

    #[test]
    fn land_hint_only_for_proven_runs_with_a_diff() {
        let dir = scratch("landhint");
        let id = "01M06A2PXBBH43ZFF3GJ9VQW94";
        assert_eq!(land_hint(RunState::Pass, false, &dir, id), None);
        fs::write(dir.join("diff.patch"), "+x\n").unwrap();
        assert_eq!(
            land_hint(RunState::Pass, false, &dir, id).as_deref(),
            Some("next      kit land 01M06A2PXBBH")
        );
        assert_eq!(land_hint(RunState::Pass, true, &dir, id), None);
        assert_eq!(land_hint(RunState::Fail, false, &dir, id), None);
    }

    fn check(label: &str, status: kit_core::CheckStatus, summary: &str) -> kit_core::GateCheck {
        kit_core::GateCheck {
            label: label.into(),
            command: String::new(),
            status,
            exit_code: None,
            summary: Some(summary.into()),
            duration: Duration::ZERO,
        }
    }

    /// Every ending names one next step, and FAIL shows the first failure.
    #[test]
    fn run_summary_always_ends_with_a_next_step() {
        use kit_core::CheckStatus::{Fail, Pass};
        let gate = kit_core::GateOutcome {
            passed: false,
            checks: vec![
                check("format", Fail, "Diff in src/main.rs:1\nmore"),
                check("test", Pass, ""),
            ],
            ..kit_core::GateOutcome::vacuous()
        };
        let dir = PathBuf::from("/r");
        let fail = RunSummary {
            state: RunState::Fail,
            gate: Some(&gate),
            vacuous: false,
            dry: false,
            id: "01M06A2PXBBH43ZFF3GJ9VQW94",
            receipt_dir: &dir,
            worktree: None,
            has_diff: true,
        };
        let lines = fail.lines();
        assert_eq!(lines[0], "FAIL  format ✗ test ✓");
        assert_eq!(lines[1], "  format: Diff in src/main.rs:1");
        assert_eq!(
            lines.last().unwrap(),
            "next      kit receipt show 01M06A2PXBBH --output"
        );

        let pass = RunSummary {
            state: RunState::Pass,
            ..fail
        };
        assert_eq!(pass.next(), "kit land 01M06A2PXBBH");
        let empty = RunSummary {
            has_diff: false,
            ..pass
        };
        assert!(empty.next().starts_with("nothing to land"));
        let vacuous = RunSummary {
            vacuous: true,
            ..pass
        };
        assert_eq!(vacuous.verdict(), "UNCONFIGURED");
        assert!(vacuous.next().starts_with("kit init"));
        let dry = RunSummary { dry: true, ..pass };
        assert_eq!(dry.verdict(), "DRY RUN");
    }

    fn write_kit(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, b"kit-test-stub\n").expect("stub kit");
        path
    }

    #[test]
    fn default_agent_is_the_first_ready_one() {
        use kit_agents::AgentStatus;
        let ready = |kind| AgentStatus {
            installed: true,
            authenticated: true,
            ..AgentStatus::missing(kind)
        };
        let statuses = [
            AgentStatus::missing(AgentKind::Claude),
            ready(AgentKind::Codex),
        ];
        assert_eq!(pick_agent(&statuses).unwrap(), AgentKind::Codex);

        let unready = AgentStatus {
            installed: true,
            remedy: Some("run `claude` once to log in".into()),
            ..AgentStatus::missing(AgentKind::Claude)
        };
        let err = pick_agent(&[unready]).unwrap_err().to_string();
        assert!(
            err.starts_with("claude is installed but not ready (run `claude`"),
            "{err}"
        );

        let err = pick_agent(&[AgentStatus::missing(AgentKind::Ollama)])
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("no coding agent found") && err.contains("--dry-run"),
            "{err}"
        );
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
            Some("kit: agent done. Running the checks in kit.toml\n")
        );
        assert_eq!(delta_line(&RunDelta::State(RunState::Pass)), None);
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
        assert_eq!(
            lines[..2],
            ["kit: agent running in its worktree\n", "text: working\n"]
        );
        assert!(
            lines[2..]
                .iter()
                .any(|l| l.starts_with("kit: still running (grok), no output for")),
            "silence must be reported: {lines:?}"
        );
    }
}
