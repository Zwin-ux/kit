//! Agent adapters — the seam between Kit and each coding CLI.
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

mod claude;
mod codex;
mod grok;
mod ollama;
mod process;
pub mod skills;

use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
use tokio::sync::mpsc;

pub use claude::ClaudeAgent;
pub use codex::CodexAgent;
pub use grok::GrokAgent;
pub use ollama::OllamaAgent;

/// Why an adapter could not start.
#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("{0} is not installed or not on PATH")]
    NotInstalled(AgentKind),
    #[error("{0} is installed but not authenticated")]
    NotAuthenticated(AgentKind),
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
    }
}

/// Probe all four agents (for `kit doctor`).
pub async fn probe_all() -> Vec<AgentStatus> {
    let mut out = Vec::with_capacity(4);
    for kind in [
        AgentKind::Codex,
        AgentKind::Claude,
        AgentKind::Grok,
        AgentKind::Ollama,
    ] {
        out.push(adapter(kind).probe().await);
    }
    out
}
