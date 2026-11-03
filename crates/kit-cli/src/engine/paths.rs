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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Parallel tests can read the same clock tick; without the counter two
    // fixtures share a dir and one wipes the other mid-`git init`.
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kit-bare-repo-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("bare fixture dir");
    std::fs::write(dir.join("README.md"), "kit test fixture\n").expect("fixture file");

    let git = |args: &[&str]| {
        let mut cmd = Command::new("git");
        cmd.args([
            "-c",
            "user.name=kit",
            "-c",
            "user.email=kit@test",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(&dir)
        .env("GIT_AUTHOR_NAME", "kit")
        .env("GIT_AUTHOR_EMAIL", "kit@test")
        .env("GIT_COMMITTER_NAME", "kit")
        .env("GIT_COMMITTER_EMAIL", "kit@test")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        // Inherited GIT_DIR would init/commit against this workspace, which
        // has the product kit.toml (fmt+clippy+cargo test --workspace, 15m).
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_PREFIX")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
        cmd.status().expect("git")
    };
    assert!(git(&["init", "-q"]).success(), "git init");
    let _ = git(&["checkout", "-q", "-b", "main"]);
    assert!(git(&["add", "."]).success(), "git add");
    assert!(git(&["commit", "-q", "-m", "init"]).success(), "git commit");
    assert!(
        !dir.join("kit.toml").exists(),
        "bare_git_fixture must not contain kit.toml"
    );
    assert!(
        !dir.join("Cargo.toml").exists(),
        "bare_git_fixture must not contain Cargo.toml"
    );
    dir
}

use std::path::PathBuf;

/// Resolve Kit's home directory.
///
/// Order: `KIT_HOME` → `HOME`/`USERPROFILE` + `/.kit` → `./.kit` fallback.
/// Empty env values count as unset (e.g. a blank CI secret), so we do not
/// treat `""` as a relative root under the current directory (#61).
pub fn kit_home() -> PathBuf {
    if let Ok(p) = std::env::var("KIT_HOME")
        && !p.is_empty()
    {
        return PathBuf::from(p);
    }
    for key in ["HOME", "USERPROFILE"] {
        if let Ok(home) = std::env::var(key)
            && !home.is_empty()
        {
            return PathBuf::from(home).join(".kit");
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_git_fixture_has_no_product_gate() {
        let dir = bare_git_fixture();
        assert!(
            !dir.join("kit.toml").exists(),
            "P3 fixture must not carry workspace kit.toml"
        );
        assert!(
            !dir.join("Cargo.toml").exists(),
            "P3 fixture must not infer cargo workspace checks"
        );
        assert!(
            dir.join(".git").exists(),
            "fixture must be its own git checkout"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_kit_home_is_ignored() {
        let _lock = kit_home_test_lock();
        let prev_kit = std::env::var_os("KIT_HOME");
        let prev_home = std::env::var_os("HOME");
        let prev_profile = std::env::var_os("USERPROFILE");
        // SAFETY: kit_home_test_lock serializes env mutation for these tests.
        unsafe {
            std::env::set_var("KIT_HOME", "");
            std::env::set_var("HOME", "/tmp/kit-home-fallback-test");
            std::env::remove_var("USERPROFILE");
        }
        assert_eq!(
            kit_home(),
            PathBuf::from("/tmp/kit-home-fallback-test").join(".kit")
        );
        unsafe {
            std::env::set_var("HOME", "");
            std::env::set_var("USERPROFILE", "/tmp/kit-profile-fallback-test");
        }
        assert_eq!(
            kit_home(),
            PathBuf::from("/tmp/kit-profile-fallback-test").join(".kit")
        );
        unsafe {
            match prev_kit {
                Some(v) => std::env::set_var("KIT_HOME", v),
                None => std::env::remove_var("KIT_HOME"),
            }
            match prev_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
            match prev_profile {
                Some(v) => std::env::set_var("USERPROFILE", v),
                None => std::env::remove_var("USERPROFILE"),
            }
        }
    }
}
