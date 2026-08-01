//! The gate — Kit's differentiator.
//!
//! No run reports success without a gate result (PRD principle 2). The engine
//! lives in `kit-gate`; the shapes and the seam live here so that `Run` and
//! `Receipt` can carry a gate result without a circular dependency.
//!
//! CONTRACT FILE. Changing anything here is a Claude-only operation; agents
//! file an issue instead of editing. See `docs/dev/BUILD-ASSIGNMENT.md`.

use crate::config::GateConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Fail,
    /// Not configured for this repo. Not a failure.
    Skipped,
    /// Exceeded `GateConfig::timeout`.
    TimedOut,
}

/// Result of one command in the gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateCheck {
    /// `format`, `typecheck`, `test`, or `extra`.
    pub label: String,
    pub command: String,
    pub status: CheckStatus,
    pub exit_code: Option<i32>,
    /// First meaningful error line, surfaced directly in the Control Room.
    /// This is what the user reads instead of opening a log.
    pub summary: Option<String>,
    pub duration: Duration,
}

impl GateCheck {
    pub fn passed(&self) -> bool {
        matches!(self.status, CheckStatus::Pass | CheckStatus::Skipped)
    }
}

/// Verdict for a whole run. Attached to `Run` and frozen into `Receipt`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateOutcome {
    pub passed: bool,
    pub checks: Vec<GateCheck>,
    /// Files written outside `ScopeConfig::allow`, or inside `deny`.
    pub scope_violations: Vec<String>,
    /// Commands the firewall refused during the run.
    pub firewall_blocks: Vec<String>,
    pub duration: Duration,
}

impl GateOutcome {
    /// A run with no configured checks and no violations. Honest, not a pass:
    /// callers should infer defaults rather than claim proof that never ran.
    pub fn vacuous() -> Self {
        Self {
            passed: true,
            checks: Vec::new(),
            scope_violations: Vec::new(),
            firewall_blocks: Vec::new(),
            duration: Duration::ZERO,
        }
    }

    /// The line shown next to a failing run in the Control Room.
    pub fn first_failure(&self) -> Option<&GateCheck> {
        self.checks.iter().find(|c| !c.passed())
    }
}

/// Verdict from the blast-radius firewall on a single command.
///
/// Ported from Guardian's `guard.js`. Threat model: an agent generating a
/// destructive command in good faith — not a human evading the check. Biases
/// hard toward allowing, and fails open on Kit's own bugs (PRD principle 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirewallVerdict {
    Allow,
    /// Refused, with the reason shown to the user.
    Block {
        reason: String,
    },
    /// Logged and allowed — `FirewallMode::Warn`.
    Warn {
        reason: String,
    },
}

/// The gate seam. `kit-gate` implements this; `kit-core` and the TUI consume it.
#[async_trait::async_trait]
pub trait Gate: Send + Sync {
    /// Run the configured checks in `worktree` and return a verdict.
    ///
    /// Must never panic: a Kit defect must not block real work.
    async fn evaluate(&self, worktree: &Path, config: &GateConfig) -> GateOutcome;

    /// Judge one shell command before it executes.
    fn screen(&self, command: &str) -> FirewallVerdict;
}
