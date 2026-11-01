//! Kit's own agent as a run's agent: the `kit` binary itself, as
//! `kit -p - --profile workspace` in the worktree, with the task on stdin.
//! So `kit run --agent kit` gets the same streaming, timeout, kill, gate and
//! receipt as every other agent, with no special case.

use crate::process::spawn_streaming_with_stdin;
use crate::{Agent, AgentHandle, AgentStatus, SpawnError};
use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::sync::mpsc;

pub struct KitAgent;

/// The `kit` binary to run: `KIT_AGENT_BIN` (tests), else this executable.
fn kit_binary() -> Option<PathBuf> {
    std::env::var_os("KIT_AGENT_BIN")
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
}

/// A provider is configured: an Anthropic key, a model named in the
/// environment, or an `[agent]` section in Kit's config.
fn provider_configured() -> bool {
    let set = |k: &str| std::env::var_os(k).is_some_and(|v| !v.is_empty());
    if set("ANTHROPIC_API_KEY") || set("KIT_MODEL") {
        return true;
    }
    let home = std::env::var_os("KIT_HOME").map(PathBuf::from).or_else(|| {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(|h| PathBuf::from(h).join(".kit"))
    });
    home.and_then(|h| std::fs::read_to_string(h.join("config.toml")).ok())
        .is_some_and(|t| t.lines().any(|l| l.trim() == "[agent]"))
}

#[async_trait::async_trait]
impl Agent for KitAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Kit
    }

    async fn probe(&self) -> AgentStatus {
        let ready = provider_configured();
        AgentStatus {
            kind: AgentKind::Kit,
            installed: kit_binary().is_some(),
            authenticated: ready,
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            remedy: (!ready).then(|| {
                "set ANTHROPIC_API_KEY, or [agent] model (and base_url for a local server) in ~/.kit/config.toml".to_owned()
            }),
        }
    }

    async fn spawn(
        &self,
        spec: &RunSpec,
        worktree: &Path,
        tx: mpsc::Sender<RunDelta>,
    ) -> Result<Box<dyn AgentHandle>, SpawnError> {
        let bin = kit_binary().ok_or_else(|| SpawnError::Io {
            kind: AgentKind::Kit,
            source: std::io::Error::other("cannot find the kit executable"),
        })?;
        spawn_streaming_with_stdin(
            AgentKind::Kit,
            kit_command(&bin, worktree),
            spec.task.clone(),
            tx,
        )
        .await
    }
}

fn kit_command(bin: &Path, worktree: &Path) -> Command {
    let mut cmd = Command::new(bin);
    cmd.args(["-p", "-", "--profile", "workspace"])
        .current_dir(worktree);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn runs_itself_with_the_task_on_stdin() {
        let cmd = kit_command(Path::new("kit"), Path::new("wt"));
        let args: Vec<&OsStr> = cmd.as_std().get_args().collect();
        assert_eq!(args, ["-p", "-", "--profile", "workspace"]);
        assert_eq!(cmd.as_std().get_current_dir(), Some(Path::new("wt")));
    }
}
