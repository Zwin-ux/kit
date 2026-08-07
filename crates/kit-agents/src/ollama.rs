//! Ollama adapter — local model via `ollama run`.

use crate::process::{command_for, probe_binary, spawn_streaming_with_stdin};
use crate::skills;
use crate::{Agent, AgentHandle, AgentStatus, SpawnError};
use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
use tokio::sync::mpsc;

pub struct OllamaAgent;

fn model_name() -> String {
    std::env::var("KIT_OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".into())
}

#[async_trait::async_trait]
impl Agent for OllamaAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Ollama
    }

    async fn probe(&self) -> AgentStatus {
        let (installed, version) = probe_binary("ollama").await;
        if !installed {
            return AgentStatus::missing(AgentKind::Ollama);
        }
        AgentStatus {
            kind: AgentKind::Ollama,
            installed: true,
            authenticated: true, // local; no cloud auth
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
        let model = model_name();

        let _ = tx
            .send(RunDelta::Output(format!(
                "kit: spawning ollama run {model} (cwd {})\n",
                worktree.display()
            )))
            .await;

        // ollama run MODEL reads prompt from stdin when not interactive.
        let mut cmd = command_for("ollama");
        cmd.arg("run").arg(&model);
        cmd.current_dir(worktree);

        spawn_streaming_with_stdin(AgentKind::Ollama, cmd, prompt, tx).await
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
                    "kit: installed {n} skills for ollama context\n"
                )))
                .await;
            Some(src)
        }
        Err(_) => None,
    }
}
