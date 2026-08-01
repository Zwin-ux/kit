//! Control Room placeholder frame.
//!
//! Header, empty run table with column headers, and footer key hints from
//! `docs/dev/PRD-1.0.md` section 4.2. Real rows and detail screens come later.

use crate::app::{App, RunRow};
use kit_core::RunState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

/// Paint the Control Room into `frame`.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(3),    // table
            Constraint::Length(1), // footer
        ])
        .split(area);

    draw_header(frame, app, chunks[0]);
    draw_table(frame, app, chunks[1]);
    draw_footer(frame, chunks[2]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let left = "KIT / CONTROL ROOM";
    let right = format!(
        "{} RUNNING  {} GATED",
        app.running_count(),
        app.gated_count()
    );

    // Right-align the counts when there is room; otherwise collapse to left only.
    let gap = area.width as usize;
    let line = if left.len() + 2 + right.len() <= gap {
        let spaces = gap.saturating_sub(left.len() + right.len());
        format!("{left}{}{right}", " ".repeat(spaces))
    } else {
        left.to_string()
    };

    let mut spans = vec![Span::styled(
        line,
        Style::default().add_modifier(Modifier::BOLD),
    )];
    if let Some(err) = &app.error {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("! {err}"),
            Style::default().add_modifier(Modifier::BOLD),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_table(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("REPO"),
        Cell::from("AGENT"),
        Cell::from("TASK"),
        Cell::from("STATE"),
        Cell::from("GATE"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let rows = if app.runs.is_empty() {
        vec![Row::new(vec![
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        app.runs
            .iter()
            .enumerate()
            .map(|(i, run)| row_for(run, i == app.selected))
            .collect()
    };

    let widths = [
        Constraint::Percentage(22),
        Constraint::Percentage(12),
        Constraint::Percentage(36),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL))
        .column_spacing(1);

    frame.render_widget(table, area);
}

fn row_for(run: &RunRow, selected: bool) -> Row<'static> {
    let marker = if selected { "> " } else { "  " };
    let repo = format!("{marker}{}", run.repo);
    let state = format_state(run);
    let gate = format_gate(run);

    let style = if selected {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    };

    Row::new(vec![
        Cell::from(repo),
        Cell::from(run.agent.clone()),
        Cell::from(run.task.clone()),
        Cell::from(state),
        Cell::from(gate),
    ])
    .style(style)
}

fn format_state(run: &RunRow) -> String {
    let label = match run.state {
        RunState::Queued => "QUEUED",
        RunState::Running => "RUN",
        RunState::Gating => "GATING",
        RunState::Pass => "DONE",
        RunState::Fail => "DONE",
        RunState::Killed => "KILLED",
        RunState::Error => "ERROR",
    };
    if matches!(run.state, RunState::Running | RunState::Gating) && !run.elapsed.is_empty() {
        format!("{label} {}", run.elapsed)
    } else {
        label.to_string()
    }
}

fn format_gate(run: &RunRow) -> String {
    match (&run.state, &run.gate) {
        (_, Some(g)) if g.passed => "PASS".into(),
        (_, Some(_)) => "FAIL".into(),
        (RunState::Pass, None) => "PASS".into(),
        (RunState::Fail, None) => "FAIL".into(),
        _ => "--".into(),
    }
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    // PRD §4.2 key hints.
    let hints = " [d]ispatch  [enter] open  [g]ate log  [k]ill  [r]etry";
    frame.render_widget(Paragraph::new(hints), area);
}

/// Render the Control Room into a string for snapshot tests.
#[cfg(test)]
pub fn render_to_string(app: &App, width: u16, height: u16) -> String {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test backend");
    terminal.draw(|f| draw(f, app)).expect("draw control room");
    format!("{}", terminal.backend())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

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
}
