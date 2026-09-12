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
        // 12% gutters clip kill/retry at 14 rows. Use no vertical chrome
        // on short frames so the 30-second keys stay on screen.
        let v_gutter = if area.height < 18 {
            Constraint::Length(0)
        } else {
            Constraint::Percentage(12)
        };
        let v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([v_gutter, Constraint::Min(8), v_gutter])
            .split(area);
        let h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(8),
                Constraint::Percentage(84),
                Constraint::Percentage(8),
            ])
            .split(v[1]);
        draw_help_overlay(frame, h[1], &theme, help_lines(app));
    }
}

fn help_lines(app: &App) -> Vec<Line<'static>> {
    // No blank rows: at 14 terminal rows the panel inner height is ~12.
    // Kill and retry must appear in that window (k is kill, not nav).
    let mut lines: Vec<Line<'static>> = vec![
        Line::from("Kit Control Room — keys"),
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
                Line::from("  b          board (prefill list)"),
                Line::from("  f          filter ALL → FAIL → RUN → DONE"),
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
        insta::assert_snapshot!(frame);
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
