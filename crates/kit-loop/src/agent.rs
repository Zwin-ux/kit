//! The loop (PRD §5.1): ask the model, run the tools it calls, send the
//! results back, until it ends its turn, the budget or turn cap is hit, the
//! user interrupts, or the loop detector stops it. One turn at a time; no
//! hidden sub-loops.

use crate::meter::{Meter, Prices};
use crate::session::SessionLog;
use crate::tools::{READ_ONLY, ToolOutput, run_tool, tool_specs};
use crate::{Decision, Permissions, Workspace, prompt};
use async_trait::async_trait;
use kit_providers::{
    Block, Event, Message, Provider, ProviderError, Request, Role, StopReason, Usage,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Notify, mpsc};

/// What the loop reports as it goes.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    /// The user's prompt.
    Prompt {
        /// Its text.
        text: String,
    },
    /// Model text, as it streams.
    Text {
        /// New text.
        text: String,
    },
    /// Model reasoning, when streamed.
    Thinking {
        /// New text.
        text: String,
    },
    /// A tool call is about to run (after its permission check).
    ToolStart {
        /// Call id.
        id: String,
        /// Tool name.
        name: String,
        /// Arguments.
        args: Value,
    },
    /// A tool call finished, failed or was denied.
    ToolEnd {
        /// Call id.
        id: String,
        /// Tool name.
        name: String,
        /// What went back to the model.
        content: String,
        /// The call failed.
        is_error: bool,
        /// The call was refused before it ran.
        denied: bool,
    },
    /// Tokens and cost for one reply, and the session total.
    Usage {
        /// This reply.
        usage: Usage,
        /// This reply's cost, if known.
        cost_usd: Option<f64>,
        /// The session total, e.g. `≈$0.041`.
        total: String,
    },
    /// Something the user should see: a budget warning, a loop, a retry.
    Notice {
        /// Plain text.
        text: String,
    },
}

/// How a run ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// The model finished its turn.
    Done,
    /// The budget ran out.
    Budget,
    /// The turn cap was reached.
    MaxTurns,
    /// The user interrupted.
    Interrupted,
    /// The loop detector stopped a repeating agent.
    Looping,
    /// The model refused.
    Refused,
    /// The provider failed in a way retries did not fix.
    Error(String),
}

/// Answers boundary prompts (PRD §5.5).
#[async_trait]
pub trait Approver: Send + Sync {
    /// True to allow `tool` with `args`, which needs asking because of `reason`.
    async fn approve(&self, tool: &str, args: &Value, reason: &str) -> bool;
}

/// No one to ask (`kit -p`, CI, runs without a terminal): every prompt is a deny.
pub struct DenyAll;

#[async_trait]
impl Approver for DenyAll {
    async fn approve(&self, _tool: &str, _args: &Value, _reason: &str) -> bool {
        false
    }
}

/// Interrupts a running turn: the stream and the tool are dropped.
#[derive(Debug, Clone, Default)]
pub struct Cancel(Arc<(AtomicBool, Notify)>);

impl Cancel {
    /// Interrupt.
    pub fn cancel(&self) {
        self.0.0.store(true, Ordering::SeqCst);
        self.0.1.notify_waiters();
    }

    /// Whether [`cancel`](Self::cancel) was called.
    pub fn is_cancelled(&self) -> bool {
        self.0.0.load(Ordering::SeqCst)
    }

    /// Resolves once cancelled.
    pub async fn cancelled(&self) {
        while !self.is_cancelled() {
            let notified = self.0.1.notified();
            if self.is_cancelled() {
                return;
            }
            notified.await;
        }
    }
}

/// Settings for one agent.
#[derive(Debug, Clone)]
pub struct Config {
    /// Model id.
    pub model: String,
    /// Output cap per reply.
    pub max_tokens: u32,
    /// Replies per run before stopping.
    pub max_turns: u32,
    /// Stop when the session cost reaches this.
    pub budget_usd: Option<f64>,
    /// Prices for computing cost.
    pub prices: Prices,
    /// Fenced shell blocks instead of tool calls (PRD §6.3).
    pub text_protocol: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: String::new(),
            max_tokens: 8192,
            max_turns: 50,
            budget_usd: None,
            prices: Prices::default(),
            text_protocol: false,
        }
    }
}

/// Repeats before the loop detector asks.
const SAME_CALL_LIMIT: usize = 3;
/// Replies with no new file change and no new read before it asks.
const IDLE_LIMIT: u32 = 10;

/// One agent session.
pub struct Agent {
    provider: Arc<dyn Provider>,
    /// The workspace.
    pub ws: Workspace,
    /// The permission rules.
    pub perms: Permissions,
    /// Settings.
    pub cfg: Config,
    /// The conversation.
    pub messages: Vec<Message>,
    /// Tokens and cost so far.
    pub meter: Meter,
    /// Model replies so far, with or without usage reported.
    pub replies: u32,
    log: Option<SessionLog>,
    warned: bool,
}

