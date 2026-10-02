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

/// How to start the Control Room.
#[derive(Debug, Default)]
pub struct LaunchConfig {
    /// When true, seed the PRD §4.2 fixture runs so the product is dogfoodable
    /// before the M1 engine exists.
    pub demo: bool,
    /// When set, dispatch/kill/retry actions are forwarded here for the engine.
    pub engine_tx: Option<mpsc::Sender<crate::app::EngineCommand>>,
    /// When true (default for interactive launch), probe coding agents for the header strip.
    pub probe_agents: bool,
    /// Where the engine writes receipts. The Diff pane reads finished runs
    /// from it, and past runs are listed from it at launch (not in `demo`).
    pub runs_dir: Option<std::path::PathBuf>,
}

/// Run the Control Room until quit. Restores the terminal on every exit path.
pub async fn run(run_rx: mpsc::Receiver<(RunId, RunDelta)>) -> Result<()> {
    run_configured(LaunchConfig::default(), run_rx).await
}

/// Run with explicit launch options (demo fixture, engine channel).
pub async fn run_configured(
    config: LaunchConfig,
    run_rx: mpsc::Receiver<(RunId, RunDelta)>,
) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    if config.probe_agents {
        app.set_agent_statuses(kit_agents::probe_all().await);
    }
    app.runs_dir = config.runs_dir;
    if config.demo {
        app.load_prd_fixture();
    } else if let Some(dir) = app.runs_dir.clone() {
        // Every run, not only this session's: finished runs from receipts.
        app.load_past_runs(&dir);
    }
    let result = run_with_terminal(&mut terminal, app, run_rx, config.engine_tx).await;
    restore_terminal(&mut terminal)?;
    if let Some(message) = result? {
        println!("{message}");
    }
    Ok(())
}

