//! OpenAI-compatible Chat Completions, streaming, with tool calls.
//! Covers Ollama, LM Studio, llama.cpp server, vLLM, OpenRouter, LiteLLM and
//! gateways. Format: <https://platform.openai.com/docs/api-reference/chat/streaming>
//! (read 2026-10-31).

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

/// Any server that speaks `/v1/chat/completions`. The key is optional: local
/// servers usually need none.
pub struct OpenAiCompat {
    base_url: String,
    api_key: Option<String>,
    transport: Arc<dyn Transport>,
}

impl fmt::Debug for OpenAiCompat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = self.api_key.as_ref().map(|_| "<redacted>");
        f.debug_struct("OpenAiCompat")
            .field("base_url", &self.base_url)
            .field("api_key", &key)
            .finish()
    }
}

impl OpenAiCompat {
    /// `base_url` ends in `/v1`, e.g. `http://localhost:11434/v1` for Ollama.
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<String>,
        transport: Arc<dyn Transport>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key,
            transport,
        }
    }

    fn headers(&self) -> Vec<(String, String)> {
        match &self.api_key {
            Some(k) => vec![("authorization".into(), format!("Bearer {k}"))],
            None => vec![],
        }
    }
}

/// The request body. Usage is requested in the stream's last chunk.
pub fn body(req: &Request) -> Value {
    let mut messages = Vec::new();
    if !req.system.is_empty() {
        messages.push(json!({ "role": "system", "content": req.system }));
    }
    for m in &req.messages {
        push_message(&mut messages, m);
    }
    let mut body = json!({
        "model": req.model,
        "max_tokens": req.max_tokens,
        "stream": true,
        "stream_options": { "include_usage": true },
        "messages": messages,
    });
    if !req.tools.is_empty() {
        body["tools"] = req
            .tools
            .iter()
            .map(|t| {
                json!({ "type": "function", "function": { "name": t.name, "description": t.description, "parameters": t.schema } })
            })
            .collect();
    }
    body
}

/// One Kit message can become several chat messages: tool results are their
/// own `tool` messages, sent before any text in the same turn.
fn push_message(out: &mut Vec<Value>, m: &Message) {
    let mut text = String::new();
    let mut calls = Vec::new();
    for b in &m.content {
        match b {
            Block::Text { text: t } => text.push_str(t),
            Block::ToolUse { id, name, input } => calls.push(json!({
                "id": id, "type": "function",
                "function": { "name": name, "arguments": input.to_string() },
            })),
            Block::ToolResult {
                id,
                content,
                is_error,
            } => {
                let content = if *is_error {
                    format!("Error: {content}")
                } else {
                    content.clone()
                };
                out.push(json!({ "role": "tool", "tool_call_id": id, "content": content }));
            }
        }
    }
    match m.role {
        Role::User if !text.is_empty() => out.push(json!({ "role": "user", "content": text })),
        Role::User => {}
        Role::Assistant => {
            let mut msg = json!({ "role": "assistant", "content": if text.is_empty() { Value::Null } else { json!(text) } });
            if !calls.is_empty() {
                msg["tool_calls"] = Value::Array(calls);
            }
            out.push(msg);
        }
    }
}

