//! Grok Build adapter — `grok -p` single-turn with always-approve.

use crate::auth;
use crate::process::{probe_binary, spawn_streaming};
use crate::skills;
use crate::{Agent, AgentHandle, AgentStatus, SpawnError};
use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
use tokio::process::Command;
use tokio::sync::mpsc;

pub struct GrokAgent;

#[async_trait::async_trait]
impl Agent for GrokAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Grok
    }

    async fn probe(&self) -> AgentStatus {
        let (installed, version) = probe_binary("grok").await;
        if !installed {
            return AgentStatus::missing(AgentKind::Grok);
        }
        let api_key = std::env::var_os("XAI_API_KEY").is_some_and(|k| !k.is_empty());
        let login = auth::grok_login(auth::grok_home().as_deref(), api_key);
        auth::installed_status(AgentKind::Grok, version, login, "grok login")
    }

    async fn spawn(
        &self,
        spec: &RunSpec,
        worktree: &Path,
        tx: mpsc::Sender<RunDelta>,
    ) -> Result<Box<dyn AgentHandle>, SpawnError> {
        let prompt = skills::build_prompt(&spec.task);

        let _ = tx
            .send(RunDelta::Output(format!(
                "kit: spawning grok -p --cwd {} --always-approve\n",
                crate::process::tilde(worktree)
            )))
            .await;

        spawn_streaming(AgentKind::Grok, grok_command(&prompt, worktree), tx).await
    }
}

/// `grok -p <prompt> --cwd <worktree> …`, spawned directly — never via `cmd /C`.
///
/// Grok Build ships a native binary, so it needs no npm shim. `cmd.exe` ends a
/// `/C` command line at the first newline: the multi-line Kit prompt was cut to
/// its title and every flag after it never reached grok (Session B stall).
fn grok_command(prompt: &str, worktree: &Path) -> Command {
    let mut cmd = Command::new("grok");
    cmd.arg("-p")
        .arg(prompt)
        .arg("--cwd")
        .arg(worktree)
        .arg("--always-approve")
        // NDJSON, one ACP session update per line (grok 1.0.25 `--help`).
        .arg("--output-format")
        .arg("streaming-json");
    cmd.current_dir(worktree);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    /// Session B: through `cmd /C` the prompt stopped at its first newline and
    /// `--cwd`, `--always-approve` and `--output-format` were dropped with it.
    #[test]
    fn grok_command_hands_multiline_prompt_and_flags_to_grok_itself() {
        let worktree = Path::new("wt");
        let prompt = "# Kit Control Room\n\n## User task\n\nsay \"hi\" & exit\n";
        let command = grok_command(prompt, worktree);
        let std_cmd = command.as_std();
        assert_eq!(std_cmd.get_program(), OsStr::new("grok"));
        let args: Vec<&OsStr> = std_cmd.get_args().collect();
        assert_eq!(
            args,
            [
                "-p",
                prompt,
                "--cwd",
                "wt",
                "--always-approve",
                "--output-format",
                "streaming-json",
            ]
            .map(OsStr::new)
        );
        assert_eq!(std_cmd.get_current_dir(), Some(worktree));
    }
}
