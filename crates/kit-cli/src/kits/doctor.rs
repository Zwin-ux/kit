//! The kits part of `kit doctor`: is each installed kit still live? Skills
//! as installed, rules blocks present, MCP servers still in the agents'
//! config files, and the `[check]` commands approved at `kit add` exit 0.
//! MCP servers are started (to answer an MCP `initialize`) only with
//! `--start-mcp`: starting one can leave helpers running and send the
//! server's own usage data.

use super::install::skill_names;
use super::install::{home_dir, repo_root};
use super::lock::{Entry, Lock};
use super::manifest::McpServer;
use super::plan::{self, Applied, tilde};
use super::writers::{Agent, Scope};
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
    /// (what, detail): worth knowing, never a failure (a skill the user
    /// edited is theirs).
    pub notes: Vec<(String, String)>,
}

impl KitReport {
    pub fn ok(&self) -> bool {
        self.checks.iter().all(|(_, ok, _)| *ok)
    }
}

/// Every kit installed globally and in the current repo, checked.
pub fn check_installed(start_mcp: bool) -> Result<Vec<KitReport>> {
    let mut scopes = vec![Scope::Global { home: home_dir()? }];
    if let Some(root) = repo_root(Path::new(".")) {
        scopes.push(Scope::Repo(root));
    }
    let mut out = Vec::new();
    for scope in scopes {
        for e in Lock::load(&scope)?.kits {
            out.push(check(&scope, &e, start_mcp));
        }
    }
    Ok(out)
}

