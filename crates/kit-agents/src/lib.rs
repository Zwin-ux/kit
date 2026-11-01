//! Agent adapters — the seam between Kit and each coding CLI.
//!
//! Part of the `kit` command (crate `kitctl`), published only because
//! `cargo install` needs it. Not a stable library API: no semver promise.
//!
//! One trait, four implementations. A broken adapter degrades one agent and
//! never the control room (PRD risk table).
//!
//! **Contract:** the `Agent` trait is Claude-only to change. Adding adapters
//! in modules is ordinary work (this crate).
//!
//! Skills: every live spawn injects a skill pack (`.agents/skills` or `skills/`) into the
//! worktree and prepends a routing preamble — Codex-style headless workflow
//! with engineering discipline.

mod auth;
mod claude;
mod codex;
mod grok;
mod kit;
mod ollama;
mod process;
pub mod skills;

use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
use tokio::sync::mpsc;

pub use claude::ClaudeAgent;
pub use codex::CodexAgent;
pub use grok::GrokAgent;
pub use kit::KitAgent;
pub use ollama::OllamaAgent;
pub use process::{ProcessTree, configure_tree_kill, full_auto};

/// Why an adapter could not start.
#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("{0} is not installed or not on PATH")]
    NotInstalled(AgentKind),
    #[error("{0} is installed but not authenticated")]
    NotAuthenticated(AgentKind),
    /// The agent's headless mode has no edit-only setting, so it would run
    /// shell commands without asking. It runs only with KIT_FULL_AUTO=1.
    #[error(
        "{0} runs without approval prompts (its headless mode has no edit-only setting), \
         so Kit starts it only with KIT_FULL_AUTO=1. Set that in a sandbox, or use another agent"
    )]
    NeedsFullAuto(AgentKind),
    #[error("failed to start {kind}: {source}")]
    Io {
        kind: AgentKind,
        #[source]
        source: std::io::Error,
    },
}

/// A started agent process. Dropping this must not orphan the child.
#[async_trait::async_trait]
pub trait AgentHandle: Send + Sync {
    /// Wait for the process to exit and return its status code.
    async fn wait(&mut self) -> std::io::Result<i32>;

    /// Stop the process. Idempotent — killing an exited run is not an error.
    ///
    /// Must not deadlock with a concurrent [`Self::wait`] / [`Self::try_wait`]
    /// on another task: implementations release locks between polls.
    async fn kill(&mut self) -> std::io::Result<()>;

    /// Non-blocking exit probe. `None` if still running.
    async fn try_wait(&mut self) -> std::io::Result<Option<i32>> {
        // Default: not concurrent-safe; ChildHandle overrides.
        Ok(None)
    }
}

/// One coding agent Kit can dispatch to.
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    fn kind(&self) -> AgentKind;

    /// Whether the CLI is present and usable. Cheap — called during discovery.
    async fn probe(&self) -> AgentStatus;

    /// Start the task in `worktree`, streaming output through `tx`.
    async fn spawn(
        &self,
        spec: &RunSpec,
        worktree: &Path,
        tx: mpsc::Sender<RunDelta>,
    ) -> Result<Box<dyn AgentHandle>, SpawnError>;

    /// After the process exits: the agent's own account of the run (model,
    /// tokens, cost), for the receipt. Agents that report nothing return None.
    fn take_result(&self) -> Option<kit_core::AgentResult> {
        None
    }
}

/// Result of probing one agent, shown in Doctor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStatus {
    pub kind: AgentKind,
    pub installed: bool,
    pub authenticated: bool,
    pub version: Option<String>,
    /// What the user should run to fix it. Every error names the fix.
    pub remedy: Option<String>,
}

impl AgentStatus {
    pub fn missing(kind: AgentKind) -> Self {
        Self {
            kind,
            installed: false,
            authenticated: false,
            version: None,
            remedy: Some(format!("install the {} CLI", kind.binary())),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.installed && self.authenticated
    }
}

/// Return the adapter for a kind.
pub fn adapter(kind: AgentKind) -> Box<dyn Agent> {
    match kind {
        AgentKind::Codex => Box::new(CodexAgent),
        AgentKind::Claude => Box::new(ClaudeAgent),
        AgentKind::Grok => Box::new(GrokAgent),
        AgentKind::Ollama => Box::new(OllamaAgent),
        AgentKind::Kit => Box::new(KitAgent::default()),
    }
}

/// Probe every agent (for `kit doctor`), Kit's own last.
pub async fn probe_all() -> Vec<AgentStatus> {
    let mut out = Vec::with_capacity(5);
    for kind in [
        AgentKind::Codex,
        AgentKind::Claude,
        AgentKind::Grok,
        AgentKind::Ollama,
        AgentKind::Kit,
    ] {
        out.push(adapter(kind).probe().await);
    }
    out
}
