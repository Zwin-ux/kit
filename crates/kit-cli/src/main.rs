//! Kit command line — product entry for the Control Room.
//!
//! Default surface is the ratatui Control Room (PRD §4.2). Headless JSON parity
//! with the TypeScript CLI lands as Codex B4; until then this binary is the
//! Rust 1.0 front door.

use anyhow::Result;
use kit_core::{RunDelta, RunId};
use kit_tui::{LaunchConfig, run_configured};
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

    match args.first().map(String::as_str) {
        None | Some("tui") | Some("ui") | Some("control-room") => {
            let demo = wants_demo(&args) || std::env::var_os("KIT_DEMO").is_some();
            launch_tui(demo).await
        }
        Some("demo") => launch_tui(true).await,
        Some("doctor") => {
            print_doctor(version);
            Ok(())
        }
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
    }
}

fn wants_demo(args: &[String]) -> bool {
    args.iter().any(|a| a == "--demo" || a == "-d")
}

async fn launch_tui(demo: bool) -> Result<()> {
    // Engine channel: closed sender means no live runs yet (M1). The TUI loop
    // disables the run arm when the channel ends so idle CPU stays near zero.
    let (tx, rx) = mpsc::channel::<(RunId, RunDelta)>(64);
    drop(tx);

    run_configured(LaunchConfig { demo }, rx).await
}

fn print_help(version: &str) {
    println!("kit {version} — control room for parallel agent work");
    println!();
    println!("Usage:");
    println!("  kit                 Open the Control Room (empty)");
    println!("  kit --demo          Open with PRD fixture runs (dogfood without M1)");
    println!("  kit demo            Same as --demo");
    println!("  kit tui [--demo]    Explicit TUI entry");
    println!("  kit doctor          Environment / readiness (scaffold)");
    println!("  kit --version       Print version");
    println!("  kit --help          This help");
    println!();
    println!("Keys (Control Room):");
    println!("  ↑↓ select   Enter open   g gate   d dispatch   b board");
    println!("  k kill*     r retry*     q quit");
    println!("  * engine seams — flash until M1");
    println!();
    println!("Docs: docs/dev/PRD-1.0.md  ·  docs/dev/CURRENT.md");
}

fn print_doctor(version: &str) {
    println!("kit doctor {version}");
    println!();
    println!("status:");
    println!("  binary          ok (rust)");
    println!("  control room    ok (kit-tui)");
    println!("  gate engine     ok (kit-gate ported)");
    println!("  run engine      missing — M1 not built (dispatch queues only)");
    println!("  agent adapters  stub — kit-agents trait frozen, impls pending");
    println!();
    println!("next:");
    println!("  cargo run -p kit-cli -- --demo");
    println!("  see docs/dev/CURRENT.md");
}
