//! Production paths for Kit data under `$KIT_HOME` (default `~/.kit`).

#[cfg(test)]
use std::sync::{Mutex, OnceLock};

/// Serialize tests that mutate process-global `KIT_HOME`.
/// Recovers from poison so one failed test does not cascade.
#[cfg(test)]
pub fn kit_home_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let m = LOCK.get_or_init(|| Mutex::new(()));
    m.lock().unwrap_or_else(|p| p.into_inner())
}

use std::path::PathBuf;

/// Resolve Kit's home directory.
///
/// Order: `KIT_HOME` → `HOME`/`USERPROFILE` + `/.kit` → `./.kit` fallback.
pub fn kit_home() -> PathBuf {
    if let Ok(p) = std::env::var("KIT_HOME") {
        return PathBuf::from(p);
    }
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        return PathBuf::from(home).join(".kit");
    }
    PathBuf::from(".kit")
}

pub fn runs_dir() -> PathBuf {
    kit_home().join("runs")
}

pub fn worktrees_dir() -> PathBuf {
    kit_home().join("worktrees")
}

pub fn run_dir(id: &str) -> PathBuf {
    runs_dir().join(id)
}
