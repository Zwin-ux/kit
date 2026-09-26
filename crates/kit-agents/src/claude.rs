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
        let mut prompt = skills::build_prompt(&spec.task);

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

        let allowed = if !bypass && agent_runs_checks() {
            let (allowed, skipped) = allowed_checks(&spec.gate_checks);
            // Said in the stream, so it lands in the receipt's output.log.
            if !allowed.is_empty() {
                let _ = tx
                    .send(RunDelta::Output(format!(
                        "kit: KIT_AGENT_RUNS_CHECKS=1 — claude may run these gate checks \
                         without asking, and they run code it wrote: {}\n",
                        allowed.join(", ")
                    )))
                    .await;
            }
            if !skipped.is_empty() {
                let _ = tx
                    .send(RunDelta::Output(format!(
                        "kit: claude may not run these gate checks itself (characters Kit \
                         cannot pass on claude's command line); the gate still runs them: {}\n",
                        skipped.join(", ")
                    )))
                    .await;
            }
            allowed
        } else {
            Vec::new()
        };

        let runnable = if bypass { &spec.gate_checks } else { &allowed };
        prompt.push_str(&checks_note(&spec.gate_checks, runnable));

        let cmd = claude_command("claude", worktree, bypass, &allowed);
        spawn_streaming_with_stdin(AgentKind::Claude, cmd, prompt, tx).await
    }
}

/// `claude -p` with the prompt on stdin, never on the command line.
///
/// On Windows `command_for` runs the npm shim through `cmd /C`, and cmd.exe
/// honours no escapes: a `"` in the prompt (the retry task quotes gate output)
/// would end the quoted argument and let `&` run a host command. Only fixed
/// flags go on that line.
///
/// `checks` is empty unless the user opted in with `KIT_AGENT_RUNS_CHECKS=1`.
/// Then each gate command is allowed as an exact `Bash(<command>)` rule and
/// nothing broader. Those commands run code the agent may have just written
/// (a test file, a package.json script), so this is the user's call, never
/// the repo's. Edits to `kit.toml`, `.git` and `.claude` are denied with it.
fn claude_command(binary: &str, worktree: &Path, bypass: bool, checks: &[String]) -> Command {
    let mut cmd = command_for(binary);
    cmd.arg("-p");
    if bypass {
        cmd.arg("--dangerously-skip-permissions");
    } else {
        // The worktree is Kit's isolation, so edits inside it need no prompt.
        // Same bar as codex's `-s workspace-write`; shell commands still ask
        // unless KIT_FULL_AUTO=1. Without this, `-p` can read but not write.
        cmd.arg("--permission-mode").arg("acceptEdits");
        if !checks.is_empty() {
            // Both flags take several values; nothing else follows them.
            cmd.arg("--disallowedTools");
            for path in PROTECTED {
                rule_arg(&mut cmd, &format!("Edit({path})"));
            }
            cmd.arg("--allowedTools");
            for check in checks {
                rule_arg(&mut cmd, &format!("Bash({check})"));
            }
        }
    }
    cmd.current_dir(worktree);
    cmd
}

/// One permission rule as one argument. On Windows the line goes through
/// `cmd /C`, where a bare `(` or `)` is command syntax, so every rule is
/// quoted there, with or without spaces. Rules hold no `"` (see
/// [`allowed_checks`]), so the quotes cannot be closed early.
fn rule_arg(cmd: &mut Command, rule: &str) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.as_std_mut().raw_arg(format!("\"{rule}\""));
    }
    #[cfg(not(windows))]
    cmd.arg(rule);
}

/// Paths claude may not edit while it can run the gate's checks: the gate
/// itself, git's state and claude's own settings.
const PROTECTED: [&str; 3] = ["./kit.toml", "./.git/**", "./.claude/**"];

