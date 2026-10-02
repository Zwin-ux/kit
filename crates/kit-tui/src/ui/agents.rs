//! Local provider connections. Native sign-in owns its own terminal and credentials.
use super::common::{draw_footer, draw_header, draw_too_small, too_small};
use crate::{App, app::AGENT_ORDER, theme::Theme};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

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
    draw_header(
        frame,
        chunks[0],
        &theme,
        "KIT / AGENTS",
        "",
        app.flash_message(),
        app.error.as_deref(),
    );
    let mut lines = vec![
        Line::from("Connect through the provider's native sign-in. Kit keeps no credentials."),
        Line::from(""),
    ];
    for (index, name) in AGENT_ORDER.iter().enumerate() {
        let selected = index == app.agent_selected;
        let status = app.agent_statuses.iter().find(|s| s.kind.binary() == *name);
        let label = if status.is_some() {
            app.agent_status_label(name)
        } else {
            "not checked"
        };
        let text = format!(
            " {} {:8}  {}{}",
            if selected { "›" } else { " " },
            name,
            label,
            status
                .and_then(|s| s.version.as_deref())
                .map(|v| format!(" · {v}"))
                .unwrap_or_default()
        );
        lines.push(Line::from(Span::styled(
            text,
            if selected {
                theme.title()
            } else {
                theme.body()
            },
        )));
    }
    lines.push(Line::from(""));
    let selected = AGENT_ORDER[app.agent_selected];
    if let Some(remedy) = app
        .agent_statuses
        .iter()
        .find(|s| s.kind.binary() == selected)
        .and_then(|s| s.remedy.as_deref())
    {
        lines.push(Line::from(Span::styled(remedy.to_string(), theme.warn())));
    }
    if !app.active_run_ids().is_empty() {
        lines.push(Line::from(
            "Native sign-in waits until active runs finish or are stopped. Esc returns to runs.",
        ));
    } else {
        lines.push(Line::from(
            "Enter opens native sign-in for Claude or Codex, then returns here to check status.",
        ));
    }
    if let Some(error) = &app.error {
        lines.push(Line::from(Span::styled(error.clone(), theme.warn())));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), chunks[1]);
    draw_footer(
        frame,
        chunks[2],
        &theme,
        " [esc] back  [↑↓] select  [c]onnect  [r]efresh  [?]help",
        "",
    );
}
