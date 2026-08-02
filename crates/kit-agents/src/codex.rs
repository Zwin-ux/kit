//! Codex CLI adapter — `codex exec` headless workflow.

use crate::process::{command_for, full_auto, probe_binary, spawn_streaming};
use crate::skills;
use crate::{Agent, AgentHandle, AgentStatus, SpawnError};
use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
use tokio::sync::mpsc;

pub struct CodexAgent;

#[async_trait::async_trait]
impl Agent for CodexAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    async fn probe(&self) -> AgentStatus {
        let (installed, version) = probe_binary("codex").await;
        if !installed {
            return AgentStatus::missing(AgentKind::Codex);
        }
        AgentStatus {
            kind: AgentKind::Codex,
            installed: true,
            // Codex auth is user-global; we do not read credentials (PRD principle 4).
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
        let skills_n = prepare_skills(worktree, &spec.repo, &tx).await;
        let prompt = skills::build_prompt(&spec.task, skills_n > 0);

        let _ = tx
            .send(RunDelta::Output(format!(
                "kit: spawning codex exec in {}\n",
                worktree.display()
            )))
            .await;

        let mut cmd = command_for("codex");
        cmd.arg("exec")
            .arg("-C")
            .arg(worktree)
            .arg("-s")
            .arg("workspace-write")
            .arg("--json")
            .arg("--color")
            .arg("never");

        if full_auto() {
            cmd.arg("--dangerously-bypass-approvals-and-sandbox");
            let _ = tx
                .send(RunDelta::Output(
                    "kit: KIT_FULL_AUTO=1 — codex approvals/sandbox bypassed\n".into(),
                ))
                .await;
        }

        cmd.arg(prompt);
        cmd.current_dir(worktree);

        spawn_streaming(AgentKind::Codex, cmd, tx).await
    }
}

async fn prepare_skills(worktree: &Path, repo: &Path, tx: &mpsc::Sender<RunDelta>) -> usize {
    let Some(src) = skills::resolve_skills_dir(repo) else {
        let _ = tx
            .send(RunDelta::Output(
                "kit: no .agents/skills found — running without skill pack\n".into(),
            ))
            .await;
        return 0;
    };
    match skills::install_into_worktree(worktree, &src) {
        Ok(n) => {
            let _ = skills::ensure_agents_md(worktree);
            let _ = tx
                .send(RunDelta::Output(format!(
                    "kit: installed {n} skills into worktree from {}\n",
                    src.display()
                )))
                .await;
            n
        }
        Err(e) => {
            let _ = tx
                .send(RunDelta::Output(format!(
                    "kit: skill install failed: {e}\n"
                )))
                .await;
            0
        }
    }
}
