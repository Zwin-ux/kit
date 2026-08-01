//! The single event loop that drives the Control Room.
//!
//! Exactly three sources merge into [`AppEvent`]:
//! 1. `crossterm::event::EventStream` — keyboard, mouse, resize
//! 2. one `tokio::time::interval(TICK_INTERVAL)` — the frame clock
//! 3. an `mpsc::Receiver<(RunId, RunDelta)>` — run progress
//!
//! No other timer exists in the application. Animation asks [`Clock`] for phase.

use crate::app::{Action, App};
use crate::event::{AppEvent, TICK_INTERVAL};
use crate::ui;
use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::StreamExt;
use kit_core::{RunDelta, RunId};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};
use std::panic;
use tokio::sync::mpsc;
use tokio::time::{MissedTickBehavior, interval};

type Term = Terminal<CrosstermBackend<Stdout>>;

/// Run the Control Room until quit. Restores the terminal on every exit path.
pub async fn run(run_rx: mpsc::Receiver<(RunId, RunDelta)>) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_with_terminal(&mut terminal, run_rx).await;
    restore_terminal(&mut terminal)?;
    result
}

/// Event loop body, factored so tests can inject a backend later if needed.
async fn run_with_terminal(
    terminal: &mut Term,
    mut run_rx: mpsc::Receiver<(RunId, RunDelta)>,
) -> Result<()> {
    let mut app = App::new();
    let mut term_events = EventStream::new();
    let mut tick = interval(TICK_INTERVAL);
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    // First tick completes immediately; skip it so we do not double-advance on start.
    tick.tick().await;

    // Initial paint.
    terminal.draw(|f| ui::draw(f, &app))?;
    app.clear_dirty();

    loop {
        let event = tokio::select! {
            maybe = term_events.next() => {
                match maybe {
                    Some(Ok(ev)) => map_crossterm(ev),
                    Some(Err(err)) => Some(AppEvent::Error(err.to_string())),
                    None => Some(AppEvent::Quit),
                }
            }
            _ = tick.tick() => Some(AppEvent::AnimationTick),
            // Channel closed → None: keep the UI alive; user can still quit.
            item = run_rx.recv() => {
                item.map(|(id, delta)| AppEvent::RunUpdate(id, delta))
            }
        };

        let Some(event) = event else {
            continue;
        };

        let action = app.update(event);

        if app.is_dirty() {
            terminal.draw(|f| ui::draw(f, &app))?;
            app.clear_dirty();
        }

        if matches!(action, Action::Quit) || app.should_quit {
            break;
        }
    }

    Ok(())
}

fn map_crossterm(ev: Event) -> Option<AppEvent> {
    match ev {
        Event::Key(key) => {
            // Windows emits Press + Release; only act on Press (and Repeat for hold-nav).
            if key.kind == KeyEventKind::Release {
                return None;
            }
            Some(AppEvent::Key(key))
        }
        Event::Mouse(mouse) => Some(AppEvent::Mouse(mouse)),
        Event::Resize(w, h) => Some(AppEvent::Resize(w, h)),
        Event::FocusGained | Event::FocusLost | Event::Paste(_) => None,
    }
}

fn setup_terminal() -> Result<Term> {
    install_panic_hook();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Term) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// A panic must never leave the user's shell in raw mode or on the alt screen.
fn install_panic_hook() {
    let original = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        original(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::AppEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    #[test]
    fn map_key_press_only() {
        let press = Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert!(matches!(map_crossterm(press), Some(AppEvent::Key(_))));

        let release = Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert!(map_crossterm(release).is_none());
    }

    #[test]
    fn map_resize() {
        assert!(matches!(
            map_crossterm(Event::Resize(100, 40)),
            Some(AppEvent::Resize(100, 40))
        ));
    }
}
