//! Past runs, read from receipts at launch.
//!
//! The Control Room shows every run, not only the ones dispatched in this
//! session: runs from `kit run` (or an earlier session) are read back from
//! `<runs_dir>/<id>/receipt.json`, newest first, under an `earlier` divider.
//! Parsing lives here, not in kit-cli's store, so kit-tui never depends on
//! kit-cli (which depends on kit-tui).

use crate::app::RunRow;
use crate::persona::Persona;
use kit_core::{Receipt, RunState};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::SystemTime;

/// How many past runs the Control Room lists; the rest are counted.
pub const PAST_CAP: usize = 10;

/// Past rows to show, and how many receipts exist in all.
#[derive(Debug, Default)]
pub struct PastRuns {
    pub rows: Vec<RunRow>,
    pub total: usize,
}

/// Read up to `cap` receipts under `runs_dir`, newest first.
///
/// Run ids are ULIDs, so the folder names sort by start time; only the
/// newest `cap` readable receipts are parsed, keeping launch cheap however
/// many runs exist. A missing or unreadable folder yields no rows.
pub fn load(runs_dir: &Path, cap: usize) -> PastRuns {
    let Ok(entries) = std::fs::read_dir(runs_dir) else {
        return PastRuns::default();
    };
    let mut dirs: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.join("receipt.json").is_file())
        .collect();
    dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let total = dirs.len();
    let rows = dirs
        .iter()
        .filter_map(|dir| read_row(dir))
        .take(cap)
        .collect();
    PastRuns { rows, total }
}

/// One receipt as a read-only Control Room row.
fn read_row(dir: &Path) -> Option<RunRow> {
    let raw = std::fs::read_to_string(dir.join("receipt.json")).ok()?;
    let mut receipt: Receipt = serde_json::from_str(&raw).ok()?;
    if !receipt.state.is_terminal() {
        return None;
    }
    // Receipts written before 2.0.0 recorded a gate with no checks as a pass.
    if receipt.state == RunState::Pass
        && let Some(gate) = receipt.gate.as_mut()
        && gate.is_vacuous()
    {
        receipt.state = RunState::Unconfigured;
        gate.passed = false;
    }
    let (task, persona) = split_role(&receipt.spec.task);
    let mut row = RunRow::new(
        receipt.id.clone(),
        receipt.spec.repo.to_string_lossy().into_owned(),
        receipt.spec.agent.binary(),
        task,
    );
    row.persona = persona;
    row.state = receipt.state;
    row.gate = receipt.gate;
    row.past = true;
    row.ended_at = receipt.ended_at.or_else(|| {
        std::fs::metadata(dir.join("receipt.json"))
            .and_then(|m| m.modified())
            .ok()
    });
    row.set_diff(&receipt.diff);
    row.past_output = Some(dir.join("output.log"));
    row.output_truncated |= receipt.output_truncated;
    Some(row)
}

/// The user's task and the role it ran as. Dispatch sends
/// `Persona::wrap_task`, which puts the role brief after the task.
fn split_role(task: &str) -> (String, Persona) {
    match task.split_once("\n\n---\nRole (") {
        Some((user, role)) => {
            let persona = role
                .split_once(')')
                .and_then(|(label, _)| Persona::parse(label))
                .unwrap_or_default();
            (user.trim().to_string(), persona)
        }
        None => (task.trim().to_string(), Persona::default()),
    }
}

/// The last `cap` bytes of a file (lossy UTF-8), and whether it was cut.
pub(crate) fn read_tail(path: &Path, cap: usize) -> (String, bool) {
    let Ok(mut file) = std::fs::File::open(path) else {
        return (String::new(), false);
    };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let cut = len > cap as u64;
    if cut && file.seek(SeekFrom::Start(len - cap as u64)).is_err() {
        return (String::new(), false);
    }
    let mut bytes = Vec::with_capacity(cap.min(len as usize));
    if file.take(cap as u64).read_to_end(&mut bytes).is_err() {
        return (String::new(), false);
    }
    let text = String::from_utf8_lossy(&bytes).into_owned();
    if cut {
        // Start on a whole line.
        let start = text.find('\n').map_or(0, |i| i + 1);
        return (text[start..].to_string(), true);
    }
    (text, false)
}

