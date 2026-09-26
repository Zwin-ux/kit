//! Control Room, Run Detail, Dispatch, and Board frames.

mod board;
mod common;
mod control_room;
mod dispatch;
mod run_detail;

use crate::app::{App, Screen};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Line;

use crate::theme::Theme;
use common::draw_help_overlay;
use ratatui::widgets::Clear;

/// Paint the active screen into `frame`.
pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::ControlRoom => control_room::draw(frame, app),
        Screen::RunDetail { pane } => run_detail::draw(frame, app, pane),
        Screen::Attached => run_detail::draw_attached(frame, app),
        Screen::Dispatch => dispatch::draw(frame, app),
        Screen::Board => board::draw(frame, app),
    }
    if app.help_open {
        let theme = Theme::resolve();
        let area = frame.area();
        frame.render_widget(Clear, area);
        // At the 80×14 contract, gutters mash header/footer through the panel.
        let tight = area.width < 90 || area.height < 18;
        let lines = help_lines(app);
        // Vertical gutters only from rows the keys leave free: every line shows.
        let need = u16::try_from(lines.len())
            .unwrap_or(u16::MAX)
            .saturating_add(2);
        let v_gutter = if tight {
            Constraint::Length(0)
        } else {
            Constraint::Length((area.height.saturating_sub(need) / 2).min(area.height / 8))
        };
        let h_gutter = if tight {
            Constraint::Length(0)
        } else {
            Constraint::Percentage(8)
        };
        let v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([v_gutter, Constraint::Min(8), v_gutter])
            .split(area);
        let h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([h_gutter, Constraint::Min(20), h_gutter])
            .split(v[1]);
        draw_help_overlay(frame, h[1], &theme, lines);
    }
}

fn help_lines(app: &App) -> Vec<Line<'static>> {
    // No blank rows: at 14 terminal rows the panel inner height is ~12.
    // Kill and retry must appear in that window (k is kill, not nav).
    // The panel border already reads "help"; every row here is a key.
    let mut lines: Vec<Line<'static>> = vec![
        Line::from("Global"),
        Line::from("  ?          toggle this help"),
        Line::from("  Esc        back / close help"),
        Line::from("  q          quit (Control Room only; disabled while attached)"),
    ];
    match app.screen {
        Screen::ControlRoom => {
            lines.extend([
                Line::from("Control Room"),
                Line::from("  ↑↓         move selection"),
                Line::from("  Enter      open run detail (stream)"),
                Line::from("  g          open gate log"),
                Line::from("  d          dispatch fan-out"),
                Line::from("  k          kill selected run"),
                Line::from("  r          retry FAIL only"),
                Line::from("  b / f      board · filter ALL → FAIL → RUN → DONE"),
            ]);
        }
        Screen::RunDetail { .. } => {
            lines.extend([
                Line::from("Run detail"),
                Line::from("  Tab / 1 2 3   stream · gate · diff"),
                Line::from("  a             attach (PTY stub → 1.0.1)"),
                Line::from("  k / r         kill / retry"),
                Line::from("  End           follow stream tail"),
            ]);
        }
        Screen::Attached => {
            lines.extend([
                Line::from("Attached"),
                Line::from("  Esc        detach without killing"),
                Line::from("  q          disabled"),
            ]);
        }
        Screen::Dispatch => {
            lines.extend([
                Line::from("Dispatch"),
                Line::from("  ↑↓         move in the focused list"),
                Line::from("  Tab        next field"),
                Line::from("  Space      toggle repo/agent/persona"),
                Line::from("  type       task prompt"),
                Line::from("  Enter      submit fan-out"),
            ]);
        }
        Screen::Board => {
            lines.extend([
                Line::from("Board (prefill-only in 1.0)"),
                Line::from("  n          new task"),
                Line::from("  Enter      prefill Dispatch"),
                Line::from("  Space      toggle done"),
                Line::from("  x          remove"),
            ]);
        }
    }
    lines
}

/// Render the active screen into a string for snapshot tests.
#[cfg(test)]
pub fn render_to_string(app: &App, width: u16, height: u16) -> String {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test backend");
    terminal.draw(|f| draw(f, app)).expect("draw");
    format!("{}", terminal.backend())
}

