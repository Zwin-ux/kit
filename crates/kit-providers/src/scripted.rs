//! A provider that replays a script of replies. It drives the real loop in
//! tests and in `kit eval --scripted`, with no model and no network.

use crate::{Event, EventStream, ModelInfo, Provider, ProviderError, Request};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::VecDeque;
use std::sync::Mutex;
use tokio::sync::mpsc;

/// One scripted reply: the events to send, or an error to fail with.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Turn {
    /// Send these events.
    Events(Vec<Event>),
    /// Fail the call with this message (as `Transient`).
    Error {
        /// The message.
        error: String,
    },
}

/// Replays turns in order and records every request it was sent.
#[derive(Debug, Default)]
pub struct ScriptedProvider {
    turns: Mutex<VecDeque<Turn>>,
    requests: Mutex<Vec<Request>>,
}

impl ScriptedProvider {
    /// Replies to give, first to last.
    pub fn new(turns: Vec<Turn>) -> Self {
        Self {
            turns: Mutex::new(turns.into()),
            requests: Mutex::default(),
        }
    }

    /// A script from JSON: an array whose items are arrays of events, or
    /// `{"error": "…"}`.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Self::new(serde_json::from_str(json)?))
    }

    /// Every request so far, in order.
    pub fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }

    /// Turns not yet used.
    pub fn remaining(&self) -> usize {
        self.turns.lock().unwrap().len()
    }
}

#[async_trait]
impl Provider for ScriptedProvider {
    fn id(&self) -> &'static str {
        "scripted"
    }

    async fn models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        Ok(vec![ModelInfo {
            id: "scripted".into(),
            context_window: Some(200_000),
        }])
    }

    async fn stream(&self, req: Request) -> Result<EventStream, ProviderError> {
        self.requests.lock().unwrap().push(req);
        let turn = self
            .turns
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| ProviderError::Fatal("the script has no more replies".into()))?;
        let events = match turn {
            Turn::Events(e) => e,
            Turn::Error { error } => return Err(ProviderError::Transient(error)),
        };
        let (tx, rx) = mpsc::channel(events.len().max(1));
        for e in events {
            // The channel holds every event, so this never waits.
            let _ = tx.try_send(Ok(e));
        }
        Ok(rx)
    }
}
