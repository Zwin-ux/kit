//! Application state and pure reducer.
//!
//! All motion derives from [`Clock`], advanced only by `AppEvent::AnimationTick`.
//! The reducer is synchronous and free of I/O so it can be driven headlessly
//! in tests without a terminal.
//!
//! Screen model follows fennec-tui: navigation mutates [`Screen`] inside the
//! reducer; engine work leaves as [`Action`] for the event loop to fulfill.

use crate::event::{AppEvent, Clock, TICK_HZ, motion_enabled};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use kit_core::{GateOutcome, RunDelta, RunId, RunState};
use std::path::PathBuf;

/// Local display cap for streamed output (Principle 1: bounded by default).
/// Engine may already cap; the TUI never holds more than this for rendering.
pub const OUTPUT_DISPLAY_CAP_BYTES: usize = 512 * 1024;

/// Flash banner lifetime in ticks (~2 seconds at [`TICK_HZ`]).
const FLASH_TICKS: u64 = TICK_HZ * 2;

/// Maximum repo×agent combinations a single dispatch may create (UI guard).
pub const DISPATCH_FANOUT_CAP: usize = 16;

/// A run the engine should start (UI already inserted a Queued row).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchJob {
    pub id: RunId,
    pub repo: String,
    pub agent: String,
    pub task: String,
}

/// Commands the Control Room event loop forwards to the kit-cli engine supervisor.
///
/// Replaces a bare `DispatchJob` channel so kill/retry share one pipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineCommand {
    Start(DispatchJob),
    Kill { id: RunId },
    Retry { source_id: RunId, job: DispatchJob },
}

/// Side effects the event loop must perform after a pure state transition.
///
/// Navigation is **not** an Action — it mutates [`Screen`] in the reducer.
/// These variants are engine / future-screen seams only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// No external work; continue the loop.
    None,
    /// Restore the terminal and exit.
    Quit,
    /// User submitted the dispatch form (queued rows already in state).
    /// Forward to the run engine when wired.
    DispatchSubmitted { jobs: Vec<DispatchJob> },
    /// Request kill of the selected active run.
    KillSelected { id: RunId },
    /// Request retry of a failed run (new job already queued in UI state).
    RetrySelected { source_id: RunId, job: DispatchJob },
    /// Request PTY attach for the selected run (B2-pty).
    AttachSelected,
}

/// Which body pane is focused inside run detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailPane {
    #[default]
    Stream,
    Gate,
    Diff,
}

impl DetailPane {
    pub fn next(self) -> Self {
        match self {
            Self::Stream => Self::Gate,
            Self::Gate => Self::Diff,
            Self::Diff => Self::Stream,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Stream => Self::Diff,
            Self::Gate => Self::Stream,
            Self::Diff => Self::Gate,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Stream => "stream",
            Self::Gate => "gate",
            Self::Diff => "diff",
        }
    }
}

/// Active surface. Selection (`selected_id`) is shared across screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    ControlRoom,
    RunDetail {
        pane: DetailPane,
    },
    /// Terminal would be owned by the agent PTY. Stub until B2-pty.
    /// Esc detaches back to RunDetail without killing.
    Attached,
    /// Fan-out form: repos × agents × one task.
    Dispatch,
    /// Shared work queue (orchestrator view).
    Board,
}

/// Which field is focused in the dispatch form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DispatchFocus {
    #[default]
    Repos,
    Agents,
    Task,
}

/// Dispatch form — PRD §4.2 fan-out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchForm {
    pub repos: Vec<(String, bool)>,
    pub agents: Vec<(String, bool)>,
    pub task: String,
    pub focus: DispatchFocus,
    /// Cursor index into `repos` or `agents` when that column is focused.
    pub list_cursor: usize,
}

impl Default for DispatchForm {
    fn default() -> Self {
        Self {
            repos: vec![
                ("kit".into(), true),
                ("guardian".into(), false),
                ("trenchwire".into(), false),
            ],
            agents: vec![
                ("codex".into(), true),
                ("claude".into(), false),
                ("grok".into(), false),
                ("ollama".into(), false),
            ],
            task: String::new(),
            focus: DispatchFocus::Repos,
            list_cursor: 0,
        }
    }
}

impl DispatchForm {
    pub fn selected_repos(&self) -> Vec<&str> {
        self.repos
            .iter()
            .filter(|(_, on)| *on)
            .map(|(n, _)| n.as_str())
            .collect()
    }

    pub fn selected_agents(&self) -> Vec<&str> {
        self.agents
            .iter()
            .filter(|(_, on)| *on)
            .map(|(n, _)| n.as_str())
            .collect()
    }

    pub fn fanout_count(&self) -> usize {
        self.selected_repos().len() * self.selected_agents().len()
    }
}

/// One item on the shared Board queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardTask {
    pub id: u64,
    pub title: String,
    pub repo_hint: String,
    pub agent_hint: String,
    pub done: bool,
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
    /// Selected run identity. Stable across re-sorts so the list never jumps
    /// under the reader (PRD §4.2).
    pub selected_id: Option<RunId>,
    /// Snapshot of motion preference at construction (tests can override).
    motion: bool,
    /// Banner shown when a background operation fails.
    pub error: Option<String>,
    /// Live run table. Order in the vec is insertion order; display order is
    /// computed by [`App::display_order`].
    pub runs: Vec<RunRow>,
    /// Active surface.
    pub screen: Screen,
    /// Scroll offset (lines from top) when not following the stream/diff tail.
    pub detail_scroll: u16,
    /// When true, detail body pins to the last page of lines.
    pub stream_follow: bool,
    /// Short-lived status line for unwired engine seams. `(message, began_tick)`.
    flash: Option<(String, u64)>,
    /// Dispatch form (always held; shown when `screen == Dispatch`).
    pub dispatch: DispatchForm,
    /// Shared board queue.
    pub board: Vec<BoardTask>,
    /// Selected board row.
    pub board_selected: usize,
    /// Next board id.
    board_seq: u64,
}

/// One run as the Control Room / detail view-model (TUI-local).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRow {
    pub id: RunId,
    pub repo: String,
    pub agent: String,
    pub task: String,
    pub state: RunState,
    pub gate: Option<GateOutcome>,
    /// Clock tick when the run entered an active state (`Running` / `Gating`).
    pub active_since_tick: Option<u64>,
    /// Frozen elapsed at the moment the run left an active state, e.g. `"2m"`.
    pub elapsed_frozen: String,
    /// Creation order for stable secondary sort (age).
    pub seq: u64,
    /// Isolated worktree path once known.
    pub worktree: Option<PathBuf>,
    /// Append-only agent output (display-capped).
    pub output: String,
    /// True when the local display cap dropped older output.
    pub output_truncated: bool,
    /// Unified diff text. Empty until seeded / receipt / future contract delta.
    pub diff: String,
}

