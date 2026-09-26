//! `kit run --json` prints exactly one JSON value on stdout: automation pipes
//! it straight into a parser (docs/json-contract.md). Output from child
//! processes, such as git's "HEAD is now at …", must go to stderr instead.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "kit-json-stdout-{tag}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// One-commit git repo with no kit.toml, so the dry run's gate stays vacuous.
fn git_fixture(dir: &Path) {
    std::fs::write(dir.join("README.md"), "kit json fixture\n").unwrap();
    for args in [
        &["init", "-q"][..],
        &["add", "."],
        &["commit", "-q", "-m", "init"],
    ] {
        let status = Command::new("git")
            .args([
                "-c",
                "user.name=kit",
                "-c",
                "user.email=kit@test",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            // An inherited GIT_DIR would commit into this workspace instead.
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .status()
            .expect("git");
        assert!(status.success(), "git {args:?}");
    }
}

#[test]
fn run_json_stdout_is_exactly_one_json_value() {
    let home = scratch_dir("home");
    let repo = scratch_dir("repo");
    git_fixture(&repo);

    let out = Command::new(env!("CARGO_BIN_EXE_kit"))
        .args(["run", "--dry-run", "--json", "--task", "json stdout guard"])
        .arg("--repo")
        .arg(&repo)
        // Run from the fixture, so no path fallback can reach this workspace.
        .current_dir(&repo)
        .env("KIT_HOME", &home)
        .output()
        .expect("spawn kit");
    let stdout = String::from_utf8(out.stdout).expect("utf-8 stdout");
    let stderr = String::from_utf8_lossy(&out.stderr);

    let envelope: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("stdout is not exactly one JSON value ({e}):\n{stdout}\n--- stderr:\n{stderr}")
    });
    assert_eq!(envelope["schemaVersion"], 1, "{stdout}");
    assert_eq!(envelope["command"], "run", "{stdout}");
    assert_eq!(envelope["ok"], true, "{stdout}\n--- stderr:\n{stderr}");
    // The run really made its worktree (where git narrates) and a receipt.
    let receipt_dir = envelope["data"]["receiptDir"].as_str().expect("receiptDir");
    assert!(Path::new(receipt_dir).starts_with(&home), "{receipt_dir}");
    assert!(Path::new(receipt_dir).join("receipt.json").is_file());
    // No kit.toml and no project: one stderr line says how to write one;
    // stdout stays one value.
    assert!(stderr.contains("Write kit.toml by hand"), "{stderr}");
    assert!(
        out.status.success(),
        "{:?}\n--- stderr:\n{stderr}",
        out.status
    );

    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&repo);
}

/// A run with no gate checks proves nothing, so its receipt says so: state
/// `unconfigured`, gate not passed, and `kit receipt show` / `list` agree.
#[test]
fn vacuous_run_receipt_is_unconfigured_not_pass() {
    let home = scratch_dir("vac-home");
    let repo = scratch_dir("vac-repo");
    git_fixture(&repo);
    let kit = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(args)
            .current_dir(&repo)
            .env("KIT_HOME", &home)
            .output()
            .expect("spawn kit")
    };

    let out = kit(&["run", "--dry-run", "--json", "--task", "vacuous"]);
    let env: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(env["data"]["state"], "unconfigured", "{env}");
    assert_eq!(env["data"]["gatePassed"], false, "{env}");
    assert_eq!(env["data"]["gateVacuous"], true, "{env}");
    // A dry run makes no proof claim, so it still exits 0.
    assert!(out.status.success(), "{env}");

    let dir = Path::new(env["data"]["receiptDir"].as_str().unwrap()).to_path_buf();
    let receipt: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("receipt.json")).unwrap()).unwrap();
    assert_eq!(receipt["state"], "unconfigured", "{receipt}");
    assert_eq!(receipt["gate"]["passed"], false, "{receipt}");

    let id = env["data"]["id"].as_str().unwrap();
    let show = String::from_utf8(kit(&["receipt", "show", id]).stdout).unwrap();
    assert!(show.contains("state     unconfigured"), "{show}");
    assert!(show.contains("gate      UNCONFIGURED"), "{show}");
    assert!(show.contains("\n  took      "), "{show}");
    assert!(!show.contains("pass"), "{show}");
    let list = String::from_utf8(kit(&["receipt", "list"]).stdout).unwrap();
    assert!(list.contains("unconfigured"), "{list}");
    assert!(!list.contains(" pass "), "{list}");

    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&repo);
}
