//! Codex CLI adapter — `codex exec` headless workflow.

use crate::auth;
use crate::process::{command_for, full_auto, probe_binary, spawn_streaming_with_stdin};
use crate::skills;
use crate::{Agent, AgentHandle, AgentStatus, SpawnError};
use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
use tokio::process::Command;
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
        // Ask codex itself; kit never reads credentials (PRD principle 4).
        let login = match auth::run_status("codex", &["login", "status"]).await {
            Some(out) => {
                auth::parse_codex_status(out.success, &format!("{}\n{}", out.stdout, out.stderr))
            }
            None => auth::Login::Unknown("`codex login status` did not answer"),
        };
        auth::installed_status(AgentKind::Codex, version, login, "codex login")
    }

    async fn spawn(
        &self,
        spec: &RunSpec,
        worktree: &Path,
        tx: mpsc::Sender<RunDelta>,
    ) -> Result<Box<dyn AgentHandle>, SpawnError> {
        let skills_src = prepare_skills(worktree, &spec.repo, &tx).await;
        let prompt = skills::build_prompt(&spec.task, skills_src.as_deref());

        let _ = tx
            .send(RunDelta::Output(format!(
                "kit: spawning codex exec in {}\n",
                worktree.display()
            )))
            .await;

        let bypass = full_auto();
        if bypass {
            let _ = tx
                .send(RunDelta::Output(
                    "kit: KIT_FULL_AUTO=1 — codex approvals/sandbox bypassed\n".into(),
                ))
                .await;
        }

        let cmd = codex_command("codex", worktree, bypass);
        spawn_streaming_with_stdin(AgentKind::Codex, cmd, prompt, tx).await
    }
}

/// `codex exec … -`: the prompt goes to stdin, never onto the command line.
///
/// On Windows `command_for` runs the npm shim through `cmd /C`, and cmd.exe
/// honours no escapes: a `"` in the prompt (the retry task quotes gate output)
/// would end the quoted argument and let `&` run a host command. Only fixed
/// flags and Kit's own worktree path go on that line.
fn codex_command(binary: &str, worktree: &Path, bypass: bool) -> Command {
    let mut cmd = command_for(binary);
    cmd.arg("exec")
        .arg("-C")
        .arg(worktree)
        .arg("-s")
        .arg("workspace-write")
        .arg("--json")
        .arg("--color")
        .arg("never");
    if bypass {
        cmd.arg("--dangerously-bypass-approvals-and-sandbox");
    }
    cmd.arg("-");
    cmd.current_dir(worktree);
    cmd
}

async fn prepare_skills(
    worktree: &Path,
    repo: &Path,
    tx: &mpsc::Sender<RunDelta>,
) -> Option<std::path::PathBuf> {
    let Some(src) = skills::resolve_skills_dir(repo) else {
        let _ = tx
            .send(RunDelta::Output(
                "kit: no skill pack found (.agents/skills or skills/) — running without pack\n"
                    .into(),
            ))
            .await;
        return None;
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
            Some(src)
        }
        Err(e) => {
            let _ = tx
                .send(RunDelta::Output(format!(
                    "kit: skill install failed: {e}\n"
                )))
                .await;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    /// A prompt that closes cmd.exe's quoting and chains a host command.
    const HOSTILE: &str = "# Kit\n\nfix \"a\" & echo KIT_INJECTED & \"b\"\n";

    #[test]
    fn codex_argv_carries_no_prompt_text() {
        for bypass in [false, true] {
            let cmd = codex_command("codex", Path::new("wt"), bypass);
            let args: Vec<&OsStr> = cmd.as_std().get_args().collect();
            assert_eq!(
                args.last(),
                Some(&OsStr::new("-")),
                "prompt is read from stdin"
            );
            assert!(
                args.iter()
                    .all(|a| !a.to_string_lossy().contains("KIT_INJECTED")),
                "{args:?}"
            );
            assert_eq!(cmd.as_std().get_current_dir(), Some(Path::new("wt")));
        }
    }

    /// End to end through the real spawn path: a stand-in `codex` echoes its
    /// argv and stdin. The prompt must arrive on stdin, byte for byte.
    #[cfg(unix)]
    #[tokio::test]
    async fn codex_gets_the_prompt_on_stdin_not_argv() {
        let prompt = format!("{HOSTILE}{}", "x".repeat(256 * 1024));
        let out = crate::process::test_support::run_fake_agent(
            |fake, wt| codex_command(fake, wt, false),
            &prompt,
        )
        .await;
        assert!(!out.argv.contains("KIT_INJECTED"), "argv: {}", out.argv);
        assert_eq!(out.stdin, prompt);
    }
}
