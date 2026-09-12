//! Kit command line — product entry for the Control Room and headless runs.
//!
//! Default surface is the ratatui Control Room (PRD §4.2). `kit run` is the M1
//! headless path: worktree → dry-run stream → gate → receipt.

mod engine;

use anyhow::{Context, Result};
use engine::{RunOptions, execute, parse_agent, spawn_production};
use kit_core::{Bounds, RunDelta, RunId, RunState};
use kit_tui::{EngineCommand, LaunchConfig, run_configured};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("kit {version}");
        return Ok(());
    }

    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_help(version);
        return Ok(());
    }

    let first = args.first().map(String::as_str);
    if is_tui_invocation(first) {
        let demo =
            wants_demo(&args) || first == Some("demo") || std::env::var_os("KIT_DEMO").is_some();
        return launch_tui(demo).await;
    }

    match first {
        Some("run") => cmd_run(&args[1..]).await,
        Some("doctor") => {
            let json = args.iter().any(|a| a == "--json");
            print_doctor(version, json);
            Ok(())
        }
        Some("receipt") | Some("receipts") => cmd_receipt(&args[1..]),
        Some("version") => {
            println!("kit {version}");
            Ok(())
        }
        Some(other) => {
            eprintln!("unknown command: {other}");
            eprintln!();
            print_help(version);
            std::process::exit(2);
        }
        None => unreachable!("empty argv is a TUI launch"),
    }
}

fn wants_demo(args: &[String]) -> bool {
    args.iter().any(|a| a == "--demo" || a == "-d")
}

/// `kit`, `kit --demo`, `kit -d`, `kit demo`, and `kit tui` all open the Control Room.
fn is_tui_invocation(first: Option<&str>) -> bool {
    matches!(
        first,
        None | Some("tui")
            | Some("ui")
            | Some("control-room")
            | Some("demo")
            | Some("--demo")
            | Some("-d")
    )
}

async fn launch_tui(demo: bool) -> Result<()> {
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

async fn cmd_run(args: &[String]) -> Result<()> {
    let mut repo = ".".to_string();
    let mut agent = "codex".to_string();
    let mut task = String::new();
    // None = auto (live if installed). --dry-run forces offline.
    let mut dry_run: Option<bool> = None;
    let mut json = false;
    let mut allow_vacuous = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo" | "-C" => {
                i += 1;
                repo = args.get(i).context("--repo needs a path")?.clone();
            }
            "--agent" | "-a" => {
                i += 1;
                agent = args.get(i).context("--agent needs a name")?.clone();
            }
            "--task" | "-t" => {
                i += 1;
                task = args.get(i).context("--task needs text")?.clone();
            }
            "--dry-run" => dry_run = Some(true),
            "--live" | "--no-dry-run" => dry_run = Some(false),
            "--json" => json = true,
            "--allow-vacuous" => allow_vacuous = true,
            other if !other.starts_with('-') && task.is_empty() => {
                // Positional task fallback: kit run "do the thing"
                task = other.to_string();
            }
            other => anyhow::bail!("unknown kit run flag: {other}"),
        }
        i += 1;
    }

    if task.trim().is_empty() {
        anyhow::bail!("missing task — use --task \"…\" or a positional string");
    }

    let opts = RunOptions {
        repo,
        agent: parse_agent(&agent)?,
        task,
        dry_run,
        bounds: Bounds::default(),
    };

    let result = execute(opts, None, None).await?;

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
        println!("  state     {:?}", result.state);
        println!("  receipt   {}", result.receipt_dir.display());
        if let Some(wt) = &result.worktree {
            println!("  worktree  {} (kept — dirty)", wt.display());
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

/// CEO stamp P4 — thin JSON envelope (`schemaVersion: 1`, camelCase).
fn json_envelope(
    command: &str,
    ok: bool,
    data: serde_json::Value,
    error: Option<String>,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": 1,
        "command": command,
        "ok": ok,
        "data": data,
        "error": error,
        "warnings": [],
    })
}

fn print_help(version: &str) {
    println!("kit {version} — control room for parallel agent work");
    println!();
    println!("Usage:");
    println!("  kit                      Open the Control Room");
    println!("  kit --demo               Control Room with PRD fixture data");
    println!("  kit run --task \"…\"       One isolated run (live agent if installed)");
    println!("  kit run --agent codex --task \"…\" [--dry-run] [--json]");
    println!("  kit doctor [--json]      Environment / readiness");
    println!("  kit receipt list [--limit N] [--json]");
    println!("  kit receipt show <id> [--json] [--output]");
    println!("  kit --version            Print version");
    println!();
    println!("Run flags:");
    println!("  --repo / -C <path>       Target git repo (default .)");
    println!("  --agent / -a <name>      codex|claude|grok|ollama");
    println!("  --task / -t <text>       Prompt / task");
    println!("  --dry-run                Offline stream (no external CLI)");
    println!("  --allow-vacuous          Exit 0 even when gate is UNCONFIGURED");
    println!("  --live                   Force live agent (error → dry-run if missing)");
    println!("  --json                   Machine-readable result");
    println!("  KIT_HOME=…               Data root (default ~/.kit)");
    println!("  KIT_FULL_AUTO=1          Bypass agent approval prompts (dangerous)");
    println!("  KIT_SKILLS_DIR=…         Override skills pack path");
    println!();
    println!("Keys (Control Room):");
    println!("  ↑↓ select   f filter   Enter open   g gate   d dispatch   b board");
    println!("  k kill      r retry (fail only)   ? help   q quit");
    println!();
    println!("Docs: docs/dev/PRD-1.0.md  ·  docs/dev/CURRENT.md  ·  docs/json-contract.md");
}

