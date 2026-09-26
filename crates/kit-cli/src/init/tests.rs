//! Every file `kit init` can write must parse with kit-core's real
//! `KitConfig` (deny_unknown_fields) and give back the same gate.

use super::check::{CheckResult, parse_duration};
use super::render::{effective_gate, render};
use crate::engine::infer::detect;
use crate::engine::infer::tests::fixture;
use kit_core::KitConfig;
use std::time::Duration;

fn repos() -> Vec<std::path::PathBuf> {
    vec![
        fixture(&[
            ("Cargo.toml", "[package]\nname = \"x\"\n"),
            ("web/package.json", "{}"),
        ]),
        fixture(&[("go.mod", "module x\n")]),
        fixture(&[
            (
                "package.json",
                r#"{"scripts":{"format:check":"prettier --check .","typecheck":"tsc -p .",
                "lint":"eslint .","test":"vitest run"}}"#,
            ),
            ("pnpm-lock.yaml", ""),
        ]),
        fixture(&[
            (
                "package.json",
                r#"{"devDependencies":{"typescript":"5"},"scripts":{"test":"jest"}}"#,
            ),
            ("tsconfig.json", "{}"),
            ("yarn.lock", ""),
        ]),
        fixture(&[
            (
                "pyproject.toml",
                "[tool.ruff]\n[tool.mypy]\n[tool.pytest.ini_options]\n",
            ),
            ("poetry.lock", ""),
        ]),
    ]
}

#[test]
fn every_generated_kit_toml_round_trips_through_kit_core() {
    for repo in repos() {
        let det = detect(&repo);
        assert!(!det.gate.is_empty(), "{}", repo.display());
        let text = render(&det, &det.gate, &[]);
        let cfg: KitConfig =
            toml::from_str(&text).unwrap_or_else(|e| panic!("{e}\n--- kit.toml ---\n{text}"));
        assert_eq!(cfg.gate, det.gate, "{text}");
        assert_eq!(
            cfg,
            KitConfig {
                gate: det.gate.clone(),
                ..KitConfig::default()
            }
        );
    }
}

#[test]
fn failed_checks_become_comments_and_the_rest_still_parses() {
    let det = detect(&repos()[2]);
    let fail = |label: &str, command: &str| CheckResult {
        label: label.into(),
        command: command.into(),
        result: "fail",
        exit_code: Some(1),
        summary: None,
        duration: Duration::ZERO,
    };
    let mut results: Vec<CheckResult> = det
        .gate
        .checks()
        .into_iter()
        .map(|(l, c)| CheckResult {
            result: "pass",
            ..fail(l, c)
        })
        .collect();
    results[2] = fail("test", "pnpm run test");
    results[3] = fail("extra", "pnpm run lint");
    let (gate, left_out) = effective_gate(&det.gate, Some(&results));
    assert_eq!(gate.test, None);
    assert!(gate.extra.is_empty());
    assert_eq!(left_out.len(), 2);

    let text = render(&det, &gate, &left_out);
    assert!(text.contains("# test      = \"pnpm run test\""), "{text}");
    assert!(text.contains("# extra     = [\"pnpm run lint\"]"), "{text}");
    let cfg: KitConfig = toml::from_str(&text).unwrap();
    assert_eq!(cfg.gate, gate);
}

#[test]
fn commands_with_quotes_are_escaped() {
    let det = detect(&repos()[0]);
    let mut gate = det.gate.clone();
    gate.test = Some(r#"cargo test -- --skip "slow one""#.into());
    let cfg: KitConfig = toml::from_str(&render(&det, &gate, &[])).unwrap();
    assert_eq!(cfg.gate, gate);
}

#[test]
fn rendered_file_names_the_toolchain_and_skipped_projects() {
    let det = detect(&repos()[0]);
    let text = render(&det, &det.gate, &[]);
    assert!(text.contains("for a Rust project (Cargo.toml)"), "{text}");
    assert!(
        text.contains("# Not in the gate: web/package.json (Node)."),
        "{text}"
    );
    assert!(text.contains("timeout   = \"15m\""), "{text}");
}

#[test]
fn durations() {
    assert_eq!(parse_duration("90"), Some(Duration::from_secs(90)));
    assert_eq!(parse_duration("5m"), Some(Duration::from_secs(300)));
    assert_eq!(parse_duration("1h"), Some(Duration::from_secs(3600)));
    assert_eq!(parse_duration("0s"), None);
    assert_eq!(parse_duration("soon"), None);
}