#[async_trait]
impl Provider for OpenAiCompat {
    fn id(&self) -> &'static str {
        "openai-compatible"
    }

    async fn models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        let v = self
            .transport
            .get_json(&format!("{}/models", self.base_url), &self.headers())
            .await?;
        Ok(v["data"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|m| {
                        // Servers name the window differently; none is assumed.
                        let window = ["context_length", "max_model_len", "context_window"]
                            .iter()
                            .find_map(|k| m[*k].as_u64());
                        Some(ModelInfo {
                            id: m["id"].as_str()?.to_owned(),
                            context_window: window,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn stream(&self, req: Request) -> Result<EventStream, ProviderError> {
        let url = format!("{}/chat/completions", self.base_url);
        run_stream(
            self.transport.as_ref(),
            url,
            self.headers(),
            body(&req),
            ChatDecoder::default(),
        )
        .await
    }
}

#[derive(Default)]
struct CallDraft {
    id: String,
    name: String,
    args: String,
}

/// State for one reply. Tool calls arrive in pieces keyed by `index`.
#[derive(Default)]
pub(crate) struct ChatDecoder {
    calls: BTreeMap<u64, CallDraft>,
    stop: Option<StopReason>,
    usage_sent: bool,
    done: bool,
}

fn stop_reason(s: &str) -> StopReason {
    match s {
        "stop" => StopReason::EndTurn,
        "tool_calls" | "function_call" => StopReason::ToolUse,
        "length" => StopReason::MaxTokens,
        "content_filter" => StopReason::Refusal,
        other => StopReason::Other(other.to_owned()),
    }
}

fn usage(u: &Value) -> Usage {
    let prompt = u["prompt_tokens"].as_u64().unwrap_or(0);
    let cached = u["prompt_tokens_details"]["cached_tokens"]
        .as_u64()
        .unwrap_or(0);
    Usage {
        input: prompt.saturating_sub(cached),
        cached_input: cached,
        cache_write: 0,
        output: u["completion_tokens"].as_u64().unwrap_or(0),
        reasoning: u["completion_tokens_details"]["reasoning_tokens"]
            .as_u64()
            .unwrap_or(0),
        cost_usd: None,
    }
}

impl ChatDecoder {
    fn flush_calls(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.calls)
            .into_iter()
            .map(|(i, c)| {
                let (args, args_error) = parse_args(&c.args);
                let id = if c.id.is_empty() {
                    format!("call_{i}")
                } else {
                    c.id
                };
                Event::ToolCall {
                    id,
                    name: c.name,
                    args,
                    args_error,
                }
            })
            .collect()
    }

    fn end(&mut self) -> Vec<Event> {
        self.done = true;
        let mut out = self.flush_calls();
        let reason = self.stop.take().unwrap_or(StopReason::EndTurn);
        out.push(Event::Stop { reason });
        out
    }
}

impl Decoder for ChatDecoder {
    fn feed(&mut self, ev: &SseEvent) -> Result<Vec<Event>, ProviderError> {
        if self.done || ev.data.is_empty() {
            return Ok(vec![]);
        }
        if ev.data.trim() == "[DONE]" {
            return Ok(self.end());
        }
        let v: Value = serde_json::from_str(&ev.data)
            .map_err(|e| ProviderError::Transient(format!("unreadable stream chunk: {e}")))?;
        if let Some(err) = v.get("error") {
            let msg = err["message"].as_str().unwrap_or("error").to_owned();
            return Err(crate::retry::classify(
                err["code"].as_u64().map_or(500, |c| c as u16),
                None,
                &msg,
            ));
        }
        let mut out = Vec::new();
        for choice in v["choices"].as_array().into_iter().flatten() {
            let delta = &choice["delta"];
            if let Some(t) = delta["content"].as_str().filter(|t| !t.is_empty()) {
                out.push(Event::TextDelta { text: t.to_owned() });
            }
            for r in ["reasoning_content", "reasoning"] {
                if let Some(t) = delta[r].as_str().filter(|t| !t.is_empty()) {
                    out.push(Event::ThinkingDelta { text: t.to_owned() });
                }
            }
            for call in delta["tool_calls"].as_array().into_iter().flatten() {
                let draft = self
                    .calls
                    .entry(call["index"].as_u64().unwrap_or(0))
                    .or_default();
                if let Some(id) = call["id"].as_str() {
                    draft.id = id.to_owned();
                }
                if let Some(name) = call["function"]["name"].as_str() {
                    draft.name.push_str(name);
                }
                if let Some(args) = call["function"]["arguments"].as_str() {
                    draft.args.push_str(args);
                }
            }
            if let Some(r) = choice["finish_reason"].as_str() {
                self.stop = Some(stop_reason(r));
                // Calls are complete once the choice finishes; usage may follow.
                out.extend(self.flush_calls());
            }
        }
        if v["usage"].is_object() && !self.usage_sent {
            self.usage_sent = true;
            out.push(Event::Usage(usage(&v["usage"])));
        }
        Ok(out)
    }

    fn finish(&mut self) -> Result<Vec<Event>, ProviderError> {
        if self.done {
            Ok(vec![])
        } else if self.stop.is_some() {
            // Some servers end without `[DONE]` once they sent a finish reason.
            Ok(self.end())
        } else {
            Err(ProviderError::Transient(
                "the stream ended before the reply did".into(),
            ))
        }
    }
}
