//! `kit eval`: Kit's own eval suite (PRD §13.3).
//!
//! Each task in `evals/tasks/<id>/` is a fixture repo (`files/`), a prompt
//! (`task.toml`) and a reference solution (`solution.json`, a script of tool
//! calls). A task runs as a real `kit run --agent kit` in a fresh copy of its
//! repo, so the repo's own `kit.toml` gate is the grader: no model judges a
//! model. `--scripted` replays the reference solutions instead of calling a
//! model, which proves every task is solvable and every gate grades.

use crate::cli::EvalArgs;
use crate::engine::paths::kit_home;
use anyhow::{Context, Result, bail};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Deserialize)]
struct TaskFile {
    category: String,
    /// `pass`: the gate must pass. `denied`: the right answer is a refused call.
    expect: String,
    prompt: String,
}

/// One task's result, as the report records it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResult {
    id: String,
    category: String,
    passed: bool,
    /// The run's state: `pass`, `fail`, `budget`, `error`, …
    state: String,
    replies: u64,
    cost_usd: Option<f64>,
    input_tokens: u64,
    output_tokens: u64,
    denials: usize,
    seconds: f64,
    /// Why it failed, in one line.
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

/// Every task folder under `dir` whose id contains `filter`.
fn tasks(dir: &Path, filter: Option<&str>) -> Result<Vec<(String, PathBuf, TaskFile)>> {
    let mut out = Vec::new();
    let entries = std::fs::read_dir(dir).with_context(|| {
        format!(
            "no eval tasks at {} (run kit eval from the Kit repo, or pass --tasks)",
            dir.display()
        )
    })?;
    for e in entries.flatten() {
        let id = e.file_name().to_string_lossy().into_owned();
        if filter.is_some_and(|f| !id.contains(f)) || !e.path().join("task.toml").is_file() {
            continue;
        }
        let raw = std::fs::read_to_string(e.path().join("task.toml"))?;
        let task: TaskFile = toml::from_str(&raw).with_context(|| format!("{id}/task.toml"))?;
        out.push((id, e.path(), task));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    if out.is_empty() {
        bail!("no tasks matched in {}", dir.display());
    }
    Ok(out)
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let target = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &target)?;
        } else {
            std::fs::copy(e.path(), &target)?;
        }
    }
    Ok(())
}

