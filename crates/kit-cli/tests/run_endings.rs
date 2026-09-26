//! Every way a headless `kit run` ends leaves a receipt and, under `--json`,
//! exactly one envelope on stdout: a run that cannot start, and Ctrl-C.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

fn scratch(tag: &str) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kit-run-endings-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

fn kit(home: &Path, cwd: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kit"));
    cmd.current_dir(cwd)
        .env("KIT_HOME", home)
        .env_remove("KIT_FULL_AUTO")
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE");
    cmd
}

fn envelope(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout)
        .unwrap_or_else(|e| panic!("stdout is not one JSON value ({e}): {}", text(stdout)))
}

fn assert_receipt(env: &Value, state: &str) {
    assert_eq!(env["data"]["state"], state, "{env}");
    let dir = env["data"]["receiptDir"].as_str().expect("receiptDir");
    let receipt: Value = serde_json::from_str(
        &std::fs::read_to_string(Path::new(dir).join("receipt.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        receipt["state"]
            .as_str()
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some(state),
        "{receipt}"
    );
}

/// Before: `kit run` returned the error from `main`, so there was no receipt
/// and `--json` printed no envelope at all.
#[test]
fn run_that_cannot_start_leaves_an_error_receipt_and_envelope() {
    let home = scratch("home");
    let not_a_repo = scratch("not-a-repo");
    let out = kit(&home, &not_a_repo)
        .args(["run", "--dry-run", "--json", "--task", "smoke"])
        .output()
        .expect("run kit");
    assert_eq!(out.status.code(), Some(2), "{}", text(&out.stderr));
    let env = envelope(&out.stdout);
    assert_eq!(env["ok"], false, "{env}");
    assert!(
        env["error"]
            .as_str()
            .is_some_and(|e| e.contains("not a git repository")),
        "the envelope says why: {env}"
    );
    assert_receipt(&env, "error");
}

/// Before: Ctrl-C killed `kit` itself, leaving no receipt, no envelope and the
/// worktree behind.
#[cfg(unix)]
#[test]
fn ctrl_c_ends_the_run_as_killed_with_a_receipt() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};

    let home = scratch("home");
    let repo = scratch("repo");
    let bin = scratch("bin");
    let agent = bin.join("claude");
    std::fs::write(
        &agent,
        "#!/bin/sh\ncase \"$1\" in\n  --version) echo 0.0.0 fake ;;\n  auth) exit 1 ;;\n  *) sleep 30 ;;\nesac\n",
    )
    .unwrap();
    std::fs::set_permissions(&agent, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(repo.join("kit.toml"), "[gate]\ntest = 'git --version'\n").unwrap();
    for args in [
        &["init", "-q", "-b", "main"][..],
        &["add", "."],
        &[
            "-c",
            "user.name=kit",
            "-c",
            "user.email=kit@test",
            "commit",
            "-q",
            "-m",
            "init",
        ],
    ] {
        let ok = Command::new("git")
            .args(args)
            .current_dir(&repo)
            .status()
            .unwrap();
        assert!(ok.success(), "git {args:?}");
    }

    let path = std::env::var_os("PATH").unwrap_or_default();
    let path =
        std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(&path))).unwrap();
    let child = kit(&home, &repo)
        .args(["run", "--agent", "claude", "--json", "--task", "wait"])
        .env("PATH", path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn kit");

    // Wait until the agent is running in its worktree.
    let worktrees = home.join("worktrees");
    let deadline = Instant::now() + Duration::from_secs(20);
    while std::fs::read_dir(&worktrees).map_or(true, |mut d| d.next().is_none()) {
        assert!(Instant::now() < deadline, "run never reached its worktree");
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_millis(500));

    let sent = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(sent.success());
    let started = Instant::now();
    let out = child.wait_with_output().expect("kit exits");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "Ctrl-C must stop the agent, not wait it out"
    );
    let env = envelope(&out.stdout);
    assert_eq!(env["ok"], false, "{env}\n{}", text(&out.stderr));
    assert_receipt(&env, "killed");
}
