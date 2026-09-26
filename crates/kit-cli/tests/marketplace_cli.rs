//! `kit search | sync | new` and `kit add github:` through the real binary,
//! against a local "github" (KIT_GIT_BASE) holding an index repo and a kit
//! repo. No network.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

fn scratch(tag: &str) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kit-market-{tag}-{}-{}",
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

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
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
        .env_remove("GIT_DIR")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn kit_toml(name: &str, skill: &str) -> String {
    format!(
        "schema = 1\n[kit]\nname = \"{name}\"\ntitle = \"{name}\"\nversion = \"0.3.0\"\ndescription = \"Tips for iOS design\"\n[rules]\nfile = \"RULES.md\"\n[[skill]]\nname = \"{skill}\"\npath = \"skills/{skill}\"\nlicence = \"MIT\"\n"
    )
}

/// `base/owner/kits` (index.toml + kits/tipper) and `base/owner/solo` (a
/// kit at the repo root).
fn fake_github(base: &Path) {
    let kits = base.join("owner/kits");
    write(
        &kits.join("kits/tipper/KIT.toml"),
        &kit_toml("tipper", "tip"),
    );
    write(&kits.join("kits/tipper/RULES.md"), "Tipper rules.\n");
    write(
        &kits.join("kits/tipper/skills/tip/SKILL.md"),
        "---\nname: tip\n---\nTip.\n",
    );
    git(&kits, &["init", "-q", "-b", "main"]);
    git(&kits, &["add", "."]);
    git(&kits, &["commit", "-q", "-m", "kit"]);
    let sha = git(&kits, &["rev-parse", "HEAD"]);
    write(
        &kits.join("index.toml"),
        &format!(
            "schema = 1\n[[kit]]\nname = \"tipper\"\ntitle = \"Tipper\"\nsummary = \"Tips for iOS design\"\nsource = \"github:owner/kits\"\npath = \"kits/tipper\"\nrev = \"{sha}\"\nlevel = \"official\"\ntags = [\"ios\"]\n"
        ),
    );
    git(&kits, &["add", "."]);
    git(&kits, &["commit", "-q", "-m", "index"]);

    let solo = base.join("owner/solo");
    write(&solo.join("KIT.toml"), &kit_toml("solo", "solo-tip"));
    write(&solo.join("RULES.md"), "Solo rules.\n");
    write(
        &solo.join("skills/solo-tip/SKILL.md"),
        "---\nname: solo-tip\n---\nSolo.\n",
    );
    git(&solo, &["init", "-q", "-b", "main"]);
    git(&solo, &["add", "."]);
    git(&solo, &["commit", "-q", "-m", "kit"]);
}

struct Env {
    root: PathBuf,
    home: PathBuf,
    base: PathBuf,
}

impl Env {
    fn new(tag: &str) -> Self {
        let root = scratch(tag);
        let env = Self {
            home: root.join("home"),
            base: root.join("github"),
            root,
        };
        std::fs::create_dir_all(&env.home).unwrap();
        fake_github(&env.base);
        env
    }

    fn kit(&self, dir: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_kit"))
            .args(args)
            .current_dir(dir)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("KIT_HOME", self.home.join(".kit"))
            .env("KIT_GIT_BASE", &self.base)
            .env("KIT_INDEX", "github:owner/kits")
            .env_remove("GIT_DIR")
            .output()
            .expect("run kit")
    }

    fn ok(&self, dir: &Path, args: &[&str]) -> String {
        let out = self.kit(dir, args);
        assert!(
            out.status.success(),
            "kit {args:?}\nstdout: {}\nstderr: {}",
            text(&out.stdout),
            text(&out.stderr)
        );
        text(&out.stdout)
    }

