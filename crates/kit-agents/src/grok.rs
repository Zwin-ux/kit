//! Grok Build adapter — `grok -p` single-turn with always-approve.

use crate::process::{command_for, probe_binary, spawn_streaming};
use crate::skills;
use crate::{Agent, AgentHandle, AgentStatus, SpawnError};
use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
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
        AgentStatus {
            kind: AgentKind::Grok,
            installed: true,
            authenticated: true,
            version,
            remedy: None,
        }
    }

    async fn spawn(
        &self,
        spec: &RunSpec,
        worktree: &Path,
        tx: mpsc::Sender<RunDelta>,
    ) -> Result<Box<dyn AgentHandle>, SpawnError> {
        let skills_n = install_skills(worktree, &spec.repo, &tx).await;
        let prompt = skills::build_prompt(&spec.task, skills_n > 0);

        let _ = tx
            .send(RunDelta::Output(format!(
                "kit: spawning grok -p --cwd {} --always-approve\n",
                worktree.display()
            )))
            .await;

        let mut cmd = command_for("grok");
        cmd.arg("-p")
            .arg(&prompt)
            .arg("--cwd")
            .arg(worktree)
            .arg("--always-approve");
        // Prefer streaming JSON when available; harmless if ignored.
        cmd.arg("--output-format").arg("streaming-json");
        cmd.current_dir(worktree);

        spawn_streaming(AgentKind::Grok, cmd, tx).await
    }
}

async fn install_skills(worktree: &Path, repo: &Path, tx: &mpsc::Sender<RunDelta>) -> usize {
    let Some(src) = skills::resolve_skills_dir(repo) else {
        return 0;
    };
    match skills::install_into_worktree(worktree, &src) {
        Ok(n) => {
            let _ = skills::ensure_agents_md(worktree);
            let _ = tx
                .send(RunDelta::Output(format!(
                    "kit: installed {n} skills for grok\n"
                )))
                .await;
            n
        }
        Err(_) => 0,
    }
}
