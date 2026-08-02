//! Shared child-process handle and stream fan-out into [`RunDelta`].
//!
//! **Kill semantics:** stop the whole supervised tree, not only the direct
//! child. On Windows agents often run as `cmd /C codex.cmd → node → tools`;
//! terminating only `cmd` orphans grandchildren. We use a job object (Win)
//! or process group (Unix) so Control Room `k` and timeouts are real.

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

/// Prepare OS-level process grouping before spawn (tree kill support).
fn configure_tree_kill(cmd: &mut Command) {
    #[cfg(unix)]
    {
        // New process group: child's pid becomes the group leader.
        // kill(-pid, SIGKILL) then stops the whole group.
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    #[cfg(windows)]
    {
        // Job object is assigned after spawn; no creation flags required.
        let _ = cmd;
    }
}

/// Spawn a command, pipe stdout+stderr into `tx` as [`RunDelta::Output`].
pub async fn spawn_streaming(
    kind: AgentKind,
    mut cmd: Command,
    tx: mpsc::Sender<RunDelta>,
) -> Result<Box<dyn AgentHandle>, SpawnError> {
    configure_tree_kill(&mut cmd);
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .stdin(Stdio::null());

    let mut child = cmd
        .spawn()
        .map_err(|source| SpawnError::Io { kind, source })?;

    let tree = ProcessTree::attach(&child);

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    spawn_stream_tasks(stdout, stderr, tx);

    Ok(Box::new(ChildHandle {
        child: tokio::sync::Mutex::new(child),
        tree: tokio::sync::Mutex::new(tree),
    }))
}

/// Spawn with stdin fed a prompt (ollama).
pub async fn spawn_streaming_with_stdin(
    kind: AgentKind,
    mut cmd: Command,
    stdin_body: String,
    tx: mpsc::Sender<RunDelta>,
) -> Result<Box<dyn AgentHandle>, SpawnError> {
    configure_tree_kill(&mut cmd);
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd
        .spawn()
        .map_err(|source| SpawnError::Io { kind, source })?;

    let tree = ProcessTree::attach(&child);

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        let _ = stdin.write_all(stdin_body.as_bytes()).await;
        let _ = stdin.shutdown().await;
    }

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    spawn_stream_tasks(stdout, stderr, tx);

    Ok(Box::new(ChildHandle {
        child: tokio::sync::Mutex::new(child),
        tree: tokio::sync::Mutex::new(tree),
    }))
}

fn spawn_stream_tasks(
    stdout: Option<impl tokio::io::AsyncRead + Send + Unpin + 'static>,
    stderr: Option<impl tokio::io::AsyncRead + Send + Unpin + 'static>,
    tx: mpsc::Sender<RunDelta>,
) {
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
        tokio::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let pretty = prettify_line(&line);
                let _ = tx.send(RunDelta::Output(format!("[err] {pretty}\n"))).await;
            }
        });
    }
}

/// OS handle that can terminate the whole process tree.
struct ProcessTree {
    #[cfg(windows)]
    job: Option<win_job::Job>,
    #[cfg(unix)]
    pgid: Option<i32>,
    #[cfg(windows)]
    pid: Option<u32>,
}

impl ProcessTree {
    fn attach(child: &Child) -> Self {
        #[cfg(windows)]
        {
            let pid = child.id();
            let job = win_job::Job::new()
                .ok()
                .and_then(|j| match j.assign_child(child) {
                    Ok(()) => Some(j),
                    Err(_) => None,
                });
            Self { job, pid }
        }
        #[cfg(unix)]
        {
            // process_group(0) → group leader is the child pid.
            let pgid = child.id().map(|p| p as i32);
            Self { pgid }
        }
        #[cfg(not(any(windows, unix)))]
        {
            let _ = child;
            Self {}
        }
    }

