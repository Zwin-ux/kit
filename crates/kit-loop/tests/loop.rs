//! The real loop driven by a scripted provider: no model, no network.

use kit_loop::*;
use kit_providers::{
    Block, Event, ProviderError, Role, ScriptedProvider, StopReason, Usage, scripted::Turn,
};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::mpsc;

fn call(id: &str, name: &str, args: Value) -> Event {
    Event::ToolCall {
        id: id.into(),
        name: name.into(),
        args,
        args_error: None,
    }
}
fn stop(reason: StopReason) -> Event {
    Event::Stop { reason }
}
fn text(t: &str) -> Event {
    Event::TextDelta { text: t.into() }
}
fn done(t: &str) -> Turn {
    Turn::Events(vec![text(t), stop(StopReason::EndTurn)])
}

struct Run {
    outcome: Outcome,
    events: Vec<AgentEvent>,
    provider: Arc<ScriptedProvider>,
    agent: Agent,
    _dir: tempfile::TempDir,
}

async fn run_with(
    turns: Vec<Turn>,
    setup: impl FnOnce(&mut Agent),
    approver: &dyn Approver,
    cancel: Cancel,
) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(dir.path()).unwrap();
    ws.home = None;
    ws.skills.clear();
    let provider = Arc::new(ScriptedProvider::new(turns));
    let cfg = Config {
        model: "m".into(),
        ..Config::default()
    };
    let mut agent = Agent::new(provider.clone(), ws, Permissions::default(), cfg);
    setup(&mut agent);
    let (tx, mut rx) = mpsc::channel(1024);
    let outcome = agent.run("fix it", &tx, approver, &cancel).await;
    drop(tx);
    let mut events = Vec::new();
    while let Some(e) = rx.recv().await {
        events.push(e);
    }
    Run {
        outcome,
        events,
        provider,
        agent,
        _dir: dir,
    }
}

async fn run(turns: Vec<Turn>) -> Run {
    run_with(turns, |_| {}, &DenyAll, Cancel::default()).await
}

#[tokio::test]
async fn tool_round_trip_writes_the_file_and_returns_the_result() {
    let r = run(vec![
        Turn::Events(vec![
            text("Writing."),
            call(
                "t1",
                "Write",
                json!({"file_path": "a.txt", "content": "hi"}),
            ),
            stop(StopReason::ToolUse),
        ]),
        done("Done."),
    ])
    .await;
    assert_eq!(r.outcome, Outcome::Done);
    assert_eq!(
        std::fs::read_to_string(r.agent.ws.root.join("a.txt")).unwrap(),
        "hi"
    );
    let second = &r.provider.requests()[1];
    let last = second.messages.last().unwrap();
    assert_eq!(last.role, Role::User);
    assert!(
        matches!(&last.content[0], Block::ToolResult { id, is_error: false, .. } if id == "t1")
    );
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, AgentEvent::ToolStart { name, .. } if name == "Write"))
    );
    assert!(r.events.iter().any(|e| matches!(
        e,
        AgentEvent::ToolEnd {
            denied: false,
            is_error: false,
            ..
        }
    )));
}

#[tokio::test]
async fn a_prompt_with_no_one_to_ask_is_a_deny_and_the_model_hears_why() {
    let r = run(vec![
        Turn::Events(vec![
            call("t1", "Bash", json!({"command": "cargo build"})),
            stop(StopReason::ToolUse),
        ]),
        done("I could not build."),
    ])
    .await;
    assert_eq!(r.outcome, Outcome::Done);
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, AgentEvent::ToolStart { .. })),
        "a denied call never starts"
    );
    let reqs = r.provider.requests();
    let result = &reqs[1].messages.last().unwrap().content[0];
    assert!(
        matches!(result, Block::ToolResult { is_error: true, content, .. } if content.starts_with("denied:")),
        "{result:?}"
    );
}

struct AllowAll;
#[async_trait::async_trait]
impl Approver for AllowAll {
    async fn approve(&self, _: &str, _: &Value, _: &str) -> bool {
        true
    }
}

#[tokio::test]
async fn an_approved_prompt_runs() {
    let r = run_with(
        vec![
            Turn::Events(vec![
                call(
                    "t1",
                    "Write",
                    json!({"file_path": "kit.toml", "content": "x"}),
                ),
                stop(StopReason::ToolUse),
            ]),
            done("ok"),
        ],
        |_| {},
        &AllowAll,
        Cancel::default(),
    )
    .await;
    assert_eq!(
        std::fs::read_to_string(r.agent.ws.root.join("kit.toml")).unwrap(),
        "x",
        "protected, asked, approved"
    );
}

#[tokio::test]
async fn the_budget_stops_the_run_before_the_next_reply() {
    let big = Usage {
        input: 1_000_000,
        output: 0,
        ..Default::default()
    };
    let r = run_with(
        vec![
            Turn::Events(vec![
                call("t1", "Glob", json!({"pattern": "*"})),
                Event::Usage(big),
                stop(StopReason::ToolUse),
            ]),
            done("never sent"),
        ],
        |a| {
            a.cfg.budget_usd = Some(1.0);
            a.cfg.prices = Prices::from_toml("[models.m]\ninput = 2.0\noutput = 10.0\n").unwrap();
        },
        &DenyAll,
        Cancel::default(),
    )
    .await;
    assert_eq!(r.outcome, Outcome::Budget);
    assert_eq!(
        r.provider.remaining(),
        1,
        "the second reply was never requested"
    );
    assert_eq!(r.agent.meter.dollars(), "≈$2.000");
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, AgentEvent::Notice { text } if text.contains("budget")))
    );
}

