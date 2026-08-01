//! Kit command line.
//!
//! M0 SCAFFOLD. The headless surface and its JSON contract are ported from the
//! TypeScript CLI, whose E2E harness (`packages/cli/tests/e2e/`) is the
//! acceptance oracle.

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--version" | "-V") => println!("kit {version}"),
        _ => {
            println!("kit {version} — control room for parallel agent work");
            println!();
            println!("M0 scaffold. See docs/dev/PRD-1.0.md for the 1.0 surface.");
        }
    }
}
