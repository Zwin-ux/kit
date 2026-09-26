//! Control Room engine supervisor — kill registry + max-N concurrency.
//!
//! Every dispatched run is announced `Queued` before it waits for one of the
//! [`MAX_CONCURRENT_RUNS`](super::registry::MAX_CONCURRENT_RUNS) permits, so
//! the delta stream alone says which runs hold a slot. Tests prove the cap
//! from that stream; the task body carries no test-only timing.

use super::cancel::CancelHandle;
use super::registry::{RunRegistry, concurrency_limiter};
use super::runner::{
    RunOptions, RunResult, execute_cancellable, finalize_failed, finalize_killed_before_start,
    parse_agent,
};
use kit_core::{Bounds, RunDelta, RunId, RunState};
use kit_tui::EngineCommand;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Spawn the production supervisor (used by `kit` TUI entry).
pub fn spawn_production(
    cmd_rx: mpsc::Receiver<EngineCommand>,
    delta_tx: mpsc::Sender<(RunId, RunDelta)>,
) {
    tokio::spawn(async move {
        run_supervisor(cmd_rx, delta_tx, false).await;
    });
}

/// Supervisor loop. When `force_dry` is true, every job runs offline (tests / CI).
pub async fn run_supervisor(
    mut cmd_rx: mpsc::Receiver<EngineCommand>,
    delta_tx: mpsc::Sender<(RunId, RunDelta)>,
    force_dry: bool,
) {
    let registry = Arc::new(RunRegistry::new());
    let limiter = concurrency_limiter();

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            EngineCommand::Kill { id } => {
                let found = registry.kill(&id).await;
                if !found {
                    eprintln!("kit engine: kill {id:?} — no active handle");
                }
            }
            EngineCommand::Start(job) | EngineCommand::Retry { job, .. } => {
                // Register before the task exists: a Kill queued right behind
                // this Start must find the run, not race the task's first poll.
                let cancel = CancelHandle::new();
                registry.register(job.id.clone(), cancel.clone()).await;
                let tx = delta_tx.clone();
                let registry = registry.clone();
                let limiter = limiter.clone();
                tokio::spawn(async move {
                    // Honest fan-out: the run is Queued until a slot frees.
                    let _ = tx
                        .send((job.id.clone(), RunDelta::State(RunState::Queued)))
                        .await;

                    let agent = parse_agent(&job.agent).unwrap_or(kit_core::AgentKind::Codex);
                    let opts = RunOptions {
                        repo: job.repo,
                        agent,
                        task: job.task,
                        dry_run: if force_dry { Some(true) } else { None },
                        bounds: Bounds::default(),
                    };

                    // Owned permit: holds a slot until dropped (end of job or kill).
                    let permit = tokio::select! {
                        biased;
                        _ = cancel.cancelled() => None,
                        p = limiter.acquire_owned() => Some(p.expect("limiter alive")),
                    };
                    match permit {
                        Some(permit) if !cancel.is_cancelled() => {
                            let result = execute_cancellable(
                                opts.clone(),
                                Some(job.id.clone()),
                                Some(tx.clone()),
                                Some(cancel),
                            )
                            .await;
                            // The slot stays taken until the run's terminal state is out.
                            end_if_failed(result, opts, &job.id, tx).await;
                            drop(permit);
                        }
                        // Killed while queued, or between permit and execute:
                        // nothing started, so free the slot, then write the
                        // receipt before `Killed` goes out.
                        unused => {
                            drop(unused);
                            let result = finalize_killed_before_start(
                                opts.clone(),
                                job.id.clone(),
                                Some(tx.clone()),
                            )
                            .await;
                            end_if_failed(result, opts, &job.id, tx).await;
                        }
                    }
                    registry.unregister(&job.id).await;
                });
            }
        }
    }
}

