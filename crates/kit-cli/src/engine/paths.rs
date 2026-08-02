//! Production paths for Kit data under `$KIT_HOME` (default `~/.kit`).

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
