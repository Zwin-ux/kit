//! Application state and pure reducer.
//!
//! All motion derives from [`Clock`], advanced only by `AppEvent::AnimationTick`.
//! The reducer is synchronous and free of I/O so it can be driven headlessly
//! in tests without a terminal.
//!
//! Screen model follows fennec-tui: navigation mutates [`Screen`] inside the
//! reducer; engine work leaves as [`Action`] for the event loop to fulfill.

use crate::event::{AppEvent, Clock, TICK_HZ, motion_enabled};
use crate::persona::{Persona, default_persona_toggles};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use kit_core::{GateOutcome, RunDelta, RunId, RunState};
use std::path::{Path, PathBuf};

/// Known neighbor checkouts shown on Dispatch when they are git repos.
const KNOWN_DISPATCH_SIBLINGS: &[&str] =
    &["kit", "guardian", "trenchwire", "board-world-app", "wiki"];

/// Hard cap on the Dispatch repo list (cwd + siblings).
const DISPATCH_REPO_CAP: usize = 8;

/// Local display cap for streamed output (Principle 1: bounded by default).
/// Engine may already cap; the TUI never holds more than this for rendering.
pub const OUTPUT_DISPLAY_CAP_BYTES: usize = 512 * 1024;

/// Flash banner lifetime in ticks (~2 seconds at [`TICK_HZ`]).
const FLASH_TICKS: u64 = TICK_HZ * 2;

/// Maximum repo×agent×persona combinations a single dispatch may create (UI guard).
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
    /// Fan-out form: repos × agents × personas × one task.
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
    Personas,
    Task,
}

/// Control Room table filter (`f` cycles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunFilter {
    #[default]
    All,
    Fail,
    Running,
    Done,
}

impl RunFilter {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Fail,
            Self::Fail => Self::Running,
            Self::Running => Self::Done,
            Self::Done => Self::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "ALL",
            Self::Fail => "FAIL",
            Self::Running => "RUN",
            Self::Done => "DONE",
        }
    }

    fn matches(self, state: RunState) -> bool {
        match self {
            Self::All => true,
            Self::Fail => matches!(state, RunState::Fail | RunState::Error),
            Self::Running => matches!(
                state,
                RunState::Running | RunState::Gating | RunState::Queued
            ),
            Self::Done => matches!(state, RunState::Pass | RunState::Killed),
        }
    }
}

/// Dispatch form — PRD §4.2 fan-out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchForm {
    pub repos: Vec<(String, bool)>,
    pub agents: Vec<(String, bool)>,
    pub personas: Vec<(Persona, bool)>,
    pub task: String,
    pub focus: DispatchFocus,
    /// Cursor index into the focused list column.
    pub list_cursor: usize,
}

impl Default for DispatchForm {
    fn default() -> Self {
        Self {
            repos: seed_dispatch_repos(),
            agents: vec![
                ("codex".into(), true),
                ("claude".into(), false),
                ("grok".into(), false),
                ("ollama".into(), false),
            ],
            personas: default_persona_toggles(),
            task: String::new(),
            focus: DispatchFocus::Repos,
            list_cursor: 0,
        }
    }
}

/// Seed Dispatch repos: cwd first (selected), then parent git siblings.
///
/// Stored values are **absolute paths** so the engine can resolve them.
/// The TUI displays [`format_repo_label`] (basename, ` · this` for cwd).
fn seed_dispatch_repos() -> Vec<(String, bool)> {
    let mut repos: Vec<(String, bool)> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    if let Ok(cwd) = std::env::current_dir() {
        push_repo_path(&mut repos, &mut seen, cwd.clone(), true);
        if let Some(parent) = cwd.parent() {
            for name in KNOWN_DISPATCH_SIBLINGS {
                if repos.len() >= DISPATCH_REPO_CAP {
                    break;
                }
                let p = parent.join(name);
                if is_dispatch_sibling(&p) {
                    push_repo_path(&mut repos, &mut seen, p, false);
                }
            }
            if repos.len() < DISPATCH_REPO_CAP
                && let Ok(entries) = std::fs::read_dir(parent)
            {
                let mut extras: Vec<PathBuf> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| is_dispatch_sibling(p) && p.join("kit.toml").is_file())
                    .collect();
                extras.sort();
                for p in extras {
                    if repos.len() >= DISPATCH_REPO_CAP {
                        break;
                    }
                    push_repo_path(&mut repos, &mut seen, p, false);
                }
            }
        }
    }

    if repos.is_empty() {
        repos.push((".".into(), true));
    }
    repos
}

fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Parent sibling eligible for Dispatch: a git repo that is either a known
/// neighbor or contains `kit.toml`.
fn is_dispatch_sibling(path: &Path) -> bool {
    if !path.is_dir() || !is_git_repo(path) {
        return false;
    }
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    KNOWN_DISPATCH_SIBLINGS.contains(&name) || path.join("kit.toml").is_file()
}

fn push_repo_path(
    repos: &mut Vec<(String, bool)>,
    seen: &mut std::collections::BTreeSet<String>,
    path: PathBuf,
    on: bool,
) {
    let stored = path.to_string_lossy().into_owned();
    if seen.insert(stored.clone()) {
        repos.push((stored, on));
    }
}

