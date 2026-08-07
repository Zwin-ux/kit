//! Control Room frame — PRD §4.2 live table (1.0 craft remake).
//!
//! Visual bar: `docs/dev/DESIGN-tui.md` + concept-control-room.jpg

use super::common::{
    draw_empty_state, draw_footer, draw_header, draw_too_small, too_small, truncate,
};
use crate::app::{App, RunRow, format_gate_label, format_state_label};
use crate::theme::Theme;
use kit_core::RunState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

pub fn draw(frame: &mut Frame, app: &App) {
    let theme = Theme::resolve();
    let area = frame.area();
    if too_small(area) {
        draw_too_small(frame, area, &theme);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(3),    // table
            Constraint::Length(1), // footer
        ])
        .split(area);

    let agents = app.agents_strip();
    let stats = if agents.is_empty() {
        format!(
            "{} RUNNING  {} FAIL  {} GATED",
            app.running_count(),
            app.fail_count(),
            app.gated_count()
        )
    } else if app.runs.is_empty() {
        agents
    } else {
        format!(
            "{}  ·  {}R {}F {}G",
            agents,
            app.running_count(),
            app.fail_count(),
            app.gated_count()
        )
    };
    draw_header(
        frame,
        chunks[0],
        &theme,
        "KIT / CONTROL ROOM",
        &stats,
        app.flash_message(),
        app.error.as_deref(),
    );

    if app.runs.is_empty() {
        let (message, hint) = empty_room_copy(app);
        draw_empty_state(frame, chunks[1], &theme, message, hint);
    } else {
        draw_table(frame, app, chunks[1], &theme);
    }

    draw_footer(
        frame,
        chunks[2],
        &theme,
        " [↑↓] select  [d]ispatch  [b]oard  [enter] open  [g]ate  [k]ill  [r]etry  [?]help",
        "",
    );
}

/// Empty Control Room copy — cold-start cockpit, not a blank form.
fn empty_room_copy(app: &App) -> (&'static str, &'static str) {
    let ready = app.agents_ready_count();
    if app.agents_probe.is_empty() {
        (
            "No runs yet",
            "press d to dispatch  ·  kit --demo for fixture data  ·  ? help",
        )
    } else if ready == 0 {
        (
            "No coding agents on PATH",
            "install codex / claude / grok / ollama  ·  kit doctor  ·  kit --demo",
        )
    } else {
        (
            "Ready to dispatch",
            "press d to fan out agents  ·  kit --demo to see FAIL + retry  ·  ? help",
        )
    }
}

fn draw_table(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let header = Row::new(vec![
        Cell::from("REPO").style(theme.dim().add_modifier(Modifier::BOLD)),
        Cell::from("AGENT").style(theme.dim().add_modifier(Modifier::BOLD)),
        Cell::from("TASK").style(theme.dim().add_modifier(Modifier::BOLD)),
        Cell::from("STATE").style(theme.dim().add_modifier(Modifier::BOLD)),
        Cell::from("GATE").style(theme.dim().add_modifier(Modifier::BOLD)),
    ]);

    let order = app.display_order();
    let selected_id = app.selected_id.as_ref();

    let mut rows: Vec<Row> = Vec::with_capacity(order.len() * 2);
    for &idx in &order {
        let run = &app.runs[idx];
        let selected = selected_id.is_some_and(|id| *id == run.id);
        rows.push(row_for(run, selected, app, theme));
        if let Some(summary) = run.gate_summary() {
            rows.push(annotation_row(&summary, selected, theme));
        }
    }

    let widths = if area.width < 70 {
        [
            Constraint::Percentage(20),
            Constraint::Percentage(12),
            Constraint::Percentage(32),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
        ]
    } else {
        [
            Constraint::Percentage(22),
            Constraint::Percentage(12),
            Constraint::Percentage(36),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ]
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border(true))
                .title(" runs ")
                .title_style(theme.dim()),
        )
        .column_spacing(1);

    frame.render_widget(table, area);
}

fn row_for(run: &RunRow, selected: bool, app: &App, theme: &Theme) -> Row<'static> {
    let marker = if selected { "▶ " } else { "  " };
    let repo = format!("{marker}{}", truncate(&run.repo, 16));
    let state_label = format_state_label(run, &app.clock);
    let gate_label = format_gate_label(run);

    let base = if selected {
        theme.selected_row()
    } else if run.state == RunState::Fail {
        theme.fail_row(false)
    } else {
        theme.body()
    };

    let state_style = if selected {
        theme.selected_row()
    } else {
        theme.state_style(run.state)
    };
    let gate_style = if selected {
        theme.selected_row()
    } else {
        theme.gate_style(&gate_label)
    };

    Row::new(vec![
        Cell::from(repo).style(base),
        Cell::from(truncate(&run.agent, 10)).style(base),
        Cell::from(truncate(&run.task, 28)).style(base),
        Cell::from(state_label).style(state_style),
        Cell::from(gate_label).style(gate_style),
    ])
}

fn annotation_row(summary: &str, selected: bool, theme: &Theme) -> Row<'static> {
    let text = format!("    ^ {}", truncate(summary, 48));
    let style = if selected {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM)
    } else {
        theme.annotation()
    };
    Row::new(vec![
        Cell::from(""),
        Cell::from(""),
        Cell::from(text),
        Cell::from(""),
        Cell::from(""),
    ])
    .style(style)
}
