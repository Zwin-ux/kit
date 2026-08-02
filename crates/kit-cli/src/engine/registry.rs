//! In-memory run handle registry — kill targets + concurrency cap.
//!
//! Owned by the Control Room engine supervisor task in `main`. Not a global.

use super::cancel::CancelHandle;
use kit_core::RunId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// Default max concurrent live/dry runs (PRD / wiki P1).
pub const MAX_CONCURRENT_RUNS: usize = 8;

/// Maps active RunId → cancel handle so `k` can stop a process.
#[derive(Debug, Default)]
pub struct RunRegistry {
    inner: Mutex<HashMap<RunId, Arc<CancelHandle>>>,
}

impl RunRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn register(&self, id: RunId, cancel: Arc<CancelHandle>) {
        self.inner.lock().await.insert(id, cancel);
    }

    pub async fn unregister(&self, id: &RunId) {
        self.inner.lock().await.remove(id);
    }

    /// Request kill. Returns true if a handle was present (run still active).
    pub async fn kill(&self, id: &RunId) -> bool {
        if let Some(h) = self.inner.lock().await.get(id) {
            h.cancel();
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    pub async fn active_count(&self) -> usize {
        self.inner.lock().await.len()
    }
}

/// Shared concurrency limiter (max 8).
pub fn concurrency_limiter() -> Arc<Semaphore> {
    Arc::new(Semaphore::new(MAX_CONCURRENT_RUNS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn kill_unknown_is_false() {
        let reg = RunRegistry::new();
        assert!(!reg.kill(&RunId("nope".into())).await);
    }

    #[tokio::test]
    async fn register_kill_unregister() {
        let reg = RunRegistry::new();
        let id = RunId("run1".into());
        let cancel = CancelHandle::new();
        reg.register(id.clone(), cancel.clone()).await;
        assert_eq!(reg.active_count().await, 1);
        assert!(reg.kill(&id).await);
        assert!(cancel.is_cancelled());
        reg.unregister(&id).await;
        assert_eq!(reg.active_count().await, 0);
    }

    #[tokio::test]
    async fn semaphore_caps_at_eight() {
        let sem = concurrency_limiter();
        let mut permits = Vec::new();
        for _ in 0..MAX_CONCURRENT_RUNS {
            permits.push(sem.try_acquire().expect("slot"));
        }
        assert!(sem.try_acquire().is_err());
        drop(permits);
        assert!(sem.try_acquire().is_ok());
    }
}
