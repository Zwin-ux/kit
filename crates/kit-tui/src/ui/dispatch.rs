//! Dispatch form — repos × agents × one task (PRD §4.2 fan-out).

use super::common::truncate;
use crate::app::{App, DispatchFocus};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Min(6),    // body
            Constraint::Length(2), // task + meta
            Constraint::Length(1), // footer
        ])
        .split(area);

    let mut title = "KIT / DISPATCH".to_string();
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

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    draw_toggle_list(
        frame,
        body[0],
        " repos (space) ",
        &app.dispatch.repos,
        app.dispatch.focus == DispatchFocus::Repos,
        app.dispatch.list_cursor,
    );
    draw_toggle_list(
        frame,
        body[1],
        " agents (space) ",
        &app.dispatch.agents,
        app.dispatch.focus == DispatchFocus::Agents,
        app.dispatch.list_cursor,
    );

    let task_focus = app.dispatch.focus == DispatchFocus::Task;
    let task_title = if task_focus {
        " task (focused) "
    } else {
        " task "
    };
    let task_style = if task_focus {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let task_text = if app.dispatch.task.is_empty() {
        "(type the prompt to fan out)".into()
    } else {
        app.dispatch.task.clone()
    };
    let meta = format!("fan-out: {} run(s)", app.dispatch.fanout_count());
    let task_block = Block::default().borders(Borders::ALL).title(task_title);
    let inner = task_block.inner(chunks[2]);
    frame.render_widget(task_block, chunks[2]);
    let task_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    frame.render_widget(
        Paragraph::new(truncate(&task_text, inner.width as usize)).style(task_style),
        task_chunks[0],
    );
    frame.render_widget(Paragraph::new(meta), task_chunks[1]);

    frame.render_widget(
        Paragraph::new(" [esc] back  [tab] field  [space] toggle  [enter] submit"),
        chunks[3],
    );
}

fn draw_toggle_list(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[(String, bool)],
    focused: bool,
    cursor: usize,
) {
    let block = Block::default().borders(Borders::ALL).title(title);
    let style = if focused {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, (name, on))| {
            let mark = if *on { "[x]" } else { "[ ]" };
            let cursor_mark = if focused && i == cursor { ">" } else { " " };
            let line = format!("{cursor_mark} {mark} {name}");
            let mut item = ListItem::new(line);
            if focused && i == cursor {
                item = item.style(Style::default().add_modifier(Modifier::REVERSED));
            }
            item
        })
        .collect();
    frame.render_widget(List::new(list_items).block(block).style(style), area);
}
