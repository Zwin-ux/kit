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
    let agent_impl = adapter(opts.agent);
    execute_with(opts, id, tx, cancel, agent_impl.as_ref()).await
}

/// [`execute_cancellable`] with the agent adapter passed in (tests use fakes).
async fn execute_with(
    opts: RunOptions,
    id: Option<RunId>,
    tx: Option<mpsc::Sender<(RunId, RunDelta)>>,
    cancel: Option<Arc<CancelHandle>>,
    agent_impl: &dyn Agent,
) -> Result<RunResult> {
    ensure_layout()?;
    let id = id.unwrap_or_default();
    let repo = resolve_repo(&opts.repo)?;
    // Read the gate before any agent runs: a broken kit.toml stops the run
    // here, and nothing the agent does can change the gate it is held to.
    let config = load_kit_config(&repo)?;
    if opts.dry_run != Some(true) {
        require_agent(agent_impl.probe().await)?;
    }
    let started_at = SystemTime::now();

    send(&tx, &id, RunDelta::State(RunState::Running)).await;

    let wt_path = worktrees_dir().join(&id.0);
    let branch = branch_name(&id.0);
    let base = create_worktree(&repo, &wt_path, &branch)
        .with_context(|| format!("create worktree at {}", wt_path.display()))?;
    send(&tx, &id, RunDelta::Worktree(wt_path.clone())).await;

    let mut output = String::new();
    let mut truncated = false;

    // Once the worktree exists, every way out goes through one receipt write
    // and one clean-worktree removal: an error here (e.g. the agent cannot
    // spawn) ends as an `Error` receipt, never a leaked worktree.
    let ending = agent_and_gate(
        &opts,
        &id,
        &repo,
        &wt_path,
        &base,
        config,
        &tx,
        cancel.as_ref(),
        agent_impl,
        &mut output,
        &mut truncated,
    )
    .await;
    let (state, gate, note, agent_diff) = match ending {
        Ok(Ending::Killed(note)) => (RunState::Killed, None, Some(note), None),
        Ok(Ending::Gated {
            state,
            gate,
            agent_diff,
        }) => (state, Some(gate), None, agent_diff),
        Err(err) => (
            RunState::Error,
            None,
            Some(format!("kit: run failed: {err:#}\n")),
            None,
        ),
    };
    if let Some(note) = note {
        append_capped(
            &mut output,
            &mut truncated,
            opts.bounds.output_cap_bytes,
            &note,
        );
        send(&tx, &id, RunDelta::Output(note)).await;
    }
    let result = write_terminal(
        opts,
        id.clone(),
        repo,
        Some(branch),
        Some((wt_path, Some(base))),
        Some(started_at),
        state,
        output,
        truncated,
        gate,
        agent_diff,
    )
    .await?;
    // Proof first: the terminal state goes out only once its receipt exists.
    send(&tx, &id, RunDelta::State(state)).await;
    Ok(result)
}

/// How a run that got its worktree ended, before its receipt is written.
enum Ending {
    /// Killed by the user or the timeout; the note says which.
    Killed(String),
    /// The gate ran; `state` also records an agent that exited non-zero.
    /// `agent_diff` is the tree as the agent left it, taken before the gate.
    Gated {
        state: RunState,
        gate: GateOutcome,
        agent_diff: Option<String>,
    },
}

fn cancelled(cancel: Option<&Arc<CancelHandle>>) -> bool {
    cancel.is_some_and(|c| c.is_cancelled())
}

