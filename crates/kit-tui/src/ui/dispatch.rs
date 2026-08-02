//! Dispatch form — repos × agents × one task (1.0 craft).

use super::common::{draw_footer, draw_header, draw_too_small, too_small, truncate};
use crate::app::{App, DispatchFocus};
use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

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
            Constraint::Length(1), // title
            Constraint::Min(6),    // body
            Constraint::Length(3), // task + meta
            Constraint::Length(1), // footer
        ])
        .split(area);

    let stats = format!("fan-out {}", app.dispatch.fanout_count());
    draw_header(
        frame,
        chunks[0],
        &theme,
        "KIT / DISPATCH",
        &stats,
        app.flash_message(),
        None,
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
        &theme,
    );
    draw_toggle_list(
        frame,
        body[1],
        " agents (space) ",
        &app.dispatch.agents,
        app.dispatch.focus == DispatchFocus::Agents,
        app.dispatch.list_cursor,
        &theme,
    );

    let task_focus = app.dispatch.focus == DispatchFocus::Task;
    let task_title = if task_focus {
        " task (focused) "
    } else {
        " task "
    };
    let task_text = if app.dispatch.task.is_empty() {
        "(type the prompt to fan out)".into()
    } else {
        app.dispatch.task.clone()
    };
    let meta = format!(
        "{} run(s) · max {}",
        app.dispatch.fanout_count(),
        crate::app::DISPATCH_FANOUT_CAP
    );
    let task_block = Block::default()
        .borders(Borders::ALL)
        .title(task_title)
        .border_style(theme.border(task_focus))
        .title_style(if task_focus {
            theme.accent()
        } else {
            theme.dim()
        });
    let inner = task_block.inner(chunks[2]);
    frame.render_widget(task_block, chunks[2]);
    let task_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    let task_style = if task_focus {
        theme.body().add_modifier(Modifier::BOLD)
    } else {
        theme.dim()
    };
    frame.render_widget(
        Paragraph::new(truncate(&task_text, inner.width as usize)).style(task_style),
        task_chunks[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(meta, theme.dim()))),
        task_chunks[1],
    );

    draw_footer(
        frame,
        chunks[3],
        &theme,
        " [esc] back  [tab] field  [space] toggle  [enter] submit",
        "",
    );
}

fn draw_toggle_list(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: &[(String, bool)],
    focused: bool,
    cursor: usize,
    theme: &Theme,
) {
    let items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, (name, on))| {
            let mark = if *on { "[x]" } else { "[ ]" };
            let caret = if focused && i == cursor { "▶ " } else { "  " };
            let line = format!("{caret}{mark} {name}");
            let style = if focused && i == cursor {
                theme.selected_row()
            } else if *on {
                theme.accent()
            } else {
                theme.dim()
            };
            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(theme.border(focused))
            .title_style(if focused { theme.accent() } else { theme.dim() }),
    );
    frame.render_widget(list, area);
}
