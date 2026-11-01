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
    /// Kit's own agent (`kit -p`), run as a child process.
    Kit,
}

impl AgentKind {
    /// The executable name Kit looks for on PATH.
    pub fn binary(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Grok => "grok",
            Self::Ollama => "ollama",
            Self::Kit => "kit",
        }
    }

    /// Label shown in the Control Room.
    pub fn label(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::Grok => "grok",
            Self::Ollama => "ollama",
            Self::Kit => "kit",
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
    /// The gate had no checks to run: nothing proved the work, so it is
    /// neither pass nor fail.
    Unconfigured,
    Killed,
    Error,
    /// The agent stopped because the session reached its budget. The gate
    /// still ran on what it left; the receipt says how far it got.
    Budget,
}

impl RunState {
    /// Terminal states never transition again; their receipt is final.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Pass
                | Self::Fail
                | Self::Unconfigured
                | Self::Killed
                | Self::Error
                | Self::Budget
        )
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
    /// The gate commands the run is held to, as the engine loaded or inferred
    /// them before the agent started. Adapters name them to the agent; they
    /// never re-read `kit.toml`. Receipts leave this empty: the gate outcome
    /// records what ran.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gate_checks: Vec<String>,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// What the agent reported about itself: model, tokens, cost (version 2).
    /// Absent for agents that report nothing and in version 1 receipts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentResult>,
}

impl Receipt {
    /// 2 adds `agent` and the `budget` state. Version 1 receipts still read.
    pub const VERSION: u32 = 2;
}

/// An agent's own account of a run (receipt version 2).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AgentResult {
    /// `anthropic`, `openai-compatible`, …
    pub provider: String,
    /// Model id.
    pub model: String,
    /// Permission profile: `read`, `workspace` or `full`.
    pub profile: String,
    /// How the agent's session ended: `done`, `budget`, `max_turns`, `looping`, …
    pub outcome: String,
    /// Model replies.
    pub replies: u32,
    /// Token counts summed over the session.
    pub tokens: TokenCounts,
    /// Dollars, when every reply had a known cost.
    pub cost_usd: Option<f64>,
    /// True when any cost came from a price table, not the provider.
    #[serde(default)]
    pub cost_estimated: bool,
    /// Tool calls refused, as `Tool: reason`.
    #[serde(default)]
    pub denials: Vec<String>,
}

/// Token counts. `input` excludes cached tokens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenCounts {
    pub input: u64,
    pub cached_input: u64,
    pub cache_write: u64,
    pub output: u64,
    pub reasoning: u64,
}

#[cfg(test)]
mod receipt_contract {
    use super::*;

    fn receipt(agent: Option<AgentResult>) -> Receipt {
        Receipt {
            version: Receipt::VERSION,
            id: RunId("01TEST".into()),
            spec: RunSpec {
                repo: "/r".into(),
                agent: AgentKind::Kit,
                task: "t".into(),
                branch: None,
                bounds: Bounds::default(),
                gate_checks: vec![],
            },
            state: RunState::Budget,
            started_at: None,
            ended_at: None,
            diff: String::new(),
            gate: None,
            output_truncated: false,
            agent,
        }
    }

    /// Receipt v2 shape, pinned: `agent` is snake_case and absent when unknown;
    /// version 1 receipts (no `agent`) still read.
    #[test]
    fn v2_shape_and_v1_compat() {
        let r = receipt(Some(AgentResult {
            provider: "anthropic".into(),
            model: "m".into(),
            profile: "workspace".into(),
            outcome: "budget".into(),
            replies: 3,
            tokens: TokenCounts {
                input: 10,
                cached_input: 5,
                cache_write: 1,
                output: 2,
                reasoning: 0,
            },
            cost_usd: Some(0.5),
            cost_estimated: true,
            denials: vec!["Bash: not on the read-only list".into()],
        }));
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["version"], 2);
        assert_eq!(v["state"], "budget");
        // Sorted: serde_json's key order depends on features other crates enable.
        let mut keys: Vec<&str> = v["agent"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "cost_estimated",
                "cost_usd",
                "denials",
                "model",
                "outcome",
                "profile",
                "provider",
                "replies",
                "tokens"
            ]
        );
        assert_eq!(v["agent"]["tokens"]["cached_input"], 5);

        let none = serde_json::to_value(receipt(None)).unwrap();
        assert!(none.get("agent").is_none(), "omitted, not null");
        let mut v1 = none.clone();
        v1["version"] = 1.into();
        v1["state"] = "pass".into();
        let back: Receipt = serde_json::from_value(v1).unwrap();
        assert_eq!((back.version, back.agent), (1, None));
    }
}