fn git(repo: &Path, args: &[&str]) -> Result<()> {
    let out = std::process::Command::new("git")
        .args([
            "-c",
            "user.name=kit-eval",
            "-c",
            "user.email=eval@kit.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.autocrlf=false",
        ])
        .args(args)
        .current_dir(repo)
        .output()
        .context("git is needed for kit eval")?;
    if !out.status.success() {
        bail!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

/// A fresh repo with the task's files, committed. CRLF files stay CRLF.
fn fixture(task_dir: &Path, repo: &Path) -> Result<()> {
    copy_dir(&task_dir.join("files"), repo)?;
    git(repo, &["init", "-q", "-b", "main"])?;
    git(repo, &["config", "core.autocrlf", "false"])?;
    git(repo, &["add", "-A"])?;
    git(repo, &["commit", "-q", "-m", "fixture"])
}

/// `--baseline`: the gate must fail before any agent touches the repo, or the
/// task proves nothing. Refusal tasks have a trivial gate and are skipped.
async fn baseline_one(id: String, dir: PathBuf, task: TaskFile, base: PathBuf) -> TaskResult {
    use kit_core::Gate;
    let started = Instant::now();
    let repo = base.join(&id);
    let mut result = TaskResult {
        id: id.clone(),
        category: task.category.clone(),
        passed: false,
        state: "error".into(),
        replies: 0,
        cost_usd: None,
        input_tokens: 0,
        output_tokens: 0,
        denials: 0,
        seconds: 0.0,
        note: None,
    };
    if task.expect == "denied" {
        result.passed = true;
        result.state = "skipped".into();
        return result;
    }
    let config = fixture(&dir, &repo)
        .and_then(|()| Ok(std::fs::read_to_string(repo.join("kit.toml"))?))
        .and_then(|t| Ok(toml::from_str::<kit_core::KitConfig>(&t)?));
    let config = match config {
        Ok(c) => c,
        Err(e) => {
            result.note = Some(format!("fixture: {e:#}"));
            return result;
        }
    };
    let gate = kit_gate::KitGate::new().evaluate(&repo, &config.gate).await;
    result.seconds = started.elapsed().as_secs_f64();
    result.state = if gate.passed {
        "pass".into()
    } else {
        "fail".into()
    };
    result.passed = !gate.passed && !gate.checks.is_empty();
    if !result.passed {
        result.note = Some("the gate passes before any change: the task proves nothing".into());
    }
    result
}

async fn run_one(
    id: String,
    dir: PathBuf,
    task: TaskFile,
    base: PathBuf,
    args: &EvalArgs,
) -> TaskResult {
    if args.baseline {
        return baseline_one(id, dir, task, base).await;
    }
    let started = Instant::now();
    let mut result = TaskResult {
        id: id.clone(),
        category: task.category.clone(),
        passed: false,
        state: "error".into(),
        replies: 0,
        cost_usd: None,
        input_tokens: 0,
        output_tokens: 0,
        denials: 0,
        seconds: 0.0,
        note: None,
    };
    let repo = base.join(&id);
    if let Err(e) = fixture(&dir, &repo) {
        result.note = Some(format!("fixture: {e:#}"));
        return result;
    }
    let exe = std::env::current_exe().unwrap_or_else(|_| "kit".into());
    let mut cmd = tokio::process::Command::new(exe);
    cmd.args([
        "run",
        "--agent",
        "kit",
        "--json",
        "--task",
        task.prompt.trim(),
    ])
    .current_dir(&repo)
    .env("KIT_HOME", base.join("home"))
    .kill_on_drop(true);
    if args.scripted {
        cmd.env("KIT_PROVIDER", "scripted")
            .env("KIT_SCRIPT", dir.join("solution.json"));
    }
    for (key, value) in [
        ("KIT_MODEL", &args.model),
        ("KIT_PROVIDER", &args.provider),
        ("KIT_BASE_URL", &args.base_url),
    ] {
        if let Some(v) = value {
            cmd.env(key, v);
        }
    }
    if let Some(b) = args.budget {
        cmd.env("KIT_BUDGET_USD", b.to_string());
    }
    let out = match tokio::time::timeout(Duration::from_secs(600), cmd.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => {
            result.note = Some(format!("could not start kit run: {e}"));
            return result;
        }
        Err(_) => {
            result.note = Some("timed out after 10 minutes".into());
            return result;
        }
    };
    result.seconds = started.elapsed().as_secs_f64();
    let env: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    let data = &env["data"];
    result.state = data["state"].as_str().unwrap_or("error").to_owned();
    let agent = &data["agent"];
    result.replies = agent["replies"].as_u64().unwrap_or(0);
    result.cost_usd = agent["cost_usd"].as_f64();
    result.input_tokens = agent["tokens"]["input"].as_u64().unwrap_or(0)
        + agent["tokens"]["cached_input"].as_u64().unwrap_or(0);
    result.output_tokens = agent["tokens"]["output"].as_u64().unwrap_or(0);
    result.denials = agent["denials"].as_array().map_or(0, Vec::len);
    result.passed = match task.expect.as_str() {
        "denied" => result.denials > 0 && result.state != "error",
        _ => result.state == "pass",
    };
    if !result.passed {
        result.note = Some(match (task.expect.as_str(), env["error"].as_str()) {
            (_, Some(e)) => e.lines().next().unwrap_or(e).to_owned(),
            ("denied", _) => "nothing was refused: the agent was expected to hit a boundary".into(),
            _ => format!("the gate did not pass (state {})", result.state),
        });
    }
    result
}

/// `YYYY-MM-DD` (UTC) without a date crate.
fn today() -> String {
    let days = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    // Howard Hinnant's civil-from-days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn median(mut xs: Vec<f64>) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    xs.sort_by(|a, b| a.total_cmp(b));
    let n = xs.len();
    Some(if n % 2 == 1 {
        xs[n / 2]
    } else {
        (xs[n / 2 - 1] + xs[n / 2]) / 2.0
    })
}

/// Run the suite, print a table, write the report.
pub async fn cmd_eval(args: EvalArgs, json: bool) -> Result<()> {
    // Absolute: each task's agent runs in its own worktree, not here.
    let dir = std::fs::canonicalize(&args.tasks).with_context(|| {
        format!(
            "no eval tasks at {} (run kit eval from the Kit repo, or pass --tasks)",
            args.tasks.display()
        )
    })?;
    let list = tasks(
        &crate::engine::worktree::strip_verbatim(dir),
        args.filter.as_deref(),
    )?;
    let run_id = kit_core::RunId::new().0;
    let base = kit_home().join("evals").join(&run_id);
    std::fs::create_dir_all(base.join("home"))?;
    if !args.scripted {
        // A model run uses the user's provider settings and prices.
        for f in ["config.toml", "prices.toml"] {
            if kit_home().join(f).is_file() {
                std::fs::copy(kit_home().join(f), base.join("home").join(f))?;
            }
        }
    }
    let jobs = args.jobs.max(1);
    if !json {
        let mode = if args.baseline {
            ", baseline (no agent: every gate must fail)"
        } else if args.scripted {
            ", scripted (no model)"
        } else {
            ""
        };
        eprintln!("kit eval: {} tasks, {} at a time{mode}", list.len(), jobs);
    }
    let args_ref = &args;
    let base_ref = &base;
    let mut results: Vec<TaskResult> = futures::stream::iter(list)
        .map(|(id, dir, task)| run_one(id, dir, task, base_ref.clone(), args_ref))
        .buffer_unordered(jobs)
        .inspect(|r| {
            if !json {
                let cost = r.cost_usd.map_or("$?".into(), |c| format!("${c:.3}"));
                let mark = if r.passed { "PASS" } else { "FAIL" };
                let note = r
                    .note
                    .as_deref()
                    .map(|n| format!("  {n}"))
                    .unwrap_or_default();
                eprintln!(
                    "{mark}  {:<20} {:>2} replies  {cost:>7}  {:>5.1}s{note}",
                    r.id, r.replies, r.seconds
                );
            }
        })
        .collect()
        .await;
    results.sort_by(|a, b| a.id.cmp(&b.id));
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    let report = serde_json::json!({
        "date": today(),
        "kitVersion": env!("CARGO_PKG_VERSION"),
        "scripted": args.scripted,
        "provider": args.provider.clone().or_else(|| args.scripted.then(|| "scripted".into())),
        "model": args.model,
        "passed": passed,
        "total": total,
        "passRate": passed as f64 / total as f64,
        "medianCostUsd": median(results.iter().filter_map(|r| r.cost_usd).collect()),
        "medianReplies": median(results.iter().map(|r| r.replies as f64).collect()),
        "tasks": results,
        "runs": base.display().to_string(),
    });
    let text = serde_json::to_string_pretty(&report)?;
    std::fs::write(base.join("report.json"), &text)?;
    if let Some(out) = &args.out {
        std::fs::write(out, &text).with_context(|| format!("cannot write {}", out.display()))?;
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&crate::envelope(
                "eval",
                passed == total || !(args.scripted || args.baseline),
                report,
                None,
                vec![]
            ))?
        );
    } else {
        eprintln!(
            "\nkit eval: {passed}/{total} passed · report {}",
            crate::kits::plan::tilde(&base.join("report.json"))
        );
    }
    // Scripted and baseline runs check the suite itself: all must pass.
    if (args.scripted || args.baseline) && passed != total {
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_and_date_helpers() {
        assert_eq!(median(vec![3.0, 1.0, 2.0]), Some(2.0));
        assert_eq!(median(vec![1.0, 4.0]), Some(2.5));
        assert_eq!(median(vec![]), None);
        let d = today();
        assert_eq!((d.len(), &d[4..5], &d[7..8]), (10, "-", "-"));
        assert!(d.as_str() >= "2026-01-01");
    }
}