impl RunRow {
    pub fn new(
        id: RunId,
        repo: impl Into<String>,
        agent: impl Into<String>,
        task: impl Into<String>,
    ) -> Self {
        Self {
            id,
            repo: repo.into(),
            agent: agent.into(),
            task: task.into(),
            state: RunState::Queued,
            gate: None,
            active_since_tick: None,
            elapsed_frozen: String::new(),
            seq: 0,
            worktree: None,
            output: String::new(),
            output_truncated: false,
            diff: String::new(),
        }
    }

    /// Live or frozen elapsed label for the STATE column.
    pub fn elapsed_label(&self, clock: &Clock) -> String {
        if let Some(since) = self.active_since_tick
            && matches!(self.state, RunState::Running | RunState::Gating)
        {
            return format_elapsed_ticks(clock.tick.saturating_sub(since));
        }
        self.elapsed_frozen.clone()
    }

    /// First failure summary for the Control Room annotation line (`^ tsc: …`).
    pub fn gate_summary(&self) -> Option<String> {
        let gate = self.gate.as_ref()?;
        if gate.passed {
            return None;
        }
        if let Some(check) = gate.first_failure() {
            if let Some(summary) = &check.summary {
                return Some(summary.clone());
            }
            return Some(format!("{} failed", check.label));
        }
        if let Some(block) = gate.firewall_blocks.first() {
            return Some(block.clone());
        }
        if let Some(scope) = gate.scope_violations.first() {
            return Some(format!("scope: {scope}"));
        }
        Some("gate failed".into())
    }

    /// Append an output chunk, keeping the tail within the display cap.
    pub fn append_output(&mut self, chunk: &str) {
        self.output.push_str(chunk);
        if self.output.len() > OUTPUT_DISPLAY_CAP_BYTES {
            let overflow = self.output.len() - OUTPUT_DISPLAY_CAP_BYTES;
            // Byte cap may land mid-codepoint — walk *forward* so remaining
            // length never exceeds the cap (walking back would re-include bytes).
            let mut start = overflow.min(self.output.len());
            while start < self.output.len() && !self.output.is_char_boundary(start) {
                start += 1;
            }
            // Prefer dropping a full line so the display does not start mid-line.
            let rest = &self.output[start..];
            let cut = rest.find('\n').map(|i| start + i + 1).unwrap_or(start);
            self.output = self.output[cut..].to_string();
            self.output_truncated = true;
        }
    }

    /// Output split into display lines (preserves a trailing incomplete line).
    pub fn output_lines(&self) -> Vec<&str> {
        if self.output.is_empty() {
            return Vec::new();
        }
        self.output.split('\n').collect()
    }

    pub fn diff_lines(&self) -> Vec<&str> {
        if self.diff.is_empty() {
            return Vec::new();
        }
        self.diff.split('\n').collect()
    }
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
            dirty: true,
            size: (80, 24),
            selected_id: None,
            motion,
            error: None,
            runs: Vec::new(),
            screen: Screen::ControlRoom,
            detail_scroll: 0,
            stream_follow: true,
            flash: None,
            dispatch: DispatchForm::default(),
            board: Vec::new(),
            board_selected: 0,
            board_seq: 1,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn motion_enabled(&self) -> bool {
        self.motion
    }

    /// Active flash message, if still within its lifetime.
    pub fn flash_message(&self) -> Option<&str> {
        self.flash.as_ref().and_then(|(msg, began)| {
            if self.clock.within(*began, FLASH_TICKS) {
                Some(msg.as_str())
            } else {
                None
            }
        })
    }

    /// Whether anything on the current frame would change with the clock.
    pub fn animated_on_screen(&self) -> bool {
        if self.flash.is_some() {
            return true;
        }
        self.runs
            .iter()
            .any(|r| matches!(r.state, RunState::Running | RunState::Gating))
    }

