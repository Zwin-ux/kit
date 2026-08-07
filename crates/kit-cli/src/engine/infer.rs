//! Conservative gate inference when `kit.toml` has no checks (CEO stamp P2).
//!
//! Only emit a check when the tooling is verifiably on PATH and (for npm) the
//! script actually exists. A wrong inferred command is worse than no check.

use kit_core::GateConfig;
use std::path::Path;
use std::process::Command;

/// Infer gate checks from repo signals. Returns empty config when nothing is safe.
pub fn infer_gate(repo: &Path) -> GateConfig {
    let mut gate = GateConfig::default();

    if repo.join("Cargo.toml").is_file() && on_path("cargo") {
        gate.format = Some("cargo fmt --all --check".into());
        gate.typecheck = Some("cargo clippy --workspace --all-targets -- -D warnings".into());
        gate.test = Some("cargo test --workspace".into());
        return gate;
    }

    let pkg = repo.join("package.json");
    if pkg.is_file() {
        let runner = npm_runner(repo);
        if let Some(runner) = runner
            && let Ok(raw) = std::fs::read_to_string(&pkg)
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw)
        {
            let scripts = v.get("scripts").cloned().unwrap_or(serde_json::json!({}));
            if scripts.get("format").is_some() || scripts.get("format:check").is_some() {
                let script = if scripts.get("format:check").is_some() {
                    "format:check"
                } else {
                    "format"
                };
                gate.format = Some(format!("{runner} run {script}"));
            }
            if scripts.get("typecheck").is_some() || scripts.get("lint").is_some() {
                let script = if scripts.get("typecheck").is_some() {
                    "typecheck"
                } else {
                    "lint"
                };
                gate.typecheck = Some(format!("{runner} run {script}"));
            }
            if scripts.get("test").is_some() {
                gate.test = Some(format!("{runner} test"));
            }
        }
    }

    gate
}

/// True when outcome is a vacuous pass (no checks, no violations) — CEO: render UNCONFIGURED.
pub fn is_vacuous(outcome: &kit_core::GateOutcome) -> bool {
    outcome.passed
        && outcome.checks.is_empty()
        && outcome.scope_violations.is_empty()
        && outcome.firewall_blocks.is_empty()
}

fn npm_runner(repo: &Path) -> Option<&'static str> {
    if repo.join("pnpm-lock.yaml").is_file() && on_path("pnpm") {
        return Some("pnpm");
    }
    if repo.join("yarn.lock").is_file() && on_path("yarn") {
        return Some("yarn");
    }
    if on_path("npm") {
        return Some("npm");
    }
    None
}

fn on_path(bin: &str) -> bool {
    // Prefer a real lookup; avoid shelling for speed and sandbox friendliness.
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(bin);
            if candidate.is_file() {
                return true;
            }
            #[cfg(windows)]
            {
                for ext in ["exe", "cmd", "bat", "com"] {
                    let p = dir.join(format!("{bin}.{ext}"));
                    if p.is_file() {
                        return true;
                    }
                }
            }
        }
    }
    // Fallback: try spawning (covers Windows App Paths / PATHEXT edge cases).
    Command::new(bin)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kit_core::GateOutcome;
    use std::time::Duration;

    #[test]
    fn vacuous_detection() {
        assert!(is_vacuous(&GateOutcome::vacuous()));
        let real = GateOutcome {
            passed: true,
            checks: vec![kit_core::GateCheck {
                label: "test".into(),
                command: "cargo test".into(),
                status: kit_core::CheckStatus::Pass,
                exit_code: Some(0),
                summary: None,
                duration: Duration::from_millis(1),
            }],
            scope_violations: vec![],
            firewall_blocks: vec![],
            duration: Duration::from_millis(1),
        };
        assert!(!is_vacuous(&real));
    }

    #[test]
    fn infer_cargo_workspace_root() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        if !on_path("cargo") {
            return;
        }
        let g = infer_gate(&root);
        assert!(!g.is_empty(), "kit workspace should infer cargo checks");
        assert!(g.test.as_deref().unwrap().contains("cargo test"));
    }

    use std::path::PathBuf;
}
