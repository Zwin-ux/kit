//! Fixture repos in temp dirs, one per ecosystem and trap.

use super::*;
use kit_core::GateOutcome;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

/// A fresh temp dir holding `files` (path, contents).
pub(crate) fn fixture(files: &[(&str, &str)]) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kit-infer-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, body) in files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn checks(det: &Detection) -> Vec<(String, String)> {
    det.gate
        .checks()
        .into_iter()
        .map(|(l, c)| (l.to_string(), c.to_string()))
        .collect()
}

#[test]
fn vacuous_detection() {
    assert!(is_vacuous(&GateOutcome::vacuous()));
}

#[test]
fn rust_repo_gets_fmt_clippy_test() {
    let det = detect(&fixture(&[("Cargo.toml", "[package]\nname = \"x\"\n")]));
    assert_eq!(det.toolchain, Some(Toolchain::Rust));
    assert_eq!(det.gate.format.as_deref(), Some("cargo fmt --all --check"));
    assert!(
        det.gate
            .typecheck
            .as_deref()
            .unwrap()
            .contains("-D warnings")
    );
    assert_eq!(det.gate.test.as_deref(), Some("cargo test --workspace"));
}

#[test]
fn go_repo_format_check_fails_on_a_diff() {
    let det = detect(&fixture(&[("go.mod", "module x\n")]));
    assert_eq!(det.toolchain, Some(Toolchain::Go));
    assert_eq!(det.gate.format.as_deref(), Some("gofmt -d ."));
    assert_eq!(det.gate.typecheck.as_deref(), Some("go vet ./..."));
    assert_eq!(det.gate.test.as_deref(), Some("go test ./..."));
}

#[test]
fn node_writer_format_script_is_not_the_format_check() {
    let pkg = r#"{"scripts":{"format":"prettier --write .","lint":"eslint . --fix","test":"vitest run"}}"#;
    let det = detect(&fixture(&[
        ("package.json", pkg),
        ("package-lock.json", "{}"),
    ]));
    assert_eq!(det.gate.format, None, "{:?}", det.gate);
    assert!(det.gate.extra.is_empty(), "eslint --fix is a writer");
    assert_eq!(det.gate.test.as_deref(), Some("npm run test"));
    assert!(det.notes.iter().any(|n| n.contains("prettier --write")));
}

#[test]
fn node_prefers_check_scripts() {
    let pkg = r#"{"scripts":{"format":"prettier --write .","format:check":"prettier --check .",
        "typecheck":"tsc --noEmit","lint":"eslint .","test":"jest"}}"#;
    let det = detect(&fixture(&[("package.json", pkg), ("pnpm-lock.yaml", "")]));
    assert_eq!(
        checks(&det),
        vec![
            ("format".into(), "pnpm run format:check".into()),
            ("typecheck".into(), "pnpm run typecheck".into()),
            ("test".into(), "pnpm run test".into()),
            ("extra".into(), "pnpm run lint".into()),
        ]
    );
}

#[test]
fn node_plain_format_script_needs_a_check_flag() {
    let ok = r#"{"scripts":{"format":"prettier --check ."}}"#;
    let det = detect(&fixture(&[("package.json", ok)]));
    assert_eq!(det.gate.format.as_deref(), Some("npm run format"));
    let delegated = r#"{"scripts":{"format":"turbo run format"}}"#;
    let det = detect(&fixture(&[("package.json", delegated)]));
    assert_eq!(det.gate.format, None);
}