/// `just now`, `40s ago`, `5m ago`, `2h ago`, `3d ago`.
pub fn format_age(ended: SystemTime, now: SystemTime) -> String {
    let Ok(age) = now.duration_since(ended) else {
        return "just now".into();
    };
    let secs = age.as_secs();
    match secs {
        0..=9 => "just now".into(),
        10..=59 => format!("{secs}s ago"),
        60..=3599 => format!("{}m ago", secs / 60),
        3600..=86_399 => format!("{}h ago", secs / 3600),
        _ => format!("{}d ago", secs / 86_400),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use kit_core::{
        AgentKind, Bounds, CheckStatus, GateCheck, GateOutcome, Receipt, RunId, RunSpec,
    };
    use std::time::Duration;

    /// A throwaway runs dir, removed on drop.
    pub(crate) struct TempRuns(pub std::path::PathBuf);

    impl TempRuns {
        pub(crate) fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "kit-tui-past-{tag}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempRuns {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    pub(crate) fn gate(passed: bool) -> GateOutcome {
        GateOutcome {
            passed,
            checks: vec![GateCheck {
                label: "test".into(),
                command: "npm test".into(),
                status: if passed {
                    CheckStatus::Pass
                } else {
                    CheckStatus::Fail
                },
                exit_code: Some(if passed { 0 } else { 1 }),
                summary: (!passed).then(|| "npm test: 2 failing".into()),
                duration: Duration::from_millis(900),
            }],
            scope_violations: vec![],
            firewall_blocks: vec![],
            duration: Duration::from_millis(900),
        }
    }

    /// Write one receipt the way the engine does (receipt.json, output.log,
    /// diff.patch when there is a diff), ended `ago` before now.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn write_receipt(
        runs: &Path,
        id: &str,
        agent: AgentKind,
        task: &str,
        state: RunState,
        gate: Option<GateOutcome>,
        diff: &str,
        ago: Duration,
    ) {
        let dir = runs.join(id);
        std::fs::create_dir_all(&dir).unwrap();
        let ended = SystemTime::now() - ago;
        let receipt = Receipt {
            version: Receipt::VERSION,
            id: RunId(id.into()),
            spec: RunSpec {
                repo: "/home/you/code/shop".into(),
                agent,
                task: task.into(),
                branch: None,
                bounds: Bounds::default(),
                gate_checks: vec![],
            },
            state,
            started_at: Some(ended - Duration::from_secs(60)),
            ended_at: Some(ended),
            diff: diff.into(),
            gate,
            output_truncated: false,
        };
        std::fs::write(
            dir.join("receipt.json"),
            serde_json::to_string_pretty(&receipt).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("output.log"),
            format!("{agent}: working on {task}\nkit: agent done. Running the gate\n"),
        )
        .unwrap();
        if !diff.is_empty() {
            std::fs::write(dir.join("diff.patch"), diff).unwrap();
        }
    }

    pub(crate) const DIFF: &str = "diff --git a/greet.txt b/greet.txt\n--- /dev/null\n+++ b/greet.txt\n@@ -0,0 +1 @@\n+hello\n";

    #[test]
    fn loads_newest_first_capped_and_counts_all() {
        let tmp = TempRuns::new("order");
        for i in 0..12u64 {
            write_receipt(
                &tmp.0,
                &format!("01PAST{i:020}"),
                AgentKind::Claude,
                &format!("task {i:02}"),
                RunState::Pass,
                Some(gate(true)),
                DIFF,
                Duration::from_secs(3600 * (12 - i)),
            );
        }
        // A dir being written (no receipt.json yet) is not a run.
        std::fs::create_dir_all(tmp.0.join("01PARTIAL")).unwrap();
        let past = load(&tmp.0, PAST_CAP);
        assert_eq!(past.total, 12);
        assert_eq!(past.rows.len(), PAST_CAP);
        assert_eq!(past.rows[0].task, "task 11");
        assert_eq!(past.rows[9].task, "task 02");
        assert!(past.rows.iter().all(|r| r.past && r.ended_at.is_some()));
    }

    #[test]
    fn a_row_carries_output_gate_diff_and_role() {
        let tmp = TempRuns::new("row");
        write_receipt(
            &tmp.0,
            "01PASTROW00000000000000000",
            AgentKind::Codex,
            &Persona::Qa.wrap_task("fix the parser"),
            RunState::Fail,
            Some(gate(false)),
            DIFF,
            Duration::from_secs(300),
        );
        let past = load(&tmp.0, PAST_CAP);
        let mut row = past.rows[0].clone();
        assert!(
            row.output.is_empty(),
            "output waits until the run is opened"
        );
        row.load_past_output();
        assert_eq!(row.task, "fix the parser");
        assert_eq!(row.persona, Persona::Qa);
        assert_eq!(row.agent, "codex");
        assert_eq!(row.repo_name(), "shop");
        assert_eq!(row.state, RunState::Fail);
        assert!(row.output.contains("working on"), "{}", row.output);
        assert!(row.diff.contains("+hello"));
        assert_eq!(
            row.failure_summary().as_deref(),
            Some("npm test: 2 failing")
        );
    }

    #[test]
    fn a_missing_runs_dir_is_no_history() {
        let past = load(Path::new("/nonexistent/kit/runs"), PAST_CAP);
        assert_eq!(past.total, 0);
        assert!(past.rows.is_empty());
    }

    #[test]
    fn old_vacuous_pass_reads_as_unconfigured() {
        let tmp = TempRuns::new("vacuous");
        write_receipt(
            &tmp.0,
            "01PASTVAC00000000000000000",
            AgentKind::Claude,
            "x",
            RunState::Pass,
            Some(GateOutcome::vacuous()),
            DIFF,
            Duration::from_secs(60),
        );
        let row = &load(&tmp.0, PAST_CAP).rows[0];
        assert_eq!(row.state, RunState::Unconfigured);
    }

    #[test]
    fn output_tail_is_capped_on_a_line() {
        let tmp = TempRuns::new("tail");
        let path = tmp.0.join("output.log");
        std::fs::write(&path, "first line\nsecond line\nthird\n").unwrap();
        let (text, cut) = read_tail(&path, 16);
        assert!(cut);
        assert_eq!(text, "third\n");
    }

    #[test]
    fn ages_read_like_people_say_them() {
        let now = SystemTime::now();
        let ago = |s: u64| format_age(now - Duration::from_secs(s), now);
        assert_eq!(ago(3), "just now");
        assert_eq!(ago(42), "42s ago");
        assert_eq!(ago(5 * 60 + 3), "5m ago");
        assert_eq!(ago(2 * 3600 + 10), "2h ago");
        assert_eq!(ago(3 * 86_400 + 5), "3d ago");
        assert_eq!(format_age(now + Duration::from_secs(5), now), "just now");
    }
}
