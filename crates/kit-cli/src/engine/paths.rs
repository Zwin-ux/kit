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

/// Tiny git repo **without** `kit.toml`, for dry-run / supervisor tests.
///
/// Engine tests must not load this workspace's product gate (fmt + clippy +
/// test). A real `kit.toml` here would turn the P3 12-job harness into 12
/// full workspace builds.
#[cfg(test)]
pub fn bare_git_fixture() -> std::path::PathBuf {
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    let dir = std::env::temp_dir().join(format!(
        "kit-bare-repo-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("bare fixture dir");
    std::fs::write(dir.join("README.md"), "kit test fixture\n").expect("fixture file");

    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(&dir)
            .env("GIT_AUTHOR_NAME", "kit")
            .env("GIT_AUTHOR_EMAIL", "kit@test")
            .env("GIT_COMMITTER_NAME", "kit")
            .env("GIT_COMMITTER_EMAIL", "kit@test")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("git")
    };
    assert!(git(&["init", "-q"]).success(), "git init");
    let _ = git(&["checkout", "-q", "-b", "main"]);
    assert!(git(&["add", "."]).success(), "git add");
    assert!(git(&["commit", "-q", "-m", "init"]).success(), "git commit");
    dir
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
