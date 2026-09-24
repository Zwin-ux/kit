//! Login checks behind `probe()`, so `kit doctor` says "ready" only for an
//! agent that can run.
//!
//! Every check is local, fast (4s cap), non-interactive (stdin is null) and
//! read-only. Kit never reads credential contents (PRD principle 4): it asks
//! the CLI's own status command, or, for grok, checks that a login file exists.

use crate::AgentStatus;
use crate::process::{command_with_args, is_command_not_found};
use kit_core::AgentKind;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

/// Longest a login check may take before kit stops waiting.
const CHECK_TIMEOUT: Duration = Duration::from_secs(4);

/// What a login check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Login {
    In,
    Out,
    /// No reliable answer; the reason goes into the remedy text.
    Unknown(&'static str),
}

/// Build the probe result for an installed agent from its login check.
///
/// `Unknown` keeps `authenticated: true` (kit-core has no third state) and
/// says in the remedy that the login was not checked, so doctor never claims
/// more than kit knows.
pub(crate) fn installed_status(
    kind: AgentKind,
    version: Option<String>,
    login: Login,
    login_cmd: &str,
) -> AgentStatus {
    let (authenticated, remedy) = match login {
        Login::In => (true, None),
        Login::Out => (false, Some(format!("not logged in: run `{login_cmd}`"))),
        Login::Unknown(why) => (
            true,
            Some(format!(
                "login not checked ({why}); if a run fails, run `{login_cmd}`"
            )),
        ),
    };
    AgentStatus {
        kind,
        installed: true,
        authenticated,
        version,
        remedy,
    }
}

/// Output of a status command: exit success, stdout, stderr.
pub(crate) struct StatusOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Run `binary args…` with no stdin and a 4s cap. `None` when it could not
/// run, was not found, or timed out.
pub(crate) async fn run_status(binary: &str, args: &[&str]) -> Option<StatusOutput> {
    let mut cmd = command_with_args(binary, args);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let out = tokio::time::timeout(CHECK_TIMEOUT, cmd.output())
        .await
        .ok()?
        .ok()?;
    if is_command_not_found(out.status.code(), &out.stderr) {
        return None;
    }
    Some(StatusOutput {
        success: out.status.success(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

/// `codex login status` (codex 0.155): prints to stderr
/// `Logged in using ChatGPT` / `Logged in using an API key - …` and exits 0,
/// or `Not logged in` and exits 1. Other lines (warnings) are ignored.
pub(crate) fn parse_codex_status(success: bool, text: &str) -> Login {
    let mut lines = text.lines().map(str::trim);
    if lines.clone().any(|l| l.starts_with("Not logged in")) {
        return Login::Out;
    }
    if success && lines.any(|l| l.starts_with("Logged in")) {
        return Login::In;
    }
    Login::Unknown("`codex login status` gave no clear answer")
}

/// `claude auth status` (Claude Code 2.1): JSON on stdout with a boolean
/// `loggedIn`. Only that field is read; account fields are ignored.
pub(crate) fn parse_claude_status(stdout: &str) -> Login {
    let logged_in = serde_json::from_str::<serde_json::Value>(stdout.trim())
        .ok()
        .and_then(|v| v.get("loggedIn").and_then(serde_json::Value::as_bool));
    match logged_in {
        Some(true) => Login::In,
        Some(false) => Login::Out,
        None => Login::Unknown("`claude auth status` gave no clear answer"),
    }
}

/// grok (1.0.34) has no login status command. `grok login` writes
/// `$GROK_HOME/auth.json` (default `~/.grok`), and `XAI_API_KEY` also signs
/// in. Neither present: logged out. Either present: the token may still have
/// expired (grok tokens last 7 days), so the answer is `Unknown`.
pub(crate) fn grok_login(grok_home: Option<&Path>, api_key_set: bool) -> Login {
    let has_file = grok_home.is_some_and(|h| h.join("auth.json").is_file());
    if has_file || api_key_set {
        Login::Unknown("grok has no login status command")
    } else {
        Login::Out
    }
}

/// `$GROK_HOME`, else `~/.grok`.
pub(crate) fn grok_home() -> Option<PathBuf> {
    if let Some(h) = std::env::var_os("GROK_HOME").filter(|h| !h.is_empty()) {
        return Some(PathBuf::from(h));
    }
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|h| PathBuf::from(h).join(".grok"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Captured from the CLIs on the dev machine; account fields redacted.
    const CODEX_IN: &str = include_str!("fixtures/auth/codex-logged-in.txt");
    const CODEX_OUT: &str = include_str!("fixtures/auth/codex-logged-out.txt");
    const CLAUDE_IN: &str = include_str!("fixtures/auth/claude-logged-in.json");
    const CLAUDE_OUT: &str = include_str!("fixtures/auth/claude-logged-out.json");

    #[test]
    fn codex_status_output_parses() {
        assert_eq!(parse_codex_status(true, CODEX_IN), Login::In);
        // The logged-out capture also carries a WARNING line; it is ignored.
        assert_eq!(parse_codex_status(false, CODEX_OUT), Login::Out);
        assert_eq!(
            parse_codex_status(true, "Logged in using an API key - sk-***\n"),
            Login::In
        );
        // A "Logged in" line from a failing command is not proof.
        assert!(matches!(
            parse_codex_status(false, CODEX_IN),
            Login::Unknown(_)
        ));
        assert!(matches!(
            parse_codex_status(true, "error: unrecognized subcommand 'login'\n"),
            Login::Unknown(_)
        ));
    }

    #[test]
    fn claude_status_output_parses() {
        assert_eq!(parse_claude_status(CLAUDE_IN), Login::In);
        assert_eq!(parse_claude_status(CLAUDE_OUT), Login::Out);
        assert!(matches!(
            parse_claude_status("error: unknown command 'auth'\n"),
            Login::Unknown(_)
        ));
        assert!(matches!(parse_claude_status("{}"), Login::Unknown(_)));
    }

    #[test]
    fn grok_login_is_out_only_without_file_or_key() {
        let home = std::env::temp_dir().join(format!(
            "kit-grok-home-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&home).unwrap();
        assert_eq!(grok_login(Some(&home), false), Login::Out);
        assert_eq!(grok_login(None, false), Login::Out);
        assert!(matches!(grok_login(Some(&home), true), Login::Unknown(_)));
        std::fs::write(home.join("auth.json"), "{}").unwrap();
        assert!(matches!(grok_login(Some(&home), false), Login::Unknown(_)));
        let _ = std::fs::remove_dir_all(&home);
    }

    /// Doctor says "ready" only when the login was seen; an unchecked login
    /// says so in the remedy instead of passing in silence.
    #[test]
    fn status_is_honest_about_login() {
        let out = installed_status(AgentKind::Codex, None, Login::Out, "codex login");
        assert!(out.installed && !out.authenticated && !out.is_ready());
        assert_eq!(
            out.remedy.as_deref(),
            Some("not logged in: run `codex login`")
        );
        let ok = installed_status(AgentKind::Claude, None, Login::In, "claude auth login");
        assert!(ok.is_ready() && ok.remedy.is_none());
        let unknown = installed_status(
            AgentKind::Grok,
            None,
            Login::Unknown("grok has no login status command"),
            "grok login",
        );
        assert!(unknown.is_ready());
        assert!(unknown.remedy.unwrap().starts_with("login not checked"));
    }
}
