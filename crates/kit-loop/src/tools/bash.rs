//! `Bash`: one command in the workspace's shell, with a timeout, a trimmed
//! environment and a process-tree kill.

use super::{MAX_OUTPUT, ToolOutput, arg, truncate};
use crate::{Workspace, shell};
use serde_json::Value;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt};

const DEFAULT_MS: u64 = 120_000;
const MAX_MS: u64 = 600_000;

pub async fn run(ws: &Workspace, args: &Value) -> ToolOutput {
    let command = match arg(args, "command") {
        Ok(c) => c,
        Err(e) => return e,
    };
    let ms = args["timeout"]
        .as_u64()
        .unwrap_or(DEFAULT_MS)
        .clamp(1_000, MAX_MS);
    let mut cmd = ws.shell.command(command);
    cmd.current_dir(&ws.root)
        .env_clear()
        .envs(shell::tool_env(&ws.extra_env))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    kit_agents::configure_tree_kill(&mut cmd);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return ToolOutput::err(format!("could not start {}: {e}", ws.shell.name())),
    };
    let mut tree = kit_agents::ProcessTree::attach(&child);
    // stdout and stderr land in one buffer in the order they arrive.
    let buf = Arc::new(Mutex::new(Vec::new()));
    let readers = [
        child
            .stdout
            .take()
            .map(|s| tokio::spawn(pump(s, buf.clone()))),
        child
            .stderr
            .take()
            .map(|s| tokio::spawn(pump(s, buf.clone()))),
    ];
    let status = tokio::time::timeout(Duration::from_millis(ms), child.wait()).await;
    let timed_out = status.is_err();
    if timed_out {
        tree.kill_tree();
        let _ = child.start_kill();
        let _ = child.wait().await;
    }
    for r in readers.into_iter().flatten() {
        let _ = tokio::time::timeout(Duration::from_secs(2), r).await;
    }
    let output = String::from_utf8_lossy(&buf.lock().unwrap()).into_owned();
    let output = truncate(output.trim_end(), MAX_OUTPUT);
    match status {
        Err(_) => ToolOutput::err(format!(
            "{output}\n(timed out after {}s; the command was killed)",
            ms / 1000
        )),
        Ok(Err(e)) => ToolOutput::err(format!("{output}\n(could not wait for the command: {e})")),
        Ok(Ok(s)) => {
            let code = s.code().unwrap_or(-1);
            let text = if output.is_empty() {
                format!("(no output, exit code {code})")
            } else {
                format!("{output}\n(exit code {code})")
            };
            ToolOutput {
                content: text,
                is_error: code != 0,
            }
        }
    }
}

async fn pump(mut from: impl AsyncRead + Unpin, into: Arc<Mutex<Vec<u8>>>) {
    let mut chunk = [0u8; 8192];
    while let Ok(n) = from.read(&mut chunk).await {
        if n == 0 {
            break;
        }
        let mut b = into.lock().unwrap();
        // Hold at most four times the cap; truncate() keeps head and tail later.
        if b.len() < MAX_OUTPUT * 4 {
            b.extend_from_slice(&chunk[..n]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ws() -> (tempfile::TempDir, Workspace) {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::open(dir.path()).unwrap();
        (dir, ws)
    }

    #[tokio::test]
    async fn runs_in_the_workspace_and_reports_exit_codes() {
        let (dir, ws) = ws();
        std::fs::write(dir.path().join("marker.txt"), "").unwrap();
        let list = if ws.shell.name() == "powershell" {
            "Get-ChildItem -Name"
        } else {
            "ls"
        };
        let out = run(&ws, &json!({"command": list})).await;
        assert!(
            out.content.contains("marker.txt") && !out.is_error,
            "{}",
            out.content
        );
        let out = run(&ws, &json!({"command": "exit 3"})).await;
        assert!(
            out.is_error && out.content.contains("exit code 3"),
            "{}",
            out.content
        );
    }

    #[tokio::test]
    async fn provider_keys_are_not_in_the_environment() {
        let (_dir, mut ws) = ws();
        // SAFETY: tests in this module do not read this variable concurrently.
        unsafe { std::env::set_var("KIT_TEST_FAKE_API_KEY", "sk-should-not-leak") };
        ws.extra_env = vec!["KIT_TEST_FAKE_API_KEY".into()];
        let print = if ws.shell.name() == "powershell" {
            "Get-ChildItem env: | Out-String -Width 400"
        } else {
            "env"
        };
        let out = run(&ws, &json!({"command": print})).await;
        assert!(
            !out.content.contains("sk-should-not-leak"),
            "{}",
            out.content
        );
    }

    #[tokio::test]
    async fn a_hung_command_is_killed_at_the_timeout() {
        let (_dir, ws) = ws();
        let sleep = if ws.shell.name() == "powershell" {
            "Start-Sleep -Seconds 30"
        } else {
            "sleep 30"
        };
        let started = std::time::Instant::now();
        let out = run(&ws, &json!({"command": sleep, "timeout": 1000})).await;
        assert!(
            out.is_error && out.content.contains("timed out after 1s"),
            "{}",
            out.content
        );
        assert!(started.elapsed() < Duration::from_secs(15));
    }
}
