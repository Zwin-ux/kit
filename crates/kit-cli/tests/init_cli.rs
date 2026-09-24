//! `kit init` through the real binary: fixture repos in temp dirs.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

fn repo(files: &[(&str, &str)]) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kit-init-cli-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, body) in files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

fn kit(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kit"))
        .arg("init")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run kit")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("{e}: stdout was {}", text(&out.stdout)))
}

const NODE: &str =
    r#"{"scripts":{"format":"prettier --write .","lint":"eslint .","test":"vitest run"}}"#;

#[test]
fn writes_a_kit_toml_that_kit_core_accepts() {
    let dir = repo(&[("package.json", NODE), ("pnpm-lock.yaml", "")]);
    let out = kit(&dir, &[]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    assert!(stdout.contains("Wrote "), "{stdout}");
    assert!(stdout.contains("prettier --write"), "writer note: {stdout}");
    let raw = std::fs::read_to_string(dir.join("kit.toml")).unwrap();
    let cfg: kit_core::KitConfig = toml::from_str(&raw).unwrap();
    assert_eq!(
        cfg.gate.format, None,
        "the writer script is not the format check"
    );
    assert_eq!(cfg.gate.test.as_deref(), Some("pnpm run test"));
    assert_eq!(cfg.gate.extra, vec!["pnpm run lint".to_string()]);
}

#[test]
fn refuses_to_overwrite_without_force() {
    let dir = repo(&[
        ("go.mod", "module x\n"),
        ("kit.toml", "[gate]\ntest = \"mine\"\n"),
    ]);
    let out = kit(&dir, &[]);
    assert_eq!(out.status.code(), Some(2));
    let err = text(&out.stderr);
    assert!(
        err.contains("kit.toml already exists") && err.contains("--force"),
        "{err}"
    );
    let kept = std::fs::read_to_string(dir.join("kit.toml")).unwrap();
    assert!(kept.contains("mine"), "file must be unchanged");

    let out = kit(&dir, &["--force"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("Replaced "));
    let new = std::fs::read_to_string(dir.join("kit.toml")).unwrap();
    assert!(new.contains("go test ./..."), "{new}");
}

#[test]
fn print_writes_nothing_and_stdout_is_the_file() {
    let dir = repo(&[("Cargo.toml", "[package]\nname = \"x\"\n")]);
    let out = kit(&dir, &["--print"]);
    assert!(out.status.success());
    assert!(!dir.join("kit.toml").exists());
    let cfg: kit_core::KitConfig = toml::from_str(&text(&out.stdout)).unwrap();
    assert_eq!(cfg.gate.test.as_deref(), Some("cargo test --workspace"));
    assert!(text(&out.stderr).contains("Rust project (Cargo.toml)"));
}

#[test]
fn empty_repo_fails_with_a_hand_written_example() {
    let dir = repo(&[("README.md", "hi\n")]);
    let out = kit(&dir, &[]);
    assert_eq!(out.status.code(), Some(2));
    let err = text(&out.stderr);
    assert!(err.contains("no project found"), "{err}");
    assert!(
        err.contains("Write kit.toml by hand") && err.contains("[gate]"),
        "{err}"
    );
    assert!(!dir.join("kit.toml").exists());

    let out = kit(&dir, &["--json"]);
    assert_eq!(out.status.code(), Some(2));
    let v = json(&out);
    assert_eq!(v["command"], "init");
    assert_eq!(v["ok"], false);
    assert!(v["data"].is_null());
    assert!(v["error"].as_str().unwrap().contains("no project found"));
}

#[test]
fn json_envelope_shape() {
    let dir = repo(&[("package.json", NODE), ("yarn.lock", "")]);
    let out = kit(&dir, &["--json", "--print"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let v = json(&out);
    assert_eq!(v["schemaVersion"], 1);
    assert_eq!(v["command"], "init");
    assert_eq!(v["ok"], true);
    assert!(v["error"].is_null());
    assert!(v["warnings"].is_array());
    let d = &v["data"];
    assert_eq!(d["toolchain"], "node");
    assert_eq!(d["marker"], "package.json");
    assert_eq!(d["written"], false);
    assert_eq!(d["existed"], false);
    assert!(d["checks"].is_null(), "no --check, no results");
    assert!(
        d["notes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n.as_str().unwrap().contains("yarn"))
    );
    let cfg: kit_core::KitConfig = toml::from_str(d["toml"].as_str().unwrap()).unwrap();
    assert_eq!(cfg.gate.test.as_deref(), Some("yarn run test"));
}

/// `--check` keeps the checks that pass and comments out the ones that fail.
#[test]
fn check_comments_out_a_failing_command() {
    if Command::new("cargo").arg("--version").output().is_err() {
        return;
    }
    let dir = repo(&[
        (
            "Cargo.toml",
            "[package]\nname = \"fx\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
        ),
        (
            "src/lib.rs",
            "#[test]\nfn red() {\n    panic!(\"red\");\n}\n",
        ),
    ]);
    // A failing check must not be dropped in silence: a gate without `test`
    // could later PASS on format alone.
    let refused = kit(&dir, &["--check", "--json", "--timeout", "5m"]);
    assert_eq!(refused.status.code(), Some(2), "{}", text(&refused.stderr));
    let err = json(&refused);
    assert_eq!(err["ok"], false);
    let msg = err["error"].as_str().unwrap();
    assert!(
        msg.contains("test") && msg.contains("--drop-failing"),
        "{msg}"
    );
    assert!(
        !dir.join("kit.toml").exists(),
        "refused init wrote kit.toml"
    );

    let out = kit(
        &dir,
        &["--check", "--drop-failing", "--json", "--timeout", "5m"],
    );
    assert!(out.status.success(), "{}", text(&out.stderr));
    let v = json(&out);
    let checks = v["data"]["checks"].as_array().unwrap();
    let result =
        |label: &str| checks.iter().find(|c| c["label"] == label).unwrap()["result"].clone();
    assert_eq!(result("format"), "pass", "{checks:?}");
    assert_eq!(result("test"), "fail", "{checks:?}");
    let raw = std::fs::read_to_string(dir.join("kit.toml")).unwrap();
    assert!(
        raw.contains("# test      = \"cargo test --workspace\""),
        "{raw}"
    );
    let cfg: kit_core::KitConfig = toml::from_str(&raw).unwrap();
    assert_eq!(cfg.gate.test, None);
    assert!(cfg.gate.format.is_some());
}
