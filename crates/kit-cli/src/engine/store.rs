//! Receipt store under `~/.kit/runs/<id>/` (PRD principle 2: proof or it didn't happen).

use anyhow::{Context, Result, bail};
use kit_core::Receipt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::paths::{run_dir, runs_dir};

/// Persist a receipt and optional output log. Returns the run directory.
pub fn write_receipt(receipt: &Receipt, output: &str) -> Result<std::path::PathBuf> {
    let dir = run_dir(&receipt.id.0);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    let json = serde_json::to_string_pretty(receipt).context("serialize receipt")?;
    fs::write(dir.join("receipt.json"), json).context("write receipt.json")?;
    fs::write(dir.join("output.log"), output).context("write output.log")?;
    if !receipt.diff.is_empty() {
        fs::write(dir.join("diff.patch"), &receipt.diff).context("write diff.patch")?;
    }
    if let Some(gate) = &receipt.gate {
        let g = serde_json::to_string_pretty(gate).context("serialize gate")?;
        fs::write(dir.join("gate.json"), g).context("write gate.json")?;
    }
    Ok(dir)
}

/// Read a receipt by run id (full ULID or unique prefix).
pub fn read_receipt(id: &str) -> Result<Option<Receipt>> {
    let dir = resolve_run_dir(id)?;
    let path = dir.join("receipt.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let receipt: Receipt = serde_json::from_str(&raw).context("parse receipt")?;
    Ok(Some(receipt))
}

/// Run directory for an id or unique prefix.
pub fn resolve_run_dir(id_or_prefix: &str) -> Result<PathBuf> {
    let exact = run_dir(id_or_prefix);
    if exact.join("receipt.json").is_file() {
        return Ok(exact);
    }
    // Prefix match (short ids from `kit receipt list`).
    let root = runs_dir();
    if !root.is_dir() {
        bail!("no runs under {}", root.display());
    }
    let mut matches: Vec<PathBuf> = fs::read_dir(&root)
        .with_context(|| format!("read {}", root.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(id_or_prefix))
                && p.join("receipt.json").is_file()
        })
        .collect();
    matches.sort();
    match matches.len() {
        0 => bail!(
            "no receipt matching id/prefix `{id_or_prefix}` under {}",
            root.display()
        ),
        1 => Ok(matches.remove(0)),
        _ => bail!(
            "ambiguous prefix `{id_or_prefix}` — matches {} runs; use a longer id",
            matches.len()
        ),
    }
}

/// One row for `kit receipt list`.
#[derive(Debug, Clone)]
pub struct ReceiptSummary {
    pub id: String,
    pub dir: PathBuf,
    pub state: String,
    pub agent: String,
    pub repo: String,
    pub task: String,
    pub gate_passed: Option<bool>,
    pub modified: Option<SystemTime>,
}