#[tokio::test]
async fn the_loop_detector_stops_the_same_call_repeated() {
    let same = || {
        Turn::Events(vec![
            call("t", "Glob", json!({"pattern": "*.rs"})),
            stop(StopReason::ToolUse),
        ])
    };
    let r = run(vec![same(), same(), same(), same(), same()]).await;
    assert_eq!(r.outcome, Outcome::Looping);
    assert_eq!(r.provider.remaining(), 2);
}

#[tokio::test]
async fn interrupt_ends_the_run() {
    let cancel = Cancel::default();
    cancel.cancel();
    let r = run_with(vec![done("hi")], |_| {}, &DenyAll, cancel).await;
    assert_eq!(r.outcome, Outcome::Interrupted);
}

#[tokio::test]
async fn the_turn_cap_and_bad_arguments() {
    let r = run_with(
        vec![Turn::Events(vec![
            Event::ToolCall {
                id: "t1".into(),
                name: "Read".into(),
                args: Value::Null,
                args_error: Some("arguments are not valid JSON".into()),
            },
            stop(StopReason::ToolUse),
        ])],
        |a| a.cfg.max_turns = 1,
        &DenyAll,
        Cancel::default(),
    )
    .await;
    assert_eq!(r.outcome, Outcome::MaxTurns);
    assert!(r.events.iter().any(|e| matches!(e, AgentEvent::ToolEnd { is_error: true, content, .. } if content.contains("not valid JSON"))));
}

#[tokio::test]
async fn a_provider_error_ends_the_run_with_it() {
    let r = run(vec![Turn::Error {
        error: "overloaded".into(),
    }])
    .await;
    assert_eq!(
        r.outcome,
        Outcome::Error(ProviderError::Transient("overloaded".into()).to_string())
    );
}

#[tokio::test]
async fn text_protocol_runs_the_fenced_block_and_feeds_back_output() {
    let dir = tempfile::tempdir().unwrap();
    let shell = Workspace::open(dir.path()).unwrap().shell.name();
    let r = run_with(
        vec![
            Turn::Events(vec![
                text(&format!("```{shell}\necho kit-text-mode\n```")),
                stop(StopReason::EndTurn),
            ]),
            done("Done."),
        ],
        |a| a.cfg.text_protocol = true,
        &DenyAll,
        Cancel::default(),
    )
    .await;
    assert_eq!(r.outcome, Outcome::Done);
    let req = &r.provider.requests()[1];
    assert!(req.tools.is_empty(), "no tool schemas in text mode");
    assert!(
        matches!(&req.messages.last().unwrap().content[0], Block::Text { text } if text.contains("kit-text-mode")),
        "{req:?}"
    );
}

#[tokio::test]
async fn read_only_calls_in_one_reply_all_run() {
    let r = run(vec![
        Turn::Events(vec![
            call("a", "Glob", json!({"pattern": "*"})),
            call("b", "Grep", json!({"pattern": "x"})),
            stop(StopReason::ToolUse),
        ]),
        done("ok"),
    ])
    .await;
    let reqs = r.provider.requests();
    let results = &reqs[1].messages.last().unwrap().content;
    assert_eq!(results.len(), 2);
    assert!(
        matches!(&results[1], Block::ToolResult { id, .. } if id == "b"),
        "results keep call order"
    );
}

#[tokio::test]
async fn the_session_file_records_every_event() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("sessions");
    let mut path = None;
    let _r = run_with(
        vec![done("hello")],
        |a| {
            let log = SessionLog::create(&log_dir, "s1").unwrap();
            path = Some(log.path().to_path_buf());
            let agent = std::mem::replace(
                a,
                Agent::new(
                    Arc::new(ScriptedProvider::default()),
                    a.ws.clone(),
                    Permissions::default(),
                    Config::default(),
                ),
            );
            *a = agent.with_log(log);
        },
        &DenyAll,
        Cancel::default(),
    )
    .await;
    let text = std::fs::read_to_string(path.unwrap()).unwrap();
    let kinds: Vec<String> = text
        .lines()
        .map(|l| {
            serde_json::from_str::<Value>(l).unwrap()["type"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(kinds, ["prompt", "text"]);
}

#[test]
fn prompt_and_tool_schemas_fit_the_token_budget() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(dir.path()).unwrap();
    ws.home = None;
    ws.skills.clear();
    let tools = serde_json::to_string(&tool_specs()).unwrap();
    let bpe = tiktoken_rs::o200k_base().unwrap();
    let tokens = bpe
        .encode_with_special_tokens(&prompt::system_prompt(&ws, false))
        .len()
        + bpe.encode_with_special_tokens(&tools).len();
    assert!(
        tokens <= 1500,
        "prompt + tools = {tokens} tokens (budget 1500, PRD §2)"
    );
}