    /// Pure, synchronous reducer.
    pub fn update(&mut self, event: AppEvent) -> Action {
        let marks_dirty = match &event {
            AppEvent::AnimationTick => {
                let next = self.clock.tick.wrapping_add(1);
                // Elapsed labels: once per second. Flash expiry: every tick while live.
                let flash_active = self.flash.is_some();
                let elapsed_tick = next.is_multiple_of(TICK_HZ)
                    && self
                        .runs
                        .iter()
                        .any(|r| matches!(r.state, RunState::Running | RunState::Gating));
                self.motion && (elapsed_tick || flash_active)
            }
            other => other.is_redraw_worthy(),
        };
        let action = self.apply(event);
        if marks_dirty {
            self.dirty = true;
        }
        // Drop expired flash so it does not keep us animated forever.
        if let Some((_, began)) = &self.flash
            && !self.clock.within(*began, FLASH_TICKS)
        {
            self.flash = None;
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
                    Self::apply_gate_to_row(row, outcome, self.clock.tick);
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
        if key.kind == KeyEventKind::Release {
            return Action::None;
        }
        match self.screen {
            Screen::ControlRoom => self.on_control_room_key(key),
            Screen::RunDetail { pane } => self.on_detail_key(key, pane),
            Screen::Attached => self.on_attached_key(key),
            Screen::Dispatch => self.on_dispatch_key(key),
            Screen::Board => self.on_board_key(key),
        }
    }

    fn on_control_room_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
                Action::Quit
            }
            KeyCode::Up if !self.runs.is_empty() => {
                self.move_selection(-1);
                Action::None
            }
            KeyCode::Down if !self.runs.is_empty() => {
                self.move_selection(1);
                Action::None
            }
            KeyCode::Enter => {
                self.open_detail(DetailPane::Stream);
                Action::None
            }
            KeyCode::Char('g') => {
                // lowercase only — uppercase G is End in detail.
                self.open_detail(DetailPane::Gate);
                Action::None
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.screen = Screen::Dispatch;
                Action::None
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.screen = Screen::Board;
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Char('K') => self.request_kill(),
            KeyCode::Char('r') | KeyCode::Char('R') => self.request_retry(),
            _ => Action::None,
        }
    }

    fn on_dispatch_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::ControlRoom;
                Action::None
            }
            KeyCode::Char('q') | KeyCode::Char('Q')
                if self.dispatch.focus != DispatchFocus::Task =>
            {
                self.should_quit = true;
                Action::Quit
            }
            KeyCode::Tab => {
                self.dispatch.focus = match self.dispatch.focus {
                    DispatchFocus::Repos => DispatchFocus::Agents,
                    DispatchFocus::Agents => DispatchFocus::Task,
                    DispatchFocus::Task => DispatchFocus::Repos,
                };
                self.dispatch.list_cursor = 0;
                Action::None
            }
            KeyCode::BackTab => {
                self.dispatch.focus = match self.dispatch.focus {
                    DispatchFocus::Repos => DispatchFocus::Task,
                    DispatchFocus::Agents => DispatchFocus::Repos,
                    DispatchFocus::Task => DispatchFocus::Agents,
                };
                self.dispatch.list_cursor = 0;
                Action::None
            }
            KeyCode::Up => {
                match self.dispatch.focus {
                    DispatchFocus::Repos | DispatchFocus::Agents => {
                        self.dispatch.list_cursor = self.dispatch.list_cursor.saturating_sub(1);
                    }
                    DispatchFocus::Task => {}
                }
                Action::None
            }
            KeyCode::Down => {
                match self.dispatch.focus {
                    DispatchFocus::Repos => {
                        let max = self.dispatch.repos.len().saturating_sub(1);
                        self.dispatch.list_cursor = (self.dispatch.list_cursor + 1).min(max);
                    }
                    DispatchFocus::Agents => {
                        let max = self.dispatch.agents.len().saturating_sub(1);
                        self.dispatch.list_cursor = (self.dispatch.list_cursor + 1).min(max);
                    }
                    DispatchFocus::Task => {}
                }
                Action::None
            }
            KeyCode::Char(' ') if self.dispatch.focus != DispatchFocus::Task => {
                self.toggle_dispatch_cursor();
                Action::None
            }
            KeyCode::Enter => self.submit_dispatch(),
            KeyCode::Backspace if self.dispatch.focus == DispatchFocus::Task => {
                self.dispatch.task.pop();
                Action::None
            }
            KeyCode::Char(c) if self.dispatch.focus == DispatchFocus::Task && !c.is_control() => {
                if self.dispatch.task.len() < 240 {
                    self.dispatch.task.push(c);
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn toggle_dispatch_cursor(&mut self) {
        match self.dispatch.focus {
            DispatchFocus::Repos => {
                if let Some((_, on)) = self.dispatch.repos.get_mut(self.dispatch.list_cursor) {
                    *on = !*on;
                }
            }
            DispatchFocus::Agents => {
                if let Some((_, on)) = self.dispatch.agents.get_mut(self.dispatch.list_cursor) {
                    *on = !*on;
                }
            }
            DispatchFocus::Task => {}
        }
    }

    fn submit_dispatch(&mut self) -> Action {
        let task = self.dispatch.task.trim().to_string();
        if task.is_empty() {
            self.set_flash("task is empty — type a prompt first");
            self.dispatch.focus = DispatchFocus::Task;
            return Action::None;
        }
        let repos = self
            .dispatch
            .selected_repos()
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let agents = self
            .dispatch
            .selected_agents()
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        if repos.is_empty() || agents.is_empty() {
            self.set_flash("select at least one repo and one agent");
            return Action::None;
        }
        let n = repos.len() * agents.len();
        if n > DISPATCH_FANOUT_CAP {
            self.set_flash(format!(
                "fan-out {n} exceeds cap {DISPATCH_FANOUT_CAP} — deselect some"
            ));
            return Action::None;
        }

        let mut first_id = None;
        let mut jobs = Vec::with_capacity(n);
        for repo in &repos {
            for agent in &agents {
                let id = RunId::new();
                if first_id.is_none() {
                    first_id = Some(id.clone());
                }
                jobs.push(DispatchJob {
                    id: id.clone(),
                    repo: repo.clone(),
                    agent: agent.clone(),
                    task: task.clone(),
                });
                let mut row = RunRow::new(id, repo.clone(), agent.clone(), task.clone());
                row.state = RunState::Queued;
                self.upsert_run(row);
            }
        }
        if let Some(id) = first_id {
            self.selected_id = Some(id);
        }
        self.screen = Screen::ControlRoom;
        self.set_flash(format!("{n} run(s) queued — starting engine"));
        Action::DispatchSubmitted { jobs }
    }

    fn on_board_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::ControlRoom;
                Action::None
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
                Action::Quit
            }
            KeyCode::Up if !self.board.is_empty() => {
                self.board_selected = self.board_selected.saturating_sub(1);
                Action::None
            }
            KeyCode::Down if !self.board.is_empty() => {
                let max = self.board.len().saturating_sub(1);
                self.board_selected = (self.board_selected + 1).min(max);
                Action::None
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                // Seed a board item from the dispatch task field if set, else placeholder.
                let title = if self.dispatch.task.trim().is_empty() {
                    format!("task-{}", self.board_seq)
                } else {
                    self.dispatch.task.trim().to_string()
                };
                let id = self.board_seq;
                self.board_seq += 1;
                self.board.push(BoardTask {
                    id,
                    title,
                    repo_hint: self
                        .dispatch
                        .selected_repos()
                        .first()
                        .copied()
                        .unwrap_or("kit")
                        .to_string(),
                    agent_hint: self
                        .dispatch
                        .selected_agents()
                        .first()
                        .copied()
                        .unwrap_or("codex")
                        .to_string(),
                    done: false,
                });
                self.board_selected = self.board.len().saturating_sub(1);
                Action::None
            }
            KeyCode::Char('x') | KeyCode::Char('X') | KeyCode::Delete if !self.board.is_empty() => {
                self.board.remove(self.board_selected);
                if self.board_selected >= self.board.len() && self.board_selected > 0 {
                    self.board_selected -= 1;
                }
                Action::None
            }
            KeyCode::Char(' ') if !self.board.is_empty() => {
                if let Some(t) = self.board.get_mut(self.board_selected) {
                    t.done = !t.done;
                }
                Action::None
            }
            KeyCode::Enter if !self.board.is_empty() => {
                // Prefill dispatch from the selected board item and open form.
                let item = self.board[self.board_selected].clone();
                self.dispatch.task = item.title;
                for (name, on) in &mut self.dispatch.repos {
                    *on = *name == item.repo_hint;
                }
                for (name, on) in &mut self.dispatch.agents {
                    *on = *name == item.agent_hint;
                }
                self.dispatch.focus = DispatchFocus::Task;
                self.screen = Screen::Dispatch;
                Action::None
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.screen = Screen::Dispatch;
                Action::None
            }
            _ => Action::None,
        }
    }

    fn on_detail_key(&mut self, key: KeyEvent, pane: DetailPane) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
                Action::Quit
            }
            KeyCode::Esc => {
                self.screen = Screen::ControlRoom;
                Action::None
            }
            KeyCode::Tab | KeyCode::Right => {
                self.screen = Screen::RunDetail { pane: pane.next() };
                self.detail_scroll = 0;
                self.stream_follow = true;
                Action::None
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.screen = Screen::RunDetail { pane: pane.prev() };
                self.detail_scroll = 0;
                self.stream_follow = true;
                Action::None
            }
            KeyCode::Char('1') => {
                self.screen = Screen::RunDetail {
                    pane: DetailPane::Stream,
                };
                self.detail_scroll = 0;
                self.stream_follow = true;
                Action::None
            }
            KeyCode::Char('2') => {
                self.screen = Screen::RunDetail {
                    pane: DetailPane::Gate,
                };
                self.detail_scroll = 0;
                self.stream_follow = true;
                Action::None
            }
            KeyCode::Char('3') => {
                self.screen = Screen::RunDetail {
                    pane: DetailPane::Diff,
                };
                self.detail_scroll = 0;
                self.stream_follow = true;
                Action::None
            }
            KeyCode::Up => {
                self.scroll_detail(-1);
                Action::None
            }
            KeyCode::Down => {
                self.scroll_detail(1);
                Action::None
            }
            KeyCode::PageUp => {
                self.scroll_detail(-10);
                Action::None
            }
            KeyCode::PageDown => {
                self.scroll_detail(10);
                Action::None
            }
            KeyCode::Home => {
                self.detail_scroll = 0;
                self.stream_follow = false;
                Action::None
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.stream_follow = true;
                self.detail_scroll = 0;
                Action::None
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Enter attach mode immediately as an honest stub; Action
                // signals the loop to wire PTY when B2-pty exists.
                self.screen = Screen::Attached;
                self.set_flash("PTY not connected yet — Esc detaches");
                Action::AttachSelected
            }
            KeyCode::Char('k') | KeyCode::Char('K') => self.request_kill(),
            KeyCode::Char('r') | KeyCode::Char('R') => self.request_retry(),
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.screen = Screen::Dispatch;
                Action::None
            }
            _ => Action::None,
        }
    }

    fn on_attached_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                // Detach without killing (PRD §4.2).
                self.screen = Screen::RunDetail {
                    pane: DetailPane::Stream,
                };
                self.stream_follow = true;
                self.detail_scroll = 0;
                Action::None
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                // Refuse quit-from-attach — accidental death of agent sessions.
                self.set_flash("Esc to detach (q disabled while attached)");
                Action::None
            }
            _ => Action::None,
        }
    }

    fn open_detail(&mut self, pane: DetailPane) {
        if self.selected_id.is_none() && !self.runs.is_empty() {
            // Prefer display-order first if nothing selected.
            self.selected_id = self
                .display_order()
                .first()
                .map(|&i| self.runs[i].id.clone());
        }
        if self.selected_id.is_none() {
            return;
        }
        self.screen = Screen::RunDetail { pane };
        self.stream_follow = true;
        self.detail_scroll = 0;
    }

    fn scroll_detail(&mut self, delta: i32) {
        self.stream_follow = false;
        let max = self.detail_line_count().saturating_sub(1) as u16;
        if delta < 0 {
            self.detail_scroll = self.detail_scroll.saturating_sub((-delta) as u16);
        } else {
            self.detail_scroll = (self.detail_scroll.saturating_add(delta as u16)).min(max);
        }
    }

    /// Line count for the active detail body (for scroll clamping).
    pub fn detail_line_count(&self) -> usize {
        let Some(run) = self.selected_run() else {
            return 0;
        };
        match self.screen {
            Screen::RunDetail {
                pane: DetailPane::Stream,
            } => run.output_lines().len().max(1),
            Screen::RunDetail {
                pane: DetailPane::Diff,
            } => run.diff_lines().len().max(1),
            Screen::RunDetail {
                pane: DetailPane::Gate,
            } => gate_log_line_count(run),
            _ => 0,
        }
    }

    /// Kill the selected active run (Running / Gating / Queued).
    fn request_kill(&mut self) -> Action {
        let Some(row) = self.selected_run() else {
            self.set_flash("no run selected");
            return Action::None;
        };
        if !row.state.is_active() {
            self.set_flash("run already finished");
            return Action::None;
        }
        let id = row.id.clone();
        let short = short_run_id(&id);
        self.set_flash(format!("killing {short}…"));
        Action::KillSelected { id }
    }

    /// Fail-only retry: queue a new run with gate failure context in the task.
    fn request_retry(&mut self) -> Action {
        let Some(row) = self.selected_run().cloned() else {
            self.set_flash("no run selected");
            return Action::None;
        };
        if row.state != RunState::Fail {
            self.set_flash("retry only for failed (gate) runs");
            return Action::None;
        }

        let gate_ctx = row
            .gate_summary()
            .or_else(|| {
                row.gate.as_ref().map(|g| {
                    if g.checks.is_empty() {
                        "gate failed (no check details)".into()
                    } else {
                        format!(
                            "{} check(s) failed",
                            g.checks.iter().filter(|c| !c.passed()).count()
                        )
                    }
                })
            })
            .unwrap_or_else(|| "gate failed".into());

        let task = format!(
            "{}\n\n## Previous gate failure\n{}\n\nFix the failure, then leave the worktree green.",
            row.task.trim_end(),
            gate_ctx
        );
        let new_id = RunId::default();
        let job = DispatchJob {
            id: new_id.clone(),
            repo: row.repo.clone(),
            agent: row.agent.clone(),
            task: task.clone(),
        };

        let mut queued = RunRow::new(new_id.clone(), row.repo, row.agent, task);
        queued.seq = self.runs.len() as u64;
        self.runs.push(queued);
        self.selected_id = Some(new_id);
        self.screen = Screen::ControlRoom;
        self.set_flash(format!(
            "retry queued from {} — starting engine",
            short_run_id(&row.id)
        ));
        Action::RetrySelected {
            source_id: row.id,
            job,
        }
    }

    fn set_flash(&mut self, msg: impl Into<String>) {
        self.flash = Some((msg.into(), self.clock.tick));
    }

    /// Move selection by `delta` rows in *display* order.
    fn move_selection(&mut self, delta: isize) {
        let order = self.display_order();
        if order.is_empty() {
            return;
        }
        let current = self
            .selected_id
            .as_ref()
            .and_then(|id| order.iter().position(|&i| self.runs[i].id == *id))
            .unwrap_or(0);
        let next = if delta < 0 {
            current.saturating_sub((-delta) as usize)
        } else {
            (current + delta as usize).min(order.len() - 1)
        };
        self.selected_id = Some(self.runs[order[next]].id.clone());
    }

    /// Indices into `runs` sorted for the Control Room: state priority, then age.
    pub fn display_order(&self) -> Vec<usize> {
        let mut idxs: Vec<usize> = (0..self.runs.len()).collect();
        idxs.sort_by(|&a, &b| {
            let ra = &self.runs[a];
            let rb = &self.runs[b];
            state_rank(ra.state)
                .cmp(&state_rank(rb.state))
                .then_with(|| ra.seq.cmp(&rb.seq))
        });
        idxs
    }

    pub fn selected_display_index(&self) -> Option<usize> {
        let id = self.selected_id.as_ref()?;
        self.display_order()
            .into_iter()
            .position(|i| self.runs[i].id == *id)
    }

    pub fn selected_run(&self) -> Option<&RunRow> {
        let id = self.selected_id.as_ref()?;
        self.runs.iter().find(|r| r.id == *id)
    }

    /// Insert or replace a fully-described row (engine / fixtures use this).
    pub fn upsert_run(&mut self, mut row: RunRow) {
        if let Some(existing) = self.runs.iter_mut().find(|r| r.id == row.id) {
            row.seq = existing.seq;
            if row.active_since_tick.is_none() {
                row.active_since_tick = existing.active_since_tick;
            }
            if row.elapsed_frozen.is_empty() {
                row.elapsed_frozen = existing.elapsed_frozen.clone();
            }
            if row.output.is_empty() && !existing.output.is_empty() {
                row.output = existing.output.clone();
                row.output_truncated = existing.output_truncated;
            }
            if row.diff.is_empty() && !existing.diff.is_empty() {
                row.diff = existing.diff.clone();
            }
            if row.worktree.is_none() {
                row.worktree = existing.worktree.clone();
            }
            *existing = row;
        } else {
            row.seq = self.runs.len() as u64;
            if matches!(row.state, RunState::Running | RunState::Gating)
                && row.active_since_tick.is_none()
            {
                row.active_since_tick = Some(self.clock.tick);
            }
            let id = row.id.clone();
            self.runs.push(row);
            if self.selected_id.is_none() {
                self.selected_id = Some(id);
            }
        }
    }

    fn apply_run_update(&mut self, id: RunId, delta: RunDelta) {
        let tick = self.clock.tick;
        let row = if let Some(row) = self.runs.iter_mut().find(|r| r.id == id) {
            row
        } else {
            let mut fresh = RunRow::new(id.clone(), "", "", "");
            fresh.seq = self.runs.len() as u64;
            self.runs.push(fresh);
            if self.selected_id.is_none() {
                self.selected_id = Some(id.clone());
            }
            self.runs.last_mut().expect("just pushed")
        };

        match delta {
            RunDelta::State(state) => {
                Self::transition_state(row, state, tick);
            }
            RunDelta::Output(chunk) => {
                row.append_output(&chunk);
            }
            RunDelta::Worktree(path) => {
                row.worktree = Some(path);
            }
            RunDelta::Gate(outcome) => {
                Self::apply_gate_to_row(row, outcome, tick);
            }
        }
    }

    fn transition_state(row: &mut RunRow, state: RunState, tick: u64) {
        let was_active = matches!(row.state, RunState::Running | RunState::Gating);
        let now_active = matches!(state, RunState::Running | RunState::Gating);

        if now_active && !was_active {
            row.active_since_tick = Some(tick);
            row.elapsed_frozen.clear();
        } else if was_active
            && !now_active
            && let Some(since) = row.active_since_tick.take()
        {
            row.elapsed_frozen = format_elapsed_ticks(tick.saturating_sub(since));
        }

        row.state = state;
    }

    fn apply_gate_to_row(row: &mut RunRow, outcome: GateOutcome, tick: u64) {
        let next = if outcome.passed {
            RunState::Pass
        } else {
            RunState::Fail
        };
        Self::transition_state(row, next, tick);
        row.gate = Some(outcome);
    }

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

    /// PRD §4.2 fixture rows — Control Room + detail snapshots / visual QA.
    pub fn load_prd_fixture(&mut self) {
        use kit_core::{CheckStatus, GateCheck};
        use std::time::Duration;

        self.runs.clear();
        self.selected_id = None;
        self.screen = Screen::ControlRoom;
        self.clock = Clock {
            tick: TICK_HZ * 120,
        };

        let mut r0 = RunRow::new(
            RunId("01FIXRUN0KITCODEX000000000".into()),
            "kit",
            "codex",
            "port guard.js",
        );
        r0.state = RunState::Running;
        r0.active_since_tick = Some(0);
        r0.worktree = Some(PathBuf::from("/tmp/kit-wt-01FIXRUN0"));
        r0.output = [
            "codex: starting worktree",
            "reading crates/kit-gate/src/lib.rs",
            "porting allowlist rules…",
            "drafting firewall verdict mapping",
            "tests: pending",
        ]
        .join("\n");
        r0.seq = 0;
        self.upsert_run(r0);

        let mut r1 = RunRow::new(
            RunId("01FIXRUN1KITGROK0000000000".into()),
            "kit",
            "grok",
            "frame clock",
        );
        r1.state = RunState::Running;
        r1.active_since_tick = Some(0);
        r1.worktree = Some(PathBuf::from("/tmp/kit-wt-01FIXRUN1"));
        r1.output = "grok: event loop select! arms online\nredraw policy: idle quiet".into();
        r1.seq = 1;
        self.upsert_run(r1);

        let mut r2 = RunRow::new(
            RunId("01FIXRUN2GUARDIANCLAUDE000".into()),
            "guardian",
            "claude",
            "855-case suite",
        );
        r2.state = RunState::Pass;
        r2.elapsed_frozen = "4m".into();
        r2.output = "npm test\n\n  855 passed\n".into();
        r2.diff = [
            "diff --git a/hooks/guard.js b/hooks/guard.js",
            "--- a/hooks/guard.js",
            "+++ b/hooks/guard.js",
            "@@ -10,0 +11,3 @@",
            "+// rust port parity note",
            "+export const KIT_PARITY = true;",
        ]
        .join("\n");
        r2.gate = Some(GateOutcome {
            passed: true,
            checks: vec![GateCheck {
                label: "test".into(),
                command: "npm test".into(),
                status: CheckStatus::Pass,
                exit_code: Some(0),
                summary: None,
                duration: Duration::from_secs(12),
            }],
            scope_violations: vec![],
            firewall_blocks: vec![],
            duration: Duration::from_secs(12),
        });
        r2.seq = 2;
        self.upsert_run(r2);

        let mut r3 = RunRow::new(
            RunId("01FIXRUN3TRENCHWIRECODEX00".into()),
            "trenchwire",
            "codex",
            "fix red CI",
        );
        r3.state = RunState::Fail;
        r3.elapsed_frozen = "1m".into();
        r3.worktree = Some(PathBuf::from("/tmp/kit-wt-01FIXRUN3"));
        r3.output = [
            "codex: patching src/client.ts",
            "running tsc --noEmit",
            "src/client.ts(42,5): error TS2322: Type 'string' is not assignable…",
            "src/client.ts(88,12): error TS2345",
            "src/api.ts(3,1): error TS2307: Cannot find module './missing'",
        ]
        .join("\n");
        r3.diff = [
            "diff --git a/src/client.ts b/src/client.ts",
            "--- a/src/client.ts",
            "+++ b/src/client.ts",
            "@@ -40,3 +40,3 @@",
            "-  return data as Response;",
            "+  return data as string;",
        ]
        .join("\n");
        r3.gate = Some(GateOutcome {
            passed: false,
            checks: vec![
                GateCheck {
                    label: "format".into(),
                    command: "pnpm format:check".into(),
                    status: CheckStatus::Pass,
                    exit_code: Some(0),
                    summary: None,
                    duration: Duration::from_secs(2),
                },
                GateCheck {
                    label: "typecheck".into(),
                    command: "tsc --noEmit".into(),
                    status: CheckStatus::Fail,
                    exit_code: Some(2),
                    summary: Some("tsc: 3 errors".into()),
                    duration: Duration::from_secs(8),
                },
            ],
            scope_violations: vec![],
            firewall_blocks: vec![],
            duration: Duration::from_secs(10),
        });
        r3.seq = 3;
        self.upsert_run(r3);

        self.selected_id = self
            .display_order()
            .first()
            .map(|&i| self.runs[i].id.clone());

        // Board fixture items for F4 snapshots / QA.
        self.board = vec![
            BoardTask {
                id: 1,
                title: "port guard.js".into(),
                repo_hint: "kit".into(),
                agent_hint: "codex".into(),
                done: false,
            },
            BoardTask {
                id: 2,
                title: "frame clock".into(),
                repo_hint: "kit".into(),
                agent_hint: "grok".into(),
                done: false,
            },
            BoardTask {
                id: 3,
                title: "fix red CI".into(),
                repo_hint: "trenchwire".into(),
                agent_hint: "codex".into(),
                done: true,
            },
        ];
        self.board_seq = 4;
        self.board_selected = 0;
        self.dirty = true;
    }
}

