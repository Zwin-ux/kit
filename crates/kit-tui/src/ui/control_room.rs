//! Control Room frame — PRD §4.2 live table.

use super::common::truncate;
use crate::app::{App, RunRow, format_gate_label, format_state_label};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

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
    draw_footer(frame, app, chunks[2]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let left = "KIT / CONTROL ROOM";
    let right = format!(
        "{} RUNNING  {} GATED",
        app.running_count(),
        app.gated_count()
    );

    let gap = area.width as usize;
    let mut line = if left.len() + 2 + right.len() <= gap {
        let spaces = gap.saturating_sub(left.len() + right.len());
        format!("{left}{}{right}", " ".repeat(spaces))
    } else {
        left.to_string()
    };

    if let Some(flash) = app.flash_message() {
        let tag = format!("  · {flash}");
        if line.len() + tag.len() <= gap {
            line.push_str(&tag);
        }
    } else if let Some(err) = &app.error {
        let tag = format!("  ! {err}");
        if line.len() + tag.len() <= gap {
            line.push_str(&tag);
        }
    }

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            line,
            Style::default().add_modifier(Modifier::BOLD),
        ))),
        area,
    );
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

    let order = app.display_order();
    let selected_id = app.selected_id.as_ref();

    let rows: Vec<Row> = if order.is_empty() {
        vec![Row::new(vec![
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        let mut out = Vec::with_capacity(order.len() * 2);
        for &idx in &order {
            let run = &app.runs[idx];
            let selected = selected_id.is_some_and(|id| *id == run.id);
            out.push(row_for(run, selected, app));
            if let Some(summary) = run.gate_summary() {
                out.push(annotation_row(&summary, selected));
            }
        }
        out
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

fn row_for(run: &RunRow, selected: bool, app: &App) -> Row<'static> {
    let marker = if selected { "> " } else { "  " };
    let repo = format!("{marker}{}", truncate(&run.repo, 18));
    let style = if selected {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    };

    Row::new(vec![
        Cell::from(repo),
        Cell::from(truncate(&run.agent, 10)),
        Cell::from(truncate(&run.task, 28)),
        Cell::from(format_state_label(run, &app.clock)),
        Cell::from(format_gate_label(run)),
    ])
    .style(style)
}

fn annotation_row(summary: &str, selected: bool) -> Row<'static> {
    let text = format!("    ^ {}", truncate(summary, 48));
    let style = if selected {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM)
    } else {
        Style::default().add_modifier(Modifier::DIM)
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

fn draw_footer(frame: &mut Frame, _app: &App, area: Rect) {
    let hints = " [d]ispatch  [enter] open  [g]ate log  [k]ill  [r]etry";
    frame.render_widget(Paragraph::new(hints), area);
}
