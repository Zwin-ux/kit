//! `kit -p` end to end: the real binary against a local stand-in for an
//! OpenAI-compatible server. No network, no keys.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;

/// Serve one SSE body per request, in order; return the request bodies.
fn serve(replies: Vec<String>) -> (String, std::thread::JoinHandle<Vec<Value>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://127.0.0.1:{}/v1",
        listener.local_addr().unwrap().port()
    );
    let handle = std::thread::spawn(move || {
        let mut bodies = Vec::new();
        for reply in replies {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut len = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" || line.is_empty() {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    len = v.trim().parse().unwrap();
                }
            }
            let mut body = vec![0; len];
            reader.read_exact(&mut body).unwrap();
            bodies.push(serde_json::from_slice(&body).unwrap());
            let mut out = stream;
            write!(out, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n{reply}").unwrap();
        }
        bodies
    });
    (url, handle)
}

fn chunk(delta: Value, finish: Option<&str>) -> String {
    format!(
        "data: {}\n\n",
        json!({ "choices": [{ "index": 0, "delta": delta, "finish_reason": finish }] })
    )
}

fn tool_call(name: &str, args: Value) -> String {
    chunk(
        json!({ "tool_calls": [{ "index": 0, "id": "c1", "type": "function",
        "function": { "name": name, "arguments": args.to_string() } }] }),
        None,
    ) + &chunk(json!({}), Some("tool_calls"))
        + "data: [DONE]\n\n"
}

fn say(text: &str) -> String {
    chunk(json!({ "content": text }), None) + &chunk(json!({}), Some("stop")) + "data: [DONE]\n\n"
}

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let ok = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .status()
        .unwrap();
    assert!(ok.success());
    dir
}