fn check(scope: &Scope, e: &Entry, start_mcp: bool) -> KitReport {
    let mut checks = Vec::new();
    let mut notes = Vec::new();
    let titles: Vec<&str> = e
        .agents
        .iter()
        .filter_map(|id| Agent::ALL.into_iter().find(|a| a.id() == id))
        .map(Agent::title)
        .collect();
    // Skills are counted by name, as `kit list` counts them, however many
    // agents' folders hold a copy.
    let names = skill_names(e);
    let (mut missing, mut edited) = (Vec::new(), Vec::new());
    for a in &e.applied {
        let Applied::Skill { dir, hash } = a else {
            continue;
        };
        let name = dir.file_name().map(|n| n.to_string_lossy().into_owned());
        match plan::is_installed(dir, hash) {
            Ok(Some(true)) => {}
            Ok(Some(false)) => edited.push((name, format!("{} was changed by hand", tilde(dir)))),
            Ok(None) => missing.push((name, format!("{} is missing", tilde(dir)))),
            Err(err) => missing.push((name, format!("{}: {err:#}", tilde(dir)))),
        }
    }
    let count = |list: &[(Option<String>, String)]| {
        list.iter()
            .filter_map(|(n, _)| n.as_ref())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    };
    if let Some((_, first)) = missing.first() {
        checks.push((
            format!("{} of {} skills missing", count(&missing), names.len()),
            false,
            format!("{first}. Run kit add {} again", e.name),
        ));
    } else if !names.is_empty() {
        checks.push((
            format!(
                "{} skills as installed for {}",
                names.len(),
                super::setup::and_list(&titles)
            ),
            true,
            String::new(),
        ));
    }
    if let Some((_, first)) = edited.first() {
        notes.push((
            format!(
                "{} of {} skills changed by hand",
                count(&edited),
                names.len()
            ),
            format!(
                "{first}; kept as yours. `kit add {} --force` puts Kit's copy back",
                e.name
            ),
        ));
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
    let mut started: Vec<&str> = Vec::new();
    for a in &e.applied {
        let (name, present, place, agent) = match a {
            Applied::McpJson { file, name, .. } => {
                (name, mcp_in_json(file, name), tilde(file), Agent::Claude)
            }
            Applied::McpToml { file, name, .. } => {
                (name, mcp_in_toml(file, name), tilde(file), Agent::Codex)
            }
            Applied::ClaudeMcp { name, .. } => (
                name,
                claude_has_mcp(name),
                "~/.claude.json".to_string(),
                Agent::Claude,
            ),
            _ => continue,
        };
        let what = format!("{name} MCP configured for {}", agent.title());
        if let Err(why) = present {
            checks.push((what, false, why));
            continue;
        }
        let server = e.checks.mcp.get(name);
        let hint = if server.is_some() && !start_mcp {
            "; `kit doctor --start-mcp` starts it"
        } else {
            ""
        };
        checks.push((what, true, format!("in {place}{hint}")));
        // One start per server, whichever agents it was added for.
        let Some(server) = server else {
            continue;
        };
        if !start_mcp || started.contains(&name.as_str()) {
            continue;
        }
        started.push(name);
        let what = format!("{name} MCP starts");
        checks.push(match mcp_starts(server) {
            Ok(took) => (what, true, format!("{:.1}s", took.as_secs_f64())),
            Err(why) => (what, false, why),
        });
    }
    // Grok gets no files of its own when Claude Code is there: it reads
    // Claude Code's, which the lines above checked.
    if e.agents.iter().any(|a| a == "grok") && e.agents.iter().any(|a| a == "claude") {
        let what = "Grok reads the Claude Code files above".to_string();
        match plan::find_program("grok") {
            Some(_) => checks.push((what, true, String::new())),
            None => notes.push((what, "grok is not on PATH".into())),
        }
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
        notes,
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

/// A server `claude mcp add-json --scope user` added, read from Claude
/// Code's own config file. Never `claude mcp get` or `list`: those start
/// the server to see that it connects.
fn claude_has_mcp(name: &str) -> std::result::Result<(), String> {
    let file = match std::env::var_os("CLAUDE_CONFIG_DIR") {
        Some(dir) => Path::new(&dir).join(".claude.json"),
        None => home_dir().map_err(|e| e.to_string())?.join(".claude.json"),
    };
    mcp_in_json(&file, name).map_err(|_| "Claude Code no longer lists it. Run kit add again".into())
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

/// Stop a process and everything it started: its descendants (found by
/// parent, so one that left the process group goes too, as long as the
/// process that started it is still alive), then its process group. On
/// Windows `taskkill /T` walks the same tree.
fn stop_tree(child: &mut std::process::Child) {
    let pid = child.id();
    let quiet = |c: &mut Command| {
        let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).status();
    };
    if cfg!(unix) {
        let mut targets: Vec<String> = descendants(pid).iter().map(u32::to_string).collect();
        targets.push(format!("-{pid}"));
        quiet(Command::new("kill").arg("-KILL").arg("--").args(&targets));
    } else {
        quiet(Command::new("taskkill").args(["/T", "/F", "/PID", &pid.to_string()]));
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// Every process below `root`, from one `ps` listing.
fn descendants(root: u32) -> Vec<u32> {
    let Ok(out) = Command::new("ps")
        .args(["-A", "-o", "pid=", "-o", "ppid="])
        .stderr(Stdio::null())
        .output()
    else {
        return Vec::new();
    };
    let pairs: Vec<(u32, u32)> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace().map(str::parse::<u32>);
            Some((it.next()?.ok()?, it.next()?.ok()?))
        })
        .collect();
    let mut found = vec![root];
    let mut i = 0;
    while i < found.len() {
        let parent = found[i];
        for (pid, ppid) in &pairs {
            if *ppid == parent && !found.contains(pid) {
                found.push(*pid);
            }
        }
        i += 1;
    }
    found.remove(0);
    found
}

/// Longest an approved `[check]` command may take.
const CHECK_TIMEOUT: Duration = Duration::from_secs(120);

/// Run an approved `[check]` command in its own process group; whatever it
/// leaves running is stopped when it is done.
fn run_check(command: &str) -> std::result::Result<(), String> {
    let shell = if cfg!(windows) { "bash" } else { "sh" };
    let sh = plan::find_program(shell).ok_or_else(|| format!("{shell} is not on PATH"))?;
    let mut cmd = Command::new(sh);
    cmd.arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let stderr = child.stderr.take().expect("piped");
    let (tx, rx) = std::sync::mpsc::channel();
    // The first line that says anything. The reader drains the pipe to
    // the end, so a check that goes on writing never hits a closed pipe;
    // the end can wait for a helper left running, until it is stopped.
    std::thread::spawn(move || {
        // Bytes, not lines of text: a line that is not UTF-8 must not end
        // the read either.
        let mut first = None;
        for line in BufReader::new(stderr)
            .split(b'\n')
            .map_while(std::result::Result::ok)
        {
            let line = String::from_utf8_lossy(&line);
            if first.is_none() && !line.trim().is_empty() {
                first = Some(line.trim_end().to_string());
            }
        }
        let _ = tx.send(first);
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if start.elapsed() < CHECK_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(20));
            }
            _ => break None,
        }
    };
    stop_tree(&mut child);
    let first = rx.recv_timeout(Duration::from_secs(1)).ok().flatten();
    match status {
        Some(s) if s.success() => Ok(()),
        Some(_) => Err(first.unwrap_or_else(|| "failed".into())),
        None => Err(format!("still running after {}s", CHECK_TIMEOUT.as_secs())),
    }
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
        for (what, detail) in &r.notes {
            println!("    note  {what}   {detail}");
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
                "notes": r.notes.iter().map(|(what, detail)| serde_json::json!({
                    "note": what, "detail": detail,
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

    /// A pid that is still running (a zombie waiting for its parent to
    /// reap it is not).
    fn alive(pid: &str) -> bool {
        let out = Command::new("ps")
            .args(["-o", "stat=", "-p", pid])
            .output()
            .unwrap();
        let stat = String::from_utf8_lossy(&out.stdout);
        !stat.trim().is_empty() && !stat.trim().starts_with('Z')
    }

    fn scratch_file(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("kit-doctor-{tag}-{}", std::process::id()))
    }

    fn read_pids(file: &Path) -> Vec<String> {
        std::fs::read_to_string(file)
            .unwrap()
            .split_whitespace()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn nothing_a_started_server_leaves_behind_survives() {
        let pids = scratch_file("mcp-pids");
        let _ = std::fs::remove_file(&pids);
        // One helper in the group, one in a new session (as a detached
        // watchdog would be) when `setsid` exists.
        let s = server(&format!(
            r#"sleep 300 & echo $! >> {p}; if command -v setsid >/dev/null; then setsid sleep 300 & echo $! >> {p}; fi; read line; echo '{{"jsonrpc":"2.0","id":1,"result":{{}}}}'; sleep 300"#,
            p = pids.display()
        ));
        assert!(mcp_starts(&s).is_ok());
        std::thread::sleep(Duration::from_millis(200));
        for pid in read_pids(&pids) {
            assert!(!alive(&pid), "{pid} survived");
        }
    }

    #[test]
    fn a_check_command_leaves_nothing_running() {
        let pids = scratch_file("check-pids");
        let _ = std::fs::remove_file(&pids);
        let cmd = format!("sleep 300 & echo $! > {}", pids.display());
        assert_eq!(run_check(&cmd), Ok(()));
        std::thread::sleep(Duration::from_millis(200));
        for pid in read_pids(&pids) {
            assert!(!alive(&pid), "{pid} survived");
        }
        assert_eq!(run_check("echo nope >&2; exit 3"), Err("nope".to_string()));
        // A check that goes on writing after its first line still passes,
        // bytes that are not UTF-8 included.
        assert_eq!(
            run_check(
                r"printf '\377\376 bad\n' >&2; for i in $(seq 2000); do echo more$i >&2; done; exit 0"
            ),
            Ok(())
        );
        assert_eq!(
            run_check("echo warn >&2; for i in $(seq 2000); do echo more$i >&2; done; exit 0"),
            Ok(())
        );
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