/// Agent phase, then gate phase, inside an existing worktree. Writes nothing
/// to the receipt store: the caller owns the one terminal write.
#[allow(clippy::too_many_arguments)]
async fn agent_and_gate(
    opts: &RunOptions,
    id: &RunId,
    repo: &std::path::Path,
    wt_path: &std::path::Path,
    base: &str,
    mut config: kit_core::KitConfig,
    tx: &Option<mpsc::Sender<(RunId, RunDelta)>>,
    cancel: Option<&Arc<CancelHandle>>,
    agent_impl: &dyn Agent,
    output: &mut String,
    truncated: &mut bool,
) -> Result<Ending> {
    if cancelled(cancel) {
        return Ok(Ending::Killed("kit: cancelled before agent start\n".into()));
    }

    // Only an explicit --dry-run is offline; a missing agent was refused
    // before the worktree was made (see `require_agent`).
    let use_dry = opts.dry_run == Some(true);
    // CEO stamp P2: infer defaults on live runs only. Dry-run stays offline-fast
    // and is exempt from vacuous non-zero exit. Inferred from the repo before
    // the agent starts, so the agent is told the checks it will be held to.
    if config.gate.is_empty() && !use_dry {
        let inferred = super::infer::infer_gate(repo);
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
            append_capped(output, truncated, opts.bounds.output_cap_bytes, &line);
            send(tx, id, RunDelta::Output(line)).await;
            config.gate = inferred;
        }
    }
    let gate_checks: Vec<String> = config
        .gate
        .checks()
        .into_iter()
        .map(|(_, command)| command.to_owned())
        .collect();

    // --- agent phase ---
    let phase = if use_dry {
        dry_run_agent(opts, id, wt_path, tx, output, truncated, cancel).await?
    } else {
        live_agent(
            agent_impl,
            opts,
            id,
            repo,
            wt_path,
            gate_checks,
            tx,
            output,
            truncated,
            cancel,
        )
        .await?
    };

    match phase {
        AgentPhase::Killed => return Ok(Ending::Killed("kit: run killed by user\n".into())),
        AgentPhase::TimedOut => {
            // CEO stamp: timeout maps to Killed + reason in output (RunState frozen).
            return Ok(Ending::Killed(format!(
                "kit: run killed — reason: timeout ({:?})\n",
                opts.bounds.timeout
            )));
        }
        AgentPhase::Ok | AgentPhase::Failed => {}
    }
    if cancelled(cancel) {
        return Ok(Ending::Killed("kit: run killed before gate\n".into()));
    }

    // --- gate phase ---
    send(tx, id, RunDelta::State(RunState::Gating)).await;
    let cap = opts.bounds.output_cap_bytes;
    // The receipt records what the agent made. Files the gate writes
    // (coverage, reports, build output) are not the run's work.
    let agent_diff = worktree::worktree_diff(wt_path, base).ok();
    let gate = if config.gate.is_empty() {
        // Still empty after inference → vacuous (TUI: UNCONFIGURED, never PASS).
        let line = "gate: no checks configured and none inferred (vacuous — UNCONFIGURED)\n";
        append_capped(output, truncated, cap, line);
        send(tx, id, RunDelta::Output(line.into())).await;
        GateOutcome::vacuous()
    } else {
        KitGate::new().evaluate(wt_path, &config.gate).await
    };
    send(tx, id, RunDelta::Gate(gate.clone())).await;
    if agent_diff.is_some() && worktree::worktree_diff(wt_path, base).ok() != agent_diff {
        let line = "gate: the gate changed files in the worktree; the receipt keeps only the agent's changes\n";
        append_capped(output, truncated, cap, line);
        send(tx, id, RunDelta::Output(line.into())).await;
    }

    let state = if phase == AgentPhase::Failed {
        RunState::Error
    } else if gate.passed {
        RunState::Pass
    } else {
        RunState::Fail
    };
    Ok(Ending::Gated {
        state,
        gate,
        agent_diff,
    })
}

/// Terminal path for a run killed before it started: still queued on the
/// concurrency limiter, or cancelled between permit and execute. Nothing was
/// created (no worktree, branch, or start time), yet the run still gets a
/// receipt, and `Killed` is sent only once that receipt is written.
pub(crate) async fn finalize_killed_before_start(
    opts: RunOptions,
    id: RunId,
    tx: Option<mpsc::Sender<(RunId, RunDelta)>>,
) -> Result<RunResult> {
    let note = "kit: run killed while queued (never started)\n";
    let result = finalize_outside_run(opts, id.clone(), RunState::Killed, note, &tx).await?;
    send(&tx, &id, RunDelta::State(RunState::Killed)).await;
    Ok(result)
}

/// Terminal path for a run whose own path failed before it reported a
/// terminal state (runner paths send one only after their receipt is
/// written). The error goes into the stream and an `Error` receipt, and
/// `Error` is sent even if that receipt cannot be written, so the run's row
/// never stays active.
pub(crate) async fn finalize_failed(
    opts: RunOptions,
    id: RunId,
    err: &anyhow::Error,
    tx: Option<mpsc::Sender<(RunId, RunDelta)>>,
) -> Result<RunResult> {
    let note = format!("kit: run failed: {err:#}\n");
    let result = finalize_outside_run(opts, id.clone(), RunState::Error, &note, &tx).await;
    send(&tx, &id, RunDelta::State(RunState::Error)).await;
    result
}