/// `KIT_AGENT_RUNS_CHECKS=1`: the user lets claude run the gate's checks
/// itself. Read from the user's environment only, never from the repo.
fn agent_runs_checks() -> bool {
    matches!(
        std::env::var("KIT_AGENT_RUNS_CHECKS").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

/// The prompt line about checking. Without `--dangerously-skip-permissions`
/// or an allowed rule, claude cannot run a shell command under `-p` (nobody
/// can answer the prompt), so it is told Kit runs the checks instead of
/// spending turns on refused commands. `runnable` are the checks it may run.
fn checks_note(checks: &[String], runnable: &[String]) -> String {
    let named = |list: &[String]| {
        list.iter()
            .map(|c| format!("`{}`", c.trim()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    if checks.is_empty() {
        "\nDo not run tests or checks yourself; Kit checks the result after you finish.\n".into()
    } else if runnable.is_empty() {
        format!(
            "\nDo not run tests or checks yourself; Kit runs these after you finish: {}\n",
            named(checks)
        )
    } else {
        format!(
            "\nKit runs these checks after you finish: {}. You may run {} yourself first.\n",
            named(checks),
            named(runnable)
        )
    }
}

/// Splits gate commands into those that can go on claude's command line and
/// those left out. On Windows the arguments pass through `cmd /C` (see
/// [`claude_command`]), where a `"`, `&`, `%` or `\` would end or change the
/// argument, and claude reads a `,` in `--allowedTools` as a separator. A
/// command left out only means claude asks before running it.
fn allowed_checks(checks: &[String]) -> (Vec<String>, Vec<String>) {
    checks
        .iter()
        .map(|c| c.trim())
        .filter(|c| !c.is_empty())
        .map(str::to_owned)
        .partition(|c| c.chars().all(safe_on_command_line))
}

fn safe_on_command_line(c: char) -> bool {
    c.is_ascii_alphanumeric() || " -_./:=+@~".contains(c)
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
        let plain = claude_command("claude", Path::new("wt"), false, &[]);
        let args: Vec<&OsStr> = plain.as_std().get_args().collect();
        // Without a permission mode, `-p` can read the worktree but every
        // edit waits for an approval nobody can give (live smoke
        // 01M3F7N9N2VNT7FMWJ3208T9ZX ended FAIL that way).
        assert!(
            args.ends_with(&["-p", "--permission-mode", "acceptEdits"].map(OsStr::new)),
            "{args:?}"
        );
        let bypass = claude_command("claude", Path::new("wt"), true, &[]);
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

    /// With the user's opt-in, claude may run exactly the gate's commands, one
    /// exact `Bash(…)` rule each, and may not edit the gate, git or its own
    /// settings. The rules come last, so no later argument joins the list.
    #[test]
    fn opted_in_checks_become_exact_allowed_tools() {
        let checks = ["npm run test".to_owned(), "cargo fmt --check".to_owned()];
        let cmd = claude_command("claude", Path::new("wt"), false, &checks);
        // On Windows each rule is quoted for cmd.exe; the rule inside is the same.
        let args: Vec<String> = cmd
            .as_std()
            .get_args()
            .map(|a| a.to_string_lossy().trim_matches('"').to_owned())
            .collect();
        assert!(
            args.ends_with(
                &[
                    "--permission-mode",
                    "acceptEdits",
                    "--disallowedTools",
                    "Edit(./kit.toml)",
                    "Edit(./.git/**)",
                    "Edit(./.claude/**)",
                    "--allowedTools",
                    "Bash(npm run test)",
                    "Bash(cargo fmt --check)",
                ]
                .map(String::from)
            ),
            "{args:?}"
        );
        let bypass = claude_command("claude", Path::new("wt"), true, &checks);
        let args: Vec<&OsStr> = bypass.as_std().get_args().collect();
        assert!(!args.contains(&OsStr::new("--allowedTools")), "{args:?}");
    }

    #[test]
    fn checks_with_cmd_metacharacters_or_commas_stay_off_the_command_line() {
        let checks = [
            " cargo fmt --check ",
            "npm run lint && echo x",
            "make \"all\"",
            "echo %PATH%",
            "cargo test --features a,b",
            "scripts\\check.bat",
            "npm run test | tee out",
            "cargo clippy -- -D warnings",
            "",
        ]
        .map(str::to_owned);
        let (allowed, skipped) = allowed_checks(&checks);
        assert_eq!(
            allowed,
            ["cargo fmt --check", "cargo clippy -- -D warnings"]
        );
        assert_eq!(
            skipped,
            [
                "npm run lint && echo x",
                "make \"all\"",
                "echo %PATH%",
                "cargo test --features a,b",
                "scripts\\check.bat",
                "npm run test | tee out",
            ]
        );
    }

    /// By default claude is told Kit runs the checks, named, and not to run
    /// them itself; a repo with no checks still gets told.
    #[test]
    fn prompt_says_kit_runs_the_checks() {
        let checks = ["npm test".to_owned(), "npx tsc --noEmit".to_owned()];
        let note = checks_note(&checks, &[]);
        assert!(
            note.contains("Do not run tests or checks yourself"),
            "{note}"
        );
        assert!(note.contains("`npm test`, `npx tsc --noEmit`"), "{note}");
        let none = checks_note(&[], &[]);
        assert!(
            none.contains("Do not run tests or checks yourself"),
            "{none}"
        );
        let some = checks_note(&checks, &checks[..1]);
        assert!(some.contains("You may run `npm test` yourself"), "{some}");
    }

    /// End to end through the real spawn path: a stand-in `claude` echoes its
    /// argv and stdin. The prompt must arrive on stdin, byte for byte.
    #[cfg(unix)]
    #[tokio::test]
    async fn claude_gets_the_prompt_on_stdin_not_argv() {
        let prompt = format!("{HOSTILE}{}", "x".repeat(256 * 1024));
        let out = crate::process::test_support::run_fake_agent(
            |fake, wt| claude_command(fake, wt, false, &[]),
            &prompt,
        )
        .await;
        assert!(!out.argv.contains("KIT_INJECTED"), "argv: {}", out.argv);
        assert_eq!(out.stdin, prompt);
    }
}
