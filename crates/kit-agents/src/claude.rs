//! Claude Code adapter — `claude -p` non-interactive.

use crate::auth;
use crate::process::{command_for, full_auto, probe_binary, spawn_streaming_with_stdin};
use crate::skills;
use crate::{Agent, AgentHandle, AgentStatus, SpawnError};
use kit_core::{AgentKind, RunDelta, RunSpec};
use std::path::Path;
use tokio::process::Command;
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
        let login = match auth::run_status("claude", &["auth", "status", "--json"]).await {
            Some(out) => auth::parse_claude_status(&out.stdout),
            None => auth::Login::Unknown("`claude auth status` did not answer"),
        };
        auth::installed_status(AgentKind::Claude, version, login, "claude auth login")
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

        let bypass = full_auto();
        if bypass {
            let _ = tx
                .send(RunDelta::Output(
                    "kit: KIT_FULL_AUTO=1 — claude permission checks skipped\n".into(),
                ))
                .await;
        }

        let cmd = claude_command("claude", worktree, bypass);
        spawn_streaming_with_stdin(AgentKind::Claude, cmd, prompt, tx).await
    }
}

/// `claude -p` with the prompt on stdin, never on the command line.
///
/// On Windows `command_for` runs the npm shim through `cmd /C`, and cmd.exe
/// honours no escapes: a `"` in the prompt (the retry task quotes gate output)
/// would end the quoted argument and let `&` run a host command. Only fixed
/// flags go on that line.
fn claude_command(binary: &str, worktree: &Path, bypass: bool) -> Command {
    let mut cmd = command_for(binary);
    cmd.arg("-p");
    if bypass {
        cmd.arg("--dangerously-skip-permissions");
    } else {
        // The worktree is Kit's isolation, so edits inside it need no prompt.
        // Same bar as codex's `-s workspace-write`; shell commands still ask
        // unless KIT_FULL_AUTO=1. Without this, `-p` can read but not write.
        cmd.arg("--permission-mode").arg("acceptEdits");
    }
    cmd.current_dir(worktree);
    cmd
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    /// A prompt that closes cmd.exe's quoting and chains a host command.
    #[cfg(unix)]
    const HOSTILE: &str = "# Kit\n\nfix \"a\" & echo KIT_INJECTED & \"b\"\n";

    #[test]
    fn claude_argv_carries_no_prompt_text() {
        // On Windows `command_for` prefixes `/C claude`; the tail is ours.
        let plain = claude_command("claude", Path::new("wt"), false);
        let args: Vec<&OsStr> = plain.as_std().get_args().collect();
        // Without a permission mode, `-p` can read the worktree but every
        // edit waits for an approval nobody can give (live smoke
        // 01M3F7N9N2VNT7FMWJ3208T9ZX ended FAIL that way).
        assert!(
            args.ends_with(&["-p", "--permission-mode", "acceptEdits"].map(OsStr::new)),
            "{args:?}"
        );
        let bypass = claude_command("claude", Path::new("wt"), true);
        let args: Vec<&OsStr> = bypass.as_std().get_args().collect();
        assert!(
            args.ends_with(&["-p", "--dangerously-skip-permissions"].map(OsStr::new)),
            "{args:?}"
        );
        assert!(
            !args.contains(&OsStr::new("acceptEdits")),
            "full auto already skips every check: {args:?}"
        );
        assert!(
            args.len() <= 4,
            "nothing but the shim and fixed flags: {args:?}"
        );
        assert_eq!(bypass.as_std().get_current_dir(), Some(Path::new("wt")));
    }

    /// End to end through the real spawn path: a stand-in `claude` echoes its
    /// argv and stdin. The prompt must arrive on stdin, byte for byte.
    #[cfg(unix)]
    #[tokio::test]
    async fn claude_gets_the_prompt_on_stdin_not_argv() {
        let prompt = format!("{HOSTILE}{}", "x".repeat(256 * 1024));
        let out = crate::process::test_support::run_fake_agent(
            |fake, wt| claude_command(fake, wt, false),
            &prompt,
        )
        .await;
        assert!(!out.argv.contains("KIT_INJECTED"), "argv: {}", out.argv);
        assert_eq!(out.stdin, prompt);
    }
}