/// List receipts newest-first (by directory mtime, then id).
pub fn list_receipts(limit: usize) -> Result<Vec<ReceiptSummary>> {
    let root = runs_dir();
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut rows: Vec<ReceiptSummary> = Vec::new();
    for entry in fs::read_dir(&root).with_context(|| format!("read {}", root.display()))? {
        let entry = entry?;
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let receipt_path = dir.join("receipt.json");
        if !receipt_path.is_file() {
            continue;
        }
        let raw = match fs::read_to_string(&receipt_path) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let receipt: Receipt = match serde_json::from_str(&raw) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
        let task = truncate_task(&receipt.spec.task, 48);
        rows.push(ReceiptSummary {
            id: receipt.id.0.clone(),
            dir: dir.clone(),
            state: format!("{:?}", receipt.state).to_ascii_lowercase(),
            agent: receipt.spec.agent.label().to_string(),
            repo: receipt
                .spec
                .repo
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(".")
                .to_string(),
            task,
            gate_passed: receipt.gate.as_ref().map(|g| g.passed),
            modified,
        });
    }
    rows.sort_by(|a, b| match (b.modified, a.modified) {
        (Some(bm), Some(am)) => bm.cmp(&am).then_with(|| b.id.cmp(&a.id)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => b.id.cmp(&a.id),
    });
    if limit > 0 && rows.len() > limit {
        rows.truncate(limit);
    }
    Ok(rows)
}

/// Read raw `output.log` tail for a run (best-effort).
pub fn read_output_tail(id_or_prefix: &str, max_bytes: usize) -> Result<String> {
    let path = resolve_run_dir(id_or_prefix)?.join("output.log");
    if !path.exists() {
        return Ok(String::new());
    }
    let raw = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    if raw.len() <= max_bytes {
        return Ok(String::from_utf8_lossy(&raw).into_owned());
    }
    let start = raw.len() - max_bytes;
    // Walk forward to next char boundary / line for cleaner tail.
    let mut cut = start;
    while cut < raw.len() && raw[cut] != b'\n' {
        cut += 1;
    }
    if cut < raw.len() {
        cut += 1;
    }
    Ok(String::from_utf8_lossy(&raw[cut..]).into_owned())
}

fn truncate_task(task: &str, max: usize) -> String {
    let one_line = task.lines().next().unwrap_or(task).trim();
    if one_line.chars().count() <= max {
        return one_line.to_string();
    }
    let mut out: String = one_line.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Ensure parent kit home dirs exist (idempotent).
pub fn ensure_layout() -> Result<()> {
    fs::create_dir_all(super::paths::runs_dir())?;
    fs::create_dir_all(super::paths::worktrees_dir())?;
    Ok(())
}

/// Load kit.toml from a repo root if present; otherwise defaults.
pub fn load_kit_config(repo: &Path) -> kit_core::KitConfig {
    let path = repo.join("kit.toml");
    if !path.exists() {
        return kit_core::KitConfig::default();
    }
    match fs::read_to_string(&path) {
        Ok(raw) => toml::from_str(&raw).unwrap_or_default(),
        Err(_) => kit_core::KitConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::paths::kit_home_test_lock;
    use kit_core::{AgentKind, Bounds, RunId, RunSpec, RunState};
    use std::time::{Duration, SystemTime};

    #[test]
    fn list_and_read_roundtrip() {
        let _lock = kit_home_test_lock();
        let home = std::env::temp_dir().join(format!(
            "kit-receipt-cli-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        let _ = fs::remove_dir_all(&home);
        // SAFETY: tests serialize KIT_HOME via kit_home_test_lock.
        unsafe {
            std::env::set_var("KIT_HOME", &home);
        }
        ensure_layout().unwrap();

        let id = RunId("01TESTRECEIPTLIST00000000001".into());
        let receipt = Receipt {
            version: Receipt::VERSION,
            id: id.clone(),
            spec: RunSpec {
                repo: PathBuf::from("/tmp/kit"),
                agent: AgentKind::Codex,
                task: "smoke receipt list".into(),
                branch: None,
                bounds: Bounds::default(),
            },
            state: RunState::Pass,
            started_at: Some(SystemTime::now() - Duration::from_secs(5)),
            ended_at: Some(SystemTime::now()),
            diff: String::new(),
            gate: None,
            output_truncated: false,
        };
        write_receipt(&receipt, "hello\n").unwrap();

        let rows = list_receipts(10).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, id.0);
        assert_eq!(rows[0].agent, "codex");

        let loaded = read_receipt(&id.0).unwrap().expect("receipt");
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.spec.task, "smoke receipt list");

        let prefix = &id.0[..10];
        let by_prefix = read_receipt(prefix).unwrap().expect("prefix");
        assert_eq!(by_prefix.id, id);

        let tail = read_output_tail(prefix, 1024).unwrap();
        assert!(tail.contains("hello"));

        unsafe {
            std::env::remove_var("KIT_HOME");
        }
        let _ = fs::remove_dir_all(&home);
    }
}
