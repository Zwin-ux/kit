//! The Run — Kit's one primitive.
//!
//! Every screen is a view over Runs. See `docs/dev/PRD-1.0.md` section 4.1.
//!
//! CONTRACT FILE. Changing anything here is a Claude-only operation; agents
//! file an issue instead of editing. See `docs/dev/BUILD-ASSIGNMENT.md`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Stable, sortable run identifier (ULID — lexicographic order matches creation order).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RunId(pub String);

impl RunId {
    pub fn new() -> Self {
        Self(ulid::Ulid::new().to_string())
    }
}

impl Default for RunId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Which coding agent executes the task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    Codex,
    Claude,
    Grok,
    Ollama,
}

impl AgentKind {
    /// The executable name Kit looks for on PATH.
    pub fn binary(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Grok => "grok",
            Self::Ollama => "ollama",
        }
    }

    /// Label shown in the Control Room.
    pub fn label(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Grok => "grok",
            Self::Ollama => "ollama",
        }
    }
}

impl std::fmt::Display for AgentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Lifecycle of a run.
///
/// `Gating` is the state no competing tool has: the agent has stopped, and the
/// gate is deciding whether the work may be called done.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Queued,
    Running,
    Gating,
    Pass,
    Fail,
    Killed,
    Error,
}

impl RunState {
    /// Terminal states never transition again; their receipt is final.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Pass | Self::Fail | Self::Killed | Self::Error)
    }

    pub fn is_active(self) -> bool {
        matches!(self, Self::Queued | Self::Running | Self::Gating)
    }
}

/// Limits applied to every run. Principle 1: bounded by default, no exceptions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bounds {
    /// Wall-clock ceiling for the agent process.
    pub timeout: Duration,
    /// Maximum captured output before the run is truncated and stopped.
    pub output_cap_bytes: u64,
    /// Globs the agent may write. Empty means the whole worktree.
    pub write_allow: Vec<String>,
    /// Globs the agent may never write, applied after `write_allow`.
    pub write_deny: Vec<String>,
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30 * 60),
            output_cap_bytes: 8 * 1024 * 1024,
            write_allow: Vec::new(),
            write_deny: Vec::new(),
        }
    }
}

/// What the user dispatches. Immutable once a run starts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSpec {
    /// Repository root the run targets.
    pub repo: PathBuf,
    pub agent: AgentKind,
    /// The prompt, or a reference to a library skill.
    pub task: String,
    /// Branch name for the isolated worktree. Derived from the task when absent.
    pub branch: Option<String>,
    pub bounds: Bounds,
}

/// A run in flight or at rest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    pub id: RunId,
    pub spec: RunSpec,
    pub state: RunState,
    /// Isolated worktree path. Present once the run has started.
    pub worktree: Option<PathBuf>,
    pub started_at: Option<SystemTime>,
    pub ended_at: Option<SystemTime>,
    /// Populated when the gate has run.
    pub gate: Option<crate::GateOutcome>,
}

impl Run {
    pub fn new(spec: RunSpec) -> Self {
        Self {
            id: RunId::new(),
            spec,
            state: RunState::Queued,
            worktree: None,
            started_at: None,
            ended_at: None,
            gate: None,
        }
    }

    pub fn elapsed(&self) -> Option<Duration> {
        let start = self.started_at?;
        let end = self.ended_at.unwrap_or_else(SystemTime::now);
        end.duration_since(start).ok()
    }
}

/// Incremental update pushed from a run task to the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunDelta {
    State(RunState),
    /// One chunk of agent output. Already bounded by `Bounds::output_cap_bytes`.
    Output(String),
    Worktree(PathBuf),
    Gate(crate::GateOutcome),
}

/// Immutable record written to `~/.kit/runs/<id>/`.
///
/// Principle 2: proof or it didn't happen. A third-party tool must be able to
/// read this directory — the shape is part of Kit's public contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    /// Receipt format version. Bump on any breaking shape change.
    pub version: u32,
    pub id: RunId,
    pub spec: RunSpec,
    pub state: RunState,
    pub started_at: Option<SystemTime>,
    pub ended_at: Option<SystemTime>,
    /// Unified diff produced in the worktree. Empty when the agent changed nothing.
    pub diff: String,
    pub gate: Option<crate::GateOutcome>,
    /// Whether output hit `Bounds::output_cap_bytes`.
    pub output_truncated: bool,
}

impl Receipt {
    pub const VERSION: u32 = 1;
}