#[test]
fn node_lockfile_picks_the_package_manager() {
    for (lock, pm) in [
        ("pnpm-lock.yaml", "pnpm"),
        ("yarn.lock", "yarn"),
        ("bun.lockb", "bun"),
        ("bun.lock", "bun"),
        ("package-lock.json", "npm"),
    ] {
        let det = detect(&fixture(&[
            ("package.json", r#"{"scripts":{"test":"jest"}}"#),
            (lock, ""),
        ]));
        assert_eq!(det.gate.test, Some(format!("{pm} run test")), "{lock}");
    }
    // packageManager wins over a stale lockfile.
    let pkg = r#"{"packageManager":"yarn@4.1.0","scripts":{"test":"jest"}}"#;
    let det = detect(&fixture(&[
        ("package.json", pkg),
        ("package-lock.json", ""),
    ]));
    assert_eq!(det.gate.test.as_deref(), Some("yarn run test"));
    assert!(
        det.notes
            .iter()
            .any(|n| n.contains("another package manager: package-lock.json")),
        "{:?}",
        det.notes
    );
}

#[test]
fn node_tsc_only_when_typescript_is_installed() {
    let with = r#"{"devDependencies":{"typescript":"^5"}}"#;
    let det = detect(&fixture(&[
        ("package.json", with),
        ("tsconfig.json", "{}"),
        ("pnpm-lock.yaml", ""),
    ]));
    assert_eq!(
        det.gate.typecheck.as_deref(),
        Some("pnpm exec tsc --noEmit")
    );
    let det = detect(&fixture(&[("package.json", "{}"), ("tsconfig.json", "{}")]));
    assert_eq!(det.gate.typecheck, None);
}

#[test]
fn node_placeholder_and_watch_tests_are_not_used() {
    let pkg = r#"{"scripts":{"test":"echo \"Error: no test specified\" && exit 1"}}"#;
    assert_eq!(detect(&fixture(&[("package.json", pkg)])).gate.test, None);
    let pkg = r#"{"scripts":{"test":"jest --watch"}}"#;
    assert_eq!(detect(&fixture(&[("package.json", pkg)])).gate.test, None);
    let pkg = r#"{"scripts":{"test":"jest --watchAll=false"}}"#;
    assert!(
        detect(&fixture(&[("package.json", pkg)]))
            .gate
            .test
            .is_some()
    );
}

#[test]
fn python_uses_only_configured_tools() {
    let py = "[project]\nname = \"x\"\n[dependency-groups]\ndev = [\"ruff>=0.4\", \"pytest-cov\"]\n[tool.mypy]\nstrict = true\n";
    let det = detect(&fixture(&[("pyproject.toml", py), ("uv.lock", "")]));
    assert_eq!(det.toolchain, Some(Toolchain::Python));
    assert_eq!(
        det.gate.format.as_deref(),
        Some("uv run ruff format --check")
    );
    assert_eq!(det.gate.typecheck.as_deref(), Some("uv run mypy ."));
    assert_eq!(det.gate.test, None, "pytest-cov is not pytest");
    assert_eq!(det.gate.extra, vec!["uv run ruff check".to_string()]);

    let det = detect(&fixture(&[("requirements.txt", "flask\npytest==8.0\n")]));
    assert_eq!(checks(&det), vec![("test".into(), "pytest".into())]);

    let det = detect(&fixture(&[("pyproject.toml", "[project]\nname = \"x\"\n")]));
    assert!(det.gate.is_empty());
    assert!(det.notes.iter().any(|n| n.contains("No ruff")));
}

#[test]
fn mixed_repo_picks_root_toolchain_and_names_the_rest() {
    let det = detect(&fixture(&[
        ("Cargo.toml", "[workspace]\n"),
        ("package.json", "{}"),
        ("web/package.json", "{}"),
        ("node_modules/x/package.json", "{}"),
        ("tools/go.mod", "module t\n"),
    ]));
    assert_eq!(det.toolchain, Some(Toolchain::Rust));
    assert_eq!(
        det.skipped,
        vec![
            "package.json (Node) at the repo root".to_string(),
            "tools/go.mod (Go)".into(),
            "web/package.json (Node)".into(),
        ]
    );
}

#[test]
fn empty_repo_detects_nothing() {
    let det = detect(&fixture(&[("README.md", "hi")]));
    assert_eq!(det.toolchain, None);
    assert!(det.gate.is_empty());
}

#[test]
fn writer_detection() {
    for w in [
        "prettier --write .",
        "prettier .",
        "eslint . --fix",
        "biome check --write",
        "cargo fmt",
        "black src",
        "ruff format",
        "gofmt -w .",
        "tsc && prettier -w src",
    ] {
        assert!(is_writer(w), "{w}");
    }
    for c in [
        "prettier --check .",
        "cargo fmt --all --check",
        "gofmt -d .",
        "ruff format --check",
        "eslint .",
        "tsc --noEmit",
        "go vet ./...",
    ] {
        assert!(!is_writer(c), "{c}");
    }
}

#[test]
fn live_inference_matches_init_on_this_workspace() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    if !on_path("cargo") {
        return;
    }
    // One source of truth: with cargo on PATH, the run path infers exactly
    // what `kit init` proposes.
    assert_eq!(infer_gate(&root), detect(&root).gate);
}

#[test]
fn a_kits_extra_checks_add_to_the_inferred_ones() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    if !on_path("cargo") {
        return;
    }
    let kit_only = GateConfig {
        extra: vec!["swiftlint lint --quiet".into()],
        ..GateConfig::default()
    };
    let kits = ["swiftlint lint --quiet".to_string()];
    let gate = with_inferred(&kit_only, &root, &kits).expect("inferred");
    let inferred = infer_gate(&root);
    assert_eq!(gate.test, inferred.test);
    assert_eq!(gate.format, inferred.format);
    assert_eq!(
        gate.extra.last().map(String::as_str),
        Some("swiftlint lint --quiet")
    );
    assert_eq!(gate.timeout, inferred.timeout);

    // A gate with a named check of its own is left as it is.
    let own = GateConfig {
        test: Some("make test".into()),
        ..kit_only
    };
    assert_eq!(with_inferred(&own, &root, &kits), None);
}

#[test]
fn a_user_written_gate_with_only_extra_runs_exactly_that_gate() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let mine = GateConfig {
        extra: vec!["make lint".into()],
        ..GateConfig::default()
    };
    // No kit added it, or a kit added something else: nothing is inferred,
    // and a live run says why (a clone or CI has no record of kit lines).
    assert_eq!(with_inferred(&mine, &root, &[]), None);
    if on_path("cargo") {
        let note = inference_note(&mine, &root, &[]).expect("a note");
        assert!(
            note.contains("no Kit record") && note.contains("`kit add <kit>`"),
            "{note}"
        );
    }
    let kits = ["make lint".to_string()];
    assert_eq!(
        inference_note(&mine, &root, &kits),
        None,
        "recorded: inferred, no note"
    );
    let own = GateConfig {
        test: Some("make test".into()),
        ..mine.clone()
    };
    assert_eq!(
        inference_note(&own, &root, &[]),
        None,
        "a named check: as written"
    );
    assert_eq!(
        with_inferred(&mine, &root, &["swiftlint lint --quiet".to_string()]),
        None
    );
}