impl Agent {
    /// A new session.
    pub fn new(
        provider: Arc<dyn Provider>,
        ws: Workspace,
        perms: Permissions,
        cfg: Config,
    ) -> Self {
        Self {
            provider,
            ws,
            perms,
            cfg,
            messages: vec![],
            meter: Meter::default(),
            replies: 0,
            log: None,
            warned: false,
        }
    }

    /// Record every event to this session file.
    pub fn with_log(mut self, log: SessionLog) -> Self {
        self.log = Some(log);
        self
    }

    fn emit(&mut self, tx: &mpsc::Sender<AgentEvent>, e: AgentEvent) {
        if let Some(log) = &mut self.log {
            log.write(&e);
        }
        let _ = tx.try_send(e);
    }

    fn over_budget(&self) -> bool {
        matches!((self.cfg.budget_usd, self.meter.cost_usd), (Some(b), Some(c)) if c >= b)
    }

    /// Run `prompt` to an [`Outcome`].
    pub async fn run(
        &mut self,
        prompt: &str,
        tx: &mpsc::Sender<AgentEvent>,
        approver: &dyn Approver,
        cancel: &Cancel,
    ) -> Outcome {
        self.emit(
            tx,
            AgentEvent::Prompt {
                text: prompt.into(),
            },
        );
        self.messages.push(Message {
            role: Role::User,
            content: vec![Block::Text {
                text: prompt.into(),
            }],
        });
        let mut recent: VecDeque<String> = VecDeque::new();
        let mut reads: HashSet<String> = HashSet::new();
        let mut idle = 0;
        for _ in 0..self.cfg.max_turns {
            if self.over_budget() {
                return Outcome::Budget;
            }
            let req = Request {
                model: self.cfg.model.clone(),
                system: prompt::system_prompt(&self.ws, self.cfg.text_protocol),
                messages: self.messages.clone(),
                tools: if self.cfg.text_protocol {
                    vec![]
                } else {
                    tool_specs()
                },
                max_tokens: self.cfg.max_tokens,
            };
            let reply = tokio::select! {
                biased;
                _ = cancel.cancelled() => return Outcome::Interrupted,
                r = self.stream(req, tx) => r,
            };
            let (text, calls, stop) = match reply {
                Ok(r) => r,
                Err(ProviderError::ContextOverflow) => {
                    return Outcome::Error("the context window is full; start a new session (compaction arrives in A0.2)".into());
                }
                Err(e) => return Outcome::Error(e.to_string()),
            };
            let mut content = Vec::new();
            if !text.is_empty() {
                content.push(Block::Text { text: text.clone() });
            }
            for (id, name, args, _) in &calls {
                content.push(Block::ToolUse {
                    id: id.clone(),
                    name: name.clone(),
                    input: args.clone(),
                });
            }
            self.messages.push(Message {
                role: Role::Assistant,
                content,
            });

            let calls = if self.cfg.text_protocol {
                match crate::textproto::command(&text, self.ws.shell.name()) {
                    Some(cmd) => vec![(
                        "text".to_owned(),
                        "Bash".to_owned(),
                        json!({ "command": cmd }),
                        None,
                    )],
                    None => return Outcome::Done,
                }
            } else {
                calls
            };
            if calls.is_empty() {
                return match stop {
                    StopReason::Refusal => Outcome::Refused,
                    _ => Outcome::Done,
                };
            }
            let results = tokio::select! {
                biased;
                _ = cancel.cancelled() => return Outcome::Interrupted,
                r = self.run_calls(&calls, tx, approver) => r,
            };
            let progress = calls.iter().zip(&results).any(|((_, name, args, _), out)| {
                let new_read = name == "Read" && reads.insert(args["file_path"].to_string());
                (!out.is_error && !READ_ONLY.contains(&name.as_str())) || new_read
            });
            self.messages.push(Message {
                role: Role::User,
                content: if self.cfg.text_protocol {
                    vec![Block::Text {
                        text: format!("Output:\n{}", results[0].content),
                    }]
                } else {
                    calls
                        .iter()
                        .zip(results)
                        .map(|((id, ..), out)| Block::ToolResult {
                            id: id.clone(),
                            content: out.content,
                            is_error: out.is_error,
                        })
                        .collect()
                },
            });

            // Loop detector: the same call again and again, or no progress for a while.
            for (_, name, args, _) in &calls {
                recent.push_back(format!("{name}{args}"));
                if recent.len() > SAME_CALL_LIMIT {
                    recent.pop_front();
                }
            }
            idle = if progress { 0 } else { idle + 1 };
            let repeating =
                recent.len() == SAME_CALL_LIMIT && recent.iter().all(|c| c == &recent[0]);
            if repeating || idle >= IDLE_LIMIT {
                let why = if repeating {
                    "it made the same call 3 times in a row"
                } else {
                    "10 replies changed nothing new"
                };
                self.emit(
                    tx,
                    AgentEvent::Notice {
                        text: format!("The agent may be stuck: {why}."),
                    },
                );
                if !approver.approve("Loop", &json!({}), why).await {
                    return Outcome::Looping;
                }
                recent.clear();
                idle = 0;
            }
        }
        Outcome::MaxTurns
    }

