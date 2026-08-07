//! Claude Code adapter — `claude -p` non-interactive.

use crate::process::{command_for, full_auto, probe_binary, spawn_streaming};
use crate::skills;
use crate::{Agent, AgentHandle, AgentStatus, SpawnError};
use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
use tokio::sync::mpsc;

pub struct ClaudeAgent;

#[async_trait::async_trait]
impl Agent for ClaudeAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    async fn probe(&self) -> AgentStatus {
        let (installed, version) = probe_binary("claude").await;
        if !installed {
            return AgentStatus::missing(AgentKind::Claude);
        }
        AgentStatus {
            kind: AgentKind::Claude,
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
        let skills_src = install_skills(worktree, &spec.repo, &tx).await;
        let prompt = skills::build_prompt(&spec.task, skills_src.as_deref());

        let _ = tx
            .send(RunDelta::Output(format!(
                "kit: spawning claude -p in {}\n",
                worktree.display()
            )))
            .await;

        let mut cmd = command_for("claude");
        cmd.arg("-p").arg(&prompt);
        if full_auto() {
            cmd.arg("--dangerously-skip-permissions");
            let _ = tx
                .send(RunDelta::Output(
                    "kit: KIT_FULL_AUTO=1 — claude permission checks skipped\n".into(),
                ))
                .await;
        }
        cmd.current_dir(worktree);

        spawn_streaming(AgentKind::Claude, cmd, tx).await
    }
}

async fn install_skills(
    worktree: &Path,
    repo: &Path,
    tx: &mpsc::Sender<RunDelta>,
) -> Option<std::path::PathBuf> {
    let src = skills::resolve_skills_dir(repo)?;
    match skills::install_into_worktree(worktree, &src) {
        Ok(n) => {
            let _ = skills::ensure_agents_md(worktree);
            let _ = tx
                .send(RunDelta::Output(format!(
                    "kit: installed {n} skills for claude\n"
                )))
                .await;
            Some(src)
        }
        Err(_) => None,
    }
}