/// Runner paths send their terminal state only after their receipt is
/// written, so an `Err` means the run has not ended yet. End it here as
/// `Error`, with a receipt when the disk allows, rather than strand its row.
async fn end_if_failed(
    result: anyhow::Result<RunResult>,
    opts: RunOptions,
    id: &RunId,
    tx: mpsc::Sender<(RunId, RunDelta)>,
) {
    let Err(err) = result else {
        return;
    };
    eprintln!("kit engine: {err:#}");
    if let Err(receipt_err) = finalize_failed(opts, id.clone(), &err, Some(tx)).await {
        eprintln!("kit engine: {receipt_err:#}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::paths::{
        bare_git_fixture, kit_home_test_lock, run_dir, runs_dir, worktrees_dir,
    };
    use crate::engine::registry::MAX_CONCURRENT_RUNS;
    use crate::engine::store::load_kit_config;
    use crate::engine::worktree::{resolve_repo, strip_verbatim};
    use kit_core::Receipt;
    use kit_tui::DispatchJob;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    /// Ceiling that turns a wedged engine into a failure instead of a hung CI
    /// job. No phase waits on it: each one advances on an observed delta.
    const DEADLINE: Duration = Duration::from_secs(180);

    /// The whole life of a run that held a permit and passed its gate.
    const PASSED: [RunState; 4] = [
        RunState::Queued,
        RunState::Running,
        RunState::Gating,
        RunState::Pass,
    ];

    /// Temp `KIT_HOME` plus a bare fixture repo whose only gate check blocks
    /// until [`LatchedRepo::open`]. A run keeps its permit through Gating, so
    /// shut latches pin the pool full for exactly as long as a test needs:
    /// saturation comes from the fixture, never from a sleep in the engine.
    struct LatchedRepo {
        home: PathBuf,
        repo: PathBuf,
        release: PathBuf,
    }

    impl LatchedRepo {
        /// Points `KIT_HOME` at a fresh dir; callers hold [`kit_home_test_lock`].
        fn new(tag: &str) -> Self {
            let home = std::env::temp_dir().join(format!(
                "kit-{tag}-home-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            let _ = std::fs::remove_dir_all(&home);
            std::fs::create_dir_all(&home).unwrap();
            let release = home.join("gate-release");

            // Poll for the release file. cmd has no sub-second sleep that works
            // without a console (`timeout` rejects redirected stdin), so ping
            // paces the Windows loop.
            #[cfg(windows)]
            let (script, body) = (
                home.join("gate-latch.cmd"),
                format!(
                    "@echo off\r\n:wait\r\nif exist \"{}\" exit /b 0\r\nping -n 2 127.0.0.1 >nul\r\ngoto wait\r\n",
                    release.display()
                ),
            );
            #[cfg(not(windows))]
            let (script, body) = (
                home.join("gate-latch.sh"),
                format!(
                    "#!/bin/sh\nwhile [ ! -f \"{}\" ]; do sleep 0.2; done\n",
                    release.display()
                ),
            );
            std::fs::write(&script, body).unwrap();
            #[cfg(windows)]
            let check = format!("cmd /C \"{}\"", script.display());
            #[cfg(not(windows))]
            let check = format!("sh \"{}\"", script.display());

            let repo = bare_git_fixture();
            std::fs::write(repo.join("kit.toml"), format!("[gate]\ntest = '{check}'\n")).unwrap();
            // The latch is the entire gate (never this workspace's product
            // gate), and runs resolve to the fixture (never to cwd).
            assert_eq!(
                load_kit_config(&repo).expect("kit.toml").gate.checks(),
                [("test", check.as_str())]
            );
            let resolved = resolve_repo(&repo.to_string_lossy()).expect("resolve fixture");
            assert_eq!(resolved, strip_verbatim(repo.canonicalize().unwrap()));

            // SAFETY: callers hold kit_home_test_lock.
            unsafe {
                std::env::set_var("KIT_HOME", &home);
            }
            Self {
                home,
                repo,
                release,
            }
        }

        fn open(&self) {
            std::fs::write(&self.release, b"open\n").unwrap();
        }

        /// `n` dry-run jobs. Fixed-width ids and tasks: none is a substring of
        /// another, which the log-isolation checks rely on.
        fn jobs(&self, tag: &str, n: usize) -> Vec<DispatchJob> {
            (0..n)
                .map(|i| DispatchJob {
                    id: RunId(format!("01{tag}{i:04}")),
                    repo: self.repo.to_string_lossy().into_owned(),
                    agent: "codex".into(),
                    task: format!("{tag} task {i:04}"),
                })
                .collect()
        }
    }

    impl Drop for LatchedRepo {
        fn drop(&mut self) {
            // Never strand a gate child, even after a failed assertion.
            let _ = std::fs::write(&self.release, b"open\n");
            // A test that panics before `drain` leaves its job tasks running
            // until the runtime shuts down. Unsetting KIT_HOME now would send
            // their receipts and worktrees into the real ~/.kit, so on a panic
            // KIT_HOME stays on this scratch home and the dirs are kept.
            if std::thread::panicking() {
                return;
            }
            unsafe {
                std::env::remove_var("KIT_HOME");
            }
            let _ = std::fs::remove_dir_all(&self.home);
            let _ = std::fs::remove_dir_all(&self.repo);
        }
    }

    /// After a panic, teardown leaves KIT_HOME on the scratch home (and keeps
    /// that home), so a failed test's stray jobs cannot reach the real ~/.kit.
    #[test]
    fn panicking_teardown_keeps_kit_home_on_scratch() {
        let _lock = kit_home_test_lock();
        let mut dirs = None;
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let fx = LatchedRepo::new("panic");
            dirs = Some((fx.home.clone(), fx.repo.clone()));
            panic!("simulated test failure before drain");
        }));
        assert!(failed.is_err());
        let (home, repo) = dirs.expect("fixture was built");
        assert_eq!(
            std::env::var_os("KIT_HOME"),
            Some(home.clone().into_os_string())
        );
        assert!(
            home.is_dir(),
            "a panicking teardown must keep the scratch home"
        );
        // Clean up what the panicking teardown deliberately kept.
        unsafe {
            std::env::remove_var("KIT_HOME");
        }
        let _ = std::fs::remove_dir_all(&home);
        let _ = std::fs::remove_dir_all(&repo);
    }

    /// The delta stream in channel order: exactly what the Control Room sees.
    #[derive(Default)]
    struct Ledger {
        states: HashMap<RunId, Vec<RunState>>,
        worktrees: HashMap<RunId, Vec<PathBuf>>,
        output: HashMap<RunId, String>,
        /// Runs between `Running` and a terminal state.
        running: HashSet<RunId>,
    }

    impl Ledger {
        /// Checks the stream invariants on every delta. A run opens with
        /// `Queued` and never sees it again: a late Queued would regress a
        /// Running row, or revive a finished one, in the Control Room. Nothing
        /// follows a terminal state, and no more than the cap are ever
        /// `Running` at once.
        fn record(&mut self, id: RunId, delta: RunDelta) {
            match delta {
                RunDelta::State(state) => {
                    let seen = self.states.entry(id.clone()).or_default();
                    assert!(
                        !seen.is_empty() || state == RunState::Queued,
                        "{id}: first state {state:?}, expected Queued"
                    );
                    assert!(
                        seen.is_empty() || state != RunState::Queued,
                        "{id}: Queued after {seen:?}"
                    );
                    assert!(
                        !seen.last().is_some_and(|s| s.is_terminal()),
                        "{id}: {state:?} after terminal {seen:?}"
                    );
                    seen.push(state);
                    if state == RunState::Running {
                        self.running.insert(id);
                    } else if state.is_terminal() {
                        self.running.remove(&id);
                    }
                    assert!(
                        self.running.len() <= MAX_CONCURRENT_RUNS,
                        "{} runs Running at once, cap is {MAX_CONCURRENT_RUNS}",
                        self.running.len()
                    );
                }
                RunDelta::Output(chunk) => self.output.entry(id).or_default().push_str(&chunk),
                RunDelta::Worktree(path) => self.worktrees.entry(id).or_default().push(path),
                RunDelta::Gate(_) => {}
            }
        }

        /// Runs whose latest state is `state`.
        fn ids(&self, state: RunState) -> Vec<RunId> {
            self.states
                .iter()
                .filter(|(_, seen)| seen.last() == Some(&state))
                .map(|(id, _)| id.clone())
                .collect()
        }
    }

    /// Drives the real supervisor body (the one `spawn_production` runs),
    /// forced dry so no live agent can spawn: codex may be on PATH.
    struct Harness {
        cmd_tx: Option<mpsc::Sender<EngineCommand>>,
        delta_rx: mpsc::Receiver<(RunId, RunDelta)>,
        deadline: tokio::time::Instant,
        ledger: Ledger,
    }

    impl Harness {
        fn start() -> Self {
            let (cmd_tx, cmd_rx) = mpsc::channel(64);
            let (delta_tx, delta_rx) = mpsc::channel(256);
            tokio::spawn(run_supervisor(cmd_rx, delta_tx, true));
            Self {
                cmd_tx: Some(cmd_tx),
                delta_rx,
                deadline: tokio::time::Instant::now() + DEADLINE,
                ledger: Ledger::default(),
            }
        }

        async fn send(&self, cmd: EngineCommand) {
            let tx = self.cmd_tx.as_ref().expect("command pipe open");
            tx.send(cmd).await.expect("supervisor alive");
        }

        /// Record deltas until `done` holds.
        async fn until(&mut self, what: &str, done: impl Fn(&Ledger) -> bool) {
            while !done(&self.ledger) {
                let Some((id, delta)) = self.next(what).await else {
                    panic!("delta stream ended before {what}");
                };
                self.ledger.record(id, delta);
            }
        }

        /// Close the command pipe and record the rest of the stream. It ends
        /// only once the supervisor and every job task have dropped their
        /// senders, which is after the last receipt is on disk. By then every
        /// run must have ended: a terminal state (`record` forbids a second
        /// one) and a receipt.
        async fn drain(&mut self) {
            self.cmd_tx = None;
            while let Some((id, delta)) = self.next("the stream closed").await {
                self.ledger.record(id, delta);
            }
            for (id, seen) in &self.ledger.states {
                assert!(
                    seen.last().is_some_and(|s| s.is_terminal()),
                    "{id} never ended: {seen:?}"
                );
                assert!(
                    run_dir(&id.0).join("receipt.json").is_file(),
                    "{id} ended without a receipt"
                );
            }
        }

        async fn next(&mut self, what: &str) -> Option<(RunId, RunDelta)> {
            tokio::time::timeout_at(self.deadline, self.delta_rx.recv())
                .await
                .unwrap_or_else(|_| panic!("deadline passed before {what}"))
        }
    }

    fn receipt(id: &RunId) -> Receipt {
        let path = run_dir(&id.0).join("receipt.json");
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{id}: no receipt at {}: {e}", path.display()));
        serde_json::from_str(&raw).expect("parse receipt")
    }

    /// P3 from product state: 12 dispatches (one a retry) against a cap of 8.
    /// The stream never shows more than 8 `Running`, the pool provably fills
    /// while the rest wait `Queued`, and every run leaves its own receipt,
    /// worktree, and log.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn p3_twelve_jobs_never_exceed_eight_concurrent() {
        let _lock = kit_home_test_lock();
        let fx = LatchedRepo::new("p3");
        let jobs = fx.jobs("P3PROOF", 12);
        let mut h = Harness::start();
        let (retry, starts) = jobs.split_last().unwrap();
        for job in starts {
            h.send(EngineCommand::Start(job.clone())).await;
        }
        h.send(EngineCommand::Retry {
            source_id: jobs[0].id.clone(),
            job: retry.clone(),
        })
        .await;

        // Shut latches pin 8 runs in Gating; the other 4 must wait Queued.
        let waiting = jobs.len() - MAX_CONCURRENT_RUNS;
        h.until("the pool filled", |l| {
            l.ids(RunState::Gating).len() == MAX_CONCURRENT_RUNS
                && l.ids(RunState::Queued).len() == waiting
        })
        .await;
        assert_eq!(h.ledger.running.len(), MAX_CONCURRENT_RUNS);
        for id in h.ledger.ids(RunState::Gating) {
            // Each pinned run works in its own live checkout...
            let dest = worktrees_dir().join(&id.0);
            assert_eq!(h.ledger.worktrees[&id], std::slice::from_ref(&dest));
            assert!(
                dest.join("README.md").is_file(),
                "{id}: no checkout at {}",
                dest.display()
            );
        }
        for id in h.ledger.ids(RunState::Queued) {
            // ...while a queued run has created nothing yet.
            assert!(
                !worktrees_dir().join(&id.0).exists(),
                "{id}: queued run already has a worktree"
            );
        }

        fx.open();
        h.drain().await;

        // One receipt dir per run, keyed by its id, and nothing else.
        let mut dirs: Vec<String> = std::fs::read_dir(runs_dir())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        dirs.sort();
        let mut ids: Vec<String> = jobs.iter().map(|j| j.id.0.clone()).collect();
        ids.sort();
        assert_eq!(dirs, ids);

        let mut dests = HashSet::new();
        for job in &jobs {
            let id = &job.id;
            assert_eq!(h.ledger.states[id], PASSED, "{id}");
            let receipt = receipt(id);
            assert_eq!(&receipt.id, id);
            assert_eq!(receipt.state, RunState::Pass, "{id}");
            assert_eq!(receipt.spec.task, job.task);
            assert!(receipt.ended_at.is_some(), "{id}: no ended_at");
            assert!(
                receipt
                    .gate
                    .as_ref()
                    .is_some_and(|g| g.passed && g.checks.len() == 1),
                "{id}: the latch check must have run"
            );

            // One worktree per run, keyed by its id.
            let dest = worktrees_dir().join(&id.0);
            assert_eq!(h.ledger.worktrees[id], std::slice::from_ref(&dest));
            assert!(dests.insert(dest.clone()), "{id}: shared worktree");

            // Its log, on disk and on the wire, is its own and only its own.
            let log = std::fs::read_to_string(run_dir(&id.0).join("output.log")).unwrap();
            for text in [&log, &h.ledger.output[id]] {
                assert!(
                    text.contains(&format!("task: {}", job.task)),
                    "{id}: log lacks its task"
                );
                assert!(
                    text.contains(&dest.display().to_string()),
                    "{id}: log lacks its worktree"
                );
                for other in jobs.iter().filter(|o| o.id != *id) {
                    assert!(
                        !text.contains(&other.id.0),
                        "{id}: log mentions {}",
                        other.id
                    );
                    assert!(
                        !text.contains(&other.task),
                        "{id}: log has {}'s task",
                        other.id
                    );
                }
            }
        }
        assert_eq!(dests.len(), jobs.len());
    }

    /// Kill a run still waiting for a permit: it never starts and gets no
    /// worktree, yet its killed receipt is on disk before `Killed` is sent.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn kill_while_queued_writes_receipt_without_worktree() {
        let _lock = kit_home_test_lock();
        let fx = LatchedRepo::new("killq");
        let jobs = fx.jobs("KILLQ", MAX_CONCURRENT_RUNS + 1);
        let mut h = Harness::start();
        for job in &jobs {
            h.send(EngineCommand::Start(job.clone())).await;
        }
        h.until("the pool filled", |l| {
            l.ids(RunState::Gating).len() == MAX_CONCURRENT_RUNS
                && l.ids(RunState::Queued).len() == 1
        })
        .await;
        let victim = h.ledger.ids(RunState::Queued).remove(0);

        h.send(EngineCommand::Kill { id: victim.clone() }).await;
        h.until("the queued run was killed", |l| {
            l.states[&victim].last() == Some(&RunState::Killed)
        })
        .await;
        // Read the moment `Killed` arrives: the receipt must already exist.
        let killed = receipt(&victim);
        assert_eq!(killed.state, RunState::Killed);
        assert!(killed.ended_at.is_some(), "a killed receipt needs ended_at");
        assert_eq!(killed.started_at, None, "a queued run never started");
        assert_eq!(killed.spec.branch, None, "no worktree, so no branch");
        assert!(killed.diff.is_empty());
        assert!(killed.gate.is_none());
        let log = std::fs::read_to_string(run_dir(&victim.0).join("output.log")).unwrap();
        assert!(log.contains("killed while queued"), "{log}");
        assert_eq!(
            h.ledger.states[&victim],
            [RunState::Queued, RunState::Killed]
        );
        assert!(!h.ledger.worktrees.contains_key(&victim));
        assert!(
            !worktrees_dir().join(&victim.0).exists(),
            "a queued kill must not create a worktree"
        );

        fx.open();
        h.drain().await;
        for job in jobs.iter().filter(|j| j.id != victim) {
            assert_eq!(h.ledger.states[&job.id], PASSED, "{}", job.id);
            assert_eq!(receipt(&job.id).state, RunState::Pass);
        }
        assert!(!worktrees_dir().join(&victim.0).exists());
    }

    /// A Kill right behind its Start must still find the run. Current-thread
    /// flavor: the job task cannot run before the supervisor has handled both
    /// commands, so registering inside the task would lose this kill every time.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test]
    async fn kill_right_after_start_is_not_lost() {
        let _lock = kit_home_test_lock();
        let fx = LatchedRepo::new("killstart");
        // A lost kill should fail fast as a Pass, not wait on the latch.
        fx.open();
        let job = fx.jobs("KILLSTART", 1).remove(0);
        let mut h = Harness::start();
        h.send(EngineCommand::Start(job.clone())).await;
        h.send(EngineCommand::Kill { id: job.id.clone() }).await;
        h.drain().await;
        assert_eq!(
            h.ledger.states[&job.id],
            [RunState::Queued, RunState::Killed]
        );
        assert_eq!(receipt(&job.id).state, RunState::Killed);
        assert!(!worktrees_dir().join(&job.id.0).exists());
    }

    /// A run whose own path fails (here its repo cannot resolve) still ends:
    /// the error is in its stream and log, it gets an `Error` receipt, and it
    /// sends one terminal state instead of sticking at Queued or Running.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn failed_run_ends_with_error_receipt() {
        let _lock = kit_home_test_lock();
        let fx = LatchedRepo::new("fail");
        // A plain file is never a git checkout, whatever directory holds it.
        let not_a_repo = fx.home.join("not-a-repo.txt");
        std::fs::write(&not_a_repo, b"plain file\n").unwrap();
        let mut job = fx.jobs("FAIL", 1).remove(0);
        job.repo = not_a_repo.to_string_lossy().into_owned();
        let mut h = Harness::start();
        h.send(EngineCommand::Start(job.clone())).await;
        h.drain().await;
        assert_eq!(
            h.ledger.states[&job.id],
            [RunState::Queued, RunState::Error]
        );
        assert!(
            h.ledger.output[&job.id].contains("not a git repository"),
            "{}",
            h.ledger.output[&job.id]
        );
        let failed = receipt(&job.id);
        assert_eq!(failed.state, RunState::Error);
        assert!(failed.ended_at.is_some(), "an error receipt needs ended_at");
        let log = std::fs::read_to_string(run_dir(&job.id.0).join("output.log")).unwrap();
        assert!(log.contains("not a git repository"), "{log}");
    }
}
