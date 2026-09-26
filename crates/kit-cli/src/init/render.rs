//! The exact `kit.toml` text `kit init` prints and writes.

use super::check::CheckResult;
use crate::engine::infer::Detection;
use kit_core::GateConfig;
use std::time::Duration;

/// The gate to write: the proposal minus the checks that did not pass
/// `--check`. Returns the gate and the `(label, command)` pairs left out.
pub fn effective_gate(
    gate: &GateConfig,
    results: Option<&[CheckResult]>,
) -> (GateConfig, Vec<(String, String)>) {
    let Some(results) = results else {
        return (gate.clone(), Vec::new());
    };
    let failed = |label: &str, cmd: &str| {
        results
            .iter()
            .any(|r| r.label == label && r.command == cmd && !r.passed())
    };
    let mut out = gate.clone();
    let mut left_out = Vec::new();
    for (label, slot) in [
        ("format", &mut out.format),
        ("typecheck", &mut out.typecheck),
        ("test", &mut out.test),
    ] {
        if let Some(cmd) = slot.clone()
            && failed(label, &cmd)
        {
            left_out.push((label.to_string(), cmd));
            *slot = None;
        }
    }
    out.extra.retain(|cmd| {
        let bad = failed("extra", cmd);
        if bad {
            left_out.push(("extra".into(), cmd.clone()));
        }
        !bad
    });
    (out, left_out)
}

/// Render `kit.toml`. `left_out` checks become comments the user can restore.
pub fn render(det: &Detection, gate: &GateConfig, left_out: &[(String, String)]) -> String {
    let mut s = String::from("# Kit gate. Every command must exit 0 for a run to PASS.\n");
    if let (Some(tc), Some(marker)) = (det.toolchain, &det.marker) {
        s.push_str(&format!(
            "# Written by `kit init` for a {} project ({marker}).\n",
            tc.title()
        ));
    }
    for skipped in &det.skipped {
        s.push_str(&format!(
            "# Not in the gate: {skipped}. Add its commands to extra.\n"
        ));
    }
    s.push_str("\n[gate]\n");
    for (label, cmd) in [
        ("format", &gate.format),
        ("typecheck", &gate.typecheck),
        ("test", &gate.test),
    ] {
        if let Some(cmd) = cmd {
            s.push_str(&line(label, &quote(cmd)));
        }
    }
    if !gate.extra.is_empty() {
        let items: Vec<String> = gate.extra.iter().map(|c| quote(c)).collect();
        s.push_str(&line("extra", &format!("[{}]", items.join(", "))));
    }
    s.push_str(&line("timeout", &quote(&duration(gate.timeout))));
    if !left_out.is_empty() {
        s.push_str("\n# Failed at `kit init`. Fix each one, then remove the \"# \".\n");
        for (label, cmd) in left_out {
            let value = if label == "extra" {
                format!("[{}]", quote(cmd))
            } else {
                quote(cmd)
            };
            s.push_str(&format!("# {}", line(label, &value)));
        }
    }
    s
}

fn line(key: &str, value: &str) -> String {
    format!("{key:<9} = {value}\n")
}

/// A TOML basic string, escaped by the toml crate.
fn quote(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

fn duration(d: Duration) -> String {
    let secs = d.as_secs();
    match secs {
        s if s > 0 && s % 3600 == 0 => format!("{}h", s / 3600),
        s if s > 0 && s % 60 == 0 => format!("{}m", s / 60),
        s => format!("{s}s"),
    }
}
