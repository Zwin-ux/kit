//! Shared child-process handle and stream fan-out into [`RunDelta`].

use crate::{AgentHandle, SpawnError};
use kit_core::{AgentKind, RunDelta};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

/// Build a command that works for npm/cmd shims on Windows.
pub fn command_for(binary: &str) -> Command {
    #[cfg(windows)]
    {
        // npm installs `codex.cmd` / `claude.cmd`; bare CreateProcess often fails.
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(binary);
        cmd
    }
    #[cfg(not(windows))]
    {
        Command::new(binary)
    }
}

/// Append args after the binary (handles Windows `cmd /C binary …`).
pub fn command_with_args(binary: &str, args: &[&str]) -> Command {
    let mut cmd = command_for(binary);
    #[cfg(windows)]
    {
        for a in args {
            cmd.arg(a);
        }
    }
    #[cfg(not(windows))]
    {
        cmd.args(args);
    }
    cmd
}

/// Spawn a command, pipe stdout+stderr into `tx` as [`RunDelta::Output`].
pub async fn spawn_streaming(
    kind: AgentKind,
    mut cmd: Command,
    tx: mpsc::Sender<RunDelta>,
) -> Result<Box<dyn AgentHandle>, SpawnError> {
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .stdin(Stdio::null());

    let mut child = cmd
        .spawn()
        .map_err(|source| SpawnError::Io { kind, source })?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(out) = stdout {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let pretty = prettify_line(&line);
                let _ = tx.send(RunDelta::Output(format!("{pretty}\n"))).await;
            }
        });
    }
    if let Some(err) = stderr {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let pretty = prettify_line(&line);
                let _ = tx.send(RunDelta::Output(format!("[err] {pretty}\n"))).await;
            }
        });
    }

    Ok(Box::new(ChildHandle {
        child: tokio::sync::Mutex::new(child),
    }))
}

/// Spawn with stdin fed a prompt (ollama).
pub async fn spawn_streaming_with_stdin(
    kind: AgentKind,
    mut cmd: Command,
    stdin_body: String,
    tx: mpsc::Sender<RunDelta>,
) -> Result<Box<dyn AgentHandle>, SpawnError> {
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd
        .spawn()
        .map_err(|source| SpawnError::Io { kind, source })?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        let _ = stdin.write_all(stdin_body.as_bytes()).await;
        let _ = stdin.shutdown().await;
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    if let Some(out) = stdout {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let pretty = prettify_line(&line);
                let _ = tx.send(RunDelta::Output(format!("{pretty}\n"))).await;
            }
        });
    }
    if let Some(err) = stderr {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let pretty = prettify_line(&line);
                let _ = tx.send(RunDelta::Output(format!("[err] {pretty}\n"))).await;
            }
        });
    }

    Ok(Box::new(ChildHandle {
        child: tokio::sync::Mutex::new(child),
    }))
}

struct ChildHandle {
    /// Mutex so the handle is `Sync` (contract on [`AgentHandle`]).
    child: tokio::sync::Mutex<Child>,
}

#[async_trait::async_trait]
impl AgentHandle for ChildHandle {
    async fn wait(&mut self) -> std::io::Result<i32> {
        let mut child = self.child.lock().await;
        let status = child.wait().await?;
        Ok(status.code().unwrap_or(1))
    }

    async fn kill(&mut self) -> std::io::Result<()> {
        let mut child = self.child.lock().await;
        let _ = child.start_kill();
        let _ = child.wait().await;
        Ok(())
    }
}

/// Cheap PATH probe: try common version flags, then bare `--help`.
pub async fn probe_binary(binary: &str) -> (bool, Option<String>) {
    for args in [["--version"].as_slice(), ["-V"].as_slice(), ["version"].as_slice()] {
        if let Some(ver) = try_probe(binary, args).await {
            return (true, ver);
        }
    }
    // Last resort: help succeeds if the binary exists on PATH.
    if try_probe(binary, &["--help"]).await.is_some() {
        return (true, Some(binary.into()));
    }
    (false, None)
}

async fn try_probe(binary: &str, args: &[&str]) -> Option<Option<String>> {
    let mut cmd = command_with_args(binary, args);
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(std::time::Duration::from_secs(4), cmd.output()).await {
        Ok(Ok(out)) if out.status.success() || !out.stdout.is_empty() || !out.stderr.is_empty() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let text = if text.trim().is_empty() {
                String::from_utf8_lossy(&out.stderr).into_owned()
            } else {
                text.into_owned()
            };
            // Skip PowerShell noise / empty; take first informative line.
            let ver = text
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty() && !l.starts_with("At ") && !l.contains("CategoryInfo"))
                .map(|s| s.chars().take(80).collect());
            Some(ver)
        }
        _ => None,
    }
}

/// Collapse Codex/Grok JSONL noise into readable text when possible.
fn prettify_line(line: &str) -> String {
    let trimmed = line.trim();
    if !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return line.to_string();
    }
    let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return line.to_string();
    };
    // Common shapes: { "type": "...", "item": { "text": "..." } } etc.
    if let Some(t) = v.get("type").and_then(|x| x.as_str()) {
        if let Some(msg) = v
            .pointer("/item/text")
            .or_else(|| v.pointer("/message"))
            .or_else(|| v.pointer("/text"))
            .and_then(|x| x.as_str())
        {
            return format!("{t}: {msg}");
        }
        if let Some(msg) = v.get("msg").and_then(|x| x.as_str()) {
            return format!("{t}: {msg}");
        }
        return format!("event:{t}");
    }
    line.to_string()
}

pub fn full_auto() -> bool {
    matches!(
        std::env::var("KIT_FULL_AUTO").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}
