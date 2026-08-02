//! Board — shared task queue (PRD §4.2 orchestrator view).

use super::common::truncate;
use crate::app::App;
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
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    let open = app.board.iter().filter(|t| !t.done).count();
    let mut title = format!("KIT / BOARD    {open} open  {} total", app.board.len());
    if let Some(flash) = app.flash_message() {
        title.push_str("  ·  ");
        title.push_str(flash);
    }
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))),
        chunks[0],
    );

    draw_table(frame, app, chunks[1]);
    frame.render_widget(
        Paragraph::new(
            " [esc] back  [n]ew  [enter] dispatch  [space] done  [x] remove  [d]ispatch",
        ),
        chunks[2],
    );
}

fn draw_table(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("#"),
        Cell::from("TASK"),
        Cell::from("REPO"),
        Cell::from("AGENT"),
        Cell::from("STATE"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = if app.board.is_empty() {
        vec![Row::new(vec![
            Cell::from(""),
            Cell::from("(empty — press n to add)"),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        app.board
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let selected = i == app.board_selected;
                let marker = if selected { "> " } else { "  " };
                let state = if t.done { "DONE" } else { "OPEN" };
                let style = if selected {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                Row::new(vec![
                    Cell::from(format!("{marker}{}", t.id)),
                    Cell::from(truncate(&t.title, 32)),
                    Cell::from(truncate(&t.repo_hint, 12)),
                    Cell::from(truncate(&t.agent_hint, 10)),
                    Cell::from(state),
                ])
                .style(style)
            })
            .collect()
    };

    let widths = [
        Constraint::Length(6),
        Constraint::Percentage(45),
        Constraint::Percentage(18),
        Constraint::Percentage(15),
        Constraint::Percentage(12),
    ];
    frame.render_widget(
        Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL))
            .column_spacing(1),
        area,
    );
}