/// Receipt for a run that ends outside its own run path, with `note` in the
/// stream and the log. A worktree the run got before failing is diffed against
/// its HEAD and kept: its base commit is not known here, so Kit cannot prove
/// the agent left no commit in it. The start time is not known either.
async fn finalize_outside_run(
    opts: RunOptions,
    id: RunId,
    state: RunState,
    note: &str,
    tx: &Option<mpsc::Sender<(RunId, RunDelta)>>,
) -> Result<RunResult> {
    // Record the repo the way a started run would; keep the raw token if it
    // does not resolve (the run is over either way).
    let repo = resolve_repo(&opts.repo).unwrap_or_else(|_| PathBuf::from(&opts.repo));
    let wt = worktrees_dir().join(&id.0);
    let wt_path = wt.is_dir().then_some((wt, None));
    let branch = wt_path.as_ref().map(|_| branch_name(&id.0));
    let mut output = String::new();
    let mut truncated = false;
    append_capped(
        &mut output,
        &mut truncated,
        opts.bounds.output_cap_bytes,
        note,
    );
    send(tx, &id, RunDelta::Output(note.into())).await;
    write_terminal(
        opts, id, repo, branch, wt_path, None, state, output, truncated, None, None,
    )
    .await
}

/// Persist the receipt, then remove the worktree if the run left it clean.
/// `worktree` is `None` when the run never got one; else its path and base
/// commit (`None` when not known, and then the worktree is always kept).
#[allow(clippy::too_many_arguments)]
async fn write_terminal(
    opts: RunOptions,
    id: RunId,
    repo: PathBuf,
    branch: Option<String>,
    worktree: Option<(PathBuf, Option<String>)>,
    started_at: Option<SystemTime>,
    state: RunState,
    mut output: String,
    mut truncated: bool,
    gate: Option<GateOutcome>,
    agent_diff: Option<String>,
) -> Result<RunResult> {
    let ended_at = SystemTime::now();
    let (wt_path, base) = match worktree {
        Some((wt, base)) => (Some(wt), base),
        None => (None, None),
    };
    // Diff against the pinned base, so commits the agent made are in it.
    // Without a base, the worktree HEAD is the best Kit knows.
    let diff_base = base.clone().or_else(|| {
        wt_path
            .as_deref()
            .and_then(|wt| worktree::head_commit(wt).ok())
    });
    let diff = match (agent_diff, wt_path.as_deref(), diff_base.as_deref()) {
        (Some(d), _, _) => d,
        (None, Some(wt), Some(b)) => worktree::worktree_diff(wt, b).unwrap_or_else(|err| {
            // Loud: an empty diff here would under-report the run. Said in
            // the receipt's log, not on stderr, which would paint over the TUI.
            let note = format!("kit: cannot record the diff of {}: {err:#}\n", wt.display());
            append_capped(
                &mut output,
                &mut truncated,
                opts.bounds.output_cap_bytes,
                &note,
            );
            String::new()
        }),
        _ => String::new(),
    };

    let receipt = Receipt {
        version: Receipt::VERSION,
        id: id.clone(),
        spec: RunSpec {
            repo: repo.clone(),
            agent: opts.agent,
            task: opts.task.clone(),
            branch,
            bounds: opts.bounds.clone(),
            gate_checks: Vec::new(),
        },
        state,
        started_at,
        ended_at: Some(ended_at),
        diff,
        gate: gate.clone(),
        output_truncated: truncated,
    };

    let written = write_receipt(&receipt, &output, base.as_deref());
    // A clean worktree holds nothing the receipt lacks, so it goes even when
    // the receipt write failed: an error never leaks a worktree.
    let removed = match (wt_path.as_deref(), base.as_deref()) {
        (Some(wt), Some(b)) => remove_if_clean(&repo, wt, b).unwrap_or(false),
        _ => false,
    };
    let receipt_dir = written?;

    Ok(RunResult {
        id,
        state,
        receipt_dir,
        worktree: if removed { None } else { wt_path },
        worktree_removed: removed,
        gate,
    })
}