/// Display sort rank — lower is higher in the table.
fn state_rank(state: RunState) -> u8 {
    match state {
        RunState::Running => 0,
        RunState::Gating => 1,
        RunState::Queued => 2,
        RunState::Fail => 3,
        RunState::Error => 4,
        RunState::Pass => 5,
        RunState::Killed => 6,
    }
}

fn format_elapsed_ticks(ticks: u64) -> String {
    let secs = ticks / TICK_HZ;
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        let m = secs / 60;
        format!("{m}m")
    } else {
        let h = secs / 3600;
        format!("{h}h")
    }
}

/// Short id for flash banners (first 8 chars of RunId).
fn short_run_id(id: &RunId) -> String {
    let s = id.0.as_str();
    if s.len() <= 8 {
        s.to_string()
    } else {
        s[..8].to_string()
    }
}

/// Gate log height for scroll clamping — must match rendered `gate_log_lines`.
fn gate_log_line_count(run: &RunRow) -> usize {
    gate_log_lines(run).len()
}

/// Public helpers used by the UI for state/gate labels.
pub fn format_state_label(run: &RunRow, clock: &Clock) -> String {
    let label = match run.state {
        RunState::Queued => "QUEUED",
        RunState::Running => "RUN",
        RunState::Gating => "GATING",
        RunState::Pass => "DONE",
        RunState::Fail => "DONE",
        RunState::Killed => "KILLED",
        RunState::Error => "ERROR",
    };
    let elapsed = run.elapsed_label(clock);
    if matches!(run.state, RunState::Running | RunState::Gating) && !elapsed.is_empty() {
        format!("{label} {elapsed}")
    } else {
        label.to_string()
    }
}