fn kit(dir: &Path, home: &Path, args: &[&str]) -> (i32, Value, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_kit"))
        .args(args)
        .current_dir(dir)
        .env("KIT_HOME", home)
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("KIT_MODEL")
        .env_remove("KIT_BASE_URL")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or(Value::Null);
    (
        out.status.code().unwrap_or(-1),
        json,
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn the_agent_writes_a_file_and_reports_the_session() {
    let dir = repo();
    let home = tempfile::tempdir().unwrap();
    let (url, server) = serve(vec![
        tool_call(
            "Write",
            json!({ "file_path": "a.txt", "content": "from kit" }),
        ),
        say("Done."),
    ]);
    let (code, out, err) = kit(
        dir.path(),
        home.path(),
        &[
            "-p",
            "make a.txt",
            "--base-url",
            &url,
            "--model",
            "m",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    assert_eq!(out["ok"], true);
    assert_eq!(out["data"]["outcome"], "done");
    assert_eq!(out["data"]["text"], "Done.");
    assert_eq!(out["data"]["provider"], "openai-compatible");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
        "from kit"
    );
    let session = out["data"]["session"].as_str().unwrap();
    assert!(Path::new(session).starts_with(home.path()) && Path::new(session).exists());
    let bodies = server.join().unwrap();
    assert_eq!(bodies[0]["model"], "m");
    assert!(
        bodies[0]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["function"]["name"] == "Write")
    );
    let last = bodies[1]["messages"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(last["role"], "tool");
}

#[test]
fn a_boundary_prompt_is_denied_without_a_terminal() {
    let dir = repo();
    let home = tempfile::tempdir().unwrap();
    let (url, server) = serve(vec![
        tool_call("Bash", json!({ "command": "cargo build" })),
        say("Could not build."),
    ]);
    let (code, out, err) = kit(
        dir.path(),
        home.path(),
        &[
            "-p",
            "build it",
            "--base-url",
            &url,
            "--model",
            "m",
            "--json",
        ],
    );
    assert_eq!(code, 0, "{out} {err}");
    let bodies = server.join().unwrap();
    let result = bodies[1]["messages"].as_array().unwrap().last().unwrap()["content"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(result.contains("denied"), "{result}");
}

#[test]
fn plain_output_streams_text_to_stdout_and_cards_to_stderr() {
    let dir = repo();
    let home = tempfile::tempdir().unwrap();
    let (url, _server) = serve(vec![
        tool_call("Glob", json!({ "pattern": "*" })),
        say("Nothing here."),
    ]);
    let out = Command::new(env!("CARGO_BIN_EXE_kit"))
        .args(["-p", "look", "--base-url", &url, "--model", "m"])
        .current_dir(dir.path())
        .env("KIT_HOME", home.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "Nothing here.");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("▸ Glob  *") && err.contains("kit: done"),
        "{err}"
    );
}

#[test]
fn setup_errors_say_how_to_fix_them() {
    let dir = repo();
    let home = tempfile::tempdir().unwrap();
    let (code, out, _) = kit(
        dir.path(),
        home.path(),
        &["-p", "x", "--provider", "anthropic", "--json"],
    );
    assert_eq!(code, 2);
    assert!(
        out["error"].as_str().unwrap().contains("ANTHROPIC_API_KEY"),
        "{out}"
    );
    let (code, out, _) = kit(dir.path(), home.path(), &["-p", "x", "--json"]);
    assert_eq!(code, 2);
    assert!(
        out["error"].as_str().unwrap().contains("name a model"),
        "{out}"
    );
    let (code, _, err) = kit(dir.path(), home.path(), &["--model", "m"]);
    assert_eq!(code, 2, "agent flags need -p: {err}");
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args([
            "-c",
            "user.name=kit",
            "-c",
            "user.email=kit@test",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .status()
        .unwrap();
    assert!(ok.success(), "git {args:?}");
}

#[test]
fn kit_run_with_the_kit_agent_is_gated_and_receipted() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    std::fs::write(repo.join("README.md"), "app\n").unwrap();
    std::fs::write(repo.join("kit.toml"), "[gate]\ntest = 'git --version'\n").unwrap();
    git(repo, &["init", "-q", "-b", "main"]);
    git(repo, &["config", "core.autocrlf", "false"]);
    git(repo, &["add", "."]);
    git(repo, &["commit", "-q", "-m", "init"]);
    let home = tempfile::tempdir().unwrap();
    let (url, server) = serve(vec![
        tool_call(
            "Write",
            json!({ "file_path": "a.txt", "content": "from kit\n" }),
        ),
        say("Done."),
    ]);
    // The run's child `kit -p` finds its model through the environment.
    let out = Command::new(env!("CARGO_BIN_EXE_kit"))
        .args(["run", "--agent", "kit", "--json", "--task", "make a.txt"])
        .current_dir(repo)
        .env("KIT_HOME", home.path())
        .env("KIT_BASE_URL", &url)
        .env("KIT_MODEL", "m")
        .env_remove("ANTHROPIC_API_KEY")
        .output()
        .unwrap();
    let env: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{e}: {}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    assert_eq!(
        env["data"]["state"],
        "pass",
        "{env}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let receipt = Path::new(env["data"]["receiptDir"].as_str().unwrap());
    let diff = std::fs::read_to_string(receipt.join("diff.patch")).unwrap();
    assert!(
        diff.contains("diff --git a/a.txt b/a.txt") && diff.contains("+from kit"),
        "{diff}"
    );
    assert!(
        !repo.join("a.txt").exists(),
        "the run worked in its own worktree"
    );
    let bodies = server.join().unwrap();
    assert_eq!(bodies.len(), 2);
    // Receipt v2: the agent's own account.
    let agent = &env["data"]["agent"];
    assert_eq!(
        (agent["model"].as_str(), agent["provider"].as_str()),
        (Some("m"), Some("openai-compatible")),
        "{env}"
    );
    assert_eq!(
        (agent["outcome"].as_str(), agent["replies"].as_u64()),
        (Some("done"), Some(2))
    );
    let receipt: Value =
        serde_json::from_str(&std::fs::read_to_string(receipt.join("receipt.json")).unwrap())
            .unwrap();
    assert_eq!(receipt["version"], 2);
    assert_eq!(receipt["agent"]["profile"], "workspace");
}

#[test]
fn a_session_that_runs_out_of_budget_is_a_budget_run() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    std::fs::write(repo.join("kit.toml"), "[gate]\ntest = 'git --version'\n").unwrap();
    git(repo, &["init", "-q", "-b", "main"]);
    git(repo, &["add", "."]);
    git(repo, &["commit", "-q", "-m", "init"]);
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        "[agent]\nbudget_usd = 0.5\n",
    )
    .unwrap();
    std::fs::write(
        home.path().join("prices.toml"),
        "[models.m]\ninput = 1000.0\noutput = 1.0\n",
    )
    .unwrap();
    // One reply that costs $1.00 (1,000 input tokens at $1,000 per million), then a tool call.
    let reply = chunk(
        json!({ "tool_calls": [{ "index": 0, "id": "c1", "type": "function",
            "function": { "name": "Glob", "arguments": "{\"pattern\":\"*\"}" } }] }),
        None,
    ) + &chunk(json!({}), Some("tool_calls"))
        + &format!(
            "data: {}\n\n",
            json!({ "choices": [], "usage": { "prompt_tokens": 1000, "completion_tokens": 0 } })
        )
        + "data: [DONE]\n\n";
    let (url, server) = serve(vec![reply]);
    let out = Command::new(env!("CARGO_BIN_EXE_kit"))
        .args(["run", "--agent", "kit", "--json", "--task", "look around"])
        .current_dir(repo)
        .env("KIT_HOME", home.path())
        .env("KIT_BASE_URL", &url)
        .env("KIT_MODEL", "m")
        .env_remove("ANTHROPIC_API_KEY")
        .output()
        .unwrap();
    let env: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        env["data"]["state"],
        "budget",
        "{env}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(env["data"]["agent"]["outcome"], "budget");
    assert_eq!(env["data"]["agent"]["cost_estimated"], true);
    assert_eq!(
        server.join().unwrap().len(),
        1,
        "no second reply after the budget ran out"
    );
}
