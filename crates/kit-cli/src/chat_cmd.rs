//! `kit chat`: Kit's own agent in the session view (PRD §7.1). The agent runs
//! on its own task; the view owns the terminal. Boundary prompts are asked
//! inline and answered with `y`, `a` or `n`.

use crate::agent_cmd::{Built, build};
use crate::cli::AgentOpts;
use anyhow::Result;
use async_trait::async_trait;
use kit_loop::{Approver, Cancel};
use kit_tui::session::{self, Answer, Approval, Cmd, Control, Out, Session, Status};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

/// Asks the view, and remembers "always" answers for the rest of the session.
struct AskTheView {
    control: mpsc::Sender<Control>,
    always: Mutex<HashSet<String>>,
}

#[async_trait]
impl Approver for AskTheView {
    async fn approve(&self, tool: &str, args: &Value, reason: &str) -> bool {
        let target = session::state::target(args);
        let key = format!("{tool}\u{0}{target}");
        if self.always.lock().is_ok_and(|a| a.contains(&key)) {
            return true;
        }
        let (reply, answer) = oneshot::channel();
        let approval = Approval {
            tool: tool.into(),
            target,
            reason: reason.into(),
        };
        if self
            .control
            .send(Control::Ask { approval, reply })
            .await
            .is_err()
        {
            return false;
        }
        match answer.await {
            Ok(Answer::Once) => true,
            Ok(Answer::Always) => {
                if let Ok(mut a) = self.always.lock() {
                    a.insert(key);
                }
                true
            }
            Ok(Answer::Deny) | Err(_) => false,
        }
    }
}

fn branch(root: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    let name = String::from_utf8(out.stdout).ok()?.trim().to_owned();
    (out.status.success() && !name.is_empty() && name != "HEAD").then_some(name)
}

pub async fn cmd_chat(opts: &AgentOpts) -> Result<()> {
    let Built {
        mut agent,
        model,
        provider_id,
        profile,
        session,
    } = build(opts)?;
    let profile = format!("{profile:?}").to_ascii_lowercase();
    let status = Status {
        model: model.clone(),
        profile: profile.clone(),
        branch: branch(&agent.ws.root),
        ..Status::default()
    };
    let intro = vec![
        Out::Dim(format!(
            "kit · {model} via {provider_id} · {profile} profile, unsandboxed"
        )),
        Out::Dim(format!("session {}", crate::kits::plan::tilde(&session))),
        Out::Dim("Enter send · Esc interrupt · Ctrl-C twice quit · ? help".into()),
        Out::Blank,
    ];

    // The loop drops events when its channel is full, so give the view room.
    let (events_tx, events_rx) = mpsc::channel(4096);
    let (control_tx, control_rx) = mpsc::channel(16);
    let (prompt_tx, mut prompt_rx) = mpsc::channel::<String>(4);
    let current = Arc::new(Mutex::new(Cancel::default()));
    let approver = AskTheView {
        control: control_tx.clone(),
        always: Mutex::new(HashSet::new()),
    };

    let turns = current.clone();
    let worker = tokio::spawn(async move {
        while let Some(prompt) = prompt_rx.recv().await {
            // A fresh interrupt flag per turn: the last one stays tripped.
            let cancel = Cancel::default();
            if let Ok(mut c) = turns.lock() {
                *c = cancel.clone();
            }
            let outcome = agent.run(&prompt, &events_tx, &approver, &cancel).await;
            if control_tx.send(Control::TurnEnded(outcome)).await.is_err() {
                break;
            }
        }
    });

    let result = session::run(
        Session::new(status),
        intro,
        events_rx,
        control_rx,
        |cmd| match cmd {
            Cmd::Send(text) => {
                let _ = prompt_tx.try_send(text);
            }
            Cmd::Interrupt => {
                if let Ok(c) = current.lock() {
                    c.cancel();
                }
            }
            Cmd::Answer(_) | Cmd::Quit => {}
        },
    )
    .await;
    if let Ok(c) = current.lock() {
        c.cancel();
    }
    worker.abort();
    eprintln!(
        "kit: session saved to {}",
        crate::kits::plan::tilde(&session)
    );
    result
}
