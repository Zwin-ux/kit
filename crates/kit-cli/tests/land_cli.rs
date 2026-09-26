//! `kit land` through the real binary. Each test makes a scratch repo and a
//! fake `claude` on PATH that writes files (a new text file, a new binary
//! file, an edit), so `kit run --agent claude` ends PASS with a real,
//! non-vacuous diff. The gate only runs `git --version`: files the gate
//! writes never reach the receipt.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

const SEED: &[u8] = &[0, 1, 2, 159, 146, 150, 0, 255, 10, 13, 0];

fn scratch(tag: &str) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kit-land-cli-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(dir: &Path, args: &[&str]) -> String {
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
    assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
    text(&out.stdout).trim().to_owned()
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Scratch repo on branch `main` with a committed kit.toml, and a fake
/// agent that `writes` (true) changes files, or (false) changes nothing.
struct Fixture {
    repo: PathBuf,
    home: PathBuf,
    bin: PathBuf,
}

/// A fake `claude`: answers the readiness probe, and on a real run makes
/// the edits in its working directory (the run's worktree).
fn fake_agent(bin: &Path, writes: bool) {
    if cfg!(windows) {
        let edits = if writes {
            "echo hello> created.txt\r\ncopy /b seed.bin new.bin >nul\r\necho edited>> README.md\r\n"
        } else {
            ""
        };
        let body = format!(
            "@echo off\r\nif \"%1\"==\"--version\" (echo 0.0.0 fake& exit /b 0)\r\nif \"%1\"==\"auth\" exit /b 1\r\n{edits}exit /b 0\r\n"
        );
        std::fs::write(bin.join("claude.cmd"), body).unwrap();
    } else {
        let edits = if writes {
            "echo hello > created.txt && cp seed.bin new.bin && echo edited >> README.md"
        } else {
            "true"
        };
        let body = format!(
            "#!/bin/sh\ncase \"$1\" in\n  --version) echo 0.0.0 fake ;;\n  auth) exit 1 ;;\n  *) {edits} ;;\nesac\n"
        );
        let path = bin.join("claude");
        std::fs::write(&path, body).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
}

impl Fixture {
    fn new(tag: &str, writes: bool) -> Self {
        let repo = scratch(tag);
        let home = scratch(&format!("{tag}-home"));
        std::fs::write(repo.join("README.md"), "fixture\n").unwrap();
        std::fs::write(repo.join("seed.bin"), SEED).unwrap();
        let bin = scratch(&format!("{tag}-bin"));
        fake_agent(&bin, writes);
        std::fs::write(repo.join("kit.toml"), "[gate]\ntest = 'git --version'\n").unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        git(&repo, &["config", "core.autocrlf", "false"]);
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-q", "-m", "init"]);
        Self { repo, home, bin }
    }

    fn kit(&self, args: &[&str]) -> Output {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let path = std::env::join_paths(
            std::iter::once(self.bin.clone()).chain(std::env::split_paths(&path)),
        )
        .unwrap();
        Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(args)
            .env("PATH", path)
            .env_remove("KIT_FULL_AUTO")
            .current_dir(&self.repo)
            .env("KIT_HOME", &self.home)
            .env("GIT_AUTHOR_NAME", "kit")
            .env("GIT_AUTHOR_EMAIL", "kit@test")
            .env("GIT_COMMITTER_NAME", "kit")
            .env("GIT_COMMITTER_EMAIL", "kit@test")
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .expect("run kit")
    }

    /// A run of the fake agent; returns its receipt id.
    fn run(&self) -> String {
        let out = self.kit(&[
            "run",
            "--agent",
            "claude",
            "--json",
            "--task",
            "Add greeting\n\nmore",
        ]);
        let env = json(&out);
        assert_eq!(env["data"]["state"], "pass", "{env}\n{}", text(&out.stderr));
        env["data"]["id"].as_str().unwrap().to_owned()
    }

    /// Copy run `id` to a new receipt id with `edit` applied to its JSON.
    fn craft(&self, id: &str, new_id: &str, edit: impl FnOnce(&mut Value)) {
        let runs = self.home.join("runs");
        let (from, to) = (runs.join(id), runs.join(new_id));
        std::fs::create_dir_all(&to).unwrap();
        for f in ["diff.patch", "base.txt", "output.log"] {
            std::fs::copy(from.join(f), to.join(f)).unwrap();
        }
        let raw = std::fs::read_to_string(from.join("receipt.json")).unwrap();
        let mut v: Value = serde_json::from_str(&raw).unwrap();
        v["id"] = Value::String(new_id.into());
        edit(&mut v);
        std::fs::write(to.join("receipt.json"), v.to_string()).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.repo);
        let _ = std::fs::remove_dir_all(&self.home);
        let _ = std::fs::remove_dir_all(&self.bin);
    }
}

fn json(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{e}: stdout {}\nstderr {}",
            text(&out.stdout),
            text(&out.stderr)
        )
    })
}