    fn repo(&self) -> PathBuf {
        let repo = self.root.join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        git(&repo, &["init", "-q"]);
        repo
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn search_finds_starter_and_index_kits_best_first() {
    let env = Env::new("search");
    let dir = env.root.clone();
    let out = env.ok(&dir, &["search", "ios"]);
    assert!(out.starts_with("tipper"), "{out}");
    // Only Kit's own index can call a kit Official.
    assert!(out.contains("tipper   Index"), "{out}");

    let out = env.ok(&dir, &["search"]);
    for name in ["essentials", "frontend-design", "llm-engineer", "tipper"] {
        assert!(out.contains(name), "{name} missing: {out}");
    }
    let out = env.ok(&dir, &["search", "frontend"]);
    assert!(out.starts_with("frontend-design"), "{out}");

    let out = env.ok(&dir, &["search", "quantum", "widgets"]);
    assert!(out.contains("No kits match \"quantum widgets\""), "{out}");
    assert!(out.contains("kit new quantum"), "{out}");

    let out = env.ok(&dir, &["search", "ios", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["command"], "search");
    assert_eq!(v["data"]["kits"][0]["name"], "tipper");
    assert_eq!(v["data"]["kits"][0]["installed"], false);

    // Offline with no cache: the starter kits still show, with a note.
    std::fs::remove_dir_all(env.base.join("owner")).unwrap();
    let out = env.ok(&dir, &["search", "design", "--refresh"]);
    assert!(out.contains("could not refresh"), "{out}");
    std::fs::remove_dir_all(env.home.join(".kit/index")).unwrap();
    let out = env.ok(&dir, &["search", "design"]);
    assert!(out.contains("frontend-design"), "{out}");
    assert!(out.contains("could not be read"), "{out}");
}

#[test]
fn index_and_github_kits_install_and_sync_restores_exactly_what_the_lock_pins() {
    let env = Env::new("sync");
    let repo = env.repo();

    let out = env.ok(&repo, &["add", "tipper", "-a", "claude", "--yes"]);
    assert!(out.contains("Index: listed in the kit index"), "{out}");
    assert!(out.contains("github:owner/kits/kits/tipper@"), "{out}");
    assert!(repo.join(".claude/skills/tip/SKILL.md").is_file());
    let lock: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(repo.join("kit.lock")).unwrap()).unwrap();
    assert_eq!(lock["kits"][0]["source"], "tipper");
    assert!(
        lock["kits"][0]["pin"]
            .as_str()
            .unwrap()
            .starts_with("github:owner/kits/kits/tipper@")
    );

    let out = env.ok(
        &repo,
        &["add", "github:owner/solo", "-a", "claude", "--yes"],
    );
    assert!(out.contains("Direct source, not reviewed by Kit"), "{out}");
    assert!(repo.join(".claude/skills/solo-tip/SKILL.md").is_file());

    let out = env.ok(&repo, &["sync"]);
    assert!(out.contains("In sync: kit.lock pins 2 kits"), "{out}");
    env.ok(&repo, &["sync", "--check"]);

    // A teammate's clone is missing a skill and a rules block.
    std::fs::remove_dir_all(repo.join(".claude/skills/tip")).unwrap();
    std::fs::remove_file(repo.join("CLAUDE.md")).unwrap();
    let out = env.kit(&repo, &["sync", "--check"]);
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(1), "out of sync is exit 1");
    let err = text(&out.stdout);
    assert!(err.contains("3 of"), "{err}");
    assert!(err.contains(".claude/skills/tip"), "{err}");
    assert!(
        !repo.join(".claude/skills/tip").exists(),
        "--check writes nothing"
    );

    // Upstream moved on: sync still installs the pinned commit.
    let kits = env.base.join("owner/kits");
    write(
        &kits.join("kits/tipper/skills/tip/SKILL.md"),
        "---\nname: tip\n---\nNEW.\n",
    );
    git(&kits, &["commit", "-q", "-am", "move"]);

    let out = env.ok(&repo, &["sync", "--yes"]);
    assert!(out.contains("Missing   3 of"), "{out}");
    assert!(out.contains("This repo matches kit.lock."), "{out}");
    let skill = std::fs::read_to_string(repo.join(".claude/skills/tip/SKILL.md")).unwrap();
    assert!(skill.contains("Tip."), "{skill}");
    assert!(
        std::fs::read_to_string(repo.join("CLAUDE.md"))
            .unwrap()
            .contains("<!-- kit:tipper")
    );
    let out = env.ok(&repo, &["sync"]);
    assert!(out.contains("In sync"), "{out}");

    // A lock whose hash does not match the pinned source is refused.
    let lock_file = repo.join("kit.lock");
    let raw = std::fs::read_to_string(&lock_file).unwrap();
    let mut lock: serde_json::Value = serde_json::from_str(&raw).unwrap();
    for kit in lock["kits"].as_array_mut().unwrap() {
        for a in kit["applied"].as_array_mut().unwrap() {
            if a["kind"] == "skill" {
                a["hash"] = "sha256:0000".into();
            }
        }
    }
    std::fs::write(&lock_file, serde_json::to_string(&lock).unwrap()).unwrap();
    std::fs::remove_dir_all(repo.join(".claude/skills/tip")).unwrap();
    let out = env.kit(&repo, &["sync", "--yes"]);
    assert!(!out.status.success());
    assert!(
        text(&out.stderr).contains("is not what kit.lock pins"),
        "{}",
        text(&out.stderr)
    );
    assert!(!repo.join(".claude/skills/tip").exists());
    std::fs::write(&lock_file, raw).unwrap();

    // A new machine: the same kits, for all projects, from a copied lock.
    let copied = env.root.join("old-kit.lock");
    std::fs::copy(&lock_file, &copied).unwrap();
    let out = env.ok(
        &repo,
        &[
            "sync",
            "--global",
            "--from",
            copied.to_str().unwrap(),
            "--yes",
        ],
    );
    assert!(out.contains("This machine matches"), "{out}");
    assert!(env.home.join(".claude/skills/tip/SKILL.md").is_file());
    assert!(env.home.join(".claude/skills/solo-tip/SKILL.md").is_file());
    let out = env.ok(&repo, &["sync", "--global"]);
    assert!(out.contains("In sync"), "{out}");
}

#[test]
fn a_new_kit_is_valid_shows_plans_and_can_start_from_another() {
    let env = Env::new("new");
    let dir = env.root.clone();
    let out = env.ok(&dir, &["new", "ios-team", "--extends", "essentials"]);
    assert!(out.starts_with("Created ios-team/"), "{out}");
    assert!(out.contains("kit add github:<you>/ios-team"), "{out}");
    for f in [
        "KIT.toml",
        "RULES.md",
        "README.md",
        "skills/ios-team-house-style/SKILL.md",
    ] {
        assert!(dir.join("ios-team").join(f).is_file(), "{f}");
    }
    let out = env.ok(&dir, &["show", "./ios-team"]);
    assert!(out.contains("extends   essentials"), "{out}");
    // A kit on its own (no upstream fetch) plans without writing.
    env.ok(&dir, &["new", "plain", "--description", "Plain rules"]);
    let repo = env.repo();
    let spec = dir.join("plain");
    let out = env.ok(
        &repo,
        &["add", spec.to_str().unwrap(), "-a", "codex", "--print"],
    );
    assert!(out.contains("plain-house-style"), "{out}");
    assert!(out.contains("Nothing was written"), "{out}");

    let out = env.kit(&dir, &["new", "ios-team"]);
    assert!(
        text(&out.stderr).contains("not empty"),
        "{}",
        text(&out.stderr)
    );
    let out = env.kit(&dir, &["new", "Bad_Name"]);
    assert!(text(&out.stderr).contains("lowercase"));
    let out = env.kit(&dir, &["new", "essentials"]);
    assert!(text(&out.stderr).contains("--from essentials"));

    let out = env.ok(&dir, &["new", "my-tips", "--from", "tipper", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["data"]["from"], "tipper");
    let toml = std::fs::read_to_string(dir.join("my-tips/KIT.toml")).unwrap();
    assert!(toml.contains("name = \"my-tips\""), "{toml}");
    assert!(toml.contains("version = \"0.1.0\""), "{toml}");
    assert!(dir.join("my-tips/skills/tip/SKILL.md").is_file());
    assert_eq!(
        std::fs::read_to_string(dir.join("my-tips/RULES.md")).unwrap(),
        "Tipper rules.\n"
    );

    env.ok(&dir, &["new", "web-plus", "--from", "frontend-design"]);
    let out = env.ok(&dir, &["show", "./web-plus"]);
    assert!(out.contains("chrome-devtools"), "{out}");
}
