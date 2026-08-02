//! One-run executor: worktree → agent stream → gate → receipt.
//!
//! Production shape for M1. Agent body is dry-run or live adapters; gate and
//! receipt are always real. Never reports PASS without a gate outcome.
//! Cancel-aware: optional [`CancelHandle`] stops agent mid-run → `Killed`.

use super::cancel::CancelHandle;
use super::paths::worktrees_dir;
use super::store::{ensure_layout, load_kit_config, write_receipt};
use super::worktree::{self, branch_name, create_worktree, remove_if_clean, resolve_repo};
use anyhow::{Context, Result};
use kit_agents::{Agent, adapter};
use kit_core::{AgentKind, Bounds, Gate, GateOutcome, Receipt, RunDelta, RunId, RunSpec, RunState};
use kit_gate::KitGate;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Options for a single run (headless or Control Room).
#[derive(Debug, Clone)]
pub struct RunOptions {
    pub repo: String,
    pub agent: AgentKind,
    pub task: String,
    /// When true, skip external CLIs and stream a dry-run transcript.
    /// When `None`, auto: live if the agent binary is on PATH, else dry-run.
    pub dry_run: Option<bool>,
    pub bounds: Bounds,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            repo: ".".into(),
            agent: AgentKind::Codex,
            task: String::new(),
            dry_run: None,
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

/// How the agent phase ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentPhase {
    Ok,
    Failed,
    Killed,
    TimedOut,
}

/// Execute one run, streaming deltas to `tx` when provided.
///
/// When `cancel` is set and cancelled mid-agent, the run ends as [`RunState::Killed`]
/// without claiming a gate PASS.
pub async fn execute(
    opts: RunOptions,
    id: Option<RunId>,
    tx: Option<mpsc::Sender<(RunId, RunDelta)>>,
) -> Result<RunResult> {
    execute_cancellable(opts, id, tx, None).await
}

