//! Anthropic Messages API, streaming, with tool use and prompt caching.
//! Format: <https://docs.anthropic.com/en/api/messages-streaming> (read 2026-10-31).

use crate::sse::SseEvent;
use crate::{
    Block, Decoder, Event, EventStream, Message, ModelInfo, Provider, ProviderError, Request, Role,
    StopReason, Transport, Usage, parse_args, run_stream,
};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

const API_VERSION: &str = "2023-06-01";

/// The Anthropic API, with an API key. Never a subscription token.
pub struct Anthropic {
    base_url: String,
    api_key: String,
    transport: Arc<dyn Transport>,
}

impl fmt::Debug for Anthropic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Anthropic")
            .field("base_url", &self.base_url)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

impl Anthropic {
    /// `base_url` is usually `https://api.anthropic.com`.
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        transport: Arc<dyn Transport>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
            transport,
        }
    }

    fn headers(&self) -> Vec<(String, String)> {
        vec![
            ("x-api-key".into(), self.api_key.clone()),
            ("anthropic-version".into(), API_VERSION.into()),
        ]
    }
}

/// The request body. The system prompt and the last tool carry cache
/// breakpoints, so the stable prefix is cached across turns.
pub fn body(req: &Request) -> Value {
    let mut tools: Vec<Value> = req
        .tools
        .iter()
        .map(|t| json!({ "name": t.name, "description": t.description, "input_schema": t.schema }))
        .collect();
    if let Some(last) = tools.last_mut() {
        last["cache_control"] = json!({ "type": "ephemeral" });
    }
    let mut body = json!({
        "model": req.model,
        "max_tokens": req.max_tokens,
        "stream": true,
        "messages": req.messages.iter().map(message).collect::<Vec<_>>(),
    });
    if !req.system.is_empty() {
        body["system"] = json!([{ "type": "text", "text": req.system, "cache_control": { "type": "ephemeral" } }]);
    }
    if !tools.is_empty() {
        body["tools"] = Value::Array(tools);
    }
    body
}

fn message(m: &Message) -> Value {
    let role = match m.role {
        Role::User => "user",
        Role::Assistant => "assistant",
    };
    let content: Vec<Value> = m
        .content
        .iter()
        .map(|b| match b {
            Block::Text { text } => json!({ "type": "text", "text": text }),
            Block::ToolUse { id, name, input } => json!({ "type": "tool_use", "id": id, "name": name, "input": input }),
            Block::ToolResult { id, content, is_error } => {
                json!({ "type": "tool_result", "tool_use_id": id, "content": content, "is_error": is_error })
            }
        })
        .collect();
    json!({ "role": role, "content": content })
}