/// Refuse a live run whose agent is not installed.
///
/// A silent dry-run fallback runs no agent, so the gate checks an unchanged
/// tree and can PASS: a receipt that proves nothing. Stop before any worktree.
fn require_agent(status: kit_agents::AgentStatus) -> Result<()> {
    if status.installed {
        return Ok(());
    }
    let agent = status.kind;
    anyhow::bail!(
        "{agent} is not installed. Install the {} CLI, choose another agent (--agent \
         codex|claude|grok|ollama), or use --dry-run to test without an agent",
        agent.binary()
    )
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
    gate_checks: Vec<String>,
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
        gate_checks,
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

    // The adapter's pipe readers hold the only senders. Once the agent exits
    // they drop, and a closed channel is always ready: under `biased` it would
    // starve the exit poll below (Session B: kit spun until the run timeout).
    let mut pipes_open = true;
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
            maybe = local_rx.recv(), if pipes_open => {
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
                        pipes_open = false;
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
    if *truncated || buf.len() >= cap {
        *truncated = true;
        return;
    }
    let room = cap - buf.len();
    if chunk.len() > room {
        // Cut on a char boundary: slicing inside a multi-byte char panics.
        let mut end = room;
        while !chunk.is_char_boundary(end) {
            end -= 1;
        }
        buf.push_str(&chunk[..end]);
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
    use crate::engine::paths::kit_home_test_lock;
    use kit_core::CheckStatus;

    /// Agent output that hits the cap inside a multi-byte char used to panic.
    #[test]
    fn output_cap_cuts_on_a_char_boundary() {
        let (mut buf, mut truncated) = (String::from("ab"), false);
        // cap 4: room is 2 bytes, but "é" is 2 bytes starting at byte 1.
        append_capped(&mut buf, &mut truncated, 4, "xé✓");
        assert!(truncated);
        assert_eq!(buf, "abx");
        append_capped(&mut buf, &mut truncated, 4, "more");
        assert_eq!(buf, "abx");
    }

    /// A missing agent must stop the run, never fall back to a dry run that
    /// could PASS the gate on an unchanged tree.
    #[test]
    fn missing_agent_is_refused_with_the_fix() {
        let err = require_agent(kit_agents::AgentStatus::missing(AgentKind::Grok))
            .unwrap_err()
            .to_string();
        assert!(err.contains("grok is not installed"), "{err}");
        assert!(err.contains("--dry-run"), "{err}");
        let mut ready = kit_agents::AgentStatus::missing(AgentKind::Codex);
        ready.installed = true;
        assert!(require_agent(ready).is_ok());
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
        let _guard = kit_home_test_lock();
        unsafe {
            std::env::set_var("KIT_HOME", home);
        }
        let out = f().await;
        unsafe {
            std::env::remove_var("KIT_HOME");
        }
        out
    }

    #[tokio::test]
    async fn dry_run_writes_receipt_and_cleans_worktree() {
        let root = crate::engine::paths::bare_git_fixture();
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
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn parse_agent_labels() {
        assert!(matches!(parse_agent("codex").unwrap(), AgentKind::Codex));
        assert!(parse_agent("nope").is_err());
    }

    #[tokio::test]
    async fn dry_run_gate_does_not_write_parent_cargo_target_dir() {
        let root = crate::engine::paths::bare_git_fixture();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let home = std::env::temp_dir().join(format!(
            "kit-test-home-target-{}-{stamp}",
            std::process::id()
        ));
        let parent_target =
            std::env::temp_dir().join(format!("kit-parent-target-{}-{stamp}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        let _ = std::fs::remove_dir_all(&parent_target);
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&parent_target).unwrap();
        let sentinel = parent_target.join("sentinel");
        std::fs::write(&sentinel, b"parent-target\n").unwrap();

        #[cfg(windows)]
        let script = home.join("p1-probe.cmd");
        #[cfg(not(windows))]
        let script = home.join("p1-probe.sh");
        #[cfg(windows)]
        std::fs::write(
            &script,
            "@echo off\r\nif not exist \"%CARGO_TARGET_DIR%\" mkdir \"%CARGO_TARGET_DIR%\"\r\necho p1>\"%CARGO_TARGET_DIR%\\p1-marker\"\r\n",
        )
        .unwrap();
        #[cfg(not(windows))]
        std::fs::write(
            &script,
            "#!/bin/sh\nmkdir -p \"$CARGO_TARGET_DIR\"\necho p1 > \"$CARGO_TARGET_DIR/p1-marker\"\n",
        )
        .unwrap();
        #[cfg(windows)]
        let test_cmd = format!("cmd /C \"{}\"", script.display());
        #[cfg(not(windows))]
        let test_cmd = format!("sh \"{}\"", script.display());
        let kit_toml = format!("[gate]\ntest = '{test_cmd}'\ntimeout = \"30s\"\n");
        std::fs::write(root.join("kit.toml"), kit_toml).unwrap();
        let loaded = crate::engine::store::load_kit_config(&root).expect("kit.toml");
        assert!(
            !loaded.gate.is_empty(),
            "fixture kit.toml must parse so the probe actually runs"
        );

        let before_count = std::fs::read_dir(&parent_target).unwrap().count();
        let before_dir_mtime = std::fs::metadata(&parent_target)
            .unwrap()
            .modified()
            .unwrap();
        let before_sentinel = std::fs::metadata(&sentinel).unwrap().modified().unwrap();

        let result = with_kit_home(&home, || async {
            let prev = std::env::var_os("CARGO_TARGET_DIR");
            unsafe {
                std::env::set_var("CARGO_TARGET_DIR", &parent_target);
            }
            struct Restore(Option<std::ffi::OsString>);
            impl Drop for Restore {
                fn drop(&mut self) {
                    match &self.0 {
                        Some(v) => unsafe { std::env::set_var("CARGO_TARGET_DIR", v) },
                        None => unsafe { std::env::remove_var("CARGO_TARGET_DIR") },
                    }
                }
            }
            let _restore = Restore(prev);
            let opts = RunOptions {
                repo: root.to_string_lossy().into_owned(),
                agent: AgentKind::Codex,
                task: "p1 cargo target isolation".into(),
                dry_run: Some(true),
                bounds: Bounds::default(),
            };
            execute(opts, None, None).await.expect("execute")
        })
        .await;

        let after_count = std::fs::read_dir(&parent_target).unwrap().count();
        let after_dir_mtime = std::fs::metadata(&parent_target)
            .unwrap()
            .modified()
            .unwrap();
        let after_sentinel = std::fs::metadata(&sentinel).unwrap().modified().unwrap();
        println!(
            "parent target count {before_count}->{after_count} dir_mtime {before_dir_mtime:?}->{after_dir_mtime:?}"
        );

        let gate = result.gate.as_ref().expect("dry-run still runs the gate");
        assert_eq!(
            gate.checks.len(),
            1,
            "probe check must run, got {:?}",
            gate.checks
        );
        assert_eq!(
            gate.checks[0].status,
            CheckStatus::Pass,
            "probe must actually run: {:?}",
            gate.checks[0]
        );
        assert_eq!(
            after_count, before_count,
            "parent CARGO_TARGET_DIR file count changed"
        );
        assert_eq!(
            after_sentinel, before_sentinel,
            "parent sentinel mtime changed"
        );
        assert_eq!(
            after_dir_mtime, before_dir_mtime,
            "parent target dir mtime changed"
        );
        assert!(
            !parent_target.join("p1-marker").exists(),
            "probe wrote into the parent CARGO_TARGET_DIR"
        );

        let _ = std::fs::remove_dir_all(&home);
        let _ = std::fs::remove_dir_all(&parent_target);
        let _ = std::fs::remove_dir_all(&root);
        if let Some(wt) = result.worktree {
            let _ = std::fs::remove_dir_all(&wt);
        }
    }

    #[tokio::test]
    async fn cancel_before_start_yields_killed() {
        let root = crate::engine::paths::bare_git_fixture();
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
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Real child that exits at once. As in kit-agents' `spawn_streaming`, the
    /// only sender lives in the stdout reader and drops at EOF (agent exit).
    struct ExitAtOnce;

    #[async_trait::async_trait]
    impl Agent for ExitAtOnce {
        fn kind(&self) -> AgentKind {
            AgentKind::Codex
        }

        async fn probe(&self) -> kit_agents::AgentStatus {
            kit_agents::AgentStatus::missing(AgentKind::Codex)
        }

        async fn spawn(
            &self,
            _spec: &RunSpec,
            worktree: &std::path::Path,
            tx: mpsc::Sender<RunDelta>,
        ) -> std::result::Result<Box<dyn kit_agents::AgentHandle>, kit_agents::SpawnError> {
            let mut cmd = if cfg!(windows) {
                let mut c = tokio::process::Command::new("cmd");
                c.args(["/C", "exit", "0"]);
                c
            } else {
                tokio::process::Command::new("true")
            };
            let mut child = cmd
                .current_dir(worktree)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .kill_on_drop(true)
                .spawn()
                .map_err(|source| kit_agents::SpawnError::Io {
                    kind: AgentKind::Codex,
                    source,
                })?;
            let stdout = child.stdout.take().expect("piped stdout");
            tokio::spawn(async move {
                use tokio::io::AsyncBufReadExt;
                let mut lines = tokio::io::BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx.send(RunDelta::Output(format!("{line}\n"))).await;
                }
            });
            Ok(Box::new(ExitHandle(std::sync::Mutex::new(child))))
        }
    }

    /// `AgentHandle` must be `Sync`; `tokio::process::Child` is not.
    struct ExitHandle(std::sync::Mutex<tokio::process::Child>);

    #[async_trait::async_trait]
    impl kit_agents::AgentHandle for ExitHandle {
        async fn wait(&mut self) -> std::io::Result<i32> {
            let child = self.0.get_mut().expect("child");
            Ok(child.wait().await?.code().unwrap_or(1))
        }

        async fn kill(&mut self) -> std::io::Result<()> {
            self.0.get_mut().expect("child").kill().await
        }

        async fn try_wait(&mut self) -> std::io::Result<Option<i32>> {
            let child = self.0.get_mut().expect("child");
            Ok(child.try_wait()?.map(|status| status.code().unwrap_or(1)))
        }
    }

    /// Installed, but its program cannot start (`spawn grok: program not found`).
    struct SpawnFails;

    #[async_trait::async_trait]
    impl Agent for SpawnFails {
        fn kind(&self) -> AgentKind {
            AgentKind::Grok
        }

        async fn probe(&self) -> kit_agents::AgentStatus {
            let mut status = kit_agents::AgentStatus::missing(AgentKind::Grok);
            status.installed = true;
            status
        }

        async fn spawn(
            &self,
            _spec: &RunSpec,
            _worktree: &std::path::Path,
            _tx: mpsc::Sender<RunDelta>,
        ) -> std::result::Result<Box<dyn kit_agents::AgentHandle>, kit_agents::SpawnError> {
            Err(kit_agents::SpawnError::Io {
                kind: AgentKind::Grok,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "program not found"),
            })
        }
    }

    /// `git worktree list --porcelain` entries for `repo`.
    fn git_worktrees(repo: &std::path::Path) -> Vec<String> {
        let out = std::process::Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(repo)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .expect("git worktree list");
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.strip_prefix("worktree ").map(str::to_string))
            .collect()
    }

    /// An agent that fails to spawn after the worktree exists must not leak
    /// it: the dir and its `git worktree` registration go, and the run ends
    /// with an `Error` receipt that names the cause, on the CLI path too.
    #[tokio::test]
    async fn spawn_failure_removes_worktree_and_writes_error_receipt() {
        let root = crate::engine::paths::bare_git_fixture();
        let home = std::env::temp_dir().join(format!(
            "kit-test-spawnfail-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        let id = RunId("01TESTSPAWNFAILS0000000001".into());

        let (result, wt_dir, receipt, log) = with_kit_home(&home, || async {
            let opts = RunOptions {
                repo: root.to_string_lossy().into_owned(),
                agent: AgentKind::Grok,
                task: "spawn fails".into(),
                dry_run: Some(false),
                bounds: Bounds::default(),
            };
            let result = execute_with(opts, Some(id.clone()), None, None, &SpawnFails).await;
            let wt_dir = worktrees_dir().join(&id.0);
            let receipt = crate::engine::store::read_receipt(&id.0);
            let log = crate::engine::store::read_output_tail(&id.0, 4096);
            (result, wt_dir, receipt, log)
        })
        .await;

        let result = result.expect("a spawn failure still ends the run");
        assert_eq!(result.state, RunState::Error);
        assert!(result.worktree_removed, "clean worktree must be removed");
        assert!(!wt_dir.exists(), "leaked {}", wt_dir.display());
        assert_eq!(git_worktrees(&root).len(), 1, "{:?}", git_worktrees(&root));
        let receipt = receipt.expect("read").expect("an Error receipt");
        assert_eq!(receipt.state, RunState::Error);
        assert!(receipt.gate.is_none());
        let log = log.expect("output.log");
        assert!(log.contains("program not found"), "{log}");

        let _ = std::fs::remove_dir_all(&home);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Changes files in the worktree like a real agent, then exits 0:
    /// edits a tracked file, creates a text file and a binary file, and
    /// (when `commit` is set) commits the edit, as some agents do.
    struct WritesFiles {
        commit: bool,
    }

    #[async_trait::async_trait]
    impl Agent for WritesFiles {
        fn kind(&self) -> AgentKind {
            AgentKind::Codex
        }

        async fn probe(&self) -> kit_agents::AgentStatus {
            let mut status = kit_agents::AgentStatus::missing(AgentKind::Codex);
            status.installed = true;
            status
        }

        async fn spawn(
            &self,
            spec: &RunSpec,
            worktree: &std::path::Path,
            tx: mpsc::Sender<RunDelta>,
        ) -> std::result::Result<Box<dyn kit_agents::AgentHandle>, kit_agents::SpawnError> {
            std::fs::write(worktree.join("README.md"), "kit test fixture\nedited\n").unwrap();
            if self.commit {
                let git = |args: &[&str]| {
                    std::process::Command::new("git")
                        .args(["-c", "user.name=kit", "-c", "user.email=kit@test"])
                        .args(args)
                        .current_dir(worktree)
                        .env_remove("GIT_DIR")
                        .env_remove("GIT_INDEX_FILE")
                        .output()
                        .unwrap()
                };
                git(&["commit", "-q", "-am", "agent commit"]);
            }
            std::fs::create_dir_all(worktree.join("src")).unwrap();
            std::fs::write(worktree.join("src").join("created.txt"), "new file\n").unwrap();
            std::fs::write(worktree.join("blob.bin"), [0u8, 159, 146, 150, 0, 255, 1]).unwrap();
            ExitAtOnce.spawn(spec, worktree, tx).await
        }
    }

    /// Run `agent` live against a fresh fixture; returns (result, receipt, repo, home).
    async fn run_with_agent(
        tag: &str,
        agent: &dyn Agent,
    ) -> (RunResult, Receipt, std::path::PathBuf, std::path::PathBuf) {
        let root = crate::engine::paths::bare_git_fixture();
        let home = std::env::temp_dir().join(format!(
            "kit-test-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&home).unwrap();
        let (result, receipt) = with_kit_home(&home, || async {
            let opts = RunOptions {
                repo: root.to_string_lossy().into_owned(),
                agent: AgentKind::Codex,
                task: "write files".into(),
                dry_run: Some(false),
                bounds: Bounds::default(),
            };
            let result = execute_with(opts, None, None, None, agent)
                .await
                .expect("run");
            let receipt = crate::engine::store::read_receipt(&result.id.0)
                .unwrap()
                .unwrap();
            (result, receipt)
        })
        .await;
        (result, receipt, root, home)
    }

    /// `git apply --check` of `patch` against `repo`'s (clean) working tree.
    fn applies_cleanly(repo: &std::path::Path, patch: &std::path::Path) -> (bool, String) {
        let out = std::process::Command::new("git")
            .args(["apply", "--check", "--binary"])
            .arg(patch)
            .current_dir(repo)
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .unwrap();
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    /// Proof gap: a file the agent CREATES must be in the receipt diff, binary
    /// files too, and diff.patch must apply to the base commit. The worktree
    /// is kept exactly when the diff is not empty.
    #[tokio::test]
    async fn receipt_diff_includes_new_and_binary_files() {
        let (result, receipt, root, home) =
            run_with_agent("newfiles", &WritesFiles { commit: false }).await;
        assert!(receipt.diff.contains("src/created.txt"), "{}", receipt.diff);
        assert!(receipt.diff.contains("blob.bin"), "{}", receipt.diff);
        assert!(
            receipt.diff.contains("GIT binary patch"),
            "{}",
            receipt.diff
        );
        assert!(receipt.diff.contains("+edited"), "{}", receipt.diff);
        let patch = result.receipt_dir.join("diff.patch");
        let (ok, err) = applies_cleanly(&root, &patch);
        assert!(ok, "diff.patch does not apply to the base: {err}");
        assert!(!result.worktree_removed, "a changed worktree is kept");
        let _ = std::fs::remove_dir_all(&home);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Files the gate writes (coverage, reports) are not the agent's work:
    /// the receipt, and so `kit land`, must hold only what the agent made.
    #[tokio::test]
    async fn gate_output_files_stay_out_of_the_receipt() {
        if std::process::Command::new("node")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let root = crate::engine::paths::bare_git_fixture();
        std::fs::write(
            root.join("kit.toml"),
            "[gate]\ntest = \"node -e \\\"require('fs').writeFileSync('gate-artifact.txt','x')\\\"\"\n",
        )
        .unwrap();
        let home = std::env::temp_dir().join(format!(
            "kit-test-gatefiles-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&home).unwrap();
        let (result, receipt, log) = with_kit_home(&home, || async {
            let opts = RunOptions {
                repo: root.to_string_lossy().into_owned(),
                agent: AgentKind::Codex,
                task: "write files".into(),
                dry_run: Some(false),
                bounds: Bounds::default(),
            };
            let agent = WritesFiles { commit: false };
            let result = execute_with(opts, None, None, None, &agent)
                .await
                .expect("run");
            let receipt = crate::engine::store::read_receipt(&result.id.0)
                .unwrap()
                .unwrap();
            let log = std::fs::read_to_string(result.receipt_dir.join("output.log")).unwrap();
            (result, receipt, log)
        })
        .await;
        assert_eq!(result.state, RunState::Pass, "{log}");
        assert!(receipt.diff.contains("src/created.txt"), "{}", receipt.diff);
        assert!(
            !receipt.diff.contains("gate-artifact.txt"),
            "gate output leaked into the receipt: {}",
            receipt.diff
        );
        assert!(log.contains("the gate changed files"), "{log}");
        let _ = std::fs::remove_dir_all(&home);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// An agent that commits in the worktree moves its HEAD. The receipt diff
    /// is taken against the base commit, so the committed edit is not lost.
    #[tokio::test]
    async fn receipt_diff_includes_agent_commits() {
        let (result, receipt, root, home) =
            run_with_agent("commits", &WritesFiles { commit: true }).await;
        assert!(receipt.diff.contains("+edited"), "{}", receipt.diff);
        assert!(receipt.diff.contains("src/created.txt"), "{}", receipt.diff);
        let (ok, err) = applies_cleanly(&root, &result.receipt_dir.join("diff.patch"));
        assert!(ok, "diff.patch does not apply to the base: {err}");
        // The agent's commit is only reachable from the worktree: keep it.
        assert!(
            !result.worktree_removed,
            "worktree with a commit was removed"
        );
        assert!(result.worktree.as_ref().is_some_and(|w| w.is_dir()));
        let base = crate::engine::store::read_base(&result.receipt_dir).expect("base.txt");
        assert_eq!(base, worktree::head_commit(&root).unwrap());
        let _ = std::fs::remove_dir_all(&home);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Session B: once the agent exits its pipes close and `recv()` yields
    /// `None` on every poll. The biased select must still reach the exit poll.
    #[tokio::test]
    async fn live_agent_sees_exit_after_output_pipes_close() {
        let dir = std::env::temp_dir();
        let opts = RunOptions {
            task: "exit at once".into(),
            dry_run: Some(false),
            bounds: Bounds {
                timeout: Duration::from_secs(10),
                ..Bounds::default()
            },
            ..RunOptions::default()
        };
        let id = RunId::default();
        let (mut output, mut truncated) = (String::new(), false);
        let run = live_agent(
            &ExitAtOnce,
            &opts,
            &id,
            &dir,
            &dir,
            Vec::new(),
            &None,
            &mut output,
            &mut truncated,
            None,
        );
        let phase = tokio::time::timeout(Duration::from_secs(2), run)
            .await
            .expect("exit poll starved: live_agent never saw the agent exit")
            .expect("live_agent");
        assert_eq!(phase, AgentPhase::Ok);
        assert!(output.contains("kit: codex exited with code 0"), "{output}");
    }
}
