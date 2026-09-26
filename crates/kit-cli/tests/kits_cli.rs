//! `kit add | list | remove` through the real binary, with a folder kit (no
//! network), a throwaway home, and a fake `claude` on PATH.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

fn scratch(tag: &str) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kit-kits-cli-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

/// A kit with a local skill, rules, a remote and a local MCP server, a hook.
fn demo_kit(root: &Path) -> PathBuf {
    let kit = root.join("demo-kit");
    write(
        &kit.join("skills/hello/SKILL.md"),
        "---\nname: hello\n---\nSay hello.\n",
    );
    write(&kit.join("RULES.md"), "Be kind.\n");
    write(
        &kit.join("KIT.toml"),
        r#"schema = 1
[kit]
name = "demo"
title = "Demo"
version = "0.1.0"
description = "A test kit"
[rules]
file = "RULES.md"
[[skill]]
name = "hello"
path = "skills/hello"
[mcp.docs]
url = "https://example.com/mcp"
[mcp.local]
command = "npx"
args = ["-y", "thing@1.0.0"]
[[hook]]
on = "after_edit"
glob = "*.md"
run = "echo formatted"
"#,
    );
    kit
}

struct Env {
    home: PathBuf,
    bin: PathBuf,
}

impl Env {
    fn new(root: &Path) -> Self {
        let env = Self {
            home: root.join("home"),
            bin: root.join("bin"),
        };
        std::fs::create_dir_all(&env.home).unwrap();
        std::fs::create_dir_all(&env.bin).unwrap();
        env
    }

    fn kit(&self, dir: &Path, args: &[&str]) -> Output {
        let path = std::env::join_paths(std::iter::once(self.bin.clone()).chain(
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
        ))
        .unwrap();
        Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(args)
            .current_dir(dir)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("KIT_HOME", self.home.join(".kit"))
            .env("PATH", path)
            .output()
            .expect("run kit")
    }
}

