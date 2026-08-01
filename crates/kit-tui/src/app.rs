//! Application state and pure reducer.
//!
//! All motion derives from [`Clock`], advanced only by `AppEvent::AnimationTick`.
//! The reducer is synchronous and free of I/O so it can be driven headlessly
//! in tests without a terminal.

use crate::event::{AppEvent, Clock, motion_enabled};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use kit_core::{GateOutcome, RunDelta, RunId, RunState};

/// Side effects the event loop must perform after a pure state transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// No external work; continue the loop.
    None,
    /// Restore the terminal and exit.
    Quit,
}

/// Kit Control Room application state.
#[derive(Debug, Clone)]
pub struct App {
    /// The single frame clock. Advanced only by `AnimationTick`.
    pub clock: Clock,
    /// When true the loop restores the terminal and returns.
    pub should_quit: bool,
    /// Frame needs to be painted. Cleared after a successful draw.
    dirty: bool,
    /// Last known terminal size (`Resize` events).
    pub size: (u16, u16),
    /// Selected row in the run table (placeholder selection for M0).
    pub selected: usize,
    /// Snapshot of motion preference at construction (tests can override).
    motion: bool,
    /// Banner shown when a background operation fails.
    pub error: Option<String>,
    /// Placeholder run list — real rows land in a later milestone.
    /// Kept so `RunUpdate` / `GateResult` have a place to land and be tested.
    pub runs: Vec<RunRow>,
}

/// Minimal row shown in the Control Room table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRow {
    pub id: RunId,
    pub repo: String,
    pub agent: String,
    pub task: String,
    pub state: RunState,
    pub gate: Option<GateOutcome>,
    /// Elapsed label, e.g. `"2m"`. Empty until a later milestone fills it.
    pub elapsed: String,
}

impl App {
    /// Build a fresh Control Room with motion preference taken from the environment.
    pub fn new() -> Self {
        Self::with_motion(motion_enabled())
    }

    /// Build with an explicit motion flag — used by tests and headless drivers.
    pub fn with_motion(motion: bool) -> Self {
        Self {
            clock: Clock::default(),
            should_quit: false,
            dirty: true, // first frame always paints
            size: (80, 24),
            selected: 0,
            motion,
            error: None,
            runs: Vec::new(),
        }
    }

    /// Whether the next draw should run.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear the dirty flag after a successful draw.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Force a redraw (e.g. after terminal setup).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Motion preference currently in effect for this session.
    pub fn motion_enabled(&self) -> bool {
        self.motion
    }

    /// Whether anything on the current frame would change with the clock.
    ///
    /// The empty Control Room placeholder has no animated widgets, so an idle
    /// Kit never redraws on tick alone — the M0 idle-CPU kill criterion.
    pub fn animated_on_screen(&self) -> bool {
        // Later milestones: mascot, blink cursors, live elapsed counters.
        false
    }

    /// Pure, synchronous reducer. Advances state and returns any loop action.
    ///
    /// Redraw policy lives here so headless tests can assert dirty flags
    /// without spinning a terminal:
    /// - events with `is_redraw_worthy()` mark the frame dirty
    /// - `AnimationTick` marks dirty only when motion is on *and* something
    ///   animated is actually drawn
    pub fn update(&mut self, event: AppEvent) -> Action {
        let marks_dirty = match &event {
            AppEvent::AnimationTick => self.motion && self.animated_on_screen(),
            other => other.is_redraw_worthy(),
        };
        let action = self.apply(event);
        if marks_dirty {
            self.dirty = true;
        }
        action
    }

