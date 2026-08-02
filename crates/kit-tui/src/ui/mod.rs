//! Control Room and Run Detail frames.

mod common;
mod control_room;
mod run_detail;

use crate::app::{App, Screen};
use ratatui::Frame;

/// Paint the active screen into `frame`.
pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::ControlRoom => control_room::draw(frame, app),
        Screen::RunDetail { pane } => run_detail::draw(frame, app, pane),
        Screen::Attached => run_detail::draw_attached(frame, app),
    }
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
        // Select the FAIL row (display index 2: two Running, then Fail).
        app.update(code(KeyCode::Down));
        app.update(code(KeyCode::Down));
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
        // FAIL row has a diff.
        app.update(code(KeyCode::Down));
        app.update(code(KeyCode::Down));
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
}
