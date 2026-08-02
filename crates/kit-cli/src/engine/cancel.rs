//! Cooperative cancellation for in-flight runs.
//!
//! Lightweight alternative to `tokio_util::sync::CancellationToken` — no extra
//! dependency. Engine supervisor holds `Arc<CancelHandle>` per active RunId;
//! kill flips the flag and notifies waiters so agent loops exit promptly.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Notify;

/// Shared cancel flag for one run.
#[derive(Debug, Default)]
pub struct CancelHandle {
    cancelled: AtomicBool,
    notify: Notify,
}

impl CancelHandle {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Mark cancelled and wake anyone in [`Self::cancelled`].
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Completes when [`Self::cancel`] is called (or already cancelled).
    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        // Race: cancel between check and notified — re-check after wait.
        self.notify.notified().await;
        // If we woke spuriously without cancel, loop.
        while !self.is_cancelled() {
            self.notify.notified().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn cancel_wakes_waiter() {
        let h = CancelHandle::new();
        let h2 = h.clone();
        let task = tokio::spawn(async move {
            h2.cancelled().await;
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        h.cancel();
        tokio::time::timeout(Duration::from_secs(1), task)
            .await
            .expect("timeout")
            .expect("join");
        assert!(h.is_cancelled());
    }

    #[tokio::test]
    async fn already_cancelled_returns_immediately() {
        let h = CancelHandle::new();
        h.cancel();
        tokio::time::timeout(Duration::from_millis(50), h.cancelled())
            .await
            .expect("should not wait");
    }
}