/// Event loop body, factored so tests can inject a backend later if needed.
async fn run_with_terminal(
    terminal: &mut Term,
    mut app: App,
    mut run_rx: mpsc::Receiver<(RunId, RunDelta)>,
    engine_tx: Option<mpsc::Sender<crate::app::EngineCommand>>,
) -> Result<Option<String>> {
    let mut term_events = Some(EventStream::new());
    let (probe_tx, mut probe_rx) = mpsc::channel(1);
    let mut probe_task: Option<tokio::task::JoinHandle<()>> = None;
    let mut land_task: Option<LandTask> = None;
    let mut tick = interval(TICK_INTERVAL);
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    // First tick completes immediately; skip it so we do not double-advance on start.
    tick.tick().await;

    // Initial paint.
    terminal.draw(|f| ui::draw(f, &app))?;
    app.clear_dirty();

    // A closed `mpsc::Receiver` yields `None` immediately and forever, so the
    // branch must be disabled once it ends. Without this guard `select!` picks
    // the always-ready arm every iteration and the loop spins at 100% CPU —
    // which would defeat the M0 idle-CPU criterion outright.
    let mut runs_open = true;

    loop {
        let event = tokio::select! {
            result = async { (&mut land_task.as_mut().expect("land task present").1).await }, if land_task.is_some() => {
                let (id,_)=land_task.take().expect("completed land task");
                app.complete_land(id,result.unwrap_or_else(|e|Err(format!("Kit land task failed: {e}"))));
                terminal.draw(|f|ui::draw(f,&app))?;app.clear_dirty();continue;
            }
            probe = probe_rx.recv(), if probe_task.is_some() => {
                if let Some(probe)=probe { app.set_agent_statuses(probe); }
                probe_task=None;
                terminal.draw(|f| ui::draw(f, &app))?;
                app.clear_dirty();
                continue;
            }
            maybe = term_events.as_mut().expect("event reader restored").next() => {
                match maybe {
                    Some(Ok(ev)) => map_crossterm(ev),
                    Some(Err(err)) => Some(AppEvent::Error(err.to_string())),
                    None => Some(AppEvent::Quit),
                }
            }
            _ = tick.tick() => Some(AppEvent::AnimationTick),
            // Channel closed → keep the UI alive, but stop polling this arm.
            item = run_rx.recv(), if runs_open => {
                let (event, still_open) = map_run_item(item);
                runs_open = still_open;
                event
            }
        };

        let Some(event) = event else {
            continue;
        };

        let action = app.update(event);

        // Engine seams — forward start/kill/retry when a channel is wired (kit-cli).
        match &action {
            Action::RefreshAgents => {
                if probe_task.is_none() {
                    let tx = probe_tx.clone();
                    probe_task = Some(tokio::spawn(async move {
                        let _ = tx.send(kit_agents::probe_all().await).await;
                    }));
                    app.set_flash("checking provider sign-in…");
                }
            }
            Action::ConnectAgent { kind } => {
                if app.active_run_ids().is_empty() {
                    stop_probe(&mut probe_task, &mut probe_rx).await;
                    release_event_reader(&mut term_events);
                    restore_terminal(terminal)?;
                    let result = kit_agents::connection::connect(*kind).await;
                    *terminal = setup_terminal()?;
                    term_events = Some(EventStream::new());
                    app.set_agent_statuses(kit_agents::probe_all().await);
                    app.error = result.err().map(|e| e.to_string());
                    app.set_flash("native sign-in ended; status checked again");
                } else {
                    app.set_flash("finish or stop active runs before native sign-in");
                }
            }
            Action::LandSelected { id } => {
                if land_task.is_none() {
                    let id = id.clone();
                    if let Some(dir) = app.runs_dir.clone() {
                        let diff = app
                            .selected_run()
                            .filter(|r| r.id == id)
                            .map(|r| r.diff.clone())
                            .unwrap_or_default();
                        let worker_id = id.clone();
                        land_task = Some((
                            id,
                            tokio::spawn(async move {
                                crate::accept::land(&worker_id, &dir, &diff).await
                            }),
                        ));
                    } else {
                        app.complete_land(
                            id,
                            Err("No saved receipt directory is configured".into()),
                        );
                    }
                }
            }
            Action::DispatchSubmitted { jobs } => {
                if let Some(tx) = &engine_tx {
                    for job in jobs {
                        let _ = tx.send(crate::app::EngineCommand::Start(job.clone())).await;
                    }
                }
            }
            Action::KillSelected { id } => {
                if let Some(tx) = &engine_tx {
                    let _ = tx
                        .send(crate::app::EngineCommand::Kill { id: id.clone() })
                        .await;
                }
            }
            Action::RetrySelected { source_id, job } => {
                if let Some(tx) = &engine_tx {
                    let _ = tx
                        .send(crate::app::EngineCommand::Retry {
                            source_id: source_id.clone(),
                            job: job.clone(),
                        })
                        .await;
                }
            }
            Action::Quit | Action::None | Action::AttachSelected => {}
        }

        if app.is_dirty() {
            terminal.draw(|f| ui::draw(f, &app))?;
            app.clear_dirty();
        }

        if matches!(action, Action::Quit) || app.should_quit {
            let stopped = stop_active_runs(
                terminal,
                &mut app,
                &mut run_rx,
                runs_open,
                engine_tx.as_ref(),
            )
            .await;
            // An approved branch operation must finish and report before Kit exits.
            await_land_on_quit(&mut app, &mut land_task).await;
            stopped?;
            break;
        }
    }

    stop_probe(&mut probe_task, &mut probe_rx).await;
    Ok(app
        .land_result
        .map(|(id, message)| format!("{id}: {message}")))
}

type LandTask = (
    RunId,
    tokio::task::JoinHandle<Result<crate::accept::Accepted, String>>,
);
async fn await_land_on_quit(app: &mut App, task: &mut Option<LandTask>) {
    if let Some((id, task)) = task.take() {
        app.complete_land(
            id,
            task.await
                .unwrap_or_else(|e| Err(format!("Kit land task failed: {e}"))),
        );
    }
}

/// Join cancellation before draining: a cancelled sender may enqueue while dropping.
async fn stop_probe(
    task: &mut Option<tokio::task::JoinHandle<()>>,
    rx: &mut mpsc::Receiver<Vec<kit_agents::AgentStatus>>,
) {
    if let Some(task) = task.take() {
        task.abort();
        let _ = task.await;
    }
    while rx.try_recv().is_ok() {}
}

/// Stop reading stdin before handing the restored terminal to the provider.
fn release_event_reader<T>(reader: &mut Option<T>) {
    drop(reader.take());
}

/// How long quitting waits for stopped runs to write their receipts.
const STOP_GRACE: std::time::Duration = std::time::Duration::from_secs(15);

