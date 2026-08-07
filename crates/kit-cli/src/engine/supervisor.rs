//! Control Room engine supervisor — kill registry + max-N concurrency.
//!
//! Extracted from `main` so P3 kill criteria can be proven in tests without
//! a full TUI: dispatch N jobs, never more than [`MAX_CONCURRENT_RUNS`] hold
//! a permit at once, all reach a terminal outcome.

use super::cancel::CancelHandle;
use super::registry::{MAX_CONCURRENT_RUNS, RunRegistry, concurrency_limiter};
use super::runner::{RunOptions, execute_cancellable, parse_agent};
use kit_core::{Bounds, RunDelta, RunId, RunState};
use kit_tui::{DispatchJob, EngineCommand};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;

/// Live counts for P3 proof / observability.
#[derive(Debug, Default)]
#[allow(dead_code)] // constructed in tests + proof harness
pub struct ConcurrencyProbe {
    /// Jobs currently holding a semaphore permit (agent+gate in flight).
    pub in_flight: AtomicUsize,
    /// High-water mark of `in_flight`.
    pub max_in_flight: AtomicUsize,
    /// Jobs that finished (any terminal path).
    pub finished: AtomicUsize,
}

impl ConcurrencyProbe {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn enter(&self) {
        let n = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_in_flight.fetch_max(n, Ordering::SeqCst);
    }

    fn leave(&self) {
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        self.finished.fetch_add(1, Ordering::SeqCst);
    }

    pub fn max_seen(&self) -> usize {
        self.max_in_flight.load(Ordering::SeqCst)
    }

    pub fn finished_count(&self) -> usize {
        self.finished.load(Ordering::SeqCst)
    }

    pub fn in_flight_now(&self) -> usize {
        self.in_flight.load(Ordering::SeqCst)
    }
}

/// Spawn the production supervisor (used by `kit` TUI entry).
pub fn spawn_production(
    cmd_rx: mpsc::Receiver<EngineCommand>,
    delta_tx: mpsc::Sender<(RunId, RunDelta)>,
) {
    tokio::spawn(async move {
        run_supervisor(cmd_rx, delta_tx, None, false).await;
    });
}

/// Supervisor loop. When `probe` is set, tracks concurrency for tests.
/// When `force_dry` is true, every job runs offline (P3 harness / CI).
pub async fn run_supervisor(
    mut cmd_rx: mpsc::Receiver<EngineCommand>,
    delta_tx: mpsc::Sender<(RunId, RunDelta)>,
    probe: Option<Arc<ConcurrencyProbe>>,
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
                let tx = delta_tx.clone();
                let registry = registry.clone();
                let limiter = limiter.clone();
                let probe = probe.clone();
                let cancel = CancelHandle::new();
                let job_id = job.id.clone();
                let reg_for_register = registry.clone();
                let cancel_for_register = cancel.clone();
                tokio::spawn(async move {
                    reg_for_register
                        .register(job_id.clone(), cancel_for_register)
                        .await;

                    // Owned permit: holds a slot until dropped (end of job or kill).
                    let permit = tokio::select! {
                        biased;
                        _ = cancel.cancelled() => {
                            registry.unregister(&job_id).await;
                            let _ = tx
                                .send((job_id.clone(), RunDelta::State(RunState::Killed)))
                                .await;
                            if let Some(p) = &probe {
                                p.finished.fetch_add(1, Ordering::SeqCst);
                            }
                            return;
                        }
                        p = limiter.clone().acquire_owned() => p.expect("limiter alive"),
                    };

                    if cancel.is_cancelled() {
                        drop(permit);
                        registry.unregister(&job_id).await;
                        let _ = tx
                            .send((job_id.clone(), RunDelta::State(RunState::Killed)))
                            .await;
                        if let Some(p) = &probe {
                            p.finished.fetch_add(1, Ordering::SeqCst);
                        }
                        return;
                    }

                    // Count in-flight strictly inside the permit hold window.
                    // leave() MUST run before drop(permit) or another task can
                    // acquire and enter while this task still shows as in-flight.
                    if let Some(p) = &probe {
                        p.enter();
                        let now = p.in_flight_now();
                        assert!(
                            now <= MAX_CONCURRENT_RUNS,
                            "concurrency breach: {now} > {MAX_CONCURRENT_RUNS}"
                        );
                        // Hold the slot briefly so the 12-job harness saturates
                        // the pool (dry-run alone can finish before the 9th starts).
                        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                    }

                    let agent = parse_agent(&job.agent).unwrap_or(kit_core::AgentKind::Codex);
                    let opts = RunOptions {
                        repo: job.repo,
                        agent,
                        task: job.task,
                        dry_run: if force_dry { Some(true) } else { None },
                        bounds: Bounds::default(),
                    };
                    if let Err(err) =
                        execute_cancellable(opts, Some(job.id.clone()), Some(tx), Some(cancel))
                            .await
                    {
                        eprintln!("kit engine: {err:#}");
                    }
                    if let Some(p) = &probe {
                        p.leave();
                    }
                    drop(permit);
                    registry.unregister(&job.id).await;
                });
            }
        }
    }
}