/// Same as [`execute`] with an optional shared cancel handle (Control Room kill).
pub async fn execute_cancellable(
    opts: RunOptions,
    id: Option<RunId>,
    tx: Option<mpsc::Sender<(RunId, RunDelta)>>,
    cancel: Option<Arc<CancelHandle>>,
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

    if cancelled(&cancel) {
        return finalize_killed(
            opts,
            id,
            repo,
            branch,
            wt_path,
            started_at,
            output,
            truncated,
            tx,
            "kit: cancelled before agent start\n",
        )
        .await;
    }

    // --- agent phase ---
    let force_dry = opts.dry_run == Some(true);
    let force_live = opts.dry_run == Some(false);
    let agent_impl = adapter(opts.agent);
    let status = agent_impl.probe().await;
    let use_dry = if force_dry {
        true
    } else if force_live {
        if !status.installed {
            append_capped(
                &mut output,
                &mut truncated,
                opts.bounds.output_cap_bytes,
                &format!(
                    "kit: {} not on PATH — cannot force live; install {}\n",
                    opts.agent,
                    opts.agent.binary()
                ),
            );
            send(
                &tx,
                &id,
                RunDelta::Output(format!(
                    "kit: {} missing; falling back to dry-run\n",
                    opts.agent.binary()
                )),
            )
            .await;
            true
        } else {
            false
        }
    } else {
        // Auto: live when installed.
        !status.installed
    };

    let phase = if use_dry {
        if !force_dry && !status.installed {
            send(
                &tx,
                &id,
                RunDelta::Output(format!(
                    "kit: {} not installed — dry-run (install CLI for live agents)\n",
                    opts.agent.binary()
                )),
            )
            .await;
        }
        dry_run_agent(
            &opts,
            &id,
            &wt_path,
            &tx,
            &mut output,
            &mut truncated,
            cancel.as_ref(),
        )
        .await?
    } else {
        live_agent(
            agent_impl.as_ref(),
            &opts,
            &id,
            &repo,
            &wt_path,
            &tx,
            &mut output,
            &mut truncated,
            cancel.as_ref(),
        )
        .await?
    };

    match phase {
        AgentPhase::Killed => {
            return finalize_killed(
                opts,
                id,
                repo,
                branch,
                wt_path,
                started_at,
                output,
                truncated,
                tx,
                "kit: run killed by user\n",
            )
            .await;
        }
        AgentPhase::TimedOut => {
            // CEO stamp: timeout maps to Killed + reason in output (RunState frozen).
            let line = format!(
                "kit: run killed — reason: timeout ({:?})\n",
                opts.bounds.timeout
            );
            return finalize_killed(
                opts, id, repo, branch, wt_path, started_at, output, truncated, tx, &line,
            )
            .await;
        }
        AgentPhase::Ok | AgentPhase::Failed => {}
    }

    if cancelled(&cancel) {
        return finalize_killed(
            opts,
            id,
            repo,
            branch,
            wt_path,
            started_at,
            output,
            truncated,
            tx,
            "kit: run killed before gate\n",
        )
        .await;
    }

    // --- gate phase ---
    send(&tx, &id, RunDelta::State(RunState::Gating)).await;
    let mut config = load_kit_config(&repo);
    // CEO stamp P2: infer defaults on live runs only. Dry-run stays offline-fast
    // and is exempt from vacuous non-zero exit.
    if config.gate.is_empty() && !use_dry {
        let inferred = super::infer::infer_gate(&repo);
        if !inferred.is_empty() {
            let line = format!(
                "gate: inferred checks (no kit.toml gate) — {}\n",
                inferred
                    .checks()
                    .iter()
                    .map(|(l, c)| format!("{l}:{c}"))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
            append_capped(
                &mut output,
                &mut truncated,
                opts.bounds.output_cap_bytes,
                &line,
            );
            send(&tx, &id, RunDelta::Output(line)).await;
            config.gate = inferred;
        }
    }
    let gate_engine = KitGate::new();
    let gate = if config.gate.is_empty() {
        // Still empty after inference → vacuous (TUI: UNCONFIGURED, never PASS).
        let line = "gate: no checks configured and none inferred (vacuous — UNCONFIGURED)\n";
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

    let state = if phase == AgentPhase::Failed {
        RunState::Error
    } else if gate.passed {
        RunState::Pass
    } else {
        RunState::Fail
    };
    send(&tx, &id, RunDelta::State(state)).await;

    write_terminal(
        opts,
        id,
        repo,
        branch,
        wt_path,
        started_at,
        state,
        output,
        truncated,
        Some(gate),
        tx,
    )
    .await
}

fn cancelled(cancel: &Option<Arc<CancelHandle>>) -> bool {
    cancel.as_ref().is_some_and(|c| c.is_cancelled())
}

#[allow(clippy::too_many_arguments)]
async fn finalize_killed(
    opts: RunOptions,
    id: RunId,
    repo: PathBuf,
    branch: String,
    wt_path: PathBuf,
    started_at: SystemTime,
    mut output: String,
    mut truncated: bool,
    tx: Option<mpsc::Sender<(RunId, RunDelta)>>,
    note: &str,
) -> Result<RunResult> {
    append_capped(
        &mut output,
        &mut truncated,
        opts.bounds.output_cap_bytes,
        note,
    );
    send(&tx, &id, RunDelta::Output(note.into())).await;
    send(&tx, &id, RunDelta::State(RunState::Killed)).await;
    write_terminal(
        opts,
        id,
        repo,
        branch,
        wt_path,
        started_at,
        RunState::Killed,
        output,
        truncated,
        None,
        tx,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn write_terminal(
    opts: RunOptions,
    id: RunId,
    repo: PathBuf,
    branch: String,
    wt_path: PathBuf,
    started_at: SystemTime,
    state: RunState,
    output: String,
    truncated: bool,
    gate: Option<GateOutcome>,
    _tx: Option<mpsc::Sender<(RunId, RunDelta)>>,
) -> Result<RunResult> {
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
        gate: gate.clone(),
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
        gate,
    })
}

async fn dry_run_agent(
    opts: &RunOptions,
    id: &RunId,
    worktree: &std::path::Path,
    tx: &Option<mpsc::Sender<(RunId, RunDelta)>>,
    output: &mut String,
    truncated: &mut bool,
    cancel: Option<&Arc<CancelHandle>>,
) -> Result<AgentPhase> {
    let lines = [
        format!("kit dry-run · agent={}", opts.agent),
        format!("task: {}", opts.task),
        format!("worktree: {}", worktree.display()),
        "status: streaming (no external CLI invoked)".into(),
        "hint: install codex/claude/grok/ollama for live TUI dispatch".into(),
    ];
    let deadline = tokio::time::Instant::now() + opts.bounds.timeout;
    for line in lines {
        if cancel.is_some_and(|c| c.is_cancelled()) {
            return Ok(AgentPhase::Killed);
        }
        if tokio::time::Instant::now() >= deadline {
            return Ok(AgentPhase::TimedOut);
        }
        let chunk = format!("{line}\n");
        append_capped(output, truncated, opts.bounds.output_cap_bytes, &chunk);
        send(tx, id, RunDelta::Output(chunk)).await;
        sleep(Duration::from_millis(15)).await;
    }
    if cancel.is_some_and(|c| c.is_cancelled()) {
        return Ok(AgentPhase::Killed);
    }
    Ok(AgentPhase::Ok)
}

/// Live agent via kit-agents adapter; tee deltas into local output buffer.
///
/// Uses `try_wait` polling so kill/timeout never race a long `wait()` borrow.
#[allow(clippy::too_many_arguments)]
async fn live_agent(
    agent: &dyn Agent,
    opts: &RunOptions,
    id: &RunId,
    repo: &std::path::Path,
    worktree: &std::path::Path,
    tx: &Option<mpsc::Sender<(RunId, RunDelta)>>,
    output: &mut String,
    truncated: &mut bool,
    cancel: Option<&Arc<CancelHandle>>,
) -> Result<AgentPhase> {
    let (local_tx, mut local_rx) = mpsc::channel::<RunDelta>(256);
    let tee_tx = tx.clone();
    let id_tee = id.clone();
    let cap = opts.bounds.output_cap_bytes;

    let spec = RunSpec {
        repo: repo.to_path_buf(),
        agent: opts.agent,
        task: opts.task.clone(),
        branch: None,
        bounds: opts.bounds.clone(),
    };

    let mut handle = agent
        .spawn(&spec, worktree, local_tx)
        .await
        .with_context(|| format!("spawn {}", opts.agent))?;

    let deadline = tokio::time::Instant::now() + opts.bounds.timeout;
    let mut poll = tokio::time::interval(Duration::from_millis(40));
    poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // First tick is immediate; consume so we do not race spawn.
    poll.tick().await;

    let outcome = loop {
        if cancel.is_some_and(|c| c.is_cancelled()) {
            let _ = handle.kill().await;
            break AgentPhase::Killed;
        }
        if tokio::time::Instant::now() >= deadline {
            let _ = handle.kill().await;
            break AgentPhase::TimedOut;
        }

        tokio::select! {
            biased;
            _ = async {
                if let Some(c) = cancel {
                    c.cancelled().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => {
                let _ = handle.kill().await;
                break AgentPhase::Killed;
            }
            _ = tokio::time::sleep_until(deadline) => {
                let _ = handle.kill().await;
                break AgentPhase::TimedOut;
            }
            maybe = local_rx.recv() => {
                match maybe {
                    Some(delta) => {
                        if let RunDelta::Output(chunk) = &delta {
                            append_capped(output, truncated, cap, chunk);
                        }
                        if let Some(ui) = &tee_tx {
                            let _ = ui.send((id_tee.clone(), delta)).await;
                        }
                    }
                    None => {
                        // Output pipes closed; keep polling exit until done/kill/timeout.
                    }
                }
            }
            _ = poll.tick() => {
                match handle.try_wait().await {
                    Ok(Some(code)) => {
                        while let Ok(delta) = local_rx.try_recv() {
                            if let RunDelta::Output(chunk) = &delta {
                                append_capped(output, truncated, cap, chunk);
                            }
                            if let Some(ui) = &tee_tx {
                                let _ = ui.send((id_tee.clone(), delta)).await;
                            }
                        }
                        let line = format!("kit: {} exited with code {code}\n", opts.agent);
                        append_capped(output, truncated, cap, &line);
                        send(tx, id, RunDelta::Output(line)).await;
                        break if code == 0 {
                            AgentPhase::Ok
                        } else {
                            AgentPhase::Failed
                        };
                    }
                    Ok(None) => {}
                    Err(err) => {
                        let line = format!("kit: wait error: {err}\n");
                        append_capped(output, truncated, cap, &line);
                        send(tx, id, RunDelta::Output(line)).await;
                        break AgentPhase::Failed;
                    }
                }
            }
        }
    };

    Ok(outcome)
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
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    /// KIT_HOME is process-global — serialize tests that mutate it.
    fn kit_home_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    /// Holding a std Mutex across await is intentional here: tests must not
    /// interleave KIT_HOME mutation. Clippy would prefer tokio::Mutex; that
    /// would not prevent other threads from racing the env var.
    #[allow(clippy::await_holding_lock)]
    async fn with_kit_home<F, Fut, T>(home: &std::path::Path, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let _guard = kit_home_lock().lock().unwrap();
        unsafe {
            std::env::set_var("KIT_HOME", home);
        }
        let out = f().await;
        unsafe {
            std::env::remove_var("KIT_HOME");
        }
        out
    }

    fn kit_repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root")
    }

    #[tokio::test]
    async fn dry_run_writes_receipt_and_cleans_worktree() {
        let root = kit_repo_root();
        let home = std::env::temp_dir().join(format!(
            "kit-test-home-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();

        let result = with_kit_home(&home, || async {
            let opts = RunOptions {
                repo: root.to_string_lossy().into_owned(),
                agent: AgentKind::Codex,
                task: "m1 skeleton smoke".into(),
                dry_run: Some(true),
                bounds: Bounds::default(),
            };
            execute(opts, None, None).await.expect("execute")
        })
        .await;

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
        // kit_home may have been cleared after with_kit_home — assert receipt under home.
        assert!(result.receipt_dir.starts_with(&home));

        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn parse_agent_labels() {
        assert!(matches!(parse_agent("codex").unwrap(), AgentKind::Codex));
        assert!(parse_agent("nope").is_err());
    }

    #[tokio::test]
    async fn cancel_before_start_yields_killed() {
        let root = kit_repo_root();
        let home = std::env::temp_dir().join(format!(
            "kit-test-kill-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();

        let cancel = CancelHandle::new();
        cancel.cancel();

        let result = with_kit_home(&home, || async {
            let opts = RunOptions {
                repo: root.to_string_lossy().into_owned(),
                agent: AgentKind::Codex,
                task: "should not run".into(),
                dry_run: Some(true),
                bounds: Bounds::default(),
            };
            execute_cancellable(opts, None, None, Some(cancel))
                .await
                .expect("execute")
        })
        .await;

        assert_eq!(result.state, RunState::Killed);
        assert!(result.receipt_dir.join("receipt.json").exists());
        let raw = std::fs::read_to_string(result.receipt_dir.join("receipt.json")).unwrap();
        assert!(raw.contains("\"killed\"") || raw.contains("killed"));

        let _ = std::fs::remove_dir_all(&home);
    }
}
