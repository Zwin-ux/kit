//! Receipt store under `~/.kit/runs/<id>/` (PRD principle 2: proof or it didn't happen).

use anyhow::{Context, Result};
use kit_core::Receipt;
use std::fs;
use std::path::Path;

use super::paths::run_dir;

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

/// Read a receipt if present (for future `kit receipt show`).
#[allow(dead_code)]
pub fn read_receipt(id: &str) -> Result<Option<Receipt>> {
    let path = run_dir(id).join("receipt.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let receipt: Receipt = serde_json::from_str(&raw).context("parse receipt")?;
    Ok(Some(receipt))
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
