//! One-run executor: worktree → agent stream → gate → receipt.
//!
//! Production shape for M1. Agent body is dry-run until real adapters land;
//! gate and receipt are always real. Never reports PASS without a gate outcome.

use super::paths::worktrees_dir;
use super::store::{ensure_layout, load_kit_config, write_receipt};
use super::worktree::{self, branch_name, create_worktree, remove_if_clean, resolve_repo};
use anyhow::{Context, Result};
use kit_core::{AgentKind, Bounds, Gate, GateOutcome, Receipt, RunDelta, RunId, RunSpec, RunState};
use kit_gate::KitGate;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Options for a single run (headless or Control Room).
#[derive(Debug, Clone)]
pub struct RunOptions {
    pub repo: String,
    pub agent: AgentKind,
    pub task: String,
    /// When true (default for M1), do not invoke external CLIs — stream a
    /// bounded dry-run transcript. Real adapters flip this off per agent probe.
    pub dry_run: bool,
    pub bounds: Bounds,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            repo: ".".into(),
            agent: AgentKind::Codex,
            task: String::new(),
            dry_run: true,
            bounds: Bounds::default(),
        }
    }
}

/// Result returned to CLI printers / tests.
#[derive(Debug)]
pub struct RunResult {
    pub id: RunId,
    pub state: RunState,
    pub receipt_dir: PathBuf,
    pub worktree: Option<PathBuf>,
    pub worktree_removed: bool,
    pub gate: Option<GateOutcome>,
}

/// Execute one run, streaming deltas to `tx` when provided.
pub async fn execute(
    opts: RunOptions,
    id: Option<RunId>,
    tx: Option<mpsc::Sender<(RunId, RunDelta)>>,
) -> Result<RunResult> {
    ensure_layout()?;
    let id = id.unwrap_or_default();
    let repo = resolve_repo(&opts.repo)?;
    let started_at = SystemTime::now();

    send(&tx, &id, RunDelta::State(RunState::Running)).await;

    let wt_path = worktrees_dir().join(&id.0);
    let branch = branch_name(&id.0);
    create_worktree(&repo, &wt_path, &branch)
        .with_context(|| format!("create worktree at {}", wt_path.display()))?;
    send(&tx, &id, RunDelta::Worktree(wt_path.clone())).await;

    let mut output = String::new();
    let mut truncated = false;

    // --- agent phase ---
    let agent_ok = if opts.dry_run {
        dry_run_agent(&opts, &id, &wt_path, &tx, &mut output, &mut truncated).await?
    } else {
        // Real adapters land as B2; fail closed to dry-run until then.
        append_capped(
            &mut output,
            &mut truncated,
            opts.bounds.output_cap_bytes,
            &format!(
                "kit: real agent adapters not implemented yet; refusing non-dry run for {}\n",
                opts.agent
            ),
        );
        send(
            &tx,
            &id,
            RunDelta::Output(format!(
                "kit: use --dry-run (default) until {} adapter lands\n",
                opts.agent
            )),
        )
        .await;
        false
    };

    // --- gate phase ---
    send(&tx, &id, RunDelta::State(RunState::Gating)).await;
    let config = load_kit_config(&repo);
    let gate_engine = KitGate::new();
    let gate = if config.gate.is_empty() {
        // Vacuous is honest: no proof claimed. Still attach so receipt is complete.
        let line = "gate: no checks configured in kit.toml (vacuous — not a substitute for CI)\n";
        append_capped(
            &mut output,
            &mut truncated,
            opts.bounds.output_cap_bytes,
            line,
        );
        send(&tx, &id, RunDelta::Output(line.into())).await;
        GateOutcome::vacuous()
    } else {
        gate_engine.evaluate(&wt_path, &config.gate).await
    };
    send(&tx, &id, RunDelta::Gate(gate.clone())).await;

    let state = if !agent_ok {
        RunState::Error
    } else if gate.passed {
        RunState::Pass
    } else {
        RunState::Fail
    };
    send(&tx, &id, RunDelta::State(state)).await;

    let ended_at = SystemTime::now();
    let diff = worktree::worktree_diff(&wt_path).unwrap_or_default();

    let receipt = Receipt {
        version: Receipt::VERSION,
        id: id.clone(),
        spec: RunSpec {
            repo: repo.clone(),
            agent: opts.agent,
            task: opts.task.clone(),
            branch: Some(branch),
            bounds: opts.bounds.clone(),
        },
        state,
        started_at: Some(started_at),
        ended_at: Some(ended_at),
        diff,
        gate: Some(gate.clone()),
        output_truncated: truncated,
    };

    let receipt_dir = write_receipt(&receipt, &output)?;
    let removed = remove_if_clean(&repo, &wt_path).unwrap_or(false);

    Ok(RunResult {
        id,
        state,
        receipt_dir,
        worktree: if removed { None } else { Some(wt_path) },
        worktree_removed: removed,
        gate: Some(gate),
    })
}

