//! Commit message and text output for `kit land`.

use super::Landed;
use crate::engine::infer;
use kit_core::{CheckStatus, Receipt};
use std::path::Path;

/// Longest subject line, in characters.
const SUBJECT_MAX: usize = 72;

/// Subject = first line of the task (max 72 chars). Body = receipt id,
/// agent, gate checks. Trailer `Kit-Receipt: <id>` marks the commit, so a
/// second land can find it.
pub fn commit_message(r: &Receipt, forced: bool) -> String {
    let id = &r.id.0;
    let first = r.spec.task.lines().map(str::trim).find(|l| !l.is_empty());
    let subject = match first {
        Some(line) => truncate(line, SUBJECT_MAX),
        None => format!("Kit run {id}"),
    };
    let mut body = vec![
        format!("Kit run {id}"),
        format!("Agent: {}", r.spec.agent.label()),
        format!("Gate: {}", gate_summary(r)),
    ];
    if forced {
        body.push("Landed with --force: the gate did not prove this run.".into());
    }
    format!("{subject}\n\n{}\n\nKit-Receipt: {id}\n", body.join("\n"))
}

/// `PASS (format pass, test pass)`; `UNCONFIGURED`; `FAIL (test fail)`.
pub fn gate_summary(r: &Receipt) -> String {
    let Some(g) = &r.gate else {
        return "none".into();
    };
    if infer::is_vacuous(g) {
        return "UNCONFIGURED (no checks)".into();
    }
    let verdict = if g.passed { "PASS" } else { "FAIL" };
    let checks: Vec<String> = g
        .checks
        .iter()
        .map(|c| {
            let s = match c.status {
                CheckStatus::Pass => "pass",
                CheckStatus::Fail => "fail",
                CheckStatus::Skipped => "skipped",
                CheckStatus::TimedOut => "timed out",
            };
            format!("{} {s}", c.label)
        })
        .collect();
    if checks.is_empty() {
        verdict.into()
    } else {
        format!("{verdict} ({})", checks.join(", "))
    }
}

fn truncate(line: &str, max: usize) -> String {
    if line.chars().count() <= max {
        return line.to_string();
    }
    let mut out: String = line.chars().take(max - 3).collect();
    out.push_str("...");
    out
}

fn short(sha: &str) -> &str {
    sha.get(..10).unwrap_or(sha)
}

pub fn print_text(r: &Receipt, repo: &Path, l: &Landed, worktree_removed: bool, next: &str) {
    let id = &r.id.0;
    match (l.mode, l.already, &l.branch) {
        ("branch", true, Some(b)) => println!("Run {id} is already landed on {b}."),
        ("branch", false, Some(b)) => println!("Landed run {id} on branch {b}."),
        (_, true, _) => println!("Run {id} is already applied to {}.", repo.display()),
        _ => println!(
            "Applied run {id} to the working tree of {}. No commit.",
            repo.display()
        ),
    }
    if let Some(c) = &l.commit {
        println!("  commit    {}", short(c));
    }
    if let Some(b) = &l.base {
        println!("  base      {} ({})", short(b), l.base_source);
    } else if l.three_way {
        println!("  base      HEAD (3-way)");
    }
    if !l.files.is_empty() {
        println!("  files     {}", l.files.len());
    }
    println!("  gate      {}", gate_summary(r));
    if worktree_removed {
        println!("  worktree  removed");
    }
    println!();
    println!("Next: {next}");
    if let (Some(b), false) = (&l.branch, l.already) {
        println!("  or:  git switch {b}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_is_first_task_line_cut_at_72() {
        assert_eq!(truncate("short", 72), "short");
        let long = "x".repeat(100);
        let cut = truncate(&long, SUBJECT_MAX);
        assert_eq!(cut.chars().count(), SUBJECT_MAX);
        assert!(cut.ends_with("..."));
    }
}
