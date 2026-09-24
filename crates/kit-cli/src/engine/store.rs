//! Receipt store under `~/.kit/runs/<id>/` (PRD principle 2: proof or it didn't happen).

use anyhow::{Context, Result, bail};
use kit_core::Receipt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::paths::{run_dir, runs_dir};

/// Persist a receipt and its output log. Returns the run directory.
///
/// Write-once: the run dir must not exist yet, so a second write for the same
/// id fails and never replaces proof. `receipt.json` is published last, by
/// rename, so a write that dies part way leaves no file that reads as a
/// complete receipt (readers skip a dir without `receipt.json`).
///
/// `base` is the commit the run's worktree started at. It goes in `base.txt`
/// (the frozen `Receipt` shape has no field for it); `kit land` applies
/// `diff.patch` onto it.
pub fn write_receipt(
    receipt: &Receipt,
    output: &str,
    base: Option<&str>,
) -> Result<std::path::PathBuf> {
    let dir = run_dir(&receipt.id.0);
    if let Some(parent) = dir.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::create_dir(&dir).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            anyhow::anyhow!(
                "receipt for run {} already exists at {} (receipts are write-once)",
                receipt.id,
                dir.display()
            )
        } else {
            anyhow::Error::new(e).context(format!("create {}", dir.display()))
        }
    })?;

    write_new(&dir.join("output.log"), output.as_bytes())?;
    if !receipt.diff.is_empty() {
        write_new(&dir.join("diff.patch"), receipt.diff.as_bytes())?;
    }
    if let Some(base) = base {
        write_new(&dir.join(BASE_FILE), format!("{base}\n").as_bytes())?;
    }
    if let Some(gate) = &receipt.gate {
        let g = serde_json::to_string_pretty(gate).context("serialize gate")?;
        write_new(&dir.join("gate.json"), g.as_bytes())?;
    }
    let json = serde_json::to_string_pretty(receipt).context("serialize receipt")?;
    let tmp = dir.join("receipt.json.tmp");
    write_new(&tmp, json.as_bytes())?;
    fs::rename(&tmp, dir.join("receipt.json")).context("publish receipt.json")?;
    Ok(dir)
}

/// Sidecar in the run dir: the base commit sha, one line.
pub const BASE_FILE: &str = "base.txt";

/// The base commit recorded for a run dir, if any.
pub fn read_base(dir: &Path) -> Option<String> {
    let raw = fs::read_to_string(dir.join(BASE_FILE)).ok()?;
    let sha = raw.trim();
    (!sha.is_empty()).then(|| sha.to_string())
}