pub fn format_gate_label(run: &RunRow) -> String {
    match (&run.state, &run.gate) {
        (_, Some(g)) if g.passed => "PASS".into(),
        (_, Some(_)) => "FAIL".into(),
        (RunState::Pass, None) => "PASS".into(),
        (RunState::Fail, None) => "FAIL".into(),
        _ => "--".into(),
    }
}

/// Build the gate log lines for the detail Gate pane.
pub fn gate_log_lines(run: &RunRow) -> Vec<String> {
    use kit_core::CheckStatus;
    match &run.gate {
        None if run.state.is_terminal() => {
            vec!["No gate result recorded.".into()]
        }
        None => vec!["Gate has not run yet.".into()],
        Some(g) => {
            let mut lines = Vec::new();
            lines.push(if g.passed {
                format!("OVERALL  PASS  ({}ms)", g.duration.as_millis())
            } else {
                format!("OVERALL  FAIL  ({}ms)", g.duration.as_millis())
            });
            lines.push(String::new());
            for c in &g.checks {
                let status = match c.status {
                    CheckStatus::Pass => "PASS",
                    CheckStatus::Fail => "FAIL",
                    CheckStatus::Skipped => "SKIP",
                    CheckStatus::TimedOut => "TIME",
                };
                let mut line = format!("{status}  {}  {}", c.label, c.command);
                if let Some(s) = &c.summary {
                    line.push_str("  ·  ");
                    line.push_str(s);
                }
                lines.push(line);
            }
            if !g.firewall_blocks.is_empty() {
                lines.push(String::new());
                lines.push("FIREWALL".into());
                for b in &g.firewall_blocks {
                    lines.push(format!("  block  {b}"));
                }
            }
            if !g.scope_violations.is_empty() {
                lines.push(String::new());
                lines.push("SCOPE".into());
                for s in &g.scope_violations {
                    lines.push(format!("  violate  {s}"));
                }
            }
            lines
        }
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

    fn code(code: KeyCode) -> AppEvent {
        AppEvent::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn animation_tick_advances_clock_but_not_dirty_when_motion_off() {
        let mut app = App::with_motion(false);
        app.clear_dirty();
        let action = app.update(AppEvent::AnimationTick);
        assert_eq!(action, Action::None);
        assert_eq!(app.clock.tick, 1);
        assert!(!app.is_dirty());
    }

    #[test]
    fn animation_tick_does_not_dirty_idle_room_even_with_motion() {
        let mut app = App::with_motion(true);
        app.clear_dirty();
        app.update(AppEvent::AnimationTick);
        assert!(!app.is_dirty());
    }

    #[test]
    fn active_run_dirties_once_per_second_for_elapsed() {
        let mut app = App::with_motion(true);
        let id = RunId("01ACTIVE000000000000000000".into());
        app.upsert_run({
            let mut r = RunRow::new(id, "kit", "codex", "tick");
            r.state = RunState::Running;
            r.active_since_tick = Some(0);
            r
        });
        app.clear_dirty();
        for _ in 0..19 {
            app.update(AppEvent::AnimationTick);
        }
        assert!(!app.is_dirty());
        app.update(AppEvent::AnimationTick);
        assert!(app.is_dirty());
    }

    #[test]
    fn scripted_reducer_sequence_is_deterministic() {
        let mut app = App::with_motion(false);
        app.clear_dirty();
        assert_eq!(app.update(AppEvent::Resize(120, 40)), Action::None);
        assert_eq!(app.size, (120, 40));
        let id = RunId("01TESTRUN00000000000000000".into());
        app.update(AppEvent::RunUpdate(id, RunDelta::State(RunState::Running)));
        assert_eq!(app.running_count(), 1);
        assert_eq!(app.update(key('q')), Action::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn quit_event_requests_exit() {
        let mut app = App::with_motion(false);
        assert_eq!(app.update(AppEvent::Quit), Action::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn selection_is_stable_across_resort() {
        let mut app = App::with_motion(false);
        let pass_id = RunId("01PASS00000000000000000000".into());
        let run_id = RunId("01RUN000000000000000000000".into());
        app.upsert_run({
            let mut r = RunRow::new(pass_id.clone(), "a", "codex", "done");
            r.state = RunState::Pass;
            r
        });
        app.upsert_run({
            let mut r = RunRow::new(run_id.clone(), "b", "grok", "live");
            r.state = RunState::Running;
            r
        });
        app.selected_id = Some(pass_id.clone());
        app.update(AppEvent::RunUpdate(run_id, RunDelta::State(RunState::Pass)));
        assert_eq!(app.selected_id.as_ref(), Some(&pass_id));
    }

    #[test]
    fn arrow_keys_move_in_display_order() {
        let mut app = App::with_motion(false);
        let a = RunId("01A00000000000000000000000".into());
        let b = RunId("01B00000000000000000000000".into());
        app.upsert_run({
            let mut r = RunRow::new(a.clone(), "a", "codex", "x");
            r.state = RunState::Pass;
            r
        });
        app.upsert_run({
            let mut r = RunRow::new(b.clone(), "b", "grok", "y");
            r.state = RunState::Running;
            r
        });
        app.selected_id = Some(a.clone());
        app.update(code(KeyCode::Up));
        assert_eq!(app.selected_id.as_ref(), Some(&b));
        app.update(code(KeyCode::Down));
        assert_eq!(app.selected_id.as_ref(), Some(&a));
    }

    #[test]
    fn enter_opens_stream_detail_and_esc_returns() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        assert_eq!(app.screen, Screen::ControlRoom);
        app.update(code(KeyCode::Enter));
        assert_eq!(
            app.screen,
            Screen::RunDetail {
                pane: DetailPane::Stream
            }
        );
        assert!(app.stream_follow);
        app.update(code(KeyCode::Esc));
        assert_eq!(app.screen, Screen::ControlRoom);
    }

    #[test]
    fn g_opens_gate_pane() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(key('g'));
        assert_eq!(
            app.screen,
            Screen::RunDetail {
                pane: DetailPane::Gate
            }
        );
    }

    #[test]
    fn tab_cycles_detail_panes() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(code(KeyCode::Enter));
        app.update(code(KeyCode::Tab));
        assert_eq!(
            app.screen,
            Screen::RunDetail {
                pane: DetailPane::Gate
            }
        );
        app.update(code(KeyCode::Tab));
        assert_eq!(
            app.screen,
            Screen::RunDetail {
                pane: DetailPane::Diff
            }
        );
        app.update(code(KeyCode::Tab));
        assert_eq!(
            app.screen,
            Screen::RunDetail {
                pane: DetailPane::Stream
            }
        );
    }

    #[test]
    fn output_delta_appends_and_truncates() {
        let mut app = App::with_motion(false);
        let id = RunId("01OUT000000000000000000000".into());
        app.upsert_run(RunRow::new(id.clone(), "kit", "codex", "x"));
        app.update(AppEvent::RunUpdate(
            id.clone(),
            RunDelta::Output("hello\n".into()),
        ));
        assert_eq!(app.runs[0].output, "hello\n");

        // Force over-cap.
        let big = "x".repeat(OUTPUT_DISPLAY_CAP_BYTES + 100);
        app.update(AppEvent::RunUpdate(id, RunDelta::Output(big)));
        assert!(app.runs[0].output.len() <= OUTPUT_DISPLAY_CAP_BYTES);
        assert!(app.runs[0].output_truncated);
    }

    #[test]
    fn output_truncation_stays_on_char_boundary() {
        let mut row = RunRow::new(
            RunId("01UTF8000000000000000000000".into()),
            "kit",
            "codex",
            "x",
        );
        // Multi-byte UTF-8 (each 雪 is 3 bytes) so a naive byte cut can panic.
        let snow = "雪".repeat((OUTPUT_DISPLAY_CAP_BYTES / 3) + 80);
        row.append_output(&snow);
        assert!(row.output_truncated);
        assert!(row.output.len() <= OUTPUT_DISPLAY_CAP_BYTES);
        assert!(
            row.output.is_char_boundary(row.output.len()),
            "truncated buffer must remain valid UTF-8"
        );
        // Must not panic when re-slicing / displaying.
        let _ = row.output_lines();
    }

    #[test]
    fn worktree_delta_is_stored() {
        let mut app = App::with_motion(false);
        let id = RunId("01WT0000000000000000000000".into());
        app.upsert_run(RunRow::new(id.clone(), "kit", "codex", "x"));
        app.update(AppEvent::RunUpdate(
            id,
            RunDelta::Worktree(PathBuf::from("/tmp/wt")),
        ));
        assert_eq!(
            app.runs[0].worktree.as_deref(),
            Some(std::path::Path::new("/tmp/wt"))
        );
    }

    #[test]
    fn scroll_disables_follow_end_reenables() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(code(KeyCode::Enter));
        assert!(app.stream_follow);
        app.update(code(KeyCode::Up));
        assert!(!app.stream_follow);
        app.update(code(KeyCode::End));
        assert!(app.stream_follow);
    }

    #[test]
    fn attach_and_detach_without_quit() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(code(KeyCode::Enter));
        assert_eq!(app.update(key('a')), Action::AttachSelected);
        assert_eq!(app.screen, Screen::Attached);
        // q does not quit while attached.
        assert_eq!(app.update(key('q')), Action::None);
        assert!(!app.should_quit);
        app.update(code(KeyCode::Esc));
        assert_eq!(
            app.screen,
            Screen::RunDetail {
                pane: DetailPane::Stream
            }
        );
    }

    #[test]
    fn engine_keys_emit_actions_with_flash() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        // Fixture first row is Running — kill is valid.
        let kill = app.update(key('k'));
        match kill {
            Action::KillSelected { id } => {
                assert_eq!(id.0, "01FIXRUN0KITCODEX000000000");
            }
            other => panic!("expected KillSelected, got {other:?}"),
        }
        assert!(app.flash_message().is_some());

        // Select a Fail row for retry.
        let fail_id = app
            .runs
            .iter()
            .find(|r| r.state == RunState::Fail)
            .map(|r| r.id.clone())
            .expect("fixture has Fail run");
        app.selected_id = Some(fail_id.clone());
        let before = app.runs.len();
        let retry = app.update(key('r'));
        match retry {
            Action::RetrySelected { source_id, job } => {
                assert_eq!(source_id, fail_id);
                assert!(job.task.contains("Previous gate failure"));
            }
            other => panic!("expected RetrySelected, got {other:?}"),
        }
        assert_eq!(app.runs.len(), before + 1);
    }

    #[test]
    fn retry_rejects_non_fail() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        // Running row selected by default.
        assert_eq!(app.update(key('r')), Action::None);
        assert!(
            app.flash_message()
                .is_some_and(|m| m.contains("retry only"))
        );
    }

    #[test]
    fn d_opens_dispatch_and_esc_returns() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(key('d'));
        assert_eq!(app.screen, Screen::Dispatch);
        app.update(code(KeyCode::Esc));
        assert_eq!(app.screen, Screen::ControlRoom);
    }

    #[test]
    fn b_opens_board() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(key('b'));
        assert_eq!(app.screen, Screen::Board);
        assert!(!app.board.is_empty());
    }

    #[test]
    fn dispatch_submit_fans_out_queued_runs() {
        let mut app = App::with_motion(false);
        app.dispatch.task = "ship the gate".into();
        // kit + guardian × codex = 2 when we enable guardian
        app.dispatch.repos[1].1 = true; // guardian
        let before = app.runs.len();
        let action = app.update(key('d'));
        assert_eq!(action, Action::None);
        assert_eq!(app.screen, Screen::Dispatch);
        let action = app.update(code(KeyCode::Enter));
        match action {
            Action::DispatchSubmitted { jobs } => assert_eq!(jobs.len(), 2),
            other => panic!("expected DispatchSubmitted, got {other:?}"),
        }
        assert_eq!(app.screen, Screen::ControlRoom);
        assert_eq!(app.runs.len(), before + 2);
        assert!(
            app.runs
                .iter()
                .filter(|r| r.task == "ship the gate")
                .all(|r| r.state == RunState::Queued)
        );
    }

    #[test]
    fn board_enter_prefills_dispatch() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(key('b'));
        app.update(code(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Dispatch);
        assert_eq!(app.dispatch.task, "port guard.js");
    }

    #[test]
    fn prd_fixture_matches_section_4_2_shape() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        assert_eq!(app.running_count(), 2);
        assert_eq!(app.runs.len(), 4);
        let order = app.display_order();
        assert_eq!(app.runs[order[0]].state, RunState::Running);
        assert_eq!(app.runs[order[2]].state, RunState::Fail);
        let fail = app.runs.iter().find(|r| r.state == RunState::Fail).unwrap();
        assert_eq!(fail.gate_summary().as_deref(), Some("tsc: 3 errors"));
        assert!(!fail.diff.is_empty());
        assert!(!fail.output.is_empty());
    }

    #[test]
    fn detail_arrows_do_not_change_selected_run() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let before = app.selected_id.clone();
        app.update(code(KeyCode::Enter));
        app.update(code(KeyCode::Down));
        app.update(code(KeyCode::Down));
        assert_eq!(app.selected_id, before);
    }

    #[test]
    fn elapsed_freezes_when_run_finishes() {
        let mut app = App::with_motion(false);
        let id = RunId("01ELAPSED00000000000000000".into());
        app.upsert_run({
            let mut r = RunRow::new(id.clone(), "kit", "codex", "x");
            r.state = RunState::Running;
            r.active_since_tick = Some(0);
            r
        });
        for _ in 0..(TICK_HZ * 45) {
            app.update(AppEvent::AnimationTick);
        }
        assert_eq!(app.runs[0].elapsed_label(&app.clock), "45s");
        app.update(AppEvent::RunUpdate(id, RunDelta::State(RunState::Pass)));
        assert_eq!(app.runs[0].elapsed_label(&app.clock), "45s");
    }
}
