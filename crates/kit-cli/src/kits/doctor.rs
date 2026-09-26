//! The kits part of `kit doctor`: is each installed kit still live? Skills
//! as installed, rules blocks present, and the `[check]` approved at
//! `kit add`: MCP servers answer an MCP `initialize`, commands exit 0.

use super::install::{home_dir, repo_root};
use super::lock::{Entry, Lock};
use super::manifest::McpServer;
use super::plan::{self, Applied, tilde};
use super::writers::Scope;
use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// First start of an `npx` server downloads it; later starts are quick.
const MCP_TIMEOUT: Duration = Duration::from_secs(60);

pub struct KitReport {
    pub scope: String,
    pub name: String,
    pub version: String,
    pub agents: Vec<String>,
    /// (what, passed, detail)
    pub checks: Vec<(String, bool, String)>,
}

impl KitReport {
    pub fn ok(&self) -> bool {
        self.checks.iter().all(|(_, ok, _)| *ok)
    }
}

/// Every kit installed globally and in the current repo, checked.
pub fn check_installed() -> Result<Vec<KitReport>> {
    let mut scopes = vec![Scope::Global { home: home_dir()? }];
    if let Some(root) = repo_root(Path::new(".")) {
        scopes.push(Scope::Repo(root));
    }
    let mut out = Vec::new();
    for scope in scopes {
        for e in Lock::load(&scope)?.kits {
            out.push(check(&scope, &e));
        }
    }
    Ok(out)
}

fn check(scope: &Scope, e: &Entry) -> KitReport {
    let mut checks = Vec::new();
    let skills: Vec<&Applied> = e
        .applied
        .iter()
        .filter(|a| matches!(a, Applied::Skill { .. }))
        .collect();
    if !skills.is_empty() {
        let drift: Vec<String> = skills
            .iter()
            .filter_map(|a| plan::drifted(a).ok().flatten())
            .collect();
        checks.push(match drift.first() {
            None => (
                format!("{} skills as installed", skills.len()),
                true,
                String::new(),
            ),
            Some(first) => (
                format!("{} of {} skills changed", drift.len(), skills.len()),
                false,
                first.clone(),
            ),
        });
    }
    for a in &e.applied {
        if let Applied::Rules { file, kit, .. } = a {
            let present = std::fs::read_to_string(file)
                .is_ok_and(|t| t.contains(&format!("<!-- kit:{kit} ")));
            checks.push((
                format!("rules block in {}", tilde(file)),
                present,
                if present {
                    String::new()
                } else {
                    format!("the kit:{kit} block is gone. Run kit add {} again", e.name)
                },
            ));
        }
    }

    // The kit's [check], as approved at `kit add` and kept in Kit's own
    // record. Never re-read from a KIT.toml or a repo's kit.lock.
    for a in &e.applied {
        let (name, present) = match a {
            Applied::McpJson { file, name, .. } => (name, mcp_in_json(file, name)),
            Applied::McpToml { file, name, .. } => (name, mcp_in_toml(file, name)),
            Applied::ClaudeMcp { name, .. } => (name, claude_has_mcp(name)),
            _ => continue,
        };
        if let Err(why) = present {
            checks.push((format!("{name} MCP configured"), false, why));
            continue;
        }
        let Some(server) = e.checks.mcp.get(name) else {
            continue;
        };
        let what = format!("{name} MCP starts");
        checks.push(match mcp_starts(server) {
            Ok(took) => (what, true, format!("{:.1}s", took.as_secs_f64())),
            Err(why) => (what, false, why),
        });
    }
    for cmd in &e.checks.commands {
        checks.push(match run_check(cmd) {
            Ok(()) => (format!("`{cmd}` runs"), true, String::new()),
            Err(why) => (format!("`{cmd}` runs"), false, why),
        });
    }
    KitReport {
        scope: scope.label(),
        name: e.name.clone(),
        version: e.version.clone(),
        agents: e.agents.clone(),
        checks,
    }
}

fn mcp_in_json(file: &Path, name: &str) -> std::result::Result<(), String> {
    let doc: serde_json::Value = std::fs::read_to_string(file)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    doc["mcpServers"]
        .get(name)
        .map(|_| ())
        .ok_or_else(|| format!("gone from {}. Run kit add again", tilde(file)))
}

fn mcp_in_toml(file: &Path, name: &str) -> std::result::Result<(), String> {
    let doc: toml_edit::DocumentMut = std::fs::read_to_string(file)
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or_default();
    doc.get("mcp_servers")
        .and_then(|t| t.get(name))
        .map(|_| ())
        .ok_or_else(|| format!("gone from {}. Run kit add again", tilde(file)))
}