/// P3 harness: fan out `n` dry-run jobs through the real supervisor.
/// Returns probe stats once all jobs finish (or timeout).
#[cfg_attr(not(test), allow(dead_code))]
pub async fn proof_dispatch_n(
    n: usize,
    repo: String,
    timeout: std::time::Duration,
) -> Result<Arc<ConcurrencyProbe>, String> {
    let (delta_tx, mut delta_rx) = mpsc::channel::<(RunId, RunDelta)>(256);
    let (cmd_tx, cmd_rx) = mpsc::channel::<EngineCommand>(64);
    let probe = ConcurrencyProbe::new();
    let probe_sup = probe.clone();

    let sup = tokio::spawn(async move {
        run_supervisor(cmd_rx, delta_tx, Some(probe_sup), true).await;
    });

    for i in 0..n {
        let id = RunId(format!("01P3PROOF{i:020}"));
        let job = DispatchJob {
            id,
            repo: repo.clone(),
            agent: "codex".into(),
            task: format!("p3 proof job {i}"),
        };
        cmd_tx
            .send(EngineCommand::Start(job))
            .await
            .map_err(|e| e.to_string())?;
    }
    // Close command stream after enqueue so supervisor can idle-exit when done.
    drop(cmd_tx);

    // Drain deltas until finished count hits n (or timeout).
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if probe.finished_count() >= n {
            break;
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(format!(
                "timeout: finished {}/{} max_in_flight={}",
                probe.finished_count(),
                n,
                probe.max_seen()
            ));
        }
        tokio::select! {
            _ = delta_rx.recv() => {}
            _ = tokio::time::sleep(std::time::Duration::from_millis(20)) => {}
        }
    }

    // Supervisor task ends when cmd channel closed and children finish.
    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), sup).await;
    Ok(probe)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::paths::kit_home_test_lock;
    use std::path::PathBuf;
    use std::time::Duration;

    fn kit_repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root")
    }

    /// Full P3 harness: 12 dry-run jobs through the real supervisor path.
    #[allow(clippy::await_holding_lock)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn p3_twelve_jobs_never_exceed_eight_concurrent() {
        let _guard = kit_home_test_lock();
        let home = std::env::temp_dir().join(format!(
            "kit-p3-home-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        unsafe {
            std::env::set_var("KIT_HOME", &home);
        }

        let repo = kit_repo_root().to_string_lossy().into_owned();
        let result = proof_dispatch_n(12, repo, Duration::from_secs(180)).await;

        unsafe {
            std::env::remove_var("KIT_HOME");
        }
        let _ = std::fs::remove_dir_all(&home);

        let probe = result.expect("p3 proof");
        assert_eq!(
            probe.finished_count(),
            12,
            "all 12 jobs must reach a terminal path"
        );
        assert!(
            probe.max_seen() <= MAX_CONCURRENT_RUNS,
            "max in-flight {} exceeds cap {}",
            probe.max_seen(),
            MAX_CONCURRENT_RUNS
        );
        assert!(
            probe.max_seen() >= 1,
            "expected at least one concurrent slot used"
        );
        // With 12 jobs and worktree work, we should saturate the pool.
        assert_eq!(
            probe.max_seen(),
            MAX_CONCURRENT_RUNS,
            "expected full saturation at {MAX_CONCURRENT_RUNS}, saw {}",
            probe.max_seen()
        );
        assert_eq!(probe.in_flight_now(), 0, "no leaked permits");
    }

    #[tokio::test]
    async fn synthetic_semaphore_caps_at_eight_under_load() {
        let limiter = concurrency_limiter();
        let probe = ConcurrencyProbe::new();
        let mut handles = Vec::new();
        for _ in 0..12 {
            let limiter = limiter.clone();
            let probe = probe.clone();
            handles.push(tokio::spawn(async move {
                let _p = limiter.acquire().await.unwrap();
                probe.enter();
                assert!(probe.in_flight_now() <= MAX_CONCURRENT_RUNS);
                tokio::time::sleep(Duration::from_millis(30)).await;
                probe.leave();
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(probe.max_seen(), MAX_CONCURRENT_RUNS);
        assert_eq!(probe.finished_count(), 12);
    }
}