    fn apply(&mut self, event: AppEvent) -> Action {
        match event {
            AppEvent::Key(key) => self.on_key(key),
            AppEvent::Mouse(_) => Action::None,
            AppEvent::Resize(w, h) => {
                self.size = (w, h);
                Action::None
            }
            AppEvent::AnimationTick => {
                self.clock.advance();
                Action::None
            }
            AppEvent::RunUpdate(id, delta) => {
                self.apply_run_update(id, delta);
                Action::None
            }
            AppEvent::GateResult(id, outcome) => {
                if let Some(row) = self.runs.iter_mut().find(|r| r.id == id) {
                    row.gate = Some(outcome.clone());
                    row.state = if outcome.passed {
                        RunState::Pass
                    } else {
                        RunState::Fail
                    };
                }
                Action::None
            }
            AppEvent::Error(msg) => {
                self.error = Some(msg);
                Action::None
            }
            AppEvent::Quit => {
                self.should_quit = true;
                Action::Quit
            }
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> Action {
        // Windows emits Press + Release; only act on Press.
        if key.kind == KeyEventKind::Release {
            return Action::None;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
                Action::Quit
            }
            KeyCode::Up if !self.runs.is_empty() => {
                self.selected = self.selected.saturating_sub(1);
                Action::None
            }
            KeyCode::Down if !self.runs.is_empty() => {
                let max = self.runs.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
                Action::None
            }
            // Footer bindings (d/enter/g/k/r) land as real commands later.
            _ => Action::None,
        }
    }

    fn apply_run_update(&mut self, id: RunId, delta: RunDelta) {
        let row = if let Some(row) = self.runs.iter_mut().find(|r| r.id == id) {
            row
        } else {
            self.runs.push(RunRow {
                id: id.clone(),
                repo: String::new(),
                agent: String::new(),
                task: String::new(),
                state: RunState::Queued,
                gate: None,
                elapsed: String::new(),
            });
            self.runs.last_mut().expect("just pushed")
        };

        match delta {
            RunDelta::State(state) => row.state = state,
            RunDelta::Output(_) => {
                // Streamed into run detail later; Control Room ignores chunks.
            }
            RunDelta::Worktree(_) => {}
            RunDelta::Gate(outcome) => {
                row.gate = Some(outcome.clone());
                row.state = if outcome.passed {
                    RunState::Pass
                } else {
                    RunState::Fail
                };
            }
        }
    }

    /// Counts used by the Control Room header.
    pub fn running_count(&self) -> usize {
        self.runs
            .iter()
            .filter(|r| matches!(r.state, RunState::Running))
            .count()
    }

    pub fn gated_count(&self) -> usize {
        self.runs
            .iter()
            .filter(|r| matches!(r.state, RunState::Gating))
            .count()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use kit_core::{RunDelta, RunId, RunState};

    fn key(c: char) -> AppEvent {
        AppEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE))
    }

    #[test]
    fn animation_tick_advances_clock_but_not_dirty_when_motion_off() {
        let mut app = App::with_motion(false);
        app.clear_dirty();
        assert!(!app.is_dirty());
        assert_eq!(app.clock.tick, 0);

        let action = app.update(AppEvent::AnimationTick);
        assert_eq!(action, Action::None);
        assert_eq!(app.clock.tick, 1);
        assert!(
            !app.is_dirty(),
            "AnimationTick must not dirty the frame when motion is disabled"
        );
    }

    #[test]
    fn animation_tick_does_not_dirty_idle_room_even_with_motion() {
        let mut app = App::with_motion(true);
        app.clear_dirty();
        app.update(AppEvent::AnimationTick);
        assert_eq!(app.clock.tick, 1);
        assert!(
            !app.is_dirty(),
            "empty Control Room has nothing animated; idle CPU stays near zero"
        );
    }

    #[test]
    fn scripted_reducer_sequence_is_deterministic() {
        let mut app = App::with_motion(false);
        app.clear_dirty();

        assert_eq!(app.update(AppEvent::Resize(120, 40)), Action::None);
        assert_eq!(app.size, (120, 40));
        assert!(app.is_dirty());
        app.clear_dirty();

        let id = RunId("01TESTRUN00000000000000000".into());
        assert_eq!(
            app.update(AppEvent::RunUpdate(
                id.clone(),
                RunDelta::State(RunState::Running)
            )),
            Action::None
        );
        assert_eq!(app.runs.len(), 1);
        assert_eq!(app.runs[0].state, RunState::Running);
        assert_eq!(app.running_count(), 1);
        assert!(app.is_dirty());
        app.clear_dirty();

        for _ in 0..5 {
            app.update(AppEvent::AnimationTick);
        }
        assert_eq!(app.clock.tick, 5);
        assert!(!app.is_dirty());

        assert_eq!(
            app.update(AppEvent::Error("probe failed".into())),
            Action::None
        );
        assert_eq!(app.error.as_deref(), Some("probe failed"));
        assert!(app.is_dirty());
        app.clear_dirty();

        assert_eq!(app.update(key('q')), Action::Quit);
        assert!(app.should_quit);
        assert!(app.is_dirty());
    }

    #[test]
    fn quit_event_requests_exit() {
        let mut app = App::with_motion(false);
        assert_eq!(app.update(AppEvent::Quit), Action::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn redraw_worthy_events_mark_dirty() {
        let mut app = App::with_motion(false);
        app.clear_dirty();
        app.update(AppEvent::Error("x".into()));
        assert!(app.is_dirty());
    }
}
