//! `kit init`: propose a gate from the repo and write `kit.toml`.
//!
//! Detection is [`crate::engine::infer::detect`], the same proposal a live run
//! uses when `kit.toml` has no gate. Refuses to overwrite without `--force`.

mod check;
mod render;
#[cfg(test)]
mod tests;

use crate::engine::infer::{self, Detection};
use anyhow::{Context, Result, bail};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

const EXAMPLE: &str = "[gate]\ntest    = \"make test\"\ntimeout = \"10m\"";

struct Opts {
    repo: PathBuf,
    force: bool,
    print: bool,
    check: bool,
    json: bool,
    timeout: Duration,
}

fn parse(args: &[String]) -> Result<Opts> {
    let mut o = Opts {
        repo: PathBuf::from("."),
        force: false,
        print: false,
        check: false,
        json: false,
        timeout: Duration::from_secs(300),
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo" | "-C" => {
                i += 1;
                o.repo = args.get(i).context("--repo needs a path")?.into();
            }
            "--force" | "-f" => o.force = true,
            "--print" | "-p" => o.print = true,
            "--check" => o.check = true,
            "--json" => o.json = true,
            "--timeout" => {
                i += 1;
                let raw = args.get(i).context("--timeout needs a value, e.g. 5m")?;
                o.timeout = check::parse_duration(raw)
                    .with_context(|| format!("--timeout {raw} is not valid. Use 90s, 5m or 1h"))?;
            }
            other => bail!("unknown kit init flag: {other}. Run `kit --help`"),
        }
        i += 1;
    }
    Ok(o)
}

pub async fn cmd_init(args: &[String]) -> Result<()> {
    let o = parse(args)?;
    let repo = std::path::absolute(&o.repo)
        .map(crate::engine::worktree::strip_verbatim)
        .unwrap_or(o.repo.clone());
    if !repo.is_dir() {
        bail!(
            "no folder at {}. Give a repo path with --repo <path>",
            repo.display()
        );
    }
    let path = repo.join("kit.toml");
    let existed = path.exists();
    if existed && !o.force && !o.print {
        bail!(
            "kit.toml already exists: {}. To replace it, run `kit init --force`. To see the proposal only, run `kit init --print`",
            path.display()
        );
    }

    let det = infer::detect(&repo);
    if det.gate.is_empty() {
        bail!(no_gate_message(&repo, &det));
    }
    let warnings: Vec<String> = infer::missing_programs(&det.gate)
        .into_iter()
        .map(|p| {
            format!("{p} is not on PATH. Install it before `kit run`, or its checks do not run.")
        })
        .collect();

    // Notes go to stderr when stdout must hold only the file or the envelope.
    let quiet_stdout = o.print || o.json;
    let say = |line: &str| {
        if quiet_stdout {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    };
    let (Some(tc), Some(marker)) = (det.toolchain, det.marker.as_deref()) else {
        unreachable!("a non-empty gate has a toolchain");
    };
    say(&format!("kit init: {} project ({marker})", tc.title()));
    for s in &det.skipped {
        say(&format!("  not in the gate: {s}"));
    }
    for n in &det.notes {
        say(&format!("  note: {n}"));
    }
    for w in &warnings {
        say(&format!("  warning: {w}"));
    }

    let results = if o.check {
        say("checking each command once in the repo:");
        Some(check::run_all(&repo, &det.gate, o.timeout, |l| say(l)).await)
    } else {
        None
    };
    let (gate, left_out) = render::effective_gate(&det.gate, results.as_deref());
    if gate.is_empty() {
        bail!(
            "no proposed check passed, so kit.toml was not written. Fix the failures above, or run `kit init` without --check to write the proposal as it is"
        );
    }
    let text = render::render(&det, &gate, &left_out);

    let written = !o.print;
    if written {
        write_file(&path, &text, o.force)?;
    }

    if o.json {
        let data = serde_json::json!({
            "repo": repo,
            "path": path,
            "toolchain": tc.label(),
            "marker": marker,
            "existed": existed,
            "written": written,
            "toml": text,
            "checks": results.as_ref().map(|r| r.iter().map(|c| c.to_json()).collect::<Vec<_>>()),
            "skipped": det.skipped,
            "notes": det.notes,
        });
        let env = crate::envelope("init", true, data, None, warnings);
        println!("{}", serde_json::to_string_pretty(&env)?);
    } else if o.print {
        print!("{text}");
    } else {
        println!();
        print!("{text}");
        println!();
        let verb = if existed { "Replaced" } else { "Wrote" };
        println!("{verb} {}.", path.display());
        println!("Next: commit kit.toml, then run `kit run --task \"...\"`.");
    }
    Ok(())
}

fn write_file(path: &std::path::Path, text: &str, force: bool) -> Result<()> {
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true);
    if force {
        opts.create(true).truncate(true);
    } else {
        // create_new: never replace a kit.toml made after the first check.
        opts.create_new(true);
    }
    let mut file = opts
        .open(path)
        .with_context(|| format!("cannot write {}", path.display()))?;
    file.write_all(text.as_bytes())
        .with_context(|| format!("cannot write {}", path.display()))
}

fn no_gate_message(repo: &std::path::Path, det: &Detection) -> String {
    let head = match (det.toolchain, &det.marker) {
        (Some(tc), Some(marker)) => {
            let mut s = format!(
                "found a {} project ({marker}) in {}, but no check is safe to run",
                tc.title(),
                repo.display()
            );
            for n in &det.notes {
                s.push_str(&format!("\n  note: {n}"));
            }
            s
        }
        _ => format!(
            "no project found in {}. kit init reads Cargo.toml, go.mod, package.json, pyproject.toml, setup.cfg and requirements.txt",
            repo.display()
        ),
    };
    format!("{head}\nWrite kit.toml by hand. Example:\n\n{EXAMPLE}\n")
}
