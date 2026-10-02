//! Provider-owned interactive sign-in. Kit never captures login output or credentials.
use crate::{AgentStatus, adapter, process::command_for};
use kit_core::AgentKind;
use std::io::IsTerminal;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Ready,
    Missing,
    SignedOut,
    Unchecked,
    NotReady,
}

impl ConnectionState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Missing => "not installed",
            Self::SignedOut => "not signed in",
            Self::Unchecked => "login unchecked",
            Self::NotReady => "not ready",
        }
    }
}

pub fn state(status: &AgentStatus) -> ConnectionState {
    if !status.installed {
        ConnectionState::Missing
    } else if status
        .remedy
        .as_deref()
        .is_some_and(|r| r.starts_with("login not checked"))
    {
        ConnectionState::Unchecked
    } else if status.authenticated {
        ConnectionState::Ready
    } else if status.kind == AgentKind::Ollama {
        ConnectionState::NotReady
    } else {
        ConnectionState::SignedOut
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("native sign-in is supported for Claude and Codex only")]
    Unsupported,
    #[error("native sign-in needs an interactive terminal; run `kit connect {0}` there")]
    NeedsTerminal(AgentKind),
    #[error("could not start {kind} sign-in: {source}; install its CLI and try again")]
    Start {
        kind: AgentKind,
        source: std::io::Error,
    },
    #[error("{kind} sign-in ended without success ({code}); return to Agents and check its status")]
    Exit { kind: AgentKind, code: String },
}

fn login_command(kind: AgentKind) -> Result<Command, ConnectionError> {
    let mut cmd = match kind {
        AgentKind::Claude => {
            let mut c = command_for("claude");
            c.args(["auth", "login"]);
            c
        }
        AgentKind::Codex => {
            let mut c = command_for("codex");
            c.arg("login");
            c
        }
        _ => return Err(ConnectionError::Unsupported),
    };
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);
    Ok(cmd)
}

async fn wait_for_login(kind: AgentKind, mut command: Command) -> Result<(), ConnectionError> {
    // Native login shares the foreground console: Ctrl-C must reach the
    // provider without terminating Kit before it can restore the workbench.
    // Register before spawning; the provider keeps its native signal handling.
    #[cfg(unix)]
    let _interrupts = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .map_err(|source| ConnectionError::Start { kind, source })?;
    #[cfg(windows)]
    let _interrupts = tokio::signal::windows::ctrl_c()
        .map_err(|source| ConnectionError::Start { kind, source })?;
    let result = command
        .status()
        .await
        .map_err(|source| ConnectionError::Start { kind, source })?;
    if result.success() {
        Ok(())
    } else {
        Err(ConnectionError::Exit {
            kind,
            code: result
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "cancelled".into()),
        })
    }
}

pub async fn connect(kind: AgentKind) -> Result<AgentStatus, ConnectionError> {
    let command = login_command(kind)?;
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(ConnectionError::NeedsTerminal(kind));
    }
    wait_for_login(kind, command).await?;
    // A zero exit is not readiness proof. Ask the provider's status command again.
    Ok(adapter(kind).probe().await)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_login_uses_only_fixed_provider_arguments() {
        for (kind, suffix) in [
            (AgentKind::Claude, vec!["auth", "login"]),
            (AgentKind::Codex, vec!["login"]),
        ] {
            let c = login_command(kind).unwrap();
            let args = c
                .as_std()
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            assert!(args.ends_with(&suffix.iter().map(|s| s.to_string()).collect::<Vec<_>>()));
            assert!(!args.iter().any(|a| a.contains("console")
                || a.contains("api-key")
                || a.contains("dangerously")));
        }
        assert!(login_command(AgentKind::Grok).is_err());
    }
    #[test]
    fn status_keeps_signed_out_and_unchecked_distinct() {
        let mut status = AgentStatus {
            kind: AgentKind::Claude,
            installed: true,
            authenticated: false,
            version: None,
            remedy: None,
        };
        assert_eq!(state(&status), ConnectionState::SignedOut);
        status.remedy = Some("login not checked (timeout)".into());
        for authenticated in [false, true] {
            status.authenticated = authenticated;
            assert_eq!(state(&status), ConnectionState::Unchecked);
        }
        status.installed = false;
        assert_eq!(state(&status), ConnectionState::Missing);
    }
    #[test]
    fn unavailable_local_server_is_not_a_login_failure() {
        let status = AgentStatus {
            kind: AgentKind::Ollama,
            installed: true,
            authenticated: false,
            version: None,
            remedy: Some("start the Ollama server".into()),
        };
        assert_eq!(state(&status), ConnectionState::NotReady);
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn failed_and_cancelled_native_login_never_claim_success() {
        for script in ["exit 7", "kill -TERM $$"] {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", script]);
            assert!(matches!(
                wait_for_login(AgentKind::Claude, cmd).await,
                Err(ConnectionError::Exit { .. })
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn foreground_interrupt_cancels_native_login_without_exiting_parent() {
        use std::os::unix::process::CommandExt;
        use std::time::{Duration, Instant};

        const CHILD_ENV: &str = "KIT_TEST_NATIVE_LOGIN_INTERRUPT";
        if let Some(root) = std::env::var_os(CHILD_ENV) {
            let root = std::path::PathBuf::from(root);
            let mut command = Command::new("sh");
            command
                .args([
                    "-c",
                    "trap 'exit 130' INT; printf ready > \"$1\"; while :; do sleep 1; done",
                    "fake-login",
                ])
                .arg(root.join("ready"))
                .kill_on_drop(true);
            let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
                tokio::time::timeout(
                    Duration::from_secs(5),
                    wait_for_login(AgentKind::Claude, command),
                )
                .await
                .expect("provider must exit after foreground interrupt")
            });
            assert!(
                matches!(result, Err(ConnectionError::Exit { .. })),
                "{result:?}"
            );
            std::fs::write(root.join("returned"), "provider cancelled; parent survived").unwrap();
            return;
        }

        let root =
            std::env::temp_dir().join(format!("kit-native-interrupt-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "connection::tests::foreground_interrupt_cancels_native_login_without_exiting_parent",
                "--nocapture",
            ])
            .env(CHILD_ENV, &root)
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !root.join("ready").exists() {
            if Instant::now() >= deadline {
                unsafe {
                    libc::kill(-(child.id() as i32), libc::SIGKILL);
                }
                let _ = child.wait();
                panic!("fake native login never became ready");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        // This isolated group contains only the test parent and fake provider.
        assert_eq!(unsafe { libc::kill(-(child.id() as i32), libc::SIGINT) }, 0);
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "parent exited on interrupt: {output:?}"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("returned")).unwrap(),
            "provider cancelled; parent survived"
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