/// `kit receipt list|show …` — proof browser for `~/.kit/runs/<id>/`.
fn cmd_receipt(args: &[String]) -> Result<()> {
    let sub = args.first().map(String::as_str).unwrap_or("list");
    match sub {
        "list" | "ls" => {
            let mut limit = 50usize;
            let mut json = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--json" => json = true,
                    "--limit" | "-n" => {
                        i += 1;
                        limit = args
                            .get(i)
                            .context("--limit needs a number")?
                            .parse()
                            .context("--limit must be an integer")?;
                    }
                    other => anyhow::bail!("unknown kit receipt list flag: {other}"),
                }
                i += 1;
            }
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
        "show" | "get" => {
            let id = args
                .get(1)
                .context("usage: kit receipt show <id-or-prefix>")?;
            let mut json = false;
            let mut show_output = false;
            for a in &args[2..] {
                match a.as_str() {
                    "--json" => json = true,
                    "--output" | "-o" => show_output = true,
                    other => anyhow::bail!("unknown kit receipt show flag: {other}"),
                }
            }
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
                println!("  state     {:?}", receipt.state);
                println!("  agent     {}", receipt.spec.agent.label());
                println!("  repo      {}", receipt.spec.repo.display());
                println!(
                    "  task      {}",
                    receipt.spec.task.lines().next().unwrap_or("")
                );
                if let Some(g) = &receipt.gate {
                    let label = if g.passed { "PASS" } else { "FAIL" };
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
            }
            Ok(())
        }
        "help" | "--help" | "-h" => {
            println!("kit receipt — browse proof under ~/.kit/runs/");
            println!();
            println!("  kit receipt list [--limit N] [--json]");
            println!("  kit receipt show <id-or-prefix> [--json] [--output]");
            Ok(())
        }
        other => {
            anyhow::bail!("unknown kit receipt subcommand: {other} (try list|show)");
        }
    }
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
    let binary_display = binary_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(unknown)".into());
    let path_collisions: Vec<String> = collisions.iter().map(|p| p.display().to_string()).collect();

    if json {
        let agents: Vec<serde_json::Value> = statuses
            .iter()
            .map(|st| {
                serde_json::json!({
                    "agent": st.kind.label(),
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
            "pathCollisions": path_collisions,
            "controlRoom": "ok",
            "gateEngine": "ok",
            "runEngine": "ok",
            "kitHome": kit_home,
            "skillsPack": skills.as_ref().map(|p| p.display().to_string()),
            "agents": agents,
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
    println!("  control room    ok (kit-tui)");
    println!("  gate engine     ok (kit-gate)");
    println!("  run engine      ok (worktree + adapters + receipt)");
    println!("  kit home        {}", kit_home.display());
    if let Some(s) = skills {
        println!("  skills pack     {}", s.display());
    } else {
        println!("  skills pack     missing (.agents/skills)");
    }
    if !collisions.is_empty() {
        println!();
        println!("warning:");
        println!("  another kit on PATH (likely npm 0.1) — this binary is the Rust control room");
        for p in &collisions {
            println!("    {}", p.display());
        }
    }
    println!();
    println!("agents:");
    for st in statuses {
        let flag = if st.is_ready() { "ready" } else { "missing" };
        let ver = st.version.as_deref().unwrap_or("-");
        println!("  {:8}  {flag:8}  {ver}", st.kind.label());
        if let Some(r) = st.remedy {
            println!("            → {r}");
        }
    }
    println!();
    println!("try:");
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

    #[test]
    fn demo_flag_is_tui_not_unknown_command() {
        assert!(is_tui_invocation(None));
        assert!(is_tui_invocation(Some("--demo")));
        assert!(is_tui_invocation(Some("-d")));
        assert!(is_tui_invocation(Some("demo")));
        assert!(is_tui_invocation(Some("tui")));
        assert!(!is_tui_invocation(Some("run")));
        assert!(!is_tui_invocation(Some("unify")));
        assert!(wants_demo(&["--demo".into()]));
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
}