fn land_json(fx: &Fixture, args: &[&str]) -> (Output, Value) {
    let mut all = vec!["land"];
    all.extend_from_slice(args);
    all.push("--json");
    let out = fx.kit(&all);
    let v = json(&out);
    (out, v)
}

#[test]
fn land_makes_a_branch_with_new_and_binary_files_and_leaves_the_user_alone() {
    let fx = Fixture::new("branch", true);
    std::fs::write(fx.repo.join("mine.txt"), "user work\n").unwrap();
    let head = git(&fx.repo, &["rev-parse", "HEAD"]);
    let status = git(&fx.repo, &["status", "--porcelain"]);
    let id = fx.run();
    let run_wt = fx.home.join("worktrees").join(&id);
    assert!(run_wt.is_dir(), "a changed run keeps its worktree");

    let (out, env) = land_json(&fx, &[&id]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(env["schemaVersion"], 1);
    assert_eq!(env["command"], "land");
    assert_eq!(env["ok"], true);
    assert!(
        env["error"].is_null() && env["warnings"].is_array(),
        "{env}"
    );
    let d = &env["data"];
    let branch = format!("kit/{}", &id[..12]);
    assert_eq!(d["branch"], branch.as_str());
    assert_eq!(d["mode"], "branch");
    assert_eq!(d["baseSource"], "recorded");
    assert_eq!(d["base"], head.as_str());
    assert_eq!(d["alreadyLanded"], false);
    assert_eq!(d["worktreeRemoved"], true);
    assert_eq!(d["next"], format!("git merge {branch}").as_str());
    assert!(!run_wt.exists(), "landed worktree must go");

    // Exact content, parent = base, message with the trailer.
    let show = |p: &str| git(&fx.repo, &["show", &format!("{branch}:{p}")]);
    assert_eq!(show("created.txt"), "hello");
    assert!(
        show("README.md").ends_with("edited"),
        "{}",
        show("README.md")
    );
    let blob = Command::new("git")
        .args(["show", &format!("{branch}:new.bin")])
        .current_dir(&fx.repo)
        .output()
        .unwrap();
    assert_eq!(blob.stdout, SEED, "binary file must land byte for byte");
    assert_eq!(git(&fx.repo, &["rev-parse", &format!("{branch}^")]), head);
    let msg = git(&fx.repo, &["log", "-1", "--format=%B", &branch]);
    assert!(msg.starts_with("Add greeting\n"), "{msg}");
    assert!(msg.contains(&format!("Kit-Receipt: {id}")), "{msg}");
    assert!(msg.contains("Gate: PASS · test"), "{msg}");

    // The user's branch, HEAD and files did not change.
    assert_eq!(git(&fx.repo, &["branch", "--show-current"]), "main");
    assert_eq!(git(&fx.repo, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(&fx.repo, &["status", "--porcelain"]), status);
    assert!(!fx.repo.join("created.txt").exists());

    // Idempotent: a second land finds the same commit.
    let (out, again) = land_json(&fx, &[&id[..14]]);
    assert!(out.status.success());
    assert_eq!(again["data"]["alreadyLanded"], true);
    assert_eq!(again["data"]["commit"], d["commit"]);
    let text_out = fx.kit(&["land", &id]);
    assert!(
        text(&text_out.stdout).contains("already landed on"),
        "{}",
        text(&text_out.stdout)
    );
}

#[test]
fn land_refuses_fail_vacuous_and_empty_runs() {
    let fx = Fixture::new("refuse", true);
    let id = fx.run();
    fx.craft(&id, "01TESTLANDFAIL0000000000001", |v| {
        v["state"] = "fail".into();
        v["gate"]["passed"] = false.into();
    });
    fx.craft(&id, "01TESTLANDVACUOUS000000001", |v| {
        v["gate"]["checks"] = Value::Array(Vec::new());
    });
    for (rid, why) in [
        ("01TESTLANDFAIL0000000000001", "is fail, not pass"),
        ("01TESTLANDVACUOUS000000001", "UNCONFIGURED"),
    ] {
        let (out, env) = land_json(&fx, &[rid]);
        assert_eq!(out.status.code(), Some(2));
        assert_eq!(env["ok"], false);
        assert_eq!(env["command"], "land");
        let err = env["error"].as_str().unwrap();
        assert!(err.contains(why) && err.contains("--force"), "{err}");
    }
    // --force lands it, loudly.
    let (out, env) = land_json(&fx, &["01TESTLANDFAIL0000000000001", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(env["data"]["forced"], true);
    assert!(
        env["warnings"][0].as_str().unwrap().contains("FORCED"),
        "{env}"
    );
    let msg = git(&fx.repo, &["log", "-1", "--format=%B", "kit/01TESTLANDFA"]);
    assert!(msg.contains("Landed with --force"), "{msg}");

    let quiet = Fixture::new("empty", false);
    let id = quiet.run();
    let (out, env) = land_json(&quiet, &[&id]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        env["error"].as_str().unwrap().contains("nothing to land"),
        "{env}"
    );
}

#[test]
fn land_apply_needs_a_clean_tree_and_makes_no_commit() {
    let fx = Fixture::new("apply", true);
    let id = fx.run();
    let head = git(&fx.repo, &["rev-parse", "HEAD"]);
    std::fs::write(fx.repo.join("mine.txt"), "user work\n").unwrap();

    let (out, env) = land_json(&fx, &[&id, "--apply"]);
    assert_eq!(out.status.code(), Some(2));
    let err = env["error"].as_str().unwrap();
    assert!(
        err.contains("uncommitted changes") && err.contains("--force"),
        "{err}"
    );
    assert!(
        !fx.repo.join("created.txt").exists(),
        "refused means untouched"
    );

    let (out, env) = land_json(&fx, &[&id, "--apply", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(env["data"]["mode"], "apply");
    assert!(env["data"]["commit"].is_null());
    assert_eq!(std::fs::read(fx.repo.join("new.bin")).unwrap(), SEED);
    assert!(fx.repo.join("created.txt").is_file());
    assert_eq!(git(&fx.repo, &["rev-parse", "HEAD"]), head, "no commit");

    let (out, env) = land_json(&fx, &[&id, "--apply"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(env["data"]["alreadyLanded"], true);
}

/// An old receipt with no base.txt (and no worktree) lands on HEAD, 3-way.
#[test]
fn land_without_a_base_uses_head_and_says_so() {
    let fx = Fixture::new("nobase", true);
    let id = fx.run();
    let rid = "01TESTLANDNOBASE0000000001";
    fx.craft(&id, rid, |_| {});
    std::fs::remove_file(fx.home.join("runs").join(rid).join("base.txt")).unwrap();
    let (out, env) = land_json(&fx, &[rid]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(env["data"]["baseSource"], "head");
    assert_eq!(env["data"]["threeWay"], true);
    assert!(
        env["warnings"].to_string().contains("no base commit"),
        "{env}"
    );
    let branch = env["data"]["branch"].as_str().unwrap().to_owned();
    assert_eq!(
        git(&fx.repo, &["show", &format!("{branch}:created.txt")]),
        "hello"
    );
}

#[test]
fn land_apply_on_a_clean_tree() {
    let fx = Fixture::new("apply-clean", true);
    let id = fx.run();
    let (out, env) = land_json(&fx, &[&id, "--apply"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(env["data"]["alreadyLanded"], false);
    assert!(fx.repo.join("created.txt").is_file());
    assert_eq!(git(&fx.repo, &["branch", "--show-current"]), "main");
}
