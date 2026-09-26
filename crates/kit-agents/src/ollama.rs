//! Ollama adapter — local model via `ollama run`.

use crate::process::{command_for, command_with_args, probe_binary, spawn_streaming_with_stdin};
use crate::skills;
use crate::{Agent, AgentHandle, AgentStatus, SpawnError};
use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
use std::time::Duration;
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
        // An installed CLI is not enough: runs need the server and the model.
        let model = model_name();
        let remedy = readiness_remedy(installed_models().await.as_deref(), &model);
        AgentStatus {
            kind: AgentKind::Ollama,
            installed: true,
            authenticated: remedy.is_none(), // local: "ready" means runnable
            version: version.filter(|v| !v.contains("could not connect")),
            remedy,
        }
    }

    async fn spawn(
        &self,
        spec: &RunSpec,
        worktree: &Path,
        tx: mpsc::Sender<RunDelta>,
    ) -> Result<Box<dyn AgentHandle>, SpawnError> {
        let model = model_name();
        // `ollama run` silently pulls a missing model (GBs): Session B's ollama
        // "hang". Fail fast with the fix. If the list is unavailable, run decides.
        if let Some(installed) = installed_models().await
            && !has_model(&installed, &model)
        {
            return Err(SpawnError::Io {
                kind: AgentKind::Ollama,
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "model '{model}' is not pulled; run `ollama pull {model}` or set \
                         KIT_OLLAMA_MODEL to an installed model [{}]",
                        installed.join(", ")
                    ),
                ),
            });
        }

        let skills_src = install_skills(worktree, &spec.repo, &tx).await;
        let prompt = skills::build_prompt(&spec.task, skills_src.as_deref());

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

/// Model names from `ollama list`; `None` when the list itself fails (server
/// down, timeout).
async fn installed_models() -> Option<Vec<String>> {
    let mut cmd = command_with_args("ollama", &["list"]);
    cmd.kill_on_drop(true);
    let out = tokio::time::timeout(Duration::from_secs(10), cmd.output())
        .await
        .ok()?
        .ok()?;
    out.status
        .success()
        .then(|| parse_model_names(&String::from_utf8_lossy(&out.stdout)))
}

/// First column of `ollama list` output, header row skipped.
fn parse_model_names(list: &str) -> Vec<String> {
    list.lines()
        .skip(1)
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_string)
        .collect()
}

/// `ollama run llama3.1` resolves to `llama3.1:latest`; tagged names match exactly.
/// What stops a run right now, or `None` when `ollama run <model>` would work.
/// `models` is `None` when `ollama list` failed: the server is not running.
fn readiness_remedy(models: Option<&[String]>, model: &str) -> Option<String> {
    match models {
        None => Some("start the Ollama server: ollama serve".into()),
        Some(models) if !has_model(models, model) => Some(format!(
            "pull the model: ollama pull {model} (or set KIT_OLLAMA_MODEL)"
        )),
        Some(_) => None,
    }
}

fn has_model(installed: &[String], model: &str) -> bool {
    installed.iter().any(|name| {
        name == model || (!model.contains(':') && name.strip_suffix(":latest") == Some(model))
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `ollama list` on the Session B host (2026-09-11).
    const LIST: &str = "NAME               ID              SIZE      MODIFIED     \n\
gemma4:e2b         7fbdbf8f5e45    7.2 GB    4 months ago    \n\
llama3.1:latest    46e0c10c039e    4.9 GB    5 months ago\n";

    #[test]
    fn parses_model_names_from_ollama_list() {
        assert_eq!(parse_model_names(LIST), ["gemma4:e2b", "llama3.1:latest"]);
        assert!(parse_model_names("NAME ID SIZE MODIFIED\n").is_empty());
    }

    #[test]
    fn untagged_model_means_latest_and_default_is_not_pulled() {
        let installed = parse_model_names(LIST);
        assert!(has_model(&installed, "llama3.1"));
        assert!(has_model(&installed, "llama3.1:latest"));
        assert!(has_model(&installed, "gemma4:e2b"));
        assert!(!has_model(&installed, "gemma4"));
        // Default model: `ollama run llama3.2` here would start a silent pull.
        assert!(!has_model(&installed, "llama3.2"));
    }

    #[test]
    fn ready_only_when_server_answers_and_model_is_pulled() {
        let installed = parse_model_names(LIST);
        assert_eq!(readiness_remedy(Some(&installed), "llama3.1"), None);
        let pull = readiness_remedy(Some(&installed), "llama3.2").unwrap();
        assert!(pull.contains("ollama pull llama3.2"), "{pull}");
        let serve = readiness_remedy(None, "llama3.1").unwrap();
        assert!(serve.contains("ollama serve"), "{serve}");
    }
}