    /// Kill every process in the tree. Best-effort; never panics.
    fn kill_tree(&mut self) {
        #[cfg(windows)]
        {
            if let Some(job) = self.job.take() {
                let _ = job.terminate();
                // Drop closes handle; KILL_ON_JOB_CLOSE is also set.
                drop(job);
            } else if let Some(pid) = self.pid {
                // Nested-job / assign failed (common in some CI sandboxes).
                // Fall back to taskkill process tree.
                let _ = std::process::Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/T", "/F"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
        #[cfg(unix)]
        {
            if let Some(pgid) = self.pgid.take() {
                // Negative pid = process group.
                unsafe {
                    let _ = libc::kill(-pgid, libc::SIGKILL);
                }
            }
        }
    }
}

struct ChildHandle {
    /// Mutex so the handle is `Sync` (contract on [`AgentHandle`]).
    /// Wait uses short `try_wait` polls so `kill` can interleave.
    child: tokio::sync::Mutex<Child>,
    tree: tokio::sync::Mutex<ProcessTree>,
}

#[async_trait::async_trait]
impl AgentHandle for ChildHandle {
    async fn wait(&mut self) -> std::io::Result<i32> {
        loop {
            if let Some(code) = self.try_wait().await? {
                return Ok(code);
            }
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        }
    }

    async fn kill(&mut self) -> std::io::Result<()> {
        // Tree first so grandchildren die even if start_kill only hits cmd.
        {
            let mut tree = self.tree.lock().await;
            tree.kill_tree();
        }
        {
            let mut child = self.child.lock().await;
            let _ = child.start_kill();
        }
        // Reap without holding the lock across a long wait so other polls work.
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
        while tokio::time::Instant::now() < deadline {
            if self.try_wait().await?.is_some() {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        // Best-effort: process may be stuck; kill_on_drop still applies.
        Ok(())
    }

    async fn try_wait(&mut self) -> std::io::Result<Option<i32>> {
        let mut child = self.child.lock().await;
        match child.try_wait()? {
            Some(status) => Ok(Some(status.code().unwrap_or(1))),
            None => Ok(None),
        }
    }
}

#[cfg(windows)]
mod win_job {
    use std::io;
    use std::ptr;
    use tokio::process::Child;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject,
    };

    /// Windows job object: kill-on-close / terminate kills the whole tree.
    pub struct Job(HANDLE);

    // SAFETY: HANDLE is closed only in Drop; Job is not Sync across threads
    // without exterior mutex (we store under Mutex in ChildHandle).
    unsafe impl Send for Job {}

    impl Job {
        pub fn new() -> io::Result<Self> {
            unsafe {
                let h = CreateJobObjectW(ptr::null(), ptr::null());
                if h.is_null() || h == INVALID_HANDLE_VALUE {
                    return Err(io::Error::last_os_error());
                }
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                let ok = SetInformationJobObject(
                    h,
                    JobObjectExtendedLimitInformation,
                    (&raw const info).cast(),
                    std::mem::size_of_val(&info) as u32,
                );
                if ok == 0 {
                    let err = io::Error::last_os_error();
                    let _ = CloseHandle(h);
                    return Err(err);
                }
                Ok(Self(h))
            }
        }

        pub fn assign_child(&self, child: &Child) -> io::Result<()> {
            // tokio::process::Child::raw_handle() → PROCESS handle on Windows.
            let Some(raw) = child.raw_handle() else {
                return Err(io::Error::other("child has no raw handle"));
            };
            let process = raw as HANDLE;
            unsafe {
                if AssignProcessToJobObject(self.0, process) == 0 {
                    return Err(io::Error::last_os_error());
                }
            }
            Ok(())
        }

        pub fn terminate(&self) -> io::Result<()> {
            unsafe {
                if TerminateJobObject(self.0, 1) == 0 {
                    return Err(io::Error::last_os_error());
                }
            }
            Ok(())
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            // KILL_ON_JOB_CLOSE: closing the last handle kills remaining processes.
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

/// Cheap PATH probe: try common version flags, then bare `--help`.
pub async fn probe_binary(binary: &str) -> (bool, Option<String>) {
    for args in [
        ["--version"].as_slice(),
        ["-V"].as_slice(),
        ["version"].as_slice(),
    ] {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn kill_stops_long_running_child_within_two_seconds() {
        let (tx, _rx) = mpsc::channel(8);
        let cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/C", "ping", "-n", "60", "127.0.0.1", ">", "nul"]);
            c
        } else {
            let mut c = Command::new("sleep");
            c.arg("60");
            c
        };

        let mut handle = spawn_streaming(AgentKind::Codex, cmd, tx)
            .await
            .expect("spawn long child");

        // Let it start.
        tokio::time::sleep(Duration::from_millis(100)).await;
        let started = Instant::now();
        handle.kill().await.expect("kill");
        let elapsed = started.elapsed();
        assert!(
            elapsed < Duration::from_secs(2),
            "kill took {elapsed:?}, want < 2s"
        );
        // Process should be reaped.
        let _code = handle.wait().await.expect("wait after kill");
    }

    #[tokio::test]
    async fn kill_is_idempotent_on_exited_process() {
        let (tx, _rx) = mpsc::channel(8);
        let cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(["/C", "exit", "/B", "0"]);
            c
        } else {
            Command::new("true")
        };
        let mut handle = spawn_streaming(AgentKind::Codex, cmd, tx)
            .await
            .expect("spawn");
        let _ = handle.wait().await;
        handle.kill().await.expect("second kill");
        handle.kill().await.expect("third kill");
    }
}