/// Basename of a stored Dispatch repo path (or the string itself if nameless).
pub(crate) fn repo_basename(stored: &str) -> &str {
    Path::new(stored)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(stored)
}

/// TUI label for a stored repo path: basename, plus ` · this` when it is cwd.
pub(crate) fn format_repo_label(stored: &str, cwd: Option<&Path>) -> String {
    let base = repo_basename(stored);
    if let Some(cwd) = cwd
        && Path::new(stored) == cwd
    {
        return format!("{base} · this");
    }
    base.to_string()
}

fn repo_matches_hint(stored: &str, hint: &str) -> bool {
    stored == hint || repo_basename(stored) == hint
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

    pub fn selected_personas(&self) -> Vec<Persona> {
        self.personas
            .iter()
            .filter(|(_, on)| *on)
            .map(|(p, _)| *p)
            .collect()
    }

    pub fn fanout_count(&self) -> usize {
        self.selected_repos().len() * self.selected_agents().len() * self.selected_personas().len()
    }
}

/// One item on the shared Board queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardTask {
    pub id: u64,
    pub title: String,
    pub repo_hint: String,
    pub agent_hint: String,
    pub persona: Persona,
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
    /// Full-screen help overlay (`?`). Esc / `?` dismiss.
    pub help_open: bool,
    /// Agent readiness from launch-time probe (`name`, ready). Empty until set.
    pub agents_probe: Vec<(String, bool)>,
    /// Control Room table filter (`f`).
    pub run_filter: RunFilter,
    /// Where receipts live. A run's diff is read from `<id>/diff.patch` here
    /// once its terminal state is out: the engine writes the receipt first.
    pub runs_dir: Option<PathBuf>,
}

/// Shown when `k` / `r` land on a `--demo` fixture row.
const DEMO_ROW_FLASH: &str = "demo row — press d to dispatch a real run";

