//! Recorded streams through each adapter, with no network.

use kit_providers::transport::Canned;
use kit_providers::*;
use serde_json::json;
use std::sync::Arc;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    // Fixtures may be checked out with CRLF; SSE accepts both.
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn request() -> Request {
    Request {
        model: "m".into(),
        system: "You are Kit.".into(),
        messages: vec![
            Message {
                role: Role::User,
                content: vec![Block::Text {
                    text: "fix the test".into(),
                }],
            },
            Message {
                role: Role::Assistant,
                content: vec![
                    Block::Text {
                        text: "Looking.".into(),
                    },
                    Block::ToolUse {
                        id: "t1".into(),
                        name: "Read".into(),
                        input: json!({"file_path": "a.rs"}),
                    },
                ],
            },
            Message {
                role: Role::User,
                content: vec![Block::ToolResult {
                    id: "t1".into(),
                    content: "fn a() {}".into(),
                    is_error: false,
                }],
            },
        ],
        tools: vec![ToolSpec {
            name: "Read".into(),
            description: "Read a file".into(),
            schema: json!({"type": "object", "properties": {"file_path": {"type": "string"}}}),
        }],
        max_tokens: 1024,
    }
}

async fn collect(mut rx: EventStream) -> Vec<Result<Event, ProviderError>> {
    let mut out = Vec::new();
    while let Some(e) = rx.recv().await {
        out.push(e);
    }
    out
}

#[tokio::test]
async fn anthropic_text_then_tool_call_with_usage() {
    let t = Arc::new(ReplayTransport::new(vec![ReplayTransport::sse(&fixture(
        "anthropic_tool_use.sse",
    ))]));
    let p = Anthropic::new("https://api.anthropic.com/", "sk-test", t.clone());
    let events: Vec<Event> = collect(p.stream(request()).await.unwrap())
        .await
        .into_iter()
        .map(Result::unwrap)
        .collect();
    assert_eq!(
        events,
        vec![
            Event::TextDelta {
                text: "Reading the ".into()
            },
            Event::TextDelta {
                text: "test first.".into()
            },
            Event::ToolCall {
                id: "toolu_01A".into(),
                name: "Read".into(),
                args: json!({"file_path": "tests/login.rs"}),
                args_error: None
            },
            Event::Usage(Usage {
                input: 412,
                cached_input: 1800,
                cache_write: 0,
                output: 38,
                reasoning: 0,
                cost_usd: None
            }),
            Event::Stop {
                reason: StopReason::ToolUse
            },
        ]
    );
    let calls = t.calls();
    assert_eq!(calls[0].0, "https://api.anthropic.com/v1/messages");
    let body = &calls[0].1;
    assert_eq!(body["stream"], true);
    assert_eq!(body["system"][0]["cache_control"]["type"], "ephemeral");
    assert_eq!(body["tools"][0]["input_schema"]["type"], "object");
    assert_eq!(body["messages"][1]["content"][1]["type"], "tool_use");
    assert_eq!(body["messages"][2]["content"][0]["tool_use_id"], "t1");
}

#[tokio::test]
async fn anthropic_stream_error_is_classified() {
    let t = Arc::new(ReplayTransport::new(vec![ReplayTransport::sse(&fixture(
        "anthropic_overloaded.sse",
    ))]));
    let p = Anthropic::new("https://api.anthropic.com", "k", t);
    let out = collect(p.stream(request()).await.unwrap()).await;
    assert!(
        matches!(out.last(), Some(Err(ProviderError::Transient(m))) if m.contains("overloaded")),
        "{out:?}"
    );
}