/// Render the active screen into a styled buffer (for colour assertions).
#[cfg(test)]
pub fn render_to_buffer(app: &App, width: u16, height: u16) -> ratatui::buffer::Buffer {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test backend");
    terminal.draw(|f| draw(f, app)).expect("draw");
    terminal.backend().buffer().clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, DetailPane, Screen};
    use crate::event::AppEvent;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn code(code: KeyCode) -> AppEvent {
        AppEvent::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn snapshot_footer(frame: &str) -> &str {
        frame
            .lines()
            .rev()
            .find(|line| line.contains('['))
            .unwrap_or(frame)
    }

    #[test]
    fn empty_control_room_snapshot() {
        let app = App::with_motion(false);
        let frame = render_to_string(&app, 80, 12);
        let footer = snapshot_footer(&frame);
        assert!(
            footer.contains('?') && footer.contains('r'),
            "80-col footer must keep help and retry: {footer}"
        );
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn empty_control_room_probe_all_missing_snapshot() {
        let mut app = App::with_motion(false);
        app.set_agents_probe(vec![
            ("codex".into(), false),
            ("claude".into(), false),
            ("grok".into(), false),
            ("ollama".into(), false),
        ]);
        let frame = render_to_string(&app, 80, 12);
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn empty_control_room_probe_ready_snapshot() {
        let mut app = App::with_motion(false);
        app.set_agents_probe(vec![("codex".into(), true), ("claude".into(), false)]);
        let frame = render_to_string(&app, 80, 12);
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn empty_control_room_snapshot_narrow() {
        let app = App::with_motion(false);
        let frame = render_to_string(&app, 60, 8);
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn empty_control_room_fox_snapshot() {
        let app = App::with_motion(false);
        let frame = render_to_string(&app, 100, 30);
        assert!(frame.contains("▄███▀████▀█████"), "the fox rests: {frame}");
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn fox_wags_on_the_clock_and_never_moves_the_copy() {
        let mut app = App::with_motion(true);
        let rest = render_to_string(&app, 100, 30);
        app.clock.tick = crate::fox::WAG_PERIOD - 9; // mid-wag
        let wag = render_to_string(&app, 100, 30);
        assert_ne!(rest, wag, "motion on: the tail moves");
        let (rest_lines, wag_lines): (Vec<_>, Vec<_>) =
            (rest.lines().collect(), wag.lines().collect());
        let moved: Vec<usize> = (0..rest_lines.len())
            .filter(|&i| rest_lines[i] != wag_lines[i])
            .collect();
        assert!(moved.len() <= 3, "only the tail rows change: {moved:?}");
        for text in ["No runs yet", "press d to dispatch"] {
            let row = |f: &str| f.lines().position(|l| l.contains(text));
            assert_eq!(row(&rest), row(&wag), "{text} stays put");
        }
    }

    #[test]
    fn fox_hides_when_the_room_is_short() {
        let app = App::with_motion(false);
        let frame = render_to_string(&app, 100, 18);
        assert!(!frame.contains('▀'), "no half-drawn fox: {frame}");
        assert!(frame.contains("No runs yet"));
    }

    #[test]
    fn too_small_snapshot() {
        let app = App::with_motion(false);
        let frame = render_to_string(&app, 40, 8);
        assert!(
            frame.contains("need 60×12") && frame.contains("40×8"),
            "too-small floor must teach the next action: {frame}"
        );
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn populated_control_room_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let frame = render_to_string(&app, 80, 14);
        assert!(
            frame.contains("^ tsc: 3 errors"),
            "FAIL first-error must stay readable at 80×14: {frame}"
        );
        assert!(
            frame.contains("GATING") && !frame.contains("GATED"),
            "header must say GATING, never 0 GATED: {frame}"
        );
        assert!(
            !frame.contains('⠋') && !frame.contains('⠙'),
            "motion-off snapshots must rest: {frame}"
        );
        let header = frame.lines().next().unwrap_or("");
        assert!(
            header.contains("FAIL ·") || header.contains("r retry"),
            "80-col header must keep the demo FAIL flash: {header}"
        );
        insta::assert_snapshot!(frame);
    }

    /// Sixteen runs against an eight-way limit: eight RUNNING, eight QUEUED.
    fn sixteen_way_app() -> App {
        use crate::app::RunRow;
        use kit_core::{RunId, RunState};
        let mut app = App::with_motion(false);
        for i in 0..16u64 {
            let mut row = RunRow::new(
                RunId(format!("01SIXTEENWAY{i:014}")),
                "kit",
                "codex",
                format!("task {i:02}"),
            );
            row.seq = i;
            if i < 8 {
                row.state = RunState::Running;
                row.active_since_tick = Some(0);
            }
            app.runs.push(row);
        }
        app.selected_id = Some(app.runs[0].id.clone());
        app
    }

    /// Rows that do not fit are counted, never silently dropped.
    #[test]
    fn control_room_counts_the_rows_it_cannot_show() {
        let app = sixteen_way_app();
        let frame = render_to_string(&app, 80, 14);
        assert!(frame.contains("task 00"), "{frame}");
        assert!(!frame.contains("task 15"), "{frame}");
        let shown = (0..16)
            .filter(|i| frame.contains(&format!("task {i:02}")))
            .count();
        assert!(
            frame.contains(&format!("↓ {} more", 16 - shown)),
            "hidden rows must be counted: {frame}"
        );
    }

    /// The selection never walks off the bottom of the table.
    #[test]
    fn control_room_keeps_the_selected_row_in_view() {
        let mut app = sixteen_way_app();
        for _ in 0..15 {
            app.update(code(KeyCode::Down));
        }
        let frame = render_to_string(&app, 80, 14);
        let selected = frame
            .lines()
            .find(|l| l.contains('▶'))
            .unwrap_or_else(|| panic!("selected row is off screen: {frame}"));
        assert!(selected.contains("task 15"), "{frame}");
        assert!(frame.contains("more above"), "{frame}");
        assert!(!frame.contains("more below"), "{frame}");
        insta::assert_snapshot!(frame);
    }

    /// Queued runs are work too: the header counts them.
    #[test]
    fn control_room_header_counts_queued_runs() {
        let app = sixteen_way_app();
        let frame = render_to_string(&app, 80, 14);
        let header = frame.lines().next().unwrap_or("");
        assert!(
            header.contains("8 RUNNING") && header.contains("8 QUEUED"),
            "{header}"
        );
        // At the 60-column floor the counts shrink rather than vanish.
        let frame = render_to_string(&app, 60, 12);
        let header = frame.lines().next().unwrap_or("");
        assert!(
            header.contains("8R") && header.contains("8Q") && header.contains("0F"),
            "{header}"
        );
    }

    /// Mid-size terminals keep every help line inside the panel.
    #[test]
    fn help_overlay_shows_its_last_line() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.help_open = true;
        for (w, h) in [(103, 19), (120, 18), (80, 14), (140, 40)] {
            let frame = render_to_string(&app, w, h);
            assert!(frame.contains("filter ALL"), "{w}x{h}\n{frame}");
        }
    }

    /// A retry task carries the gate failure on later lines. The table shows
    /// its first line only; newlines must never shift STATE and GATE.
    #[test]
    fn multi_line_task_keeps_columns_aligned() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let mut retry = app.runs[0].clone();
        retry.id = kit_core::RunId("01RETRYMULTILINE0000000000".into());
        retry.task = "fix red CI\n\n## Previous gate failure\ntsc: 3 errors".into();
        retry.state = kit_core::RunState::Queued;
        retry.seq = 9;
        app.runs.push(retry);
        let frame = render_to_string(&app, 80, 16);
        let line = frame
            .lines()
            .find(|l| l.trim_start_matches('"').starts_with('│') && l.contains("QUEUED"))
            .unwrap_or_else(|| panic!("{frame}"));
        assert!(!line.contains("Previous"), "{line}");
        let header = frame.lines().find(|l| l.contains("STATE")).unwrap();
        assert_eq!(
            line.find("QUEUED").map(|i| line[..i].chars().count()),
            header.find("STATE").map(|i| header[..i].chars().count()),
            "STATE column moved:\n{header}\n{line}"
        );
    }

    /// The FAIL wash is one solid band: every cell of the FAIL row and of its
    /// `^ first error` line, border to border.
    #[test]
    fn fail_row_wash_covers_the_whole_row() {
        let theme = crate::theme::Theme::resolve();
        if theme.fail_wash == ratatui::style::Color::Reset {
            return; // monochrome: no wash to check
        }
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let (w, h) = (80, 14);
        let buf = render_to_buffer(&app, w, h);
        let text = |y: u16| (0..w).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let fail_y = (0..h).find(|&y| text(y).contains("trenchwire")).unwrap();
        for y in [fail_y, fail_y + 1] {
            for x in 1..w - 1 {
                assert_eq!(
                    buf[(x, y)].bg,
                    theme.fail_wash,
                    "gap in the wash at ({x},{y}): {}",
                    text(y)
                );
            }
        }
    }

    /// With the agent strip showing (the real binary always probes), the
    /// counts still show, and they come before the strip.
    #[test]
    fn header_keeps_counts_beside_the_agent_strip() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.set_agents_probe(vec![
            ("codex".into(), false),
            ("claude".into(), true),
            ("grok".into(), false),
            ("ollama".into(), false),
        ]);
        // Past the demo's opening flash, which takes the width while it lives.
        for _ in 0..=crate::event::TICK_HZ * 2 {
            app.update(crate::event::AppEvent::AnimationTick);
        }
        assert_eq!(app.flash_message(), None);
        for width in [60u16, 80, 120] {
            let frame = render_to_string(&app, width, 14);
            let header = frame.lines().next().unwrap_or("");
            assert!(
                header.contains("1R") || header.contains("1 RUNNING"),
                "{width} cols: {header}"
            );
            assert!(
                header.contains("1F") || header.contains("1 FAIL"),
                "{width} cols: {header}"
            );
        }
        let frame = render_to_string(&app, 120, 14);
        let header = frame.lines().next().unwrap_or("");
        let counts = header.find("1 RUNNING").or(header.find("1R")).unwrap();
        let strip = header.find("claude").unwrap_or_else(|| panic!("{header}"));
        assert!(counts < strip, "{header}");
    }

    /// A live row's REPO cell is the folder name, as Dispatch shows it, not
    /// the absolute path the engine needs.
    #[test]
    fn repo_column_shows_the_folder_name() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.runs[0].repo = "/home/you/code/shop".into();
        let frame = render_to_string(&app, 80, 14);
        assert!(!frame.contains("/home/you"), "{frame}");
        assert!(frame.contains("shop"), "{frame}");
    }

    /// Run detail's header shows the folder name and the task's first line.
    #[test]
    fn run_detail_header_uses_folder_and_first_line() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let id = app.selected_id.clone().unwrap();
        let row = app.runs.iter_mut().find(|r| r.id == id).unwrap();
        row.repo = "/home/you/code/shop".into();
        row.task = "fix red CI\n\n## Previous gate failure\ntsc".into();
        app.screen = crate::app::Screen::RunDetail {
            pane: crate::app::DetailPane::default(),
        };
        let frame = render_to_string(&app, 100, 14);
        let header = frame.lines().next().unwrap();
        assert!(header.contains("KIT / RUN  shop"), "{header}");
        assert!(header.contains("fix red CI"), "{header}");
        assert!(!frame.contains("/home/you"), "{frame}");
    }

    /// An ERROR row has no gate; its `^` line carries the engine's reason.
    #[test]
    fn error_row_shows_its_reason() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let row = &mut app.runs[0];
        row.state = kit_core::RunState::Error;
        row.gate = None;
        row.output = "kit: run failed: codex is not installed\n".into();
        let frame = render_to_string(&app, 80, 14);
        assert!(frame.contains("^ codex is not installed"), "{frame}");
    }

    /// An agent that exits non-zero ends ERROR with a gate outcome; the row
    /// still says why the agent failed, not what the gate found.
    #[test]
    fn error_row_with_a_gate_shows_the_agent_failure() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let idx = app
            .runs
            .iter()
            .position(|r| r.gate.as_ref().is_some_and(|g| !g.passed))
            .expect("fixture has a gated FAIL run");
        let row = &mut app.runs[idx];
        row.state = kit_core::RunState::Error;
        row.output = "kit: codex exited with code 1\ngate: running\n".into();
        let frame = render_to_string(&app, 80, 14);
        assert!(frame.contains("^ codex exited with code 1"), "{frame}");
    }

    /// A live flash carries the next action: it shows whole, and the counts
    /// shrink or step aside for its 2 s.
    #[test]
    fn flash_shows_whole_beside_the_counts() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let flash = "demo row — press d to dispatch a real run";
        app.set_flash(flash);
        for w in [60u16, 80, 103, 120] {
            let frame = render_to_string(&app, w, 14);
            let header = frame.lines().next().unwrap();
            if w >= 80 {
                assert!(header.contains(flash), "{w}: {header}");
            }
            assert!(header.contains("· demo row"), "{w}: {header}");
        }
        // Without a flash the counts and the strip come back.
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let frame = render_to_string(&app, 120, 14);
        assert!(frame.lines().next().unwrap().contains("1 FAIL"), "{frame}");
    }

    /// Run detail's worktree line writes the home folder as `~`.
    #[test]
    fn worktree_path_is_tilde_shortened() {
        let Some(home) = std::env::var_os("HOME").filter(|h| !h.is_empty()) else {
            return;
        };
        let wt = std::path::Path::new(&home).join(".kit/worktrees/01ABC");
        let sep = std::path::MAIN_SEPARATOR;
        assert_eq!(
            super::common::tilde(&wt),
            format!(
                "~{sep}{}",
                std::path::Path::new(".kit/worktrees/01ABC").display()
            )
        );
        assert_eq!(
            super::common::tilde(std::path::Path::new("/elsewhere")),
            "/elsewhere"
        );
    }

    /// The selection rail on a non-FAIL row is a caret, not a reversed block.
    #[test]
    fn selection_rail_is_a_caret_off_fail_rows() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let idx = app
            .runs
            .iter()
            .position(|r| r.state == kit_core::RunState::Pass)
            .expect("fixture has a PASS run");
        app.selected_id = Some(app.runs[idx].id.clone());
        let (w, h) = (80, 14);
        let buf = render_to_buffer(&app, w, h);
        let (x, y) = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .find(|&(x, y)| buf[(x, y)].symbol() == "▶")
            .expect("a selected row");
        assert!(
            !buf[(x, y)]
                .modifier
                .contains(ratatui::style::Modifier::REVERSED),
            "rail cell is reversed"
        );
    }

    /// The board's selection bar is one band, separators included.
    #[test]
    fn board_selection_bar_has_no_gaps() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(code(KeyCode::Char('b')));
        let (w, h) = (80, 14);
        let buf = render_to_buffer(&app, w, h);
        let text = |y: u16| (0..w).map(|x| buf[(x, y)].symbol()).collect::<String>();
        let y = (0..h)
            .find(|&y| text(y).contains('▶'))
            .expect("the fixture board has a selected task");
        let start = (0..w).find(|&x| buf[(x, y)].symbol() == "▶").unwrap();
        let styled: Vec<_> = (start..w - 1).map(|x| buf[(x, y)].modifier).collect();
        assert!(
            styled.windows(2).all(|p| p[0] == p[1]),
            "selection bar breaks between cells: {}",
            text(y)
        );
    }

    #[test]
    fn populated_control_room_motion_on_snapshot() {
        let mut app = App::with_motion(true);
        app.load_prd_fixture();
        let frame = render_to_string(&app, 80, 14);
        assert!(
            frame.contains('⠋') && frame.contains("GATING") && frame.contains("RUN"),
            "motion-on live work must breathe; GATING is the wedge: {frame}"
        );
        insta::assert_snapshot!(frame);
    }

    /// At 60 columns, with the spinner on, no STATE or GATE word is clipped
    /// (`GATING…` was): AGENT gives way first.
    #[test]
    fn narrow_control_room_never_clips_state_or_gate() {
        let mut app = App::with_motion(true);
        app.load_prd_fixture();
        let frame = render_to_string(&app, 60, 16);
        for run in &app.runs {
            let state = crate::app::format_state_label(run, &app.clock, true);
            let gate = crate::app::format_gate_label(run);
            assert!(frame.contains(&state), "{state} clipped: {frame}");
            assert!(frame.contains(&gate), "{gate} clipped: {frame}");
        }
    }

    #[test]
    fn populated_control_room_snapshot_narrow() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let frame = render_to_string(&app, 60, 12);
        let footer = snapshot_footer(&frame);
        assert!(
            footer.contains('?') && footer.contains('r'),
            "60-col footer must keep help and retry: {footer}"
        );
        assert!(
            frame.contains("^ tsc: 3 errors"),
            "FAIL first-error must stay readable at 60×12: {frame}"
        );
        assert!(
            frame.contains("GATING"),
            "60-col STATE must keep GATING: {frame}"
        );
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn run_detail_stream_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(code(KeyCode::Enter));
        assert!(matches!(
            app.screen,
            Screen::RunDetail {
                pane: DetailPane::Stream
            }
        ));
        let frame = render_to_string(&app, 80, 16);
        assert!(
            frame.contains("codex·eng"),
            "run detail header must use vendor·role: {frame}"
        );
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn run_detail_gate_fail_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        // Demo fixture already selects the FAIL row.
        app.update(AppEvent::Key(KeyEvent::new(
            KeyCode::Char('g'),
            KeyModifiers::NONE,
        )));
        let frame = render_to_string(&app, 80, 16);
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn run_detail_diff_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        // FAIL row is selected and has a diff.
        app.update(code(KeyCode::Enter));
        app.update(AppEvent::Key(KeyEvent::new(
            KeyCode::Char('3'),
            KeyModifiers::NONE,
        )));
        let frame = render_to_string(&app, 80, 16);
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn run_detail_stream_narrow_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(code(KeyCode::Enter));
        let frame = render_to_string(&app, 60, 12);
        assert!(
            frame.contains("GATE FAIL"),
            "60-col run detail must keep GATE FAIL: {frame}"
        );
        let footer = snapshot_footer(&frame);
        assert!(
            footer.contains("[r]etry") || footer.contains("[r]"),
            "60-col run detail must keep retry: {footer}"
        );
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn attached_stub_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(code(KeyCode::Enter));
        app.update(AppEvent::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
        )));
        assert_eq!(app.screen, Screen::Attached);
        let frame = render_to_string(&app, 80, 12);
        assert!(
            frame.contains("codex·eng"),
            "attach header must use vendor·role: {frame}"
        );
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn dispatch_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.dispatch.task = "port guard.js across the monorepo".into();
        app.update(AppEvent::Key(KeyEvent::new(
            KeyCode::Char('d'),
            KeyModifiers::NONE,
        )));
        let frame = render_to_string(&app, 80, 16);
        assert!(
            frame.contains("[↑↓]"),
            "dispatch footer must include move keys: {frame}"
        );
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn dispatch_with_probe_snapshot() {
        let mut app = App::with_motion(false);
        app.dispatch.repos = vec![
            ("/repos/kit".into(), true),
            ("/repos/guardian".into(), false),
        ];
        app.set_agents_probe(vec![
            ("codex".into(), false),
            ("claude".into(), true),
            ("grok".into(), false),
            ("ollama".into(), false),
        ]);
        app.dispatch.task = "port guard.js across the monorepo".into();
        app.screen = Screen::Dispatch;
        let frame = render_to_string(&app, 80, 16);
        assert!(
            frame.contains("claude  ready") && frame.contains("codex  missing"),
            "probe must annotate agents without rewriting ids: {frame}"
        );
        assert!(
            !frame.contains("claude  ready") || frame.contains("[x] claude  ready"),
            "first ready agent is selected: {frame}"
        );
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn dispatch_personas_snapshot() {
        let mut app = App::with_motion(false);
        app.dispatch.repos = vec![("/repos/kit".into(), true)];
        app.dispatch.task = "empty room first paint".into();
        for (p, on) in &mut app.dispatch.personas {
            *on = matches!(
                *p,
                crate::persona::Persona::Product
                    | crate::persona::Persona::Design
                    | crate::persona::Persona::Eng
            );
        }
        app.screen = Screen::Dispatch;
        app.dispatch.focus = crate::app::DispatchFocus::Personas;
        let frame = render_to_string(&app, 80, 16);
        assert!(
            frame.contains("product") && frame.contains("design") && frame.contains("eng"),
            "persona column must be visible: {frame}"
        );
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn board_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(AppEvent::Key(KeyEvent::new(
            KeyCode::Char('b'),
            KeyModifiers::NONE,
        )));
        let frame = render_to_string(&app, 80, 14);
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn empty_board_snapshot() {
        let mut app = App::with_motion(false);
        app.update(AppEvent::Key(KeyEvent::new(
            KeyCode::Char('b'),
            KeyModifiers::NONE,
        )));
        assert_eq!(app.screen, Screen::Board);
        let frame = render_to_string(&app, 80, 14);
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn help_overlay_control_room_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(AppEvent::Key(KeyEvent::new(
            KeyCode::Char('?'),
            KeyModifiers::NONE,
        )));
        assert!(app.help_open);
        let frame = render_to_string(&app, 80, 14);
        assert!(
            !frame.contains("j/k"),
            "help must not advertise j/k nav while k=kill: {frame}"
        );
        assert!(
            !frame.contains("KIT / CONTROL ROOM") && !frame.contains("[↑↓] select"),
            "help overlay must cover the Control Room chrome: {frame}"
        );
        assert!(
            frame.contains("kill"),
            "help overlay at 80×14 must show kill: {frame}"
        );
        assert!(
            frame.contains("retry"),
            "help overlay at 80×14 must show retry: {frame}"
        );
        insta::assert_snapshot!(frame);
    }
}
