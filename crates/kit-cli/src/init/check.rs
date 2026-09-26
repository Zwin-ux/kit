//! `kit init` checks: run each proposed command once, before anything is
//! written, so the gate starts true today. Writers are never run.

use crate::engine::infer::is_writer;
use kit_core::{CheckStatus, Gate, GateConfig};
use kit_gate::KitGate;
use std::path::Path;
use std::time::Duration;

/// The result of one proposed command.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub label: String,
    pub command: String,
    /// `pass`, `fail`, `missing` (not found), `timeout` or `refused` (writer).
    pub result: &'static str,
    pub exit_code: Option<i32>,
    pub summary: Option<String>,
    pub duration: Duration,
}

impl CheckResult {
    pub fn passed(&self) -> bool {
        self.result == "pass"
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "label": self.label,
            "command": self.command,
            "result": self.result,
            "exitCode": self.exit_code,
            "summary": self.summary,
            "durationMs": self.duration.as_millis() as u64,
        })
    }
}

/// Run every check in `gate` in `repo`, one at a time, each with `timeout`.
/// `progress` gets one line per check as it finishes.
pub async fn run_all(
    repo: &Path,
    gate: &GateConfig,
    timeout: Duration,
    mut progress: impl FnMut(&str),
) -> Vec<CheckResult> {
    let engine = KitGate::new();
    let mut out = Vec::new();
    for (label, command) in gate.checks() {
        let result = if is_writer(command) {
            CheckResult {
                label: label.into(),
                command: command.into(),
                result: "refused",
                exit_code: None,
                summary: Some("the command can change files, so kit did not run it".into()),
                duration: Duration::ZERO,
            }
        } else {
            let one = GateConfig {
                extra: vec![command.to_string()],
                timeout,
                ..GateConfig::default()
            };
            let outcome = engine.evaluate(repo, &one).await;
            let check = outcome.checks.into_iter().next();
            let (result, exit_code, summary, duration) = match check {
                Some(c) => (
                    match c.status {
                        CheckStatus::Pass => "pass",
                        // kit-gate fails a command it cannot start, with no exit code.
                        CheckStatus::Fail if c.exit_code.is_none() => "missing",
                        CheckStatus::Fail => "fail",
                        CheckStatus::Skipped => "missing",
                        CheckStatus::TimedOut => "timeout",
                    },
                    c.exit_code,
                    c.summary,
                    c.duration,
                ),
                None => ("missing", None, None, Duration::ZERO),
            };
            CheckResult {
                label: label.into(),
                command: command.into(),
                result,
                exit_code,
                summary,
                duration,
            }
        };
        progress(&format!(
            "  {:<7} {:<9} {}  ({:.1}s)",
            result.result.to_ascii_uppercase(),
            result.label,
            result.command,
            result.duration.as_secs_f64()
        ));
        if !result.passed()
            && let Some(first) = result.summary.as_deref().and_then(|s| s.lines().next())
        {
            progress(&format!("          {first}"));
        }
        out.push(result);
    }
    out
}

/// Parse `90`, `90s`, `5m` or `1h`.
pub fn parse_duration(raw: &str) -> Option<Duration> {
    let raw = raw.trim();
    let (num, mult) = match raw.chars().last()? {
        's' => (&raw[..raw.len() - 1], 1),
        'm' => (&raw[..raw.len() - 1], 60),
        'h' => (&raw[..raw.len() - 1], 3600),
        _ => (raw, 1),
    };
    let n: u64 = num.parse().ok()?;
    (n > 0).then(|| Duration::from_secs(n * mult))
}