/// One run as the Control Room / detail view-model (TUI-local).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRow {
    pub id: RunId,
    pub repo: String,
    pub agent: String,
    /// Role this run is acting as. TUI-local; not on the engine contract.
    pub persona: Persona,
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
    /// `--demo` fixture row. Never reaches the engine: `k` / `r` only flash.
    pub demo: bool,
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
            persona: Persona::Eng,
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
            demo: false,
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

    /// The task's first non-blank line. Retry tasks carry the gate failure on
    /// later lines; a newline inside a one-line cell shifts what follows it.
    pub fn task_line(&self) -> &str {
        self.task
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim_end()
    }

    /// The repo's folder name, as Dispatch shows it; the engine keeps the path.
    pub fn repo_name(&self) -> &str {
        repo_basename(&self.repo)
    }

    /// Why a FAIL or ERROR row failed, for its `^ …` line. An ERROR run's
    /// reason is the engine's last `kit: …` line (`codex exited with code 1`),
    /// even when a gate ran after the agent failed; the gate comes second.
    pub fn failure_summary(&self) -> Option<String> {
        if self.state == RunState::Error
            && let Some(line) = self
                .output
                .lines()
                .rev()
                .find_map(|l| l.strip_prefix("kit: "))
        {
            return Some(line.strip_prefix("run failed: ").unwrap_or(line).to_owned());
        }
        self.gate_summary()
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

    /// Control Room AGENT cell: `codex·eng`.
    pub fn agent_cell(&self) -> String {
        format!("{}·{}", self.agent, self.persona.label())
    }

    /// Append an output chunk, keeping the tail within the display cap.
    /// Read the receipt's `diff.patch` for the Diff pane, bounded like the
    /// stream. A missing file (a run killed before it started) leaves it empty.
    pub fn load_diff(&mut self, runs_dir: &Path) {
        let Ok(bytes) = std::fs::read(runs_dir.join(&self.id.0).join("diff.patch")) else {
            return;
        };
        let text = String::from_utf8_lossy(&bytes);
        self.diff = if text.len() > OUTPUT_DISPLAY_CAP_BYTES {
            let mut end = OUTPUT_DISPLAY_CAP_BYTES;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            format!(
                "{}\n… diff cut at {} KiB; the receipt has all of it\n",
                &text[..end],
                OUTPUT_DISPLAY_CAP_BYTES / 1024
            )
        } else {
            text.into_owned()
        };
    }

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
            help_open: false,
            agents_probe: Vec::new(),
            run_filter: RunFilter::All,
            runs_dir: None,
        }
    }

    /// Seed launch-time agent readiness (from `kit_agents::probe_all`).
    pub fn set_agents_probe(&mut self, probe: Vec<(String, bool)>) {
        self.agents_probe = probe;
        self.apply_dispatch_agent_defaults();
        self.dirty = true;
    }

    /// When a probe is present, select the first ready agent and deselect the rest.
    /// Empty probe (tests / `--demo` without probe) keeps the form default (codex on).
    pub fn apply_dispatch_agent_defaults(&mut self) {
        if self.agents_probe.is_empty() {
            return;
        }
        let first_ready = self
            .agents_probe
            .iter()
            .find(|(_, ok)| *ok)
            .map(|(n, _)| n.clone());
        for (name, on) in &mut self.dispatch.agents {
            *on = first_ready.as_ref() == Some(name);
        }
    }

    /// Compact strip for the Control Room header, e.g. `codex·claude·grok ready`.
    pub fn agents_strip(&self) -> String {
        if self.agents_probe.is_empty() {
            return String::new();
        }
        let ready: Vec<&str> = self
            .agents_probe
            .iter()
            .filter(|(_, ok)| *ok)
            .map(|(n, _)| n.as_str())
            .collect();
        let missing: Vec<&str> = self
            .agents_probe
            .iter()
            .filter(|(_, ok)| !*ok)
            .map(|(n, _)| n.as_str())
            .collect();
        match (ready.is_empty(), missing.is_empty()) {
            (true, true) => String::new(),
            (false, true) => format!("{} ready", ready.join("·")),
            (true, false) => format!("{} missing — kit doctor", missing.join("·")),
            (false, false) => format!(
                "{} ready  ·  {} missing",
                ready.join("·"),
                missing.join("·")
            ),
        }
    }

    /// The strip for narrow headers: `claude ✓  3 missing`.
    pub fn agents_strip_short(&self) -> String {
        if self.agents_probe.is_empty() {
            return String::new();
        }
        let ready: Vec<&str> = self
            .agents_probe
            .iter()
            .filter(|(_, ok)| *ok)
            .map(|(n, _)| n.as_str())
            .collect();
        let missing = self.agents_probe.len() - ready.len();
        match (ready.is_empty(), missing) {
            (true, _) => "no agents — kit doctor".into(),
            (false, 0) => format!("{} ✓", ready.join("·")),
            (false, n) => format!("{} ✓  {n} missing", ready.join("·")),
        }
    }

    /// How many agents reported ready at launch.
    pub fn agents_ready_count(&self) -> usize {
        self.agents_probe.iter().filter(|(_, ok)| *ok).count()
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
                let flash_active = self.flash.is_some();
                let live = self
                    .runs
                    .iter()
                    .any(|r| matches!(r.state, RunState::Running | RunState::Gating));
                // Spinner advances every 2 ticks (10 Hz at TICK_HZ=20). Idle rooms
                // still skip redraw. KIT_MOTION=off keeps the resting label.
                let spinner_tick = live && next.is_multiple_of(2);
                self.motion && (spinner_tick || flash_active)
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
        // Help overlay captures all keys until dismissed.
        if self.help_open {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.help_open = false;
                }
                _ => {}
            }
            return Action::None;
        }
        // Global help — except while typing the dispatch task.
        if matches!(key.code, KeyCode::Char('?'))
            && !(self.screen == Screen::Dispatch && self.dispatch.focus == DispatchFocus::Task)
        {
            self.help_open = true;
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
                self.prepare_dispatch_repos();
                self.screen = Screen::Dispatch;
                Action::None
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.screen = Screen::Board;
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Char('K') => self.request_kill(),
            KeyCode::Char('r') | KeyCode::Char('R') => self.request_retry(),
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.run_filter = self.run_filter.next();
                self.set_flash(format!("filter {}", self.run_filter.label()));
                // Keep selection valid under the new filter.
                let order = self.display_order();
                if order.is_empty() {
                    self.selected_id = None;
                } else if self
                    .selected_id
                    .as_ref()
                    .is_none_or(|id| !order.iter().any(|&i| self.runs[i].id == *id))
                {
                    self.selected_id = Some(self.runs[order[0]].id.clone());
                }
                Action::None
            }
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
                // SPEC: q quits only from Control Room.
                self.set_flash("q quits from Control Room — Esc to go back");
                Action::None
            }
            KeyCode::Tab => {
                self.dispatch.focus = match self.dispatch.focus {
                    DispatchFocus::Repos => DispatchFocus::Agents,
                    DispatchFocus::Agents => DispatchFocus::Personas,
                    DispatchFocus::Personas => DispatchFocus::Task,
                    DispatchFocus::Task => DispatchFocus::Repos,
                };
                self.dispatch.list_cursor = 0;
                Action::None
            }
            KeyCode::BackTab => {
                self.dispatch.focus = match self.dispatch.focus {
                    DispatchFocus::Repos => DispatchFocus::Task,
                    DispatchFocus::Agents => DispatchFocus::Repos,
                    DispatchFocus::Personas => DispatchFocus::Agents,
                    DispatchFocus::Task => DispatchFocus::Personas,
                };
                self.dispatch.list_cursor = 0;
                Action::None
            }
            KeyCode::Up => {
                match self.dispatch.focus {
                    DispatchFocus::Repos | DispatchFocus::Agents | DispatchFocus::Personas => {
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
                    DispatchFocus::Personas => {
                        let max = self.dispatch.personas.len().saturating_sub(1);
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
            DispatchFocus::Personas => {
                if let Some((_, on)) = self.dispatch.personas.get_mut(self.dispatch.list_cursor) {
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
        let personas = self.dispatch.selected_personas();
        if repos.is_empty() || agents.is_empty() {
            self.set_flash("select at least one repo and one agent");
            return Action::None;
        }
        if personas.is_empty() {
            self.set_flash("select at least one persona — eng / product / design / qa");
            return Action::None;
        }
        let n = repos.len() * agents.len() * personas.len();
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
                for persona in &personas {
                    let id = RunId::new();
                    if first_id.is_none() {
                        first_id = Some(id.clone());
                    }
                    jobs.push(DispatchJob {
                        id: id.clone(),
                        repo: repo.clone(),
                        agent: agent.clone(),
                        task: persona.wrap_task(&task),
                    });
                    let mut row = RunRow::new(id, repo.clone(), agent.clone(), task.clone());
                    row.persona = *persona;
                    row.state = RunState::Queued;
                    self.upsert_run(row);
                }
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
                self.set_flash("q quits from Control Room — Esc to go back");
                Action::None
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
                        .map(|p| repo_basename(p).to_string())
                        .unwrap_or_else(|| "kit".into()),
                    agent_hint: self
                        .dispatch
                        .selected_agents()
                        .first()
                        .copied()
                        .unwrap_or("codex")
                        .to_string(),
                    persona: self
                        .dispatch
                        .selected_personas()
                        .first()
                        .copied()
                        .unwrap_or_default(),
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
                self.prepare_dispatch_repos();
                self.dispatch.task = item.title;
                for (name, on) in &mut self.dispatch.repos {
                    *on = repo_matches_hint(name, &item.repo_hint);
                }
                if !self
                    .dispatch
                    .repos
                    .iter()
                    .any(|(n, _)| repo_matches_hint(n, &item.repo_hint))
                {
                    let hint = Path::new(&item.repo_hint);
                    if hint.is_absolute() && hint.is_dir() {
                        self.dispatch
                            .repos
                            .insert(0, (item.repo_hint.clone(), true));
                    } else if let Some((_, on)) = self.dispatch.repos.first_mut() {
                        *on = true;
                    }
                }
                for (name, on) in &mut self.dispatch.agents {
                    *on = *name == item.agent_hint;
                }
                for (persona, on) in &mut self.dispatch.personas {
                    *on = *persona == item.persona;
                }
                self.dispatch.focus = DispatchFocus::Task;
                self.screen = Screen::Dispatch;
                Action::None
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.prepare_dispatch_repos();
                self.screen = Screen::Dispatch;
                Action::None
            }
            _ => Action::None,
        }
    }

    fn on_detail_key(&mut self, key: KeyEvent, pane: DetailPane) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.set_flash("q quits from Control Room — Esc to go back");
                Action::None
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
                self.prepare_dispatch_repos();
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
            self.set_flash("no runs — press d to dispatch  ·  kit --demo");
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
        if row.demo {
            self.set_flash(DEMO_ROW_FLASH);
            return Action::None;
        }
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
        if row.demo {
            self.set_flash(DEMO_ROW_FLASH);
            return Action::None;
        }
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
            task: row.persona.wrap_task(&task),
        };

        let mut queued = RunRow::new(new_id.clone(), row.repo, row.agent, task);
        queued.persona = row.persona;
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

    pub(crate) fn set_flash(&mut self, msg: impl Into<String>) {
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
    /// Honors [`RunFilter`].
    pub fn display_order(&self) -> Vec<usize> {
        let mut idxs: Vec<usize> = (0..self.runs.len())
            .filter(|&i| self.run_filter.matches(self.runs[i].state))
            .collect();
        idxs.sort_by(|&a, &b| {
            let ra = &self.runs[a];
            let rb = &self.runs[b];
            state_rank(ra.state)
                .cmp(&state_rank(rb.state))
                .then_with(|| ra.seq.cmp(&rb.seq))
        });
        idxs
    }

    /// Seed Dispatch repos from cwd + honest sibling paths, then apply agent defaults.
    fn prepare_dispatch_repos(&mut self) {
        self.dispatch.repos = seed_dispatch_repos();
        self.apply_dispatch_agent_defaults();
        self.dispatch.list_cursor = 0;
        self.dispatch.focus = DispatchFocus::Repos;
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
            if row.persona == Persona::Eng && existing.persona != Persona::Eng {
                row.persona = existing.persona;
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
        let runs_dir = self.runs_dir.clone();
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
                if state.is_terminal()
                    && !row.demo
                    && let Some(dir) = runs_dir
                {
                    row.load_diff(&dir);
                }
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

    pub fn queued_count(&self) -> usize {
        self.runs
            .iter()
            .filter(|r| matches!(r.state, RunState::Queued))
            .count()
    }

    pub fn gated_count(&self) -> usize {
        self.runs
            .iter()
            .filter(|r| matches!(r.state, RunState::Gating))
            .count()
    }

    pub fn fail_count(&self) -> usize {
        self.runs
            .iter()
            .filter(|r| matches!(r.state, RunState::Fail | RunState::Error))
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
        r0.persona = Persona::Eng;
        r0.seq = 0;
        self.upsert_run(r0);

        let mut r1 = RunRow::new(
            RunId("01FIXRUN1KITGROK0000000000".into()),
            "kit",
            "grok",
            "frame clock",
        );
        r1.state = RunState::Gating;
        r1.active_since_tick = Some(0);
        r1.worktree = Some(PathBuf::from("/tmp/kit-wt-01FIXRUN1"));
        r1.output = "grok: running kit.toml gate\nfmt ok\nclippy …\ncargo test --workspace".into();
        r1.persona = Persona::Eng;
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
        r2.persona = Persona::Eng;
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
                    summary: Some("tsc: 3 errors — Type 'string' is not assignable".into()),
                    duration: Duration::from_secs(8),
                },
            ],
            scope_violations: vec![],
            firewall_blocks: vec![],
            duration: Duration::from_secs(10),
        });
        r3.persona = Persona::Eng;
        r3.seq = 3;
        let fail_id = r3.id.clone();
        self.upsert_run(r3);

        // Fixture rows are for looking at; a real `k` / `r` would start a billable
        // run in the launch repo with the fixture brief.
        for row in &mut self.runs {
            row.demo = true;
        }

        // Product moment: land on FAIL so gate wash + `r` retry are visible immediately.
        self.selected_id = Some(fail_id);
        self.set_flash("FAIL · enter open · r retry");

        // Board fixture items for F4 snapshots / QA.
        self.board = vec![
            BoardTask {
                id: 1,
                title: "port guard.js".into(),
                repo_hint: "kit".into(),
                agent_hint: "codex".into(),
                persona: Persona::Eng,
                done: false,
            },
            BoardTask {
                id: 2,
                title: "frame clock".into(),
                repo_hint: "kit".into(),
                agent_hint: "grok".into(),
                persona: Persona::Design,
                done: false,
            },
            BoardTask {
                id: 3,
                title: "fix red CI".into(),
                repo_hint: "trenchwire".into(),
                agent_hint: "codex".into(),
                persona: Persona::Eng,
                done: true,
            },
        ];
        self.board_seq = 4;
        self.board_selected = 0;
        self.dirty = true;
    }
}