/// `claude mcp get <name>`: Claude Code owns its user config.
fn claude_has_mcp(name: &str) -> std::result::Result<(), String> {
    if !super::manifest::is_slug(name) {
        return Err("not a valid server name".into());
    }
    let exe = plan::find_program("claude").ok_or("Claude Code is not on PATH")?;
    let ok = Command::new(exe)
        .args(["mcp", "get", name])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if ok {
        Ok(())
    } else {
        Err("Claude Code no longer lists it. Run kit add again".into())
    }
}

/// Start a local MCP server and wait for its answer to `initialize`.
pub fn mcp_starts(server: &McpServer) -> std::result::Result<Duration, String> {
    let Some(command) = &server.command else {
        return Ok(Duration::ZERO); // remote: nothing runs here
    };
    let exe = plan::find_program(command).ok_or_else(|| format!("{command} is not on PATH"))?;
    let mut cmd = Command::new(exe);
    cmd.args(&server.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for (key, value) in &server.env {
        if let Some(v) = std::env::var_os(&value[1..]) {
            cmd.env(key, v);
        }
    }
    // Its own process group, so `npx` and the server it starts stop together.
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    let start = Instant::now();
    let mut child = cmd.spawn().map_err(|e| format!("cannot start: {e}"))?;
    let request = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "kit-doctor", "version": env!("CARGO_PKG_VERSION") },
        },
    });
    let mut stdin = child.stdin.take().expect("piped");
    let _ = writeln!(stdin, "{request}");
    let _ = stdin.flush();
    let stdout = child.stdout.take().expect("piped");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line)
                && v.get("id") == Some(&serde_json::json!(1))
            {
                let _ = tx.send(v.get("result").is_some());
                return;
            }
        }
    });
    let answer = rx.recv_timeout(MCP_TIMEOUT);
    drop(stdin);
    stop_tree(&mut child);
    match answer {
        Ok(true) => Ok(start.elapsed()),
        Ok(false) => Err("answered initialize with an error".into()),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(format!(
            "no answer to initialize in {}s",
            MCP_TIMEOUT.as_secs()
        )),
        Err(_) => Err("exited without answering initialize".into()),
    }
}

/// Stop a server and everything it started.
fn stop_tree(child: &mut std::process::Child) {
    let pid = child.id().to_string();
    let quiet = |c: &mut Command| {
        let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).status();
    };
    if cfg!(unix) {
        quiet(Command::new("kill").args(["-TERM", &format!("-{pid}")]));
    } else {
        quiet(Command::new("taskkill").args(["/T", "/F", "/PID", &pid]));
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn run_check(command: &str) -> std::result::Result<(), String> {
    let shell = if cfg!(windows) { "bash" } else { "sh" };
    let sh = plan::find_program(shell).ok_or_else(|| format!("{shell} is not on PATH"))?;
    let out = Command::new(sh)
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    Err(err
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("failed")
        .to_string())
}

/// The `kits:` section of `kit doctor`.
pub fn print(reports: &[KitReport]) {
    if reports.is_empty() {
        return;
    }
    println!();
    println!("kits:");
    for r in reports {
        println!(
            "  {} {}   {}   {}",
            r.name,
            r.version,
            r.scope,
            r.agents.join(", ")
        );
        for (what, ok, detail) in &r.checks {
            let mark = if *ok { "ok  " } else { "FAIL" };
            let detail = if detail.is_empty() {
                String::new()
            } else {
                format!("   {detail}")
            };
            println!("    {mark}  {what}{detail}");
        }
    }
}

pub fn to_json(reports: &[KitReport]) -> serde_json::Value {
    reports
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name, "version": r.version, "scope": r.scope,
                "agents": r.agents, "ok": r.ok(),
                "checks": r.checks.iter().map(|(what, ok, detail)| serde_json::json!({
                    "check": what, "ok": ok, "detail": detail,
                })).collect::<Vec<_>>(),
            })
        })
        .collect()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn server(script: &str) -> McpServer {
        McpServer {
            command: Some("sh".into()),
            url: None,
            args: vec!["-c".into(), script.into()],
            env: Default::default(),
        }
    }

    #[test]
    fn a_server_that_answers_initialize_passes() {
        let s = server(r#"read line; echo '{"jsonrpc":"2.0","id":1,"result":{}}'; sleep 5"#);
        assert!(mcp_starts(&s).is_ok());
    }

    #[test]
    fn a_server_that_exits_or_errors_fails_with_the_reason() {
        let s = server("exit 1");
        assert!(mcp_starts(&s).unwrap_err().contains("exited"));
        let s = server(r#"read line; echo '{"jsonrpc":"2.0","id":1,"error":{"code":1}}'"#);
        assert!(mcp_starts(&s).unwrap_err().contains("error"));
        let s = McpServer {
            command: Some("no-such-mcp-server".into()),
            ..server("")
        };
        assert!(mcp_starts(&s).unwrap_err().contains("not on PATH"));
    }
}
