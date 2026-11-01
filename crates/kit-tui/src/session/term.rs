//! The session view's terminal driver (PRD §7.1, decision D3).
//!
//! Scrollback-native: finished lines go above an inline viewport with
//! `insert_before` and are never redrawn; only the six-row live region below
//! them repaints, inside synchronized-output codes so it lands as one frame.
//! It redraws on input and, while the agent works, on a spinner tick; an idle
//! session wakes for nothing.

use super::state::{Answer, Approval, Cmd, Out, Session};
use super::view::{LIVE_HEIGHT, out_lines, render_live};
use crate::event::motion_enabled;
use crate::theme::Theme;
use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, Event, EventStream, KeyEventKind,
};
use crossterm::terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate};
use crossterm::{execute, terminal};
use futures::StreamExt;
use kit_loop::{AgentEvent, Outcome};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Paragraph, Widget};
use ratatui::{Terminal, TerminalOptions, Viewport};
use std::io::{Stdout, Write};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};

/// What the agent side tells the view, besides its events.
#[derive(Debug)]
pub enum Control {
    /// A boundary prompt; the answer goes back on `reply`.
    Ask {
        approval: Approval,
        reply: oneshot::Sender<Answer>,
    },
    /// `agent.run` returned.
    TurnEnded(Outcome),
}

/// Restores the terminal however the session ends.
struct Raw;

impl Raw {
    fn enter() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        let _ = execute!(std::io::stdout(), EnableBracketedPaste);
        Ok(Self)
    }
}

impl Drop for Raw {
    fn drop(&mut self) {
        let _ = execute!(std::io::stdout(), DisableBracketedPaste);
        let _ = terminal::disable_raw_mode();
    }
}

/// Run the session view until the user quits. `intro` is printed first;
/// `on_cmd` receives sends and interrupts (approvals are answered here).
pub async fn run(
    mut session: Session,
    intro: Vec<Out>,
    mut events: mpsc::Receiver<AgentEvent>,
    mut control: mpsc::Receiver<Control>,
    mut on_cmd: impl FnMut(Cmd),
) -> anyhow::Result<()> {
    let theme = Theme::resolve();
    let motion = motion_enabled();
    let _raw = Raw::enter()?;
    let mut term = Terminal::with_options(
        CrosstermBackend::new(std::io::stdout()),
        TerminalOptions {
            viewport: Viewport::Inline(LIVE_HEIGHT),
        },
    )?;
    for out in intro {
        session.push(out);
    }
    let mut keys = EventStream::new();
    let mut reply: Option<oneshot::Sender<Answer>> = None;
    let mut tick = tokio::time::interval(Duration::from_millis(125));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut events_open = true;
    paint(&mut term, &mut session, &theme, motion)?;
    loop {
        let animate = motion && session.busy.is_some();
        tokio::select! {
            biased;
            key = keys.next() => match key {
                Some(Ok(Event::Key(k))) if k.kind != KeyEventKind::Release => {
                    match session.on_key(k, Instant::now()) {
                        Some(Cmd::Quit) => break,
                        Some(Cmd::Answer(a)) => {
                            if let Some(r) = reply.take() {
                                let _ = r.send(a);
                            }
                        }
                        Some(cmd) => on_cmd(cmd),
                        None => {}
                    }
                }
                Some(Ok(Event::Paste(text))) => session.on_paste(&text),
                Some(Ok(Event::Resize(..))) => {
                    term.autoresize()?;
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err(e.into()),
                None => break,
            },
            e = events.recv(), if events_open => match e {
                Some(e) => session.on_agent(e, Instant::now()),
                None => events_open = false,
            },
            c = control.recv() => match c {
                Some(Control::Ask { approval, reply: r }) => {
                    session.ask(approval, Instant::now());
                    reply = Some(r);
                }
                Some(Control::TurnEnded(outcome)) => {
                    // Every event of the turn was queued before `run` returned.
                    while let Ok(e) = events.try_recv() {
                        session.on_agent(e, Instant::now());
                    }
                    reply = None;
                    session.turn_ended(&outcome);
                }
                None => break,
            },
            _ = tick.tick(), if animate => {}
        }
        paint(&mut term, &mut session, &theme, motion)?;
    }
    // Leave the last frame's composer behind as plain lines, then a newline.
    term.clear()?;
    let mut out = std::io::stdout();
    writeln!(out)?;
    out.flush()?;
    Ok(())
}

fn paint(
    term: &mut Terminal<CrosstermBackend<Stdout>>,
    session: &mut Session,
    theme: &Theme,
    motion: bool,
) -> anyhow::Result<()> {
    let width = term.size()?.width;
    let _ = execute!(std::io::stdout(), BeginSynchronizedUpdate);
    for out in session.drain_out() {
        let lines = out_lines(&out, theme, width);
        let height = lines.len() as u16;
        term.insert_before(height, |buf| {
            Paragraph::new(lines).render(buf.area, buf);
        })?;
    }
    let now = Instant::now();
    term.draw(|f| render_live(session, theme, f.area(), f.buffer_mut(), now, motion))?;
    let _ = execute!(std::io::stdout(), EndSynchronizedUpdate);
    Ok(())
}