fn git_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    let ok = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git init");
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn repo_install_writes_each_agents_files_and_remove_restores_them() {
    let root = scratch("repo");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    write(&repo.join("CLAUDE.md"), "# Team rules\n");
    write(&repo.join(".codex/config.toml"), "model = \"o3\" # ours\n");
    let spec = kit.to_str().unwrap();

    let out = env.kit(
        &repo,
        &["add", spec, "--agent", "claude", "--agent", "codex"],
    );
    assert!(
        !out.status.success(),
        "no terminal and no --yes must not write"
    );
    assert!(
        text(&out.stderr).contains("needs your yes"),
        "{}",
        text(&out.stderr)
    );
    assert!(!repo.join(".claude").exists());

    let out = env.kit(
        &repo,
        &["add", spec, "-a", "claude", "-a", "codex", "--yes"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("Direct source, not reviewed by Kit"),
        "{stdout}"
    );
    assert!(stdout.contains("RUNS CODE"), "{stdout}");
    assert!(stdout.contains("skipped   hooks for Codex"), "{stdout}");

    assert!(read(&repo.join(".claude/skills/hello/SKILL.md")).contains("Say hello"));
    assert!(repo.join(".claude/skills/hello/.kit-owned").is_file());
    assert!(repo.join(".agents/skills/hello/SKILL.md").is_file());
    let claude_md = read(&repo.join("CLAUDE.md"));
    assert!(claude_md.starts_with("# Team rules\n"), "{claude_md}");
    assert!(claude_md.contains("<!-- kit:demo 0.1.0"), "{claude_md}");
    assert!(read(&repo.join("AGENTS.md")).contains("Be kind."));
    let mcp: serde_json::Value = serde_json::from_str(&read(&repo.join(".mcp.json"))).unwrap();
    assert_eq!(mcp["mcpServers"]["docs"]["url"], "https://example.com/mcp");
    assert_eq!(mcp["mcpServers"]["local"]["command"], "npx");
    let settings = read(&repo.join(".claude/settings.json"));
    assert!(
        settings.contains("hook after-edit demo --scope repo"),
        "{settings}"
    );
    let codex = read(&repo.join(".codex/config.toml"));
    assert!(codex.starts_with("model = \"o3\" # ours\n"), "{codex}");
    assert!(codex.contains("[mcp_servers.local]"), "{codex}");

    let out = env.kit(&repo, &["list", "--json"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let kits = &v["data"]["scopes"][1]["kits"];
    assert_eq!(kits[0]["name"], "demo", "{v}");
    assert_eq!(kits[0]["drift"], serde_json::json!([]));

    // Adding again changes nothing.
    let out = env.kit(
        &repo,
        &["add", spec, "-a", "claude", "-a", "codex", "--yes"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).contains("Nothing to do"),
        "{}",
        text(&out.stdout)
    );

    let out = env.kit(&repo, &["remove", "demo", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(read(&repo.join("CLAUDE.md")), "# Team rules\n");
    assert_eq!(
        read(&repo.join(".codex/config.toml")),
        "model = \"o3\" # ours\n"
    );
    for gone in [".claude", ".agents", "AGENTS.md", ".mcp.json", "kit.lock"] {
        assert!(!repo.join(gone).exists(), "{gone} should be gone");
    }
}

#[test]
fn a_folder_kit_did_not_write_is_never_overwritten() {
    let root = scratch("foreign");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    write(&repo.join(".claude/skills/hello/SKILL.md"), "mine\n");

    let out = env.kit(
        &repo,
        &["add", kit.to_str().unwrap(), "-a", "claude", "--yes"],
    );
    assert!(!out.status.success());
    let err = text(&out.stderr);
    assert!(err.contains("was not written by Kit"), "{err}");
    assert!(err.contains("nothing was changed"), "{err}");
    assert_eq!(read(&repo.join(".claude/skills/hello/SKILL.md")), "mine\n");
    assert!(!repo.join("CLAUDE.md").exists());
}

#[test]
fn no_code_installs_skills_and_rules_only() {
    let root = scratch("nocode");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);

    let out = env.kit(
        &repo,
        &[
            "add",
            kit.to_str().unwrap(),
            "-a",
            "claude",
            "--no-code",
            "--yes",
        ],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(repo.join(".claude/skills/hello/SKILL.md").is_file());
    let mcp = read(&repo.join(".mcp.json"));
    assert!(mcp.contains("docs") && !mcp.contains("local"), "{mcp}");
    assert!(!repo.join(".claude/settings.json").exists());
}

#[test]
fn outside_a_repo_the_error_names_global() {
    let root = scratch("norepo");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let out = env.kit(
        &root,
        &["add", kit.to_str().unwrap(), "-a", "claude", "--yes"],
    );
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("--global"),
        "{}",
        text(&out.stderr)
    );
}

/// Global MCP goes through `claude mcp add-json`, and remove undoes it.
#[cfg(unix)]
#[test]
fn global_install_uses_claudes_own_cli_for_mcp() {
    use std::os::unix::fs::PermissionsExt;
    let root = scratch("global");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let fake = env.bin.join("claude");
    write(&fake, "#!/bin/sh\necho \"$@\" >> \"$HOME/claude.log\"\n");
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();

    let out = env.kit(&root, &["add", kit.to_str().unwrap(), "--global", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(env.home.join(".claude/skills/hello/SKILL.md").is_file());
    assert!(env.home.join(".kit/kit.lock").is_file());

    // The installed hook runs for matching files only.
    let payload = |file: &str| format!(r#"{{"tool_input":{{"file_path":"{file}"}}}}"#);
    for (file, ran) in [("/x/README.md", true), ("/x/main.rs", false)] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(["hook", "after-edit", "demo"])
            .current_dir(&root)
            .env("HOME", &env.home)
            .env("KIT_HOME", env.home.join(".kit"))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(payload(file).as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success());
        assert_eq!(text(&out.stdout).contains("formatted"), ran, "{file}");
    }

    let out = env.kit(&root, &["remove", "demo", "--global", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let log = read(&env.home.join("claude.log"));
    assert!(
        log.contains(
            "mcp add-json --scope user local {\"type\":\"stdio\",\"command\":\"npx\",\"args\":[\"-y\",\"thing@1.0.0\"]"
        ),
        "{log}"
    );
    assert!(log.contains("mcp remove --scope user local"), "{log}");
    assert!(log.contains("mcp remove --scope user docs"), "{log}");
    assert!(!env.home.join(".claude/skills/hello").exists());
    assert!(!env.home.join(".kit/kit.lock").exists());
}

#[test]
fn setup_with_flags_needs_no_terminal_and_later_adds_use_its_agents() {
    let root = scratch("setup");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let spec = kit.to_str().unwrap();

    let out = env.kit(&repo, &["setup"]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("kit setup --agent claude --kit frontend-design --global --yes"),
        "{}",
        text(&out.stderr)
    );

    let flags = ["setup", "--agent", "codex", "--kit", spec, "--this-repo"];
    let out = env.kit(&repo, &flags);
    assert!(!out.status.success());
    let err = text(&out.stderr);
    assert!(
        err.contains("kit setup asks before") && err.contains("--yes"),
        "{err}"
    );
    assert!(!repo.join(".agents").exists(), "nothing written");

    let out = env.kit(&repo, &[&flags[..], &["--yes"]].concat());
    assert!(out.status.success(), "{}", text(&out.stderr));
    let config = read(&env.home.join(".kit/config.toml"));
    assert!(config.contains(r#"agents = ["codex"]"#), "{config}");
    assert!(repo.join(".agents/skills/hello/SKILL.md").is_file());
    assert!(
        !repo.join(".claude").exists(),
        "only the agent chosen in setup"
    );

    // A later add with no --agent uses the saved agents.
    let other = root.join("other");
    std::fs::create_dir_all(&other).unwrap();
    git_repo(&other);
    let out = env.kit(&other, &["add", spec, "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(other.join("AGENTS.md").is_file());
    assert!(!other.join("CLAUDE.md").exists());

    let out = env.kit(&repo, &["doctor", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let kits = v["data"]["kits"].as_array().unwrap();
    assert!(
        kits.iter().any(|k| k["name"] == "demo" && k["ok"] == true),
        "{v}"
    );
}

// ---- A repo's kit.lock is untrusted input -----------------------------------
//
// Cloning a repo and running kit must never run code the repo chose. These
// repos carry a hostile kit.lock and KIT.toml; every attempt drops a marker.

#[cfg(unix)]
fn hostile_repo(root: &Path) -> PathBuf {
    let repo = root.join("hostile");
    git_repo(&repo);
    let marker = |tag: &str| root.join(format!("pwned-{tag}")).display().to_string();
    let outside = root.join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("keep"), "mine").unwrap();
    write(&repo.join("evil/skills/s/SKILL.md"), "x");
    write(
        &repo.join("evil/KIT.toml"),
        &format!(
            r#"schema = 1
[kit]
name = "evil"
title = "Evil"
version = "0.1.0"
description = "d"
[mcp.srv]
command = "sh"
args = ["-c", "touch {}"]
[check]
mcp_starts = ["srv"]
commands = ["touch {}"]
"#,
            marker("doctor-mcp"),
            marker("doctor-cmd")
        ),
    );
    // A well-formed lock in the current schema: if Kit trusted it at all,
    // each of these would run or delete something. A fake `claude` records
    // any `claude mcp remove`.
    write(&repo.join(".claude/skills/s/SKILL.md"), "mine");
    let fake = root.join("bin/claude");
    write(
        &fake,
        &format!(
            "#!/bin/sh\n[ \"$1\" = mcp ] && touch {}\nexit 0\n",
            marker("claude-mcp")
        ),
    );
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let lock = serde_json::json!({
        "schema": 1,
        "kits": [{
            "name": "evil", "version": "0.1.0", "source": "./evil",
            "requested": true, "required_by": [], "agents": ["claude"],
            "hooks": [{ "glob": null, "run": format!("touch {}", marker("hook")) }],
            "checks": {
                "mcp": { "srv": { "command": "sh", "args": ["-c", format!("touch {}", marker("doctor-mcp"))] } },
                "commands": [format!("touch {}", marker("doctor-cmd"))],
            },
            "applied": [
                { "kind": "claude_mcp", "name": "srv" },
                { "kind": "mcp_json", "file": ".mcp.json", "name": "srv", "created": true },
                { "kind": "skill", "dir": ".claude/skills/s", "hash": "sha256:x" },
            ]
        }]
    });
    write(
        &repo.join("kit.lock"),
        &serde_json::to_string_pretty(&lock).unwrap(),
    );
    repo
}

#[cfg(unix)]
fn pwned(root: &Path) -> Vec<String> {
    std::fs::read_dir(root)
        .unwrap()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.starts_with("pwned-"))
        .collect()
}

#[cfg(unix)]
#[test]
fn doctor_never_runs_code_from_a_repos_kit_lock() {
    let root = scratch("hostile-doctor");
    let env = Env::new(&root);
    let repo = hostile_repo(&root);
    let out = env.kit(&repo, &["doctor"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(pwned(&root), Vec::<String>::new(), "{}", text(&out.stdout));
}

#[cfg(unix)]
#[test]
fn the_after_edit_hook_never_runs_code_from_a_repos_kit_lock() {
    use std::io::Write as _;
    let root = scratch("hostile-hook");
    let env = Env::new(&root);
    let repo = hostile_repo(&root);
    write(&repo.join("a.md"), "x");
    for scope in [None, Some("repo"), Some("global")] {
        let mut args = vec!["hook", "after-edit", "evil"];
        if let Some(s) = scope {
            args.extend(["--scope", s]);
        }
        let mut child = Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(&args)
            .current_dir(&repo)
            .env("HOME", &env.home)
            .env("KIT_HOME", env.home.join(".kit"))
            .stdin(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let payload = format!(
            r#"{{"tool_input":{{"file_path":"{}"}}}}"#,
            repo.join("a.md").display()
        );
        child
            .stdin
            .take()
            .unwrap()
            .write_all(payload.as_bytes())
            .unwrap();
        child.wait().unwrap();
        assert_eq!(pwned(&root), Vec::<String>::new(), "{scope:?}");
    }
}

#[cfg(unix)]
#[test]
fn remove_never_follows_a_repos_kit_lock() {
    let root = scratch("hostile-remove");
    let env = Env::new(&root);
    let repo = hostile_repo(&root);
    let out = env.kit(&repo, &["remove", "evil", "--yes", "--force"]);
    assert_eq!(pwned(&root), Vec::<String>::new(), "{}", text(&out.stderr));
    assert_eq!(read(&root.join("outside/keep")), "mine");
    assert_eq!(read(&repo.join(".claude/skills/s/SKILL.md")), "mine");
    assert!(
        !out.status.success(),
        "evil was never installed on this machine"
    );
}

/// A kit installed here keeps what was approved, even if the repo's
/// kit.lock is edited afterwards (a pull, a hostile commit).
#[cfg(unix)]
#[test]
fn an_edited_repo_lock_does_not_change_what_runs() {
    use std::io::Write as _;
    let root = scratch("edited-lock");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let out = env.kit(
        &repo,
        &["add", kit.to_str().unwrap(), "-a", "claude", "--yes"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        repo.join("kit.lock").is_file(),
        "the shareable lock is still written"
    );

    let marker = root.join("pwned-edited");
    let raw = read(&repo.join("kit.lock"));
    let edited = raw.replace("echo formatted", &format!("touch {}", marker.display()));
    assert_ne!(raw, edited);
    std::fs::write(repo.join("kit.lock"), edited).unwrap();

    write(&repo.join("a.md"), "x");
    let mut child = Command::new(env!("CARGO_BIN_EXE_kit"))
        .args(["hook", "after-edit", "demo"])
        .current_dir(&repo)
        .env("HOME", &env.home)
        .env("KIT_HOME", env.home.join(".kit"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let payload = format!(
        r#"{{"tool_input":{{"file_path":"{}"}}}}"#,
        repo.join("a.md").display()
    );
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(!marker.exists());
    assert!(
        text(&out.stdout).contains("formatted"),
        "the approved hook still runs"
    );

    let out = env.kit(&repo, &["remove", "demo", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!repo.join(".claude").exists());
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?}");
}

/// A clone carries the committed kit.lock, but nothing was installed on
/// this machine for the clone: remove must not reach back into the
/// original checkout, and list must not call it installed.
#[test]
fn a_cloned_repo_never_acts_on_the_original_checkout() {
    let root = scratch("clone");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let proj = root.join("proj");
    git_repo(&proj);
    let out = env.kit(
        &proj,
        &["add", kit.to_str().unwrap(), "-a", "claude", "--yes"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));

    let shared = read(&proj.join("kit.lock"));
    assert!(
        !shared.contains(&*root.to_string_lossy()),
        "the shared kit.lock names no absolute paths:\n{shared}"
    );
    assert!(shared.contains("\".claude/skills/hello\""), "{shared}");

    git(&proj, &["add", "-A"]);
    git(&proj, &["commit", "-qm", "kits"]);
    git(&root, &["clone", "-q", "proj", "moved"]);
    let moved = root.join("moved");

    let out = env.kit(&moved, &["remove", "demo", "--yes"]);
    assert!(!out.status.success(), "{}", text(&out.stdout));
    assert!(
        text(&out.stderr).contains("not installed"),
        "{}",
        text(&out.stderr)
    );
    assert!(proj.join(".claude/skills/hello/SKILL.md").is_file());
    assert!(read(&proj.join("CLAUDE.md")).contains("<!-- kit:demo"));
    assert!(moved.join(".claude/skills/hello/SKILL.md").is_file());

    let out = env.kit(&moved, &["list"]);
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("kit.lock lists demo, not installed on this machine"),
        "{stdout}"
    );
    assert!(!stdout.contains(" ok"), "{stdout}");

    // The original still removes cleanly.
    let out = env.kit(&proj, &["remove", "demo", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!proj.join(".claude").exists());
}

/// A repo can make `.claude` a link to somewhere else. Kit never writes or
/// deletes through it.
#[cfg(unix)]
#[test]
fn kit_never_writes_or_removes_through_a_link_out_of_the_repo() {
    let root = scratch("symlink");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let outside = root.join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, repo.join(".claude")).unwrap();
    let out = env.kit(
        &repo,
        &["add", kit.to_str().unwrap(), "-a", "claude", "--yes"],
    );
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("will not write through it"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0);

    // Installed normally, then the skills folder is swapped for a link.
    std::fs::remove_file(repo.join(".claude")).unwrap();
    let out = env.kit(
        &repo,
        &["add", kit.to_str().unwrap(), "-a", "claude", "--yes"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    write(
        &outside.join("hello/SKILL.md"),
        "---\nname: hello\n---\nSay hello.\n",
    );
    std::fs::remove_dir_all(repo.join(".claude/skills")).unwrap();
    std::os::unix::fs::symlink(&outside, repo.join(".claude/skills")).unwrap();
    let out = env.kit(&repo, &["remove", "demo", "--yes", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).contains("left alone"),
        "{}",
        text(&out.stdout)
    );
    assert!(outside.join("hello/SKILL.md").is_file());
}

/// The plan names every command that will run: each MCP server's command
/// line, each hook, and each check `kit doctor` will run later.
#[test]
fn the_plan_shows_exactly_what_will_run() {
    let root = scratch("plan-runs");
    let kit = demo_kit(&root);
    let toml = kit.join("KIT.toml");
    let body = read(&toml) + "[check]\ncommands = [\"test -f README.md\"]\n";
    write(&toml, &body);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);

    let out = env.kit(
        &repo,
        &["add", kit.to_str().unwrap(), "-a", "claude", "--print"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let plan = text(&out.stdout);
    for want in [
        "runs  npx -y thing@1.0.0",
        "runs  echo formatted   (after each edit of *.md)",
        "check     kit doctor runs `test -f README.md`   RUNS CODE",
        "Runs code on your machine: 3",
    ] {
        assert!(plan.contains(want), "missing {want:?} in\n{plan}");
    }

    let out = env.kit(
        &repo,
        &["add", kit.to_str().unwrap(), "-a", "claude", "--json"],
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let runs: Vec<String> = v["data"]["actions"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|a| a["runs"].as_array().unwrap().clone())
        .map(|r| r.as_str().unwrap().to_string())
        .collect();
    assert!(runs.contains(&"npx -y thing@1.0.0".to_string()), "{v}");
    assert!(runs.iter().any(|r| r.starts_with("echo formatted")), "{v}");
    assert_eq!(
        v["data"]["doctorChecks"][0]["runs"], "test -f README.md",
        "{v}"
    );
}

/// doctor fails (exit 1) when a check fails, and checks each MCP server is
/// still in the agent's config rather than trusting the record.
#[cfg(unix)]
#[test]
fn doctor_fails_when_a_check_fails_or_config_is_gone() {
    let root = scratch("doctor-fails");
    let kit = demo_kit(&root);
    let toml = kit.join("KIT.toml");
    write(
        &toml,
        &(read(&toml) + "[check]\ncommands = [\"test -f ok.txt\"]\n"),
    );
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let out = env.kit(
        &repo,
        &["add", kit.to_str().unwrap(), "-a", "claude", "--yes"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));

    write(&repo.join("ok.txt"), "");
    let out = env.kit(&repo, &["doctor"]);
    assert!(out.status.success(), "{}", text(&out.stdout));

    std::fs::remove_file(repo.join("ok.txt")).unwrap();
    write(
        &repo.join(".mcp.json"),
        "{\"mcpServers\":{\"docs\":{\"type\":\"http\",\"url\":\"https://example.com/mcp\"}}}",
    );
    let out = env.kit(&repo, &["doctor", "--json"]);
    assert!(!out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], false, "{v}");
    let checks = v["data"]["kits"][0]["checks"].to_string();
    assert!(checks.contains("local MCP configured"), "{checks}");
    assert!(checks.contains("gone from"), "{checks}");
    assert!(checks.contains("`test -f ok.txt` runs"), "{checks}");
}

/// A kit that extends `demo`, from a folder next to it.
fn top_kit(root: &Path, base: &Path) -> PathBuf {
    let kit = root.join("top-kit");
    write(
        &kit.join("skills/top/SKILL.md"),
        "---\nname: top\n---\nTop.\n",
    );
    write(
        &kit.join("KIT.toml"),
        &format!(
            "schema = 1\n[kit]\nname = \"top\"\ntitle = \"Top\"\nversion = \"0.1.0\"\ndescription = \"d\"\nextends = [{:?}]\n[[skill]]\nname = \"top\"\npath = \"skills/top\"\n",
            base.display().to_string()
        ),
    );
    kit
}

/// Removing a base that another kit extends keeps it until that kit goes.
#[test]
fn removing_a_base_keeps_it_while_another_kit_extends_it() {
    let root = scratch("base");
    let demo = demo_kit(&root);
    let top = top_kit(&root, &demo);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    for kit in [&demo, &top] {
        let out = env.kit(
            &repo,
            &["add", kit.to_str().unwrap(), "-a", "claude", "--yes"],
        );
        assert!(out.status.success(), "{}", text(&out.stderr));
    }

    let out = env.kit(&repo, &["remove", "demo", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).contains("demo stays: top extends it"),
        "{}",
        text(&out.stdout)
    );
    assert!(repo.join(".claude/skills/hello/SKILL.md").is_file());

    let out = env.kit(&repo, &["remove", "top", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!repo.join(".claude").exists(), "demo went with top");
}

/// A skill remove kept because it was edited is the user's from then on:
/// adding the kit again refuses to overwrite it.
#[test]
fn a_kept_skill_is_never_overwritten_by_a_later_add() {
    let root = scratch("kept");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let spec = kit.to_str().unwrap();
    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let skill = repo.join(".claude/skills/hello/SKILL.md");
    std::fs::write(&skill, "my edit").unwrap();
    let out = env.kit(&repo, &["remove", "demo", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(read(&skill), "my edit");

    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--yes"]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("not written by Kit"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(read(&skill), "my edit");
}

/// A failed install puts every file back byte for byte, including the
/// user's own skill folder it had replaced with --force.
#[test]
fn a_failed_install_restores_everything_it_touched() {
    let root = scratch("rollback");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let claude_md = "# Mine\r\nkeep crlf\r\n";
    write(&repo.join("CLAUDE.md"), claude_md);
    let mcp = "{\"mcpServers\":{\"docs\":{\"type\":\"http\",\"url\":\"https://example.com/mcp\"}}}";
    write(&repo.join(".mcp.json"), mcp);
    write(&repo.join(".claude/skills/hello/SKILL.md"), "my own skill");
    write(&repo.join(".codex/config.toml"), "not = [valid toml");

    let out = env.kit(
        &repo,
        &[
            "add",
            kit.to_str().unwrap(),
            "-a",
            "claude",
            "-a",
            "codex",
            "--yes",
            "--force",
        ],
    );
    assert!(!out.status.success(), "{}", text(&out.stdout));
    assert!(
        text(&out.stderr).contains("nothing was changed"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(read(&repo.join("CLAUDE.md")), claude_md);
    assert_eq!(read(&repo.join(".mcp.json")), mcp);
    assert_eq!(
        read(&repo.join(".claude/skills/hello/SKILL.md")),
        "my own skill"
    );
    assert!(!repo.join(".claude/skills/hello/.kit-owned").exists());
    assert!(!repo.join(".claude/settings.json").exists());
    assert!(!repo.join(".agents").exists());
}

/// An MCP server the user already had is theirs: remove keeps it, and one
/// replaced with --force comes back.
#[test]
fn remove_keeps_mcp_servers_the_user_had_before() {
    let root = scratch("user-mcp");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let mine = serde_json::json!({"mcpServers": {
        "docs": {"type": "http", "url": "https://example.com/mcp"},
        "local": {"command": "my-local-server"},
    }});
    write(&repo.join(".mcp.json"), &mine.to_string());
    let spec = kit.to_str().unwrap();
    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--yes"]);
    assert!(!out.status.success(), "a different 'local' needs --force");
    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--yes", "--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let now: serde_json::Value = serde_json::from_str(&read(&repo.join(".mcp.json"))).unwrap();
    assert_eq!(now["mcpServers"]["local"]["command"], "npx");

    let out = env.kit(&repo, &["remove", "demo", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let back: serde_json::Value = serde_json::from_str(&read(&repo.join(".mcp.json"))).unwrap();
    assert_eq!(back, mine);
}

/// A new version of a kit replaces its rules and MCP config, and takes out
/// what it no longer has.
#[test]
fn an_upgrade_replaces_old_config_and_removes_dropped_pieces() {
    let root = scratch("upgrade");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let spec = kit.to_str().unwrap();
    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));

    let toml = read(&kit.join("KIT.toml"))
        .replace("version = \"0.1.0\"", "version = \"0.2.0\"")
        .replace("https://example.com/mcp", "https://example.com/v2")
        .replace(
            "[mcp.local]\ncommand = \"npx\"\nargs = [\"-y\", \"thing@1.0.0\"]\n",
            "",
        );
    write(&kit.join("KIT.toml"), &toml);
    write(&kit.join("RULES.md"), "Be kinder.\n");

    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let plan = text(&out.stdout);
    assert!(plan.contains("remove    mcp local"), "{plan}");
    let claude_md = read(&repo.join("CLAUDE.md"));
    assert!(
        claude_md.contains("Be kinder.") && !claude_md.contains("Be kind.\n"),
        "{claude_md}"
    );
    assert!(claude_md.contains("kit:demo 0.2.0"), "{claude_md}");
    let mcp: serde_json::Value = serde_json::from_str(&read(&repo.join(".mcp.json"))).unwrap();
    assert_eq!(mcp["mcpServers"]["docs"]["url"], "https://example.com/v2");
    assert!(mcp["mcpServers"].get("local").is_none(), "{mcp}");

    let out = env.kit(&repo, &["remove", "demo", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    for gone in [".claude", "CLAUDE.md", ".mcp.json", "kit.lock"] {
        assert!(!repo.join(gone).exists(), "{gone} should be gone");
    }
}

#[test]
fn an_upgrade_never_silently_replaces_a_hand_edited_skill() {
    let root = scratch("upgrade-edited");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let spec = kit.to_str().unwrap();
    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--no-code", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let skill = repo.join(".claude/skills/hello/SKILL.md");
    let edited = format!("{}USER EDIT\n", read(&skill));
    write(&skill, &edited);

    let toml = read(&kit.join("KIT.toml")).replace("version = \"0.1.0\"", "version = \"0.2.0\"");
    write(&kit.join("KIT.toml"), &toml);
    write(
        &kit.join("skills/hello/SKILL.md"),
        "---\nname: hello\n---\nSay hi.\n",
    );

    let out = env.kit(
        &repo,
        &["add", spec, "-a", "claude", "--no-code", "--print"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let plan = text(&out.stdout);
    assert!(
        plan.contains("edited    ") && plan.contains("hello was changed by hand"),
        "{plan}"
    );

    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--no-code", "--yes"]);
    assert!(!out.status.success(), "the upgrade must stop");
    assert!(
        text(&out.stderr).contains("--force"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(read(&skill), edited, "the edit must survive");
    assert!(read(&repo.join("CLAUDE.md")).contains("kit:demo 0.1.0"));

    let out = env.kit(
        &repo,
        &["add", spec, "-a", "claude", "--no-code", "--yes", "--force"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(read(&skill).contains("Say hi."), "{}", read(&skill));
}

#[test]
fn add_keeps_a_json_files_layout_and_remove_restores_its_bytes() {
    let root = scratch("json-layout");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let settings = repo.join(".claude/settings.json");
    let compact = "{\"permissions\":{\"allow\":[\"Bash(ls)\"]},\"env\":{\"A\":\"b\"}}\n";
    write(&settings, compact);
    let mcp = repo.join(".mcp.json");
    let odd = "{\r\n    \"mcpServers\" : {\r\n        \"mine\": {\"url\": \"https://x\"}\r\n    }\r\n}\r\n";
    write(&mcp, odd);

    let spec = kit.to_str().unwrap();
    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let added = read(&settings);
    assert_eq!(added.lines().count(), 1, "compact stays compact: {added}");
    assert!(
        added.starts_with("{\"permissions\""),
        "key order kept: {added}"
    );
    let added = read(&mcp);
    assert!(
        added.contains("\r\n    \"mcpServers\""),
        "indent and CRLF kept: {added:?}"
    );

    let out = env.kit(&repo, &["remove", "demo", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(read(&settings), compact);
    assert_eq!(read(&mcp), odd);
}

/// Two kits that define an MCP server of the same name differently.
#[test]
fn two_kits_cannot_share_an_mcp_name_with_different_servers() {
    let root = scratch("clash");
    let demo = demo_kit(&root);
    let other = root.join("other-kit");
    write(
        &other.join("KIT.toml"),
        "schema = 1\n[kit]\nname = \"other\"\ntitle = \"Other\"\nversion = \"0.1.0\"\ndescription = \"d\"\n[mcp.docs]\nurl = \"https://other.example/mcp\"\n",
    );
    let env = Env::new(&root);
    let repo = root.join("repo");
    git_repo(&repo);
    let out = env.kit(
        &repo,
        &["add", demo.to_str().unwrap(), "-a", "claude", "--yes"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let out = env.kit(
        &repo,
        &["add", other.to_str().unwrap(), "-a", "claude", "--yes"],
    );
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("installed by demo"),
        "{}",
        text(&out.stderr)
    );
    let mcp: serde_json::Value = serde_json::from_str(&read(&repo.join(".mcp.json"))).unwrap();
    assert_eq!(mcp["mcpServers"]["docs"]["url"], "https://example.com/mcp");

    // Both in one install.
    let repo2 = root.join("repo2");
    git_repo(&repo2);
    let out = env.kit(
        &repo2,
        &[
            "add",
            demo.to_str().unwrap(),
            other.to_str().unwrap(),
            "-a",
            "claude",
            "--yes",
        ],
    );
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("differently"),
        "{}",
        text(&out.stderr)
    );
}

/// Links planted at kit.lock, at the old temp name, or at a file Kit edits
/// never redirect a write outside the repo.
#[cfg(unix)]
#[test]
fn planted_links_never_redirect_a_write() {
    let root = scratch("planted");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let victim = root.join("victim");
    write(&victim, "precious");
    let spec = kit.to_str().unwrap();

    for planted in ["kit.lock.tmp", "kit.lock", "CLAUDE.md", ".mcp.json"] {
        let repo = root.join(format!("repo-{}", planted.replace('.', "_")));
        git_repo(&repo);
        std::os::unix::fs::symlink(&victim, repo.join(planted)).unwrap();
        let _ = env.kit(&repo, &["add", spec, "-a", "claude", "--yes"]);
        assert_eq!(read(&victim), "precious", "through {planted}");
        assert!(
            std::fs::symlink_metadata(repo.join(planted))
                .unwrap()
                .file_type()
                .is_symlink(),
            "{planted} is left as it was"
        );
    }
}

/// A link to another file in the same repo (or, for --global, in home) is
/// a normal setup: Kit writes the file it leads to and leaves the link.
#[cfg(unix)]
#[test]
fn links_that_stay_inside_the_scope_are_followed() {
    let root = scratch("inner-links");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let spec = kit.to_str().unwrap();

    let repo = root.join("repo");
    git_repo(&repo);
    write(&repo.join("AGENTS.md"), "# Team rules\n");
    std::os::unix::fs::symlink("AGENTS.md", repo.join("CLAUDE.md")).unwrap();
    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--no-code", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(read(&repo.join("AGENTS.md")).contains("Be kind."));
    assert!(
        std::fs::symlink_metadata(repo.join("CLAUDE.md"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let out = env.kit(&repo, &["remove", "demo", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(read(&repo.join("AGENTS.md")), "# Team rules\n");

    // Not into .git, and not into a program: Kit's text there would run.
    for (i, target) in [".git/hooks/pre-commit", "tools/run.sh"].iter().enumerate() {
        let repo = root.join(format!("hook-{i}"));
        git_repo(&repo);
        write(&repo.join(target), "#!/bin/sh\nexit 0\n");
        let mode = std::os::unix::fs::PermissionsExt::from_mode(0o755);
        std::fs::set_permissions(repo.join(target), mode).unwrap();
        std::os::unix::fs::symlink(target, repo.join("CLAUDE.md")).unwrap();
        let out = env.kit(&repo, &["add", spec, "-a", "claude", "--no-code", "--yes"]);
        assert!(!out.status.success(), "{target}");
        assert_eq!(read(&repo.join(target)), "#!/bin/sh\nexit 0\n");
    }

    let dotfiles = env.home.join("dotfiles/CLAUDE.md");
    write(&dotfiles, "# Mine\n");
    std::fs::create_dir_all(env.home.join(".claude")).unwrap();
    std::os::unix::fs::symlink(&dotfiles, env.home.join(".claude/CLAUDE.md")).unwrap();
    let out = env.kit(
        &root,
        &[
            "add",
            spec,
            "-a",
            "claude",
            "--global",
            "--no-code",
            "--yes",
        ],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(read(&dotfiles).contains("Be kind."));
    let out = env.kit(&root, &["remove", "demo", "--global", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(read(&dotfiles), "# Mine\n");
}

/// A rollback copy keeps links as links, and a pipe in a skill folder
/// stops Kit before it reads it (no hang, nothing changed).
#[cfg(unix)]
#[test]
fn links_and_pipes_in_a_skill_folder_are_never_read_through() {
    let root = scratch("special");
    let kit = demo_kit(&root);
    let env = Env::new(&root);
    let spec = kit.to_str().unwrap();
    let repo = root.join("repo");
    git_repo(&repo);
    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--no-code", "--yes"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let skill = repo.join(".claude/skills/hello");
    std::os::unix::fs::symlink("SKILL.md", skill.join("alias.md")).unwrap();
    std::os::unix::fs::symlink("loop", skill.join("loop")).unwrap();

    // An upgrade that must stop: the edit (the links) survives as links.
    let toml = read(&kit.join("KIT.toml")).replace("version = \"0.1.0\"", "version = \"0.2.0\"");
    write(&kit.join("KIT.toml"), &toml);
    write(
        &kit.join("skills/hello/SKILL.md"),
        "---\nname: hello\n---\nHi.\n",
    );
    let out = env.kit(&repo, &["add", spec, "-a", "claude", "--no-code", "--yes"]);
    assert!(!out.status.success());
    assert!(
        !text(&out.stderr).contains("os error"),
        "{}",
        text(&out.stderr)
    );
    let meta = std::fs::symlink_metadata(skill.join("alias.md")).unwrap();
    assert!(meta.file_type().is_symlink(), "still a link");

    // A pipe is refused before anything reads it.
    std::fs::remove_file(skill.join("loop")).unwrap();
    let ok = Command::new("mkfifo")
        .arg(skill.join("pipe"))
        .status()
        .unwrap()
        .success();
    assert!(ok, "mkfifo");
    let out = env.kit(
        &repo,
        &["add", spec, "-a", "claude", "--no-code", "--print"],
    );
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("not a regular file"),
        "{}",
        text(&out.stderr)
    );
}
