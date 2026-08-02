//! Board — curated Dispatch prefill list (1.0; no pull-queue).

use super::common::{
    draw_empty_state, draw_footer, draw_header, draw_too_small, too_small, truncate,
};
use crate::app::App;
use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
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
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    let open = app.board.iter().filter(|t| !t.done).count();
    let stats = format!("{open} open  {} total", app.board.len());
    draw_header(
        frame,
        chunks[0],
        &theme,
        "KIT / BOARD",
        &stats,
        app.flash_message(),
        None,
    );

    if app.board.is_empty() {
        draw_empty_state(
            frame,
            chunks[1],
            &theme,
            "Board is empty",
            "press n to add a task  ·  Enter prefills Dispatch",
        );
    } else {
        draw_table(frame, app, chunks[1], &theme);
    }

    draw_footer(
        frame,
        chunks[2],
        &theme,
        " [esc] back  [n]ew  [enter] dispatch  [space] done  [x] remove",
        "prefill-only 1.0",
    );
}

fn draw_table(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let header = Row::new(vec![
        Cell::from("#").style(theme.dim().add_modifier(Modifier::BOLD)),
        Cell::from("TASK").style(theme.dim().add_modifier(Modifier::BOLD)),
        Cell::from("REPO").style(theme.dim().add_modifier(Modifier::BOLD)),
        Cell::from("AGENT").style(theme.dim().add_modifier(Modifier::BOLD)),
        Cell::from("STATE").style(theme.dim().add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = app
        .board
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let selected = i == app.board_selected;
            let marker = if selected { "▶ " } else { "  " };
            let state = if t.done { "DONE" } else { "OPEN" };
            let base = if selected {
                theme.selected_row()
            } else {
                theme.body()
            };
            let state_style = if selected {
                theme.selected_row()
            } else if t.done {
                theme.success()
            } else {
                theme.warn()
            };
            Row::new(vec![
                Cell::from(format!("{marker}{}", t.id)).style(base),
                Cell::from(truncate(&t.title, 32)).style(base),
                Cell::from(truncate(&t.repo_hint, 12)).style(base),
                Cell::from(truncate(&t.agent_hint, 10)).style(base),
                Cell::from(state).style(state_style),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Percentage(40),
        Constraint::Percentage(18),
        Constraint::Percentage(16),
        Constraint::Percentage(14),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border(true))
                .title(" queue ")
                .title_style(theme.dim()),
        )
        .column_spacing(1);

    frame.render_widget(table, area);
}
