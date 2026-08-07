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
        // Centered panel over the current screen.
        let v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(12),
                Constraint::Percentage(76),
                Constraint::Percentage(12),
            ])
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
    let mut lines: Vec<Line<'static>> = vec![
        Line::from("Kit Control Room — keys"),
        Line::from(""),
        Line::from("Global"),
        Line::from("  ?          toggle this help"),
        Line::from("  Esc        back / close help"),
        Line::from("  q          quit (Control Room only; disabled while attached)"),
        Line::from(""),
    ];
    match app.screen {
        Screen::ControlRoom => {
            lines.extend([
                Line::from("Control Room"),
                Line::from("  ↑↓         move selection"),
                Line::from("  Enter      open run detail (stream)"),
                Line::from("  g          open gate log"),
                Line::from("  d          dispatch fan-out"),
                Line::from("  b          board (prefill list)"),
                Line::from("  f          filter ALL → FAIL → RUN → DONE"),
                Line::from("  k          kill selected run"),
                Line::from("  r          retry FAIL only"),
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
                Line::from("  Tab        next field"),
                Line::from("  Space      toggle repo/agent"),
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
    lines.push(Line::from(""));
    lines.push(Line::from("Press Esc or ? to close"));
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

    #[test]
    fn empty_control_room_snapshot() {
        let app = App::with_motion(false);
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
    fn populated_control_room_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let frame = render_to_string(&app, 80, 14);
        insta::assert_snapshot!(frame);
    }

    #[test]
    fn populated_control_room_snapshot_narrow() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        let frame = render_to_string(&app, 60, 12);
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
    fn help_overlay_control_room_snapshot() {
        let mut app = App::with_motion(false);
        app.load_prd_fixture();
        app.update(AppEvent::Key(KeyEvent::new(
            KeyCode::Char('?'),
            KeyModifiers::NONE,
        )));
        assert!(app.help_open);
        let frame = render_to_string(&app, 80, 20);
        assert!(
            !frame.contains("j/k"),
            "help must not advertise j/k nav while k=kill: {frame}"
        );
        assert!(frame.contains("kill") || frame.contains("Kill") || frame.contains("k"));
        insta::assert_snapshot!(frame);
    }
}
