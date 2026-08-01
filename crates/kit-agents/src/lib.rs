//! Agent adapters — the seam between Kit and each coding CLI.
//!
//! One trait, four implementations. A broken adapter degrades one agent and
//! never the control room (PRD risk table).
//!
//! CONTRACT FILE. Changing the `Agent` trait is a Claude-only operation;
//! agents file an issue instead of editing. Adding an implementation in
//! another module is ordinary work.

use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
use tokio::sync::mpsc;

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
    async fn kill(&mut self) -> std::io::Result<()>;
}

/// One coding agent Kit can dispatch to.
///
/// Implementations live in this crate, one module per agent. They must not
/// read, store, or copy provider credentials (PRD principle 4) — each CLI
/// uses its own existing login.
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    fn kind(&self) -> AgentKind;

    /// Whether the CLI is present and usable. Cheap — called during discovery,
    /// which must finish in under a second for all agents combined.
    async fn probe(&self) -> AgentStatus;

    /// Start the task in `worktree`, streaming output through `tx`.
    ///
    /// The adapter is responsible for translating `spec.bounds` into the CLI's
    /// own flags where possible, and for running headless with no TTY prompt.
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