#[tokio::test]
async fn a_stream_that_stops_early_is_an_error_not_a_reply() {
    let t = Arc::new(ReplayTransport::new(vec![ReplayTransport::sse(&fixture(
        "anthropic_truncated.sse",
    ))]));
    let p = Anthropic::new("https://api.anthropic.com", "k", t);
    let out = collect(p.stream(request()).await.unwrap()).await;
    assert_eq!(
        out[0],
        Ok(Event::TextDelta {
            text: "Half a rep".into()
        })
    );
    assert!(
        matches!(out.last(), Some(Err(ProviderError::Transient(_)))),
        "{out:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn rate_limit_before_the_stream_is_retried() {
    let t = Arc::new(ReplayTransport::new(vec![
        Canned::Fail(ProviderError::RateLimited {
            retry_after: Some(std::time::Duration::from_secs(2)),
        }),
        ReplayTransport::sse(&fixture("anthropic_tool_use.sse")),
    ]));
    let p = Anthropic::new("https://api.anthropic.com", "k", t.clone());
    let out = collect(p.stream(request()).await.unwrap()).await;
    assert!(matches!(out.last(), Some(Ok(Event::Stop { .. }))));
    assert_eq!(t.calls().len(), 2);
}

#[tokio::test]
async fn auth_failure_is_not_retried() {
    let t = Arc::new(ReplayTransport::new(vec![Canned::Fail(
        ProviderError::Auth,
    )]));
    let p = Anthropic::new("https://api.anthropic.com", "bad", t.clone());
    assert_eq!(p.stream(request()).await.unwrap_err(), ProviderError::Auth);
    assert_eq!(t.calls().len(), 1);
}

#[tokio::test]
async fn openai_parallel_tool_calls_assemble_from_fragments() {
    let t = Arc::new(ReplayTransport::new(vec![ReplayTransport::sse(&fixture(
        "openai_tool_calls.sse",
    ))]));
    let p = OpenAiCompat::new(
        "https://api.example.com/v1/",
        Some("sk-x".into()),
        t.clone(),
    );
    let events: Vec<Event> = collect(p.stream(request()).await.unwrap())
        .await
        .into_iter()
        .map(Result::unwrap)
        .collect();
    assert_eq!(
        events,
        vec![
            Event::ToolCall {
                id: "call_abc".into(),
                name: "Grep".into(),
                args: json!({"pattern": "CRLF"}),
                args_error: None
            },
            Event::ToolCall {
                id: "call_def".into(),
                name: "Glob".into(),
                args: json!({"pattern": "**/*.rs"}),
                args_error: None
            },
            Event::Usage(Usage {
                input: 300,
                cached_input: 600,
                cache_write: 0,
                output: 40,
                reasoning: 12,
                cost_usd: None
            }),
            Event::Stop {
                reason: StopReason::ToolUse
            },
        ]
    );
    let (url, body) = &t.calls()[0];
    assert_eq!(url, "https://api.example.com/v1/chat/completions");
    assert_eq!(body["messages"][0]["role"], "system");
    assert_eq!(
        body["messages"][2]["tool_calls"][0]["function"]["arguments"],
        r#"{"file_path":"a.rs"}"#
    );
    assert_eq!(body["messages"][3]["role"], "tool");
    assert_eq!(body["messages"][3]["tool_call_id"], "t1");
    assert_eq!(body["stream_options"]["include_usage"], true);
}

#[tokio::test]
async fn local_server_without_done_or_usage_still_ends_the_reply() {
    let t = Arc::new(ReplayTransport::new(vec![ReplayTransport::sse(&fixture(
        "ollama_text_no_done.sse",
    ))]));
    let p = OpenAiCompat::new("http://localhost:11434/v1", None, t);
    let events: Vec<Event> = collect(p.stream(request()).await.unwrap())
        .await
        .into_iter()
        .map(Result::unwrap)
        .collect();
    assert_eq!(
        events,
        vec![
            Event::TextDelta {
                text: "Fixed: ".into()
            },
            // The fixture's JSON escapes a newline; it must arrive as one.
            Event::TextDelta {
                text: "the parser split on \n only.".into()
            },
            Event::Stop {
                reason: StopReason::EndTurn
            },
        ]
    );
}

#[tokio::test]
async fn models_lists_ids_and_windows_when_given() {
    let t = Arc::new(ReplayTransport::new(vec![Canned::Json(json!({
        "data": [{"id": "llama3.1:8b", "context_length": 131072}, {"id": "qwen"}]
    }))]));
    let p = OpenAiCompat::new("http://localhost:11434/v1", None, t);
    let models = p.models().await.unwrap();
    assert_eq!(
        models[0],
        ModelInfo {
            id: "llama3.1:8b".into(),
            context_window: Some(131072)
        }
    );
    assert_eq!(
        models[1].context_window, None,
        "a missing window is never guessed"
    );
}

#[test]
fn keys_never_show_in_debug_output() {
    let t: Arc<dyn Transport> = Arc::new(ReplayTransport::default());
    let a = format!("{:?}", Anthropic::new("u", "sk-ant-secret", t.clone()));
    let o = format!("{:?}", OpenAiCompat::new("u", Some("sk-secret".into()), t));
    assert!(!a.contains("secret") && !o.contains("secret"), "{a} {o}");
}

#[tokio::test]
async fn scripted_provider_replays_turns_and_records_requests() {
    let p = ScriptedProvider::from_json(
        r#"[
            [{"type": "text_delta", "text": "hi"}, {"type": "stop", "reason": "end_turn"}],
            {"error": "boom"}
        ]"#,
    )
    .unwrap();
    let first = collect(p.stream(request()).await.unwrap()).await;
    assert_eq!(first.len(), 2);
    assert_eq!(
        p.stream(request()).await.unwrap_err(),
        ProviderError::Transient("boom".into())
    );
    assert!(matches!(
        p.stream(request()).await,
        Err(ProviderError::Fatal(_))
    ));
    assert_eq!(p.requests().len(), 3);
    assert_eq!(p.remaining(), 0);
}