async fn dry_run_agent(
    opts: &RunOptions,
    id: &RunId,
    worktree: &std::path::Path,
    tx: &Option<mpsc::Sender<(RunId, RunDelta)>>,
    output: &mut String,
    truncated: &mut bool,
) -> Result<bool> {
    let lines = [
        format!("kit dry-run · agent={}", opts.agent),
        format!("task: {}", opts.task),
        format!("worktree: {}", worktree.display()),
        "status: streaming (no external CLI invoked)".into(),
        "note: set real adapters (B2) for production agent execution".into(),
    ];
    for line in lines {
        let chunk = format!("{line}\n");
        append_capped(output, truncated, opts.bounds.output_cap_bytes, &chunk);
        send(tx, id, RunDelta::Output(chunk)).await;
        sleep(Duration::from_millis(15)).await;
    }
    Ok(true)
}

fn append_capped(buf: &mut String, truncated: &mut bool, cap: u64, chunk: &str) {
    let cap = cap as usize;
    if buf.len() >= cap {
        *truncated = true;
        return;
    }
    let room = cap - buf.len();
    if chunk.len() > room {
        buf.push_str(&chunk[..room]);
        *truncated = true;
    } else {
        buf.push_str(chunk);
    }
}

async fn send(tx: &Option<mpsc::Sender<(RunId, RunDelta)>>, id: &RunId, delta: RunDelta) {
    if let Some(tx) = tx {
        let _ = tx.send((id.clone(), delta)).await;
    }
}

/// Parse agent label into [`AgentKind`].
pub fn parse_agent(s: &str) -> Result<AgentKind> {
    match s.to_ascii_lowercase().as_str() {
        "codex" => Ok(AgentKind::Codex),
        "claude" => Ok(AgentKind::Claude),
        "grok" => Ok(AgentKind::Grok),
        "ollama" => Ok(AgentKind::Ollama),
        other => anyhow::bail!("unknown agent '{other}' (codex|claude|grok|ollama)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::paths;
    use std::path::PathBuf;

    fn kit_repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root")
    }

    #[tokio::test]
    async fn dry_run_writes_receipt_and_cleans_worktree() {
        let root = kit_repo_root();
        let home = std::env::temp_dir().join(format!("kit-test-home-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        unsafe {
            std::env::set_var("KIT_HOME", &home);
        }

        let opts = RunOptions {
            repo: root.to_string_lossy().into_owned(),
            agent: AgentKind::Codex,
            task: "m1 skeleton smoke".into(),
            dry_run: true,
            bounds: Bounds::default(),
        };

        let result = execute(opts, None, None).await.expect("execute");
        assert_eq!(result.state, RunState::Pass);
        assert!(result.receipt_dir.join("receipt.json").exists());
        assert!(result.receipt_dir.join("output.log").exists());
        assert!(
            result.worktree_removed,
            "clean dry-run should drop worktree"
        );
        assert!(result.gate.is_some());

        let raw = std::fs::read_to_string(result.receipt_dir.join("receipt.json")).unwrap();
        assert!(raw.contains("m1 skeleton smoke"));
        assert!(raw.contains("\"version\": 1"));
        assert!(paths::kit_home().starts_with(&home));

        let _ = std::fs::remove_dir_all(&home);
        unsafe {
            std::env::remove_var("KIT_HOME");
        }
    }

    #[test]
    fn parse_agent_labels() {
        assert!(matches!(parse_agent("codex").unwrap(), AgentKind::Codex));
        assert!(parse_agent("nope").is_err());
    }
}