/// Display sort rank — lower is higher in the table.
/// FAIL/ERROR surface just under live work so the proof loop stays visible.
fn state_rank(state: RunState) -> u8 {
    match state {
        RunState::Running => 0,
        RunState::Gating => 1,
        RunState::Fail => 2,
        RunState::Error => 3,
        RunState::Queued => 4,
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

/// Braille spinner — Unicode, not a Nerd Font. Resting frame is unused
/// when motion is off (label stays `RUN 2m` so snapshots and reduced-motion
/// users see a still Control Room).
const RUN_SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];

/// Public helpers used by the UI for state/gate labels.
pub fn format_state_label(run: &RunRow, clock: &Clock, motion: bool) -> String {
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
    let live = matches!(run.state, RunState::Running | RunState::Gating);
    let mut out = String::new();
    if motion && live {
        let i = clock.frame(RUN_SPINNER.len(), 2);
        out.push(RUN_SPINNER[i]);
        out.push(' ');
    }
    out.push_str(label);
    if live && !elapsed.is_empty() {
        out.push(' ');
        out.push_str(&elapsed);
    }
    out
}

pub fn format_gate_label(run: &RunRow) -> String {
    match (&run.state, &run.gate) {
        (_, Some(g)) if is_vacuous_gate(g) => "UNCONFIGURED".into(),
        (_, Some(g)) if g.passed => "PASS".into(),
        (_, Some(_)) => "FAIL".into(),
        (RunState::Pass, None) => "PASS".into(),
        (RunState::Fail, None) => "FAIL".into(),
        _ => "--".into(),
    }
}

/// Vacuous pass: no checks and no violations (CEO stamp P2 → UNCONFIGURED).
pub fn is_vacuous_gate(g: &GateOutcome) -> bool {
    g.passed && g.checks.is_empty() && g.scope_violations.is_empty() && g.firewall_blocks.is_empty()
}

/// `850ms`, `12.0s`: the gate log reads in seconds once it takes one.
fn format_gate_duration(d: std::time::Duration) -> String {
    if d.as_millis() < 1000 {
        format!("{}ms", d.as_millis())
    } else {
        format!("{:.1}s", d.as_secs_f64())
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
        Some(g) if is_vacuous_gate(g) => {
            vec![
                "OVERALL  UNCONFIGURED".into(),
                String::new(),
                "No gate checks configured and none inferred.".into(),
                "Add a [gate] section to kit.toml, or install cargo/npm tooling.".into(),
                "Live runs exit non-zero on vacuous unless --allow-vacuous.".into(),
            ]
        }
        Some(g) => {
            let mut lines = Vec::new();
            let verdict = if g.passed { "PASS" } else { "FAIL" };
            lines.push(format!(
                "OVERALL  {verdict}  ({})",
                format_gate_duration(g.duration)
            ));
            lines.push(String::new());
            let label_width = g
                .checks
                .iter()
                .map(|c| c.label.chars().count())
                .max()
                .unwrap_or(0);
            for c in &g.checks {
                let status = match c.status {
                    CheckStatus::Pass => "PASS",
                    CheckStatus::Fail => "FAIL",
                    CheckStatus::Skipped => "SKIP",
                    CheckStatus::TimedOut => "TIME",
                };
                let mut line = format!("{status}  {:<label_width$}  {}", c.label, c.command);
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
    fn motion_off_running_row_does_not_dirty_on_tick() {
        let mut app = App::with_motion(false);
        let id = RunId("01MOTIONOFF000000000000000".into());
        app.upsert_run({
            let mut r = RunRow::new(id, "kit", "codex", "tick");
            r.state = RunState::Running;
            r.active_since_tick = Some(0);
            r
        });
        app.clear_dirty();
        for _ in 0..=TICK_HZ {
            app.update(AppEvent::AnimationTick);
        }
        assert!(
            !app.is_dirty(),
            "KIT_MOTION=off must keep a RUNNING row on its resting frame"
        );
        assert_eq!(app.clock.tick, TICK_HZ + 1);
    }

    #[test]
    fn active_run_dirties_on_spinner_cadence() {
        let mut app = App::with_motion(true);
        let id = RunId("01ACTIVE000000000000000000".into());
        app.upsert_run({
            let mut r = RunRow::new(id, "kit", "codex", "tick");
            r.state = RunState::Running;
            r.active_since_tick = Some(0);
            r
        });
        app.clear_dirty();
        app.update(AppEvent::AnimationTick);
        assert!(
            !app.is_dirty(),
            "odd ticks must not redraw (spinner every 2)"
        );
        app.update(AppEvent::AnimationTick);
        assert!(app.is_dirty(), "even ticks redraw a live RUNNING row");
    }

    #[test]
    fn running_label_spins_only_when_motion_on() {
        let mut run = RunRow::new(
            RunId("01SPIN00000000000000000000".into()),
            "kit",
            "codex",
            "t",
        );
        run.state = RunState::Running;
        run.active_since_tick = Some(0);
        let clock = Clock { tick: 2 };
        let moving = format_state_label(&run, &clock, true);
        let still = format_state_label(&run, &clock, false);
        assert!(
            moving.starts_with('⠙') && moving.contains("RUN"),
            "motion-on RUNNING uses the clock frame: {moving}"
        );
        assert_eq!(still, "RUN 0s", "motion-off keeps the resting word+elapsed");
        let done = RunRow::new(
            RunId("01DONE00000000000000000000".into()),
            "kit",
            "codex",
            "t",
        );
        assert_eq!(
            format_state_label(&done, &clock, true),
            "QUEUED",
            "idle states never spin"
        );
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

    /// A real run's Diff pane shows the receipt's diff.patch once it ends.
    #[test]
    fn finished_run_loads_its_diff_from_the_receipt() {
        let dir = std::env::temp_dir().join(format!("kit-tui-diff-{}", std::process::id()));
        let id = RunId("01DIFF00000000000000000000".into());
        std::fs::create_dir_all(dir.join(&id.0)).unwrap();
        let patch = "--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n";
        std::fs::write(dir.join(&id.0).join("diff.patch"), patch).unwrap();

        let mut app = App::with_motion(false);
        app.runs_dir = Some(dir.clone());
        app.upsert_run(RunRow::new(id.clone(), "shop", "claude", "edit x"));
        app.update(AppEvent::RunUpdate(
            id.clone(),
            RunDelta::State(RunState::Gating),
        ));
        assert_eq!(app.runs[0].diff, "", "not before the run ends");
        app.update(AppEvent::RunUpdate(id, RunDelta::State(RunState::Pass)));
        assert_eq!(app.runs[0].diff, patch);
        let _ = std::fs::remove_dir_all(dir);
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

    /// Seconds, not raw milliseconds, and the check columns line up.
    #[test]
    fn gate_log_reads_as_a_table() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let row = app
            .runs
            .iter()
            .find(|r| r.repo == "trenchwire")
            .unwrap()
            .clone();
        let log = gate_log_lines(&row);
        assert_eq!(log[0], "OVERALL  FAIL  (10.0s)", "{log:?}");
        let checks: Vec<&String> = log
            .iter()
            .filter(|l| l.starts_with("PASS") || l.starts_with("FAIL"))
            .collect();
        assert!(checks.len() >= 2, "{log:?}");
        let command_col = |l: &str| {
            let rest = l[4..].trim_start();
            let label_end = l.len() - rest.len() + rest.find(' ').unwrap();
            label_end + l[label_end..].len() - l[label_end..].trim_start().len()
        };
        let first = command_col(checks[0]);
        assert!(
            checks.iter().all(|l| command_col(l) == first),
            "commands not aligned: {checks:#?}"
        );
    }

    #[test]
    fn vacuous_gate_renders_unconfigured_not_pass() {
        let mut row = RunRow::new(
            RunId("01VACUOUS00000000000000000".into()),
            "kit",
            "codex",
            "x",
        );
        row.state = RunState::Pass;
        row.gate = Some(GateOutcome::vacuous());
        assert_eq!(format_gate_label(&row), "UNCONFIGURED");
        let log = gate_log_lines(&row);
        assert!(
            log[0].contains("UNCONFIGURED"),
            "gate log must not claim PASS for vacuous: {log:?}"
        );
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
    fn question_mark_toggles_help_overlay() {
        let mut app = App::with_motion(false);
        assert!(!app.help_open);
        app.update(key('?'));
        assert!(app.help_open);
        // Other keys ignored while open.
        app.update(key('d'));
        assert!(app.help_open);
        assert_eq!(app.screen, Screen::ControlRoom);
        app.update(code(KeyCode::Esc));
        assert!(!app.help_open);
        app.update(key('?'));
        assert!(app.help_open);
        app.update(key('?'));
        assert!(!app.help_open);
    }

    #[test]
    fn q_does_not_quit_from_nested_screens() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(code(KeyCode::Enter));
        assert!(matches!(app.screen, Screen::RunDetail { .. }));
        assert_eq!(app.update(key('q')), Action::None);
        assert!(!app.should_quit);
        assert!(matches!(app.screen, Screen::RunDetail { .. }));

        app.update(code(KeyCode::Esc));
        app.update(key('b'));
        assert_eq!(app.screen, Screen::Board);
        assert_eq!(app.update(key('q')), Action::None);
        assert!(!app.should_quit);
        assert_eq!(app.screen, Screen::Board);

        app.update(code(KeyCode::Esc));
        app.update(key('d'));
        assert_eq!(app.screen, Screen::Dispatch);
        assert_eq!(app.update(key('q')), Action::None);
        assert!(!app.should_quit);
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

    /// `--demo` fixture rows must never start, kill or retry a real run (G1).
    #[test]
    fn demo_rows_never_reach_the_engine() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let before = app.runs.len();
        for state in [RunState::Running, RunState::Fail] {
            let id = app
                .runs
                .iter()
                .find(|r| r.state == state)
                .map(|r| r.id.clone())
                .expect("fixture has the state");
            app.selected_id = Some(id);
            for k in ['k', 'r'] {
                assert_eq!(app.update(key(k)), Action::None, "{k} on {state:?}");
                assert!(
                    app.flash_message()
                        .is_some_and(|m| m.contains("demo row") && m.contains("press d")),
                    "{k} on a demo row must teach d: {:?}",
                    app.flash_message()
                );
            }
        }
        assert_eq!(app.runs.len(), before, "no retry row was queued");
    }

    /// Engine rows (not fixtures) keep the real kill / retry path.
    fn load_fixture_as_engine_rows(app: &mut App) {
        app.load_prd_fixture();
        for row in &mut app.runs {
            row.demo = false;
        }
    }

    #[test]
    fn engine_keys_emit_actions_with_flash() {
        let mut app = App::with_motion(false);
        load_fixture_as_engine_rows(&mut app);
        // Demo fixture selects FAIL first — pick a Running row for kill.
        let running_id = app
            .runs
            .iter()
            .find(|r| r.state == RunState::Running)
            .map(|r| r.id.clone())
            .expect("fixture has Running run");
        app.selected_id = Some(running_id.clone());
        let kill = app.update(key('k'));
        match kill {
            Action::KillSelected { id } => {
                assert_eq!(id, running_id);
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
                assert_eq!(
                    job.task.matches("\n---\nRole (").count(),
                    1,
                    "retry must wrap the persona brief once: {}",
                    job.task
                );
            }
            other => panic!("expected RetrySelected, got {other:?}"),
        }
        assert_eq!(app.runs.len(), before + 1);
    }

    #[test]
    fn filter_cycles_and_hides_rows() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        assert_eq!(app.run_filter, RunFilter::All);
        let all_n = app.display_order().len();
        assert!(all_n >= 3);

        app.update(key('F'));
        assert_eq!(app.run_filter, RunFilter::Fail);
        let fail_n = app.display_order().len();
        assert_eq!(fail_n, 1);
        assert!(
            app.display_order()
                .iter()
                .all(|&i| matches!(app.runs[i].state, RunState::Fail | RunState::Error))
        );

        app.update(key('f'));
        assert_eq!(app.run_filter, RunFilter::Running);
        assert!(app.display_order().len() >= 2);

        app.update(key('f')); // Done
        app.update(key('f')); // All
        assert_eq!(app.run_filter, RunFilter::All);
        assert_eq!(app.display_order().len(), all_n);
    }

    #[test]
    fn retry_rejects_non_fail() {
        let mut app = App::with_motion(false);
        load_fixture_as_engine_rows(&mut app);
        // Demo selects FAIL — force a Running selection so retry is rejected.
        let running_id = app
            .runs
            .iter()
            .find(|r| r.state == RunState::Running)
            .map(|r| r.id.clone())
            .expect("fixture has Running run");
        app.selected_id = Some(running_id);
        assert_eq!(app.update(key('r')), Action::None);
        assert!(
            app.flash_message()
                .is_some_and(|m| m.contains("retry only"))
        );
    }

    #[test]
    fn empty_room_enter_and_g_flash_instead_of_silent() {
        let mut app = App::with_motion(false);
        assert!(app.runs.is_empty());
        app.update(code(KeyCode::Enter));
        assert_eq!(app.screen, Screen::ControlRoom);
        assert!(
            app.flash_message()
                .is_some_and(|m| m.contains("press d") && m.contains("demo")),
            "enter on empty room must teach the next key: {:?}",
            app.flash_message()
        );
        app.update(key('g'));
        assert_eq!(app.screen, Screen::ControlRoom);
        assert!(app.flash_message().is_some_and(|m| m.contains("press d")));
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
    fn seed_dispatch_repos_stores_absolute_cwd() {
        let repos = seed_dispatch_repos();
        assert!(!repos.is_empty());
        assert!(repos[0].1, "cwd is selected");
        assert!(
            Path::new(&repos[0].0).is_absolute() || repos[0].0 == ".",
            "stored repo must be an absolute path (or fallback .): {:?}",
            repos[0].0
        );
        assert!(repos.len() <= DISPATCH_REPO_CAP);
    }

    #[test]
    fn format_repo_label_uses_basename_and_this_suffix() {
        let cwd = std::env::current_dir().expect("cwd");
        let stored = cwd.to_string_lossy();
        let base = cwd.file_name().and_then(|n| n.to_str()).expect("basename");
        assert_eq!(
            format_repo_label(&stored, Some(&cwd)),
            format!("{base} · this")
        );
        assert_eq!(format_repo_label("/tmp/guardian", Some(&cwd)), "guardian");
    }

    #[test]
    fn dispatch_agent_defaults_select_first_ready() {
        let mut app = App::with_motion(false);
        assert!(
            app.dispatch
                .agents
                .iter()
                .any(|(n, on)| n == "codex" && *on),
            "empty probe keeps the codex default"
        );
        app.set_agents_probe(vec![("codex".into(), false), ("claude".into(), true)]);
        app.apply_dispatch_agent_defaults();
        assert_eq!(app.dispatch.selected_agents(), vec!["claude"]);
        assert!(
            !app.dispatch
                .agents
                .iter()
                .any(|(n, on)| n == "codex" && *on)
        );
    }

    #[test]
    fn dispatch_submit_sends_bare_agent_ids() {
        let mut app = App::with_motion(false);
        app.set_agents_probe(vec![("codex".into(), false), ("claude".into(), true)]);
        app.apply_dispatch_agent_defaults();
        app.dispatch.repos = vec![("kit".into(), true)];
        app.dispatch.task = "prove it".into();
        app.screen = Screen::Dispatch;
        match app.update(code(KeyCode::Enter)) {
            Action::DispatchSubmitted { jobs } => {
                assert_eq!(jobs.len(), 1);
                assert_eq!(jobs[0].agent, "claude");
                assert!(
                    !jobs[0].agent.contains("ready") && !jobs[0].agent.contains("missing"),
                    "submit must send the bare agent id, got {:?}",
                    jobs[0].agent
                );
            }
            other => panic!("expected DispatchSubmitted, got {other:?}"),
        }
    }

    #[test]
    fn dispatch_submit_fans_out_queued_runs() {
        let mut app = App::with_motion(false);
        // Explicit form — do not depend on cwd siblings for fan-out count.
        app.dispatch.repos = vec![("kit".into(), true), ("guardian".into(), true)];
        app.dispatch.agents = vec![("codex".into(), true)];
        app.dispatch.task = "ship the gate".into();
        app.screen = Screen::Dispatch;
        let before = app.runs.len();
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
    fn dispatch_fans_out_personas_and_keeps_row_task_clean() {
        let mut app = App::with_motion(false);
        app.dispatch.repos = vec![("kit".into(), true)];
        app.dispatch.agents = vec![("grok".into(), true)];
        for (p, on) in &mut app.dispatch.personas {
            *on = matches!(*p, Persona::Product | Persona::Design | Persona::Eng);
        }
        app.dispatch.task = "empty room first paint".into();
        app.screen = Screen::Dispatch;
        match app.update(code(KeyCode::Enter)) {
            Action::DispatchSubmitted { jobs } => {
                assert_eq!(jobs.len(), 3);
                assert!(jobs.iter().all(|j| j.agent == "grok"));
                assert!(jobs.iter().all(|j| j.task.contains("\n---\nRole (")));
                assert!(jobs.iter().any(|j| j.task.contains("Role (product)")));
                assert!(jobs.iter().any(|j| j.task.contains("Role (design)")));
                assert!(jobs.iter().any(|j| j.task.contains("Role (eng)")));
                // The user's task leads: it titles the receipt and the land commit.
                assert!(
                    jobs.iter()
                        .all(|j| j.task.lines().next() == Some("empty room first paint"))
                );
            }
            other => panic!("expected DispatchSubmitted, got {other:?}"),
        }
        assert_eq!(app.runs.len(), 3);
        assert!(app.runs.iter().all(|r| r.task == "empty room first paint"));
        let roles: Vec<_> = app.runs.iter().map(|r| r.persona).collect();
        assert!(roles.contains(&Persona::Product));
        assert!(roles.contains(&Persona::Design));
        assert!(roles.contains(&Persona::Eng));
    }

    #[test]
    fn dispatch_requires_a_persona() {
        let mut app = App::with_motion(false);
        app.dispatch.repos = vec![("kit".into(), true)];
        app.dispatch.agents = vec![("codex".into(), true)];
        for (_, on) in &mut app.dispatch.personas {
            *on = false;
        }
        app.dispatch.task = "no role".into();
        app.screen = Screen::Dispatch;
        assert!(matches!(app.update(code(KeyCode::Enter)), Action::None));
        assert_eq!(app.screen, Screen::Dispatch);
        assert!(
            app.flash_message().is_some_and(|m| m.contains("persona")),
            "empty persona set must teach the next key: {:?}",
            app.flash_message()
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
        assert_eq!(app.dispatch.selected_personas(), vec![Persona::Eng]);
    }

    #[test]
    fn prd_fixture_matches_section_4_2_shape() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        assert_eq!(app.running_count(), 1);
        assert_eq!(app.gated_count(), 1);
        assert_eq!(app.fail_count(), 1);
        assert_eq!(app.runs.len(), 4);
        let order = app.display_order();
        assert_eq!(app.runs[order[0]].state, RunState::Running);
        assert_eq!(app.runs[order[1]].state, RunState::Gating);
        assert_eq!(app.runs[order[2]].state, RunState::Fail);
        assert!(
            app.runs.iter().all(|r| r.persona == Persona::Eng),
            "demo table is ENG-only so AGENT reads vendor, not a third axis"
        );
        let fail = app.runs.iter().find(|r| r.state == RunState::Fail).unwrap();
        assert_eq!(
            fail.gate_summary().as_deref(),
            Some("tsc: 3 errors — Type 'string' is not assignable")
        );
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
