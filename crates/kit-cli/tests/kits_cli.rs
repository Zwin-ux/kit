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
    assert!(settings.contains("hook after-edit demo"), "{settings}");
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
        log.contains("mcp add-json --scope user local {\"args\":[\"-y\",\"thing@1.0.0\"]"),
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

    let out = env.kit(
        &repo,
        &[
            "setup",
            "--agent",
            "codex",
            "--kit",
            spec,
            "--this-repo",
            "--yes",
        ],
    );
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
