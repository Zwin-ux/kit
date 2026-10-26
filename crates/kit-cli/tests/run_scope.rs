//! The scope gate covers the receipt artifact, including agent commits.
#![cfg(unix)]

use serde_json::Value;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture {
    root: PathBuf,
    repo: PathBuf,
    bin: PathBuf,
}

fn git(repo: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "user.name=kit", "-c", "user.email=kit@test"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(repo)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

impl Fixture {
    fn new(edits: &str, gate: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "kit-run-scope-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let repo = root.join("repo");
        let bin = root.join("bin");
        std::fs::create_dir(&bin).unwrap();
        std::fs::create_dir_all(repo.join(".github")).unwrap();
        std::fs::create_dir(repo.join("src")).unwrap();
        std::fs::write(repo.join(".github/original.txt"), "original\n").unwrap();
        std::fs::write(
            repo.join("kit.toml"),
            format!(
                "[gate]\ntest = '{gate}'\n[gate.scope]\nallow = [\"src/**\"]\ndeny = [\".github/**\"]\n"
            ),
        )
        .unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "baseline"]);
        let fake = bin.join("claude");
        std::fs::write(
            &fake,
            format!(
                "#!/bin/sh\nset -e\ncase \"$1\" in\n --version) echo '0.0.0 fake'; exit 0;;\n auth) echo '{{\"loggedIn\":true}}'; exit 0;;\nesac\nmkdir -p src\n{edits}\n"
            ),
        )
        .unwrap();
        std::fs::set_permissions(fake, std::fs::Permissions::from_mode(0o755)).unwrap();
        Self { root, repo, bin }
    }

    fn kit(&self, args: &[&str]) -> (Output, Value) {
        let path = std::env::join_paths(std::iter::once(self.bin.clone()).chain(
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
        ))
        .unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(args)
            .arg("--json")
            .current_dir(&self.repo)
            .env("PATH", path)
            .env("KIT_HOME", self.root.join("kit-home"))
            // `kit land` commits; CI runners have no git identity.
            .env("GIT_AUTHOR_NAME", "kit")
            .env("GIT_AUTHOR_EMAIL", "kit@test")
            .env("GIT_COMMITTER_NAME", "kit")
            .env("GIT_COMMITTER_EMAIL", "kit@test")
            .env_remove("KIT_FULL_AUTO")
            .env_remove("KIT_AGENT_RUNS_CHECKS")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .unwrap();
        let json = serde_json::from_slice(&out.stdout).expect("one JSON envelope");
        (out, json)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

const COMMIT: &str = "git add -A\ngit -c user.name=kit -c user.email=kit@test -c commit.gpgsign=false commit -q -m agent-change";

#[test]
fn committed_denied_paths_and_rename_sources_fail_and_cannot_land() {
    for (edits, denied) in [
        ("echo denied > .github/new.txt", ".github/new.txt"),
        (
            "mv .github/original.txt src/moved.txt",
            ".github/original.txt",
        ),
    ] {
        let fx = Fixture::new(&format!("{edits}\n{COMMIT}"), "git --version");
        let base = git(&fx.repo, &["rev-parse", "HEAD"]);
        let (out, run) = fx.kit(&["run", "--agent", "claude", "change fixture"]);
        assert_eq!(out.status.code(), Some(1), "{run}");
        assert_eq!(run["data"]["state"], "fail", "{run}");
        let dir = PathBuf::from(run["data"]["receiptDir"].as_str().unwrap());
        let receipt: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("receipt.json")).unwrap())
                .unwrap();
        assert_eq!(
            receipt["gate"]["scope_violations"],
            serde_json::json!([denied])
        );
        assert!(
            std::fs::read_to_string(dir.join("diff.patch"))
                .unwrap()
                .contains(denied)
        );
        let id = run["data"]["id"].as_str().unwrap();
        let wt = fx.root.join("kit-home/worktrees").join(id);
        assert_eq!(git(&wt, &["status", "--porcelain"]), "");
        let (land, _) = fx.kit(&["land", id]);
        assert_eq!(land.status.code(), Some(2));
        assert_eq!(git(&fx.repo, &["rev-parse", "HEAD"]), base);
        assert_eq!(git(&fx.repo, &["status", "--porcelain"]), "");
    }
}

#[test]
fn allowed_agent_commit_passes_and_lands_without_changing_parent() {
    let fx = Fixture::new(
        &format!("echo allowed > src/new.txt\n{COMMIT}"),
        "git --version",
    );
    let base = git(&fx.repo, &["rev-parse", "HEAD"]);
    let (out, run) = fx.kit(&["run", "--agent", "claude", "change fixture"]);
    assert!(out.status.success(), "{run}");
    assert_eq!(run["data"]["state"], "pass");
    let (out, land) = fx.kit(&["land", run["data"]["id"].as_str().unwrap()]);
    assert!(out.status.success(), "{land}");
    let branch = land["data"]["branch"].as_str().unwrap();
    assert_eq!(
        git(&fx.repo, &["show", &format!("{branch}:src/new.txt")]),
        "allowed"
    );
    assert_eq!(git(&fx.repo, &["rev-parse", "HEAD"]), base);
    assert_eq!(git(&fx.repo, &["status", "--porcelain"]), "");
}

#[test]
fn gate_cannot_erase_a_denied_receipt_change_before_scope_check() {
    let fx = Fixture::new(
        "echo denied > .github/new.txt",
        "git reset --hard HEAD && git --version",
    );
    let (out, run) = fx.kit(&["run", "--agent", "claude", "change fixture"]);
    assert_eq!(out.status.code(), Some(1), "{run}");
    assert_eq!(run["data"]["state"], "fail");
}

/// Before: a scope check that could not run read as "no violations".
#[test]
fn scope_check_that_cannot_run_fails_the_gate() {
    let fx = Fixture::new("echo allowed > src/new.txt\nrm -f .git", "true");
    let (out, run) = fx.kit(&["run", "--agent", "claude", "change fixture"]);
    assert_ne!(out.status.code(), Some(0), "{run}");
    assert_ne!(run["data"]["state"], "pass", "{run}");
    let dir = PathBuf::from(run["data"]["receiptDir"].as_str().unwrap());
    let receipt: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("receipt.json")).unwrap()).unwrap();
    assert_eq!(receipt["gate"]["passed"], false, "{receipt}");
    assert!(
        receipt["gate"]["scope_violations"][0]
            .as_str()
            .is_some_and(|v| v.starts_with("scope check could not run")),
        "{receipt}"
    );
    let (land, _) = fx.kit(&["land", run["data"]["id"].as_str().unwrap()]);
    assert_eq!(land.status.code(), Some(2));
}
