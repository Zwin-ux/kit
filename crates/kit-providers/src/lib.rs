//! Model API adapters for Kit's own agent (docs/PRD.md §6).
//!
//! Each adapter turns a [`Request`] into one streaming HTTP call and the
//! server-sent events back into [`Event`]s. HTTP goes through a
//! [`Transport`], so tests replay recorded streams and never touch the
//! network. One file per adapter; the shared pieces are this file,
//! `sse.rs` and `retry.rs`.

#![warn(missing_docs)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc;

pub mod anthropic;
pub mod openai_compat;
pub mod retry;
pub mod scripted;
pub mod sse;
pub mod transport;

pub use anthropic::Anthropic;
pub use openai_compat::OpenAiCompat;
pub use scripted::ScriptedProvider;
pub use transport::{HttpTransport, ReplayTransport, Transport};

/// Who said a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// The person, or tool results going back to the model.
    User,
    /// The model.
    Assistant,
}

/// One piece of a message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    /// Plain text.
    Text {
        /// The text.
        text: String,
    },
    /// The model asked to run a tool.
    ToolUse {
        /// Call id, echoed back in the result.
        id: String,
        /// Tool name.
        name: String,
        /// Arguments as JSON.
        input: Value,
    },
    /// A tool's result, sent back to the model.
    ToolResult {
        /// The id of the call this answers.
        id: String,
        /// What the tool printed or returned.
        content: String,
        /// True when the tool failed or was denied.
        is_error: bool,
    },
}

/// One message in the conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Who said it.
    pub role: Role,
    /// Its content, in order.
    pub content: Vec<Block>,
}

/// A tool the model may call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Name, e.g. `Read`.
    pub name: String,
    /// What it does, for the model.
    pub description: String,
    /// JSON Schema of its arguments.
    pub schema: Value,
}

/// One model call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// Model id, as the provider names it.
    pub model: String,
    /// System prompt.
    pub system: String,
    /// The conversation so far.
    pub messages: Vec<Message>,
    /// Tools the model may call.
    pub tools: Vec<ToolSpec>,
    /// Cap on output tokens for this reply.
    pub max_tokens: u32,
}

/// Why the model stopped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// It finished its turn.
    EndTurn,
    /// It wants tool results before going on.
    ToolUse,
    /// It hit `max_tokens`.
    MaxTokens,
    /// It refused.
    Refusal,
    /// Anything else, named as the provider named it.
    Other(String),
}

/// Token counts for one reply. `input` excludes cached tokens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    /// Uncached input tokens.
    pub input: u64,
    /// Input tokens read from the prompt cache.
    pub cached_input: u64,
    /// Input tokens written to the prompt cache.
    pub cache_write: u64,
    /// Output tokens, reasoning included.
    pub output: u64,
    /// Of `output`, the reasoning tokens, when reported.
    pub reasoning: u64,
    /// Cost the provider reported, if it did.
    pub cost_usd: Option<f64>,
}

/// One thing that arrived on the stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// Visible text.
    TextDelta {
        /// The new text.
        text: String,
    },
    /// Reasoning text, when the provider streams it.
    ThinkingDelta {
        /// The new text.
        text: String,
    },
    /// A complete tool call.
    ToolCall {
        /// Call id.
        id: String,
        /// Tool name.
        name: String,
        /// Arguments. `Value::Null` when they did not parse; see `args_error`.
        args: Value,
        /// Why the arguments did not parse, if they didn't.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        args_error: Option<String>,
    },
    /// Token counts, once per reply.
    Usage(Usage),
    /// The reply ended.
    Stop {
        /// Why.
        reason: StopReason,
    },
}

/// What went wrong, in the classes the loop acts on.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ProviderError {
    /// The key was missing or rejected.
    #[error("key rejected or missing")]
    Auth,
    /// Too many requests; retry later.
    #[error("rate limited")]
    RateLimited {
        /// The server's `retry-after`, if it sent one.
        retry_after: Option<Duration>,
    },
    /// The request is larger than the model's context window.
    #[error("context window full")]
    ContextOverflow,
    /// A temporary failure: overloaded, 5xx, a dropped stream.
    #[error("temporary failure: {0}")]
    Transient(String),
    /// The server refused the request as malformed.
    #[error("bad request: {0}")]
    BadRequest(String),
    /// Anything that retrying will not fix.
    #[error("{0}")]
    Fatal(String),
}

/// A model and what Kit knows about it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model id.
    pub id: String,
    /// Context window in tokens, when the server says.
    pub context_window: Option<u64>,
}

/// The events of one reply, in order. The stream ends after `Stop` or an error.
pub type EventStream = mpsc::Receiver<Result<Event, ProviderError>>;

/// One model API.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Stable id: `anthropic`, `openai-compatible`, `scripted`.
    fn id(&self) -> &'static str;
    /// Models the server lists.
    async fn models(&self) -> Result<Vec<ModelInfo>, ProviderError>;
    /// Start one streamed reply.
    async fn stream(&self, req: Request) -> Result<EventStream, ProviderError>;
}

/// Turns server-sent events into [`Event`]s for one adapter.
pub(crate) trait Decoder: Send + 'static {
    fn feed(&mut self, ev: &sse::SseEvent) -> Result<Vec<Event>, ProviderError>;
    /// Called when the byte stream ends. A reply that never stopped is an error.
    fn finish(&mut self) -> Result<Vec<Event>, ProviderError>;
}

/// POST with retries, then decode the stream on a task into an [`EventStream`].
pub(crate) async fn run_stream(
    transport: &dyn Transport,
    url: String,
    headers: Vec<(String, String)>,
    body: Value,
    mut decoder: impl Decoder,
) -> Result<EventStream, ProviderError> {
    use futures::StreamExt;
    let mut bytes = retry::with_retries(|| transport.post_stream(&url, &headers, &body)).await?;
    let (tx, rx) = mpsc::channel(64);
    tokio::spawn(async move {
        let mut parser = sse::SseParser::default();
        while let Some(chunk) = bytes.next().await {
            let events = match chunk {
                Ok(chunk) => parser.push(&chunk),
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                    return;
                }
            };
            for ev in events {
                match decoder.feed(&ev) {
                    Ok(out) => {
                        for e in out {
                            let stop = matches!(e, Event::Stop { .. });
                            if tx.send(Ok(e)).await.is_err() || stop {
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        return;
                    }
                }
            }
        }
        for ev in parser.finish() {
            if let Err(e) = decoder.feed(&ev) {
                let _ = tx.send(Err(e)).await;
                return;
            }
        }
        match decoder.finish() {
            Ok(out) => {
                for e in out {
                    if tx.send(Ok(e)).await.is_err() {
                        return;
                    }
                }
            }
            Err(e) => {
                let _ = tx.send(Err(e)).await;
            }
        }
    });
    Ok(rx)
}

/// Parse a tool call's argument text. Empty text means no arguments.
pub(crate) fn parse_args(raw: &str) -> (Value, Option<String>) {
    if raw.trim().is_empty() {
        return (Value::Object(Default::default()), None);
    }
    match serde_json::from_str(raw) {
        Ok(v) => (v, None),
        Err(e) => (
            Value::Null,
            Some(format!("arguments are not valid JSON: {e}")),
        ),
    }
}