#[async_trait]
impl Provider for Anthropic {
    fn id(&self) -> &'static str {
        "anthropic"
    }

    async fn models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        let v = self
            .transport
            .get_json(
                &format!("{}/v1/models?limit=1000", self.base_url),
                &self.headers(),
            )
            .await?;
        Ok(v["data"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|m| {
                        Some(ModelInfo {
                            id: m["id"].as_str()?.to_owned(),
                            context_window: m["max_input_tokens"].as_u64(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn stream(&self, req: Request) -> Result<EventStream, ProviderError> {
        let url = format!("{}/v1/messages", self.base_url);
        run_stream(
            self.transport.as_ref(),
            url,
            self.headers(),
            body(&req),
            AnthropicDecoder::default(),
        )
        .await
    }
}

#[derive(Default)]
struct ToolDraft {
    id: String,
    name: String,
    json: String,
}

/// State for one reply: tool calls being assembled, usage, stop reason.
#[derive(Default)]
pub(crate) struct AnthropicDecoder {
    tools: BTreeMap<u64, ToolDraft>,
    usage: Usage,
    stop: Option<StopReason>,
    done: bool,
}

fn stop_reason(s: &str) -> StopReason {
    match s {
        "end_turn" | "stop_sequence" => StopReason::EndTurn,
        "tool_use" => StopReason::ToolUse,
        "max_tokens" => StopReason::MaxTokens,
        "refusal" => StopReason::Refusal,
        other => StopReason::Other(other.to_owned()),
    }
}

fn stream_error(v: &Value) -> ProviderError {
    let kind = v["error"]["type"].as_str().unwrap_or("error");
    let msg = v["error"]["message"].as_str().unwrap_or("").to_owned();
    match kind {
        "overloaded_error" | "api_error" => ProviderError::Transient(format!("{kind}: {msg}")),
        "rate_limit_error" => ProviderError::RateLimited { retry_after: None },
        "authentication_error" | "permission_error" => ProviderError::Auth,
        "invalid_request_error" if msg.contains("prompt is too long") => {
            ProviderError::ContextOverflow
        }
        "invalid_request_error" => ProviderError::BadRequest(msg),
        _ => ProviderError::Fatal(format!("{kind}: {msg}")),
    }
}

impl Decoder for AnthropicDecoder {
    fn feed(&mut self, ev: &SseEvent) -> Result<Vec<Event>, ProviderError> {
        if self.done || ev.data.is_empty() {
            return Ok(vec![]);
        }
        let v: Value = serde_json::from_str(&ev.data)
            .map_err(|e| ProviderError::Transient(format!("unreadable stream event: {e}")))?;
        let mut out = Vec::new();
        match v["type"].as_str().unwrap_or_default() {
            "message_start" => {
                let u = &v["message"]["usage"];
                self.usage.input = u["input_tokens"].as_u64().unwrap_or(0);
                self.usage.cached_input = u["cache_read_input_tokens"].as_u64().unwrap_or(0);
                self.usage.cache_write = u["cache_creation_input_tokens"].as_u64().unwrap_or(0);
                self.usage.output = u["output_tokens"].as_u64().unwrap_or(0);
            }
            "content_block_start" => {
                let block = &v["content_block"];
                if block["type"] == "tool_use" {
                    let draft = ToolDraft {
                        id: block["id"].as_str().unwrap_or_default().to_owned(),
                        name: block["name"].as_str().unwrap_or_default().to_owned(),
                        json: String::new(),
                    };
                    self.tools.insert(v["index"].as_u64().unwrap_or(0), draft);
                }
            }
            "content_block_delta" => {
                let d = &v["delta"];
                match d["type"].as_str().unwrap_or_default() {
                    "text_delta" => out.push(Event::TextDelta {
                        text: d["text"].as_str().unwrap_or("").to_owned(),
                    }),
                    "thinking_delta" => out.push(Event::ThinkingDelta {
                        text: d["thinking"].as_str().unwrap_or("").to_owned(),
                    }),
                    "input_json_delta" => {
                        if let Some(t) = self.tools.get_mut(&v["index"].as_u64().unwrap_or(0)) {
                            t.json.push_str(d["partial_json"].as_str().unwrap_or(""));
                        }
                    }
                    _ => {}
                }
            }
            "content_block_stop" => {
                if let Some(t) = self.tools.remove(&v["index"].as_u64().unwrap_or(0)) {
                    let (args, args_error) = parse_args(&t.json);
                    out.push(Event::ToolCall {
                        id: t.id,
                        name: t.name,
                        args,
                        args_error,
                    });
                }
            }
            "message_delta" => {
                if let Some(r) = v["delta"]["stop_reason"].as_str() {
                    self.stop = Some(stop_reason(r));
                }
                if let Some(n) = v["usage"]["output_tokens"].as_u64() {
                    self.usage.output = n;
                }
            }
            "message_stop" => {
                self.done = true;
                out.push(Event::Usage(self.usage));
                out.push(Event::Stop {
                    reason: self.stop.take().unwrap_or(StopReason::EndTurn),
                });
            }
            "error" => return Err(stream_error(&v)),
            _ => {} // ping, and types added later
        }
        Ok(out)
    }

    fn finish(&mut self) -> Result<Vec<Event>, ProviderError> {
        if self.done {
            Ok(vec![])
        } else {
            Err(ProviderError::Transient(
                "the stream ended before the reply did".into(),
            ))
        }
    }
}