/// Create `path` (never overwrite) and flush `bytes` to disk.
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("create {}", path.display()))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .with_context(|| format!("write {}", path.display()))
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
/// The repo's `kit.toml`, or the default when there is none.
///
/// A file that exists but does not parse is an error, never the default:
/// one typo (an unknown key) would otherwise switch the gate off in silence.
pub fn load_kit_config(repo: &Path) -> anyhow::Result<kit_core::KitConfig> {
    let path = repo.join("kit.toml");
    if !path.exists() {
        return Ok(kit_core::KitConfig::default());
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", path.display()))?;
    toml::from_str(&raw).map_err(|e| anyhow::anyhow!("{} is not valid: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::paths::kit_home_test_lock;
    use kit_core::{AgentKind, Bounds, RunId, RunSpec, RunState};
    use std::time::{Duration, SystemTime};

    /// `lint` is not a gate key. A typo must stop the run, not turn the gate off.
    #[test]
    fn broken_kit_toml_is_an_error_not_an_empty_gate() {
        let root = std::env::temp_dir().join(format!("kit-bad-toml-{}", RunId::default().0));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("kit.toml"), "[gate]\nlint = \"cargo clippy\"\n").unwrap();
        let err = load_kit_config(&root).unwrap_err().to_string();
        assert!(err.contains("kit.toml is not valid"), "{err}");
        assert!(err.contains("lint"), "{err}");
        fs::remove_dir_all(&root).unwrap();
        // No file at all is still the default, not an error.
        assert!(load_kit_config(&root).unwrap().gate.is_empty());
    }

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
        write_receipt(&receipt, "hello\n", None).unwrap();

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

    /// Scratch `KIT_HOME` for one store test; callers hold the lock.
    fn scratch_home(tag: &str) -> PathBuf {
        let home = std::env::temp_dir().join(format!(
            "kit-store-{tag}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&home);
        // SAFETY: tests serialize KIT_HOME via kit_home_test_lock.
        unsafe {
            std::env::set_var("KIT_HOME", &home);
        }
        ensure_layout().unwrap();
        home
    }

    fn drop_home(home: &Path) {
        unsafe {
            std::env::remove_var("KIT_HOME");
        }
        let _ = fs::remove_dir_all(home);
    }

    fn sample(id: &str, state: RunState, diff: &str) -> Receipt {
        Receipt {
            version: Receipt::VERSION,
            id: RunId(id.into()),
            spec: RunSpec {
                repo: PathBuf::from("/tmp/kit"),
                agent: AgentKind::Codex,
                task: "write once".into(),
                branch: None,
                bounds: Bounds::default(),
            },
            state,
            started_at: None,
            ended_at: Some(SystemTime::now()),
            diff: diff.into(),
            gate: None,
            output_truncated: false,
        }
    }

    /// Proof is write-once: a second receipt for the same id is refused and
    /// every byte of the first stays on disk.
    #[test]
    fn second_write_for_same_id_errors_and_keeps_original() {
        let _lock = kit_home_test_lock();
        let home = scratch_home("once");
        let id = "01TESTWRITEONCE000000000001";
        let dir = write_receipt(
            &sample(id, RunState::Pass, "+pass\n"),
            "first\n",
            Some("abc123"),
        )
        .unwrap();
        assert_eq!(read_base(&dir).as_deref(), Some("abc123"));
        let files = ["receipt.json", "output.log", "diff.patch", BASE_FILE];
        let before: Vec<Vec<u8>> = files
            .iter()
            .map(|f| fs::read(dir.join(f)).unwrap())
            .collect();

        let err = write_receipt(
            &sample(id, RunState::Error, "+other\n"),
            "second\n",
            Some("def456"),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("already exists"), "{err}");

        let after: Vec<Vec<u8>> = files
            .iter()
            .map(|f| fs::read(dir.join(f)).unwrap())
            .collect();
        assert_eq!(before, after, "the first receipt was changed");
        assert_eq!(read_receipt(id).unwrap().unwrap().state, RunState::Pass);
        assert!(!dir.join("receipt.json.tmp").exists());
        drop_home(&home);
    }

    /// A run dir without `receipt.json` (a write that died before publish)
    /// is not a receipt: list skips it and read does not find it.
    #[test]
    fn reader_ignores_dir_without_receipt_json() {
        let _lock = kit_home_test_lock();
        let home = scratch_home("torn");
        let torn = run_dir("01TESTTORNRECEIPT0000000001");
        fs::create_dir_all(&torn).unwrap();
        fs::write(torn.join("output.log"), "half\n").unwrap();
        fs::write(torn.join("receipt.json.tmp"), "{\"version\": 1").unwrap();
        write_receipt(
            &sample("01TESTWHOLERECEIPT000000001", RunState::Pass, ""),
            "",
            None,
        )
        .unwrap();

        let rows = list_receipts(0).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "01TESTWHOLERECEIPT000000001");
        assert!(read_receipt("01TESTTORN").is_err());
        drop_home(&home);
    }

    #[test]
    fn workspace_kit_toml_declares_a_real_gate() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root");
        let cfg = load_kit_config(&root).expect("kit.toml");
        assert!(
            !cfg.gate.is_empty(),
            "kit.toml must exist and declare checks so Kit-on-Kit is not vacuous"
        );
        assert!(cfg.gate.test.as_deref().unwrap().contains("cargo test"));
        assert_eq!(cfg.firewall.mode, kit_core::FirewallMode::Block);
    }
}
