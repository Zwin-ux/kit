//! A run's diff holds the agent's change and nothing Kit put there.
//!
//! Regression: with a kit installed globally for Codex (skills in
//! `~/.agents/skills`) and the repo under `~`, a run copied those skills and
//! a Kit `AGENTS.md` into the worktree. A one-line change landed as 26 files.
//! A committed repo-root `skills/` pack was copied into `.agents/skills` too.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.name=kit", "-c", "user.email=kit@test"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .expect("git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn skill(dir: &Path, name: &str) {
    let d = dir.join(name);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("SKILL.md"), format!("---\nname: {name}\n---\n")).unwrap();
}

/// A fake `claude` that answers the probe and writes one file.
fn fake_claude(bin: &Path) {
    if cfg!(windows) {
        std::fs::write(
            bin.join("claude.cmd"),
            "@echo off\r\nif \"%1\"==\"--version\" (echo 0.0.0 fake& exit /b 0)\r\nif \"%1\"==\"auth\" exit /b 1\r\necho hello> created.txt\r\nexit /b 0\r\n",
        )
        .unwrap();
    } else {
        let path = bin.join("claude");
        std::fs::write(
            &path,
            "#!/bin/sh\ncase \"$1\" in\n  --version) echo 0.0.0 fake ;;\n  auth) exit 1 ;;\n  *) echo hello > created.txt ;;\nesac\n",
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
}

#[test]
fn run_diff_has_only_the_agents_change_with_global_and_repo_skill_packs() {
    let root: PathBuf =
        std::env::temp_dir().join(format!("kit-run-diff-scope-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let (home, bin, kit_home) = (root.join("home"), root.join("bin"), root.join("kit-home"));
    // What `kit add essentials --global --agent codex` leaves in ~.
    skill(
        &home.join(".agents").join("skills"),
        "test-driven-development",
    );
    std::fs::create_dir_all(&bin).unwrap();
    fake_claude(&bin);

    // A repo under ~ with its own committed skill pack (Harness layout).
    let repo = home.join("code").join("app");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::write(repo.join("README.md"), "app\n").unwrap();
    std::fs::write(repo.join("kit.toml"), "[gate]\ntest = 'git --version'\n").unwrap();
    skill(&repo.join("skills"), "deploy");
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["config", "core.autocrlf", "false"]);
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "init"]);

    let path = std::env::var_os("PATH").unwrap_or_default();
    let path =
        std::env::join_paths(std::iter::once(bin.clone()).chain(std::env::split_paths(&path)))
            .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_kit"))
        .args(["run", "--agent", "claude", "--json", "--task", "say hello"])
        .current_dir(&repo)
        .env("PATH", path)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("KIT_HOME", &kit_home)
        .env_remove("KIT_SKILLS_DIR")
        .env_remove("KIT_FULL_AUTO")
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .expect("run kit");
    let env: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{e}: stdout {}\nstderr {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    assert_eq!(
        env["data"]["state"],
        "pass",
        "{env}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let receipt = PathBuf::from(env["data"]["receiptDir"].as_str().unwrap());
    let diff = std::fs::read_to_string(receipt.join("diff.patch")).unwrap();
    let files: Vec<&str> = diff
        .lines()
        .filter_map(|l| l.strip_prefix("diff --git a/"))
        .collect();
    assert_eq!(files, ["created.txt b/created.txt"], "diff:\n{diff}");
    let _ = std::fs::remove_dir_all(&root);
}