/// Quit never drops a run: each one still in flight is killed, and Kit waits
/// for its terminal state, which the engine sends only once the receipt is
/// written and the worktree cleaned up.
async fn stop_active_runs(
    terminal: &mut Term,
    app: &mut App,
    run_rx: &mut mpsc::Receiver<(RunId, RunDelta)>,
    runs_open: bool,
    engine_tx: Option<&mpsc::Sender<crate::app::EngineCommand>>,
) -> Result<()> {
    let active = app.active_run_ids();
    let Some(tx) = engine_tx.filter(|_| !active.is_empty() && runs_open) else {
        return Ok(());
    };
    for id in &active {
        let _ = tx
            .send(crate::app::EngineCommand::Kill { id: id.clone() })
            .await;
    }
    app.error = Some(format!(
        "stopping {} run(s) and writing their receipts…",
        active.len()
    ));
    terminal.draw(|f| ui::draw(f, app))?;
    let deadline = tokio::time::Instant::now() + STOP_GRACE;
    while !app.active_run_ids().is_empty() {
        match tokio::time::timeout_at(deadline, run_rx.recv()).await {
            Ok(Some((id, delta))) => {
                app.update(AppEvent::RunUpdate(id, delta));
            }
            // Engine gone or out of time: nothing more will arrive.
            Ok(None) | Err(_) => break,
        }
    }
    Ok(())
}

/// Map one run-channel receive into an event, reporting whether the channel is
/// still open. `None` means every sender has been dropped, and the caller must
/// stop polling that `select!` arm — see `runs_open` in [`run_with_terminal`].
fn map_run_item(item: Option<(RunId, RunDelta)>) -> (Option<AppEvent>, bool) {
    match item {
        Some((id, delta)) => (Some(AppEvent::RunUpdate(id, delta)), true),
        None => (None, false),
    }
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

    #[tokio::test]
    async fn quitting_waits_for_approved_acceptance_and_keeps_its_receipt() {
        let mut app = App::with_motion(false);
        let id = RunId::new();
        app.landing = Some(id.clone());
        let commit = "a".repeat(40);
        let expected = commit.clone();
        let mut task = Some((
            id.clone(),
            tokio::spawn(async move {
                tokio::task::yield_now().await;
                Ok(crate::accept::Accepted {
                    branch: "kit/review".into(),
                    commit,
                })
            }),
        ));
        await_land_on_quit(&mut app, &mut task).await;
        assert!(task.is_none());
        assert!(app.landing.is_none());
        let (result_id, message) = app.land_result.unwrap();
        assert_eq!(result_id, id);
        assert!(message.contains("kit/review") && message.contains(&expected));
    }

    #[tokio::test]
    async fn probe_cancellation_finishes_before_old_statuses_are_drained() {
        struct LastSend(mpsc::Sender<Vec<kit_agents::AgentStatus>>);
        impl Drop for LastSend {
            fn drop(&mut self) {
                let _ = self.0.try_send(Vec::new());
            }
        }
        let (tx, mut rx) = mpsc::channel(1);
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let mut task = Some(tokio::spawn(async move {
            let _guard = LastSend(tx);
            let _ = ready_tx.send(());
            std::future::pending::<()>().await;
        }));
        ready_rx.await.unwrap();
        stop_probe(&mut task, &mut rx).await;
        assert!(task.is_none());
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn native_sign_in_releases_event_reader_before_provider_can_read_stdin() {
        struct Reader(std::rc::Rc<std::cell::Cell<bool>>);
        impl Drop for Reader {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }
        let released = std::rc::Rc::new(std::cell::Cell::new(false));
        let mut reader = Some(Reader(released.clone()));
        release_event_reader(&mut reader);
        // This is the point at which native provider execution resumes.
        assert!(released.get());
        assert!(reader.is_none());
    }

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

    #[test]
    fn a_live_run_item_keeps_the_arm_open() {
        let (event, still_open) = map_run_item(Some((
            RunId::new(),
            RunDelta::State(kit_core::RunState::Running),
        )));
        assert!(matches!(event, Some(AppEvent::RunUpdate(..))));
        assert!(still_open);
    }

    /// Regression: a closed run channel must disable its `select!` arm.
    #[test]
    fn a_closed_run_channel_closes_the_arm() {
        let (event, still_open) = map_run_item(None);
        assert!(event.is_none());
        assert!(!still_open, "closed channel must disable the select arm");
    }

    #[tokio::test]
    async fn recv_on_a_closed_channel_is_immediately_ready_forever() {
        let (tx, mut rx) = mpsc::channel::<(RunId, RunDelta)>(4);
        drop(tx);
        for _ in 0..2 {
            let got = rx.recv().await;
            assert!(got.is_none());
            assert!(!map_run_item(got).1);
        }
    }
}