    /// One reply: stream it, forward events, account usage.
    async fn stream(
        &mut self,
        req: Request,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> Result<(String, Vec<Call>, StopReason), ProviderError> {
        let mut rx = self.provider.stream(req).await?;
        self.replies += 1;
        let (mut text, mut calls, mut stop) = (String::new(), Vec::new(), StopReason::EndTurn);
        while let Some(ev) = rx.recv().await {
            match ev? {
                Event::TextDelta { text: t } => {
                    text.push_str(&t);
                    self.emit(tx, AgentEvent::Text { text: t });
                }
                Event::ThinkingDelta { text: t } => self.emit(tx, AgentEvent::Thinking { text: t }),
                Event::ToolCall {
                    id,
                    name,
                    args,
                    args_error,
                } => calls.push((id, name, args, args_error)),
                Event::Usage(u) => {
                    let computed = self.cfg.prices.cost(&self.cfg.model, &u);
                    self.meter.add(&u, computed);
                    let total = self.meter.dollars();
                    self.emit(
                        tx,
                        AgentEvent::Usage {
                            usage: u,
                            cost_usd: u.cost_usd.or(computed),
                            total,
                        },
                    );
                    if let (Some(b), Some(c), false) =
                        (self.cfg.budget_usd, self.meter.cost_usd, self.warned)
                        && c >= 0.8 * b
                    {
                        self.warned = true;
                        let text = format!("{} of the ${b:.2} budget used", self.meter.dollars());
                        self.emit(tx, AgentEvent::Notice { text });
                    }
                }
                Event::Stop { reason } => {
                    stop = reason;
                    break;
                }
            }
        }
        Ok((text, calls, stop))
    }

    /// Run a reply's tool calls: side by side when all are read-only, else in order.
    async fn run_calls(
        &mut self,
        calls: &[Call],
        tx: &mpsc::Sender<AgentEvent>,
        approver: &dyn Approver,
    ) -> Vec<ToolOutput> {
        let mut decided = Vec::new();
        for (id, name, args, args_error) in calls {
            let verdict = match args_error {
                Some(e) => Err(ToolOutput::err(e.clone())),
                None => match self.perms.check(&self.ws, name, args) {
                    Decision::Allow => Ok(()),
                    Decision::Ask(why) if approver.approve(name, args, &why).await => Ok(()),
                    Decision::Ask(why) => Err(ToolOutput::err(format!(
                        "denied: {why}, and no one approved it"
                    ))),
                    Decision::Deny(why) => Err(ToolOutput::err(format!("denied: {why}"))),
                },
            };
            if verdict.is_ok() {
                self.emit(
                    tx,
                    AgentEvent::ToolStart {
                        id: id.clone(),
                        name: name.clone(),
                        args: args.clone(),
                    },
                );
            }
            decided.push(verdict);
        }
        let parallel = calls
            .iter()
            .all(|(_, n, ..)| READ_ONLY.contains(&n.as_str()));
        let ws = &self.ws;
        let run = |i: usize| {
            let (_, name, args, _) = &calls[i];
            async move { run_tool(ws, name, args).await }
        };
        let mut outputs: Vec<ToolOutput> = Vec::with_capacity(calls.len());
        if parallel {
            let futs = (0..calls.len()).map(|i| {
                let skip = decided[i].clone().err();
                let fut = run(i);
                async move {
                    match skip {
                        Some(out) => out,
                        None => fut.await,
                    }
                }
            });
            outputs = futures::future::join_all(futs).await;
        } else {
            for (i, verdict) in decided.iter().enumerate() {
                outputs.push(match verdict {
                    Err(out) => out.clone(),
                    Ok(()) => run(i).await,
                });
            }
        }
        for (i, out) in outputs.iter().enumerate() {
            let (id, name, ..) = &calls[i];
            let denied = decided[i].is_err();
            self.emit(
                tx,
                AgentEvent::ToolEnd {
                    id: id.clone(),
                    name: name.clone(),
                    content: out.content.clone(),
                    is_error: out.is_error,
                    denied,
                },
            );
        }
        outputs
    }
}

/// A tool call: id, name, args, and why the args did not parse.
type Call = (String, String, Value, Option<String>);
