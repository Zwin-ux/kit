//! Run detail — stream, gate log, diff panes (1.0 craft remake).

use super::common::{
    draw_footer, draw_header, draw_too_small, style_log_line, tilde, too_small, truncate,
    viewport_start,
};
use crate::app::{App, DetailPane, RunRow, format_gate_label, format_state_label, gate_log_lines};
use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn draw(frame: &mut Frame, app: &App, pane: DetailPane) {
    let theme = Theme::resolve();
    let area = frame.area();
    if too_small(area) {
        draw_too_small(frame, area, &theme);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // header
            Constraint::Length(1), // tab bar
            Constraint::Min(3),    // body
            Constraint::Length(1), // footer
        ])
        .split(area);

    let Some(run) = app.selected_run() else {
        frame.render_widget(
            Paragraph::new(Span::styled("No run selected.", theme.warn())),
            area,
        );
        return;
    };

    draw_run_header(frame, app, run, chunks[0], &theme);
    draw_tabs(frame, pane, chunks[1], &theme);
    draw_body(frame, app, run, pane, chunks[2], &theme);
    let follow = if app.stream_follow { "follow" } else { "" };
    draw_footer(frame, chunks[3], &theme, &footer_hints(run), follow);
}

/// Run detail keys: no `[k]ill` on a past run, `[l]and` on a proven run
/// with changes.
fn footer_hints(run: &RunRow) -> String {
    let mut hints = String::from(" [esc] back  [1]stream  [2]gate  [3]diff");
    if !run.past {
        hints.push_str("  [k]ill");
    }
    hints.push_str("  [r]etry");
    if run.landable() {
        hints.push_str("  [l]and");
    }
    hints
}

pub fn draw_attached(frame: &mut Frame, app: &App) {
    let theme = Theme::resolve();
    let area = frame.area();
    if too_small(area) {
        draw_too_small(frame, area, &theme);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    let title = match app.selected_run() {
        Some(r) => format!(
            "KIT / ATTACHED  {} · {} · {}",
            r.repo_name(),
            r.agent_cell(),
            truncate(r.task_line(), 24)
        ),
        None => "KIT / ATTACHED".into(),
    };

    draw_header(
        frame,
        chunks[0],
        &theme,
        &title,
        "",
        app.flash_message(),
        None,
    );

    // No key opens this screen until the agent's terminal can be attached.
    let body: Vec<Line> = vec![
        Line::from(Span::styled("Attach is not available.", theme.title())),
        Line::from(""),
        Line::from(Span::styled(
            "Esc goes back without stopping the run.",
            theme.body(),
        )),
    ];

    frame.render_widget(
        Paragraph::new(body).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" attach ")
                .border_style(theme.border(true))
                .title_style(theme.dim()),
        ),
        chunks[1],
    );
    draw_footer(frame, chunks[2], &theme, " [esc] detach  [?] help", "");
}

fn draw_run_header(frame: &mut Frame, app: &App, run: &RunRow, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let state = format_state_label(run, &app.clock, app.motion_enabled());
    let state_style = if run.no_changes() {
        theme.warn().add_modifier(Modifier::BOLD)
    } else {
        theme.state_style(run.state)
    };
    let gate = format_gate_label(run);
    let suffix = format!("  {state}  GATE {gate}");
    let title_budget = (area.width as usize).saturating_sub(suffix.chars().count());
    let prefix = format!("KIT / RUN  {} · {} · ", run.repo_name(), run.agent_cell());
    let task_budget = title_budget.saturating_sub(prefix.chars().count());
    let l1 = format!("{prefix}{}", truncate(run.task_line(), task_budget));

    // A past run's worktree is gone; its receipt holds the proof.
    let (label, path) = match (&run.worktree, &app.runs_dir) {
        (None, Some(dir)) if run.past => ("receipt ", tilde(&dir.join(&run.id.0))),
        (Some(p), _) => ("worktree", tilde(p)),
        (None, _) => ("worktree", "—".into()),
    };
    let l2 = format!(
        "{label}  {}",
        truncate(&path, area.width.saturating_sub(12) as usize)
    );

    let header_line = Line::from(vec![
        Span::styled(l1, theme.title()),
        Span::raw("  "),
        Span::styled(state, state_style),
        Span::raw("  GATE "),
        Span::styled(gate.clone(), theme.gate_style(&gate)),
    ]);
    frame.render_widget(Paragraph::new(header_line), chunks[0]);
    // A flash (the land command, a refused key) takes the second row for
    // its 2 s; the worktree line comes back after.
    let second = match app.flash_message() {
        Some(flash) => Span::styled(truncate(flash, area.width as usize), theme.accent()),
        None => Span::styled(l2, theme.dim()),
    };
    frame.render_widget(Paragraph::new(Line::from(second)), chunks[1]);
}

fn draw_tabs(frame: &mut Frame, pane: DetailPane, area: Rect, theme: &Theme) {
    let tabs = [DetailPane::Stream, DetailPane::Gate, DetailPane::Diff];
    let mut spans = Vec::new();
    for (i, t) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", theme.dim()));
        }
        let label = format!(" {} ", t.label());
        let style = if *t == pane {
            theme
                .accent()
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            theme.dim()
        };
        spans.push(Span::styled(label, style));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_body(
    frame: &mut Frame,
    app: &App,
    run: &RunRow,
    pane: DetailPane,
    area: Rect,
    theme: &Theme,
) {
    let title = match pane {
        DetailPane::Stream => " stream ",
        DetailPane::Gate => " gate ",
        DetailPane::Diff => " diff ",
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(theme.border(true))
        .title_style(theme.dim());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = match pane {
        DetailPane::Stream => {
            let raw = run.output_lines();
            let start = viewport_start(
                raw.len(),
                inner.height,
                app.detail_scroll,
                app.stream_follow,
            );
            raw.into_iter()
                .skip(start)
                .take(inner.height as usize)
                .map(|l| Line::from(Span::styled(l.to_string(), style_log_line(theme, l))))
                .collect()
        }
        DetailPane::Gate => {
            let raw = gate_log_lines(run);
            let start = viewport_start(raw.len(), inner.height, app.detail_scroll, false);
            raw.into_iter()
                .skip(start)
                .take(inner.height as usize)
                .map(|l| {
                    let style = if l.contains("UNCONFIGURED") || l.starts_with("NO CHANGES") {
                        theme.warn()
                    } else if l.starts_with("next  ") {
                        theme.accent()
                    } else if l.contains("FAIL") || l.contains("OVERALL  FAIL") {
                        theme.danger()
                    } else if l.contains("PASS") || l.contains("OVERALL  PASS") {
                        theme.success()
                    } else {
                        theme.body()
                    };
                    Line::from(Span::styled(l, style))
                })
                .collect()
        }
        DetailPane::Diff => {
            let raw = run.diff_lines();
            let start = viewport_start(raw.len(), inner.height, app.detail_scroll, false);
            raw.into_iter()
                .skip(start)
                .take(inner.height as usize)
                .map(|l| Line::from(Span::styled(l.to_string(), style_log_line(theme, l))))
                .collect()
        }
    };

    if lines.is_empty() {
        let empty = match pane {
            DetailPane::Stream => "No output yet.",
            DetailPane::Gate => "Gate has not run yet.",
            DetailPane::Diff => "No diff recorded.",
        };
        frame.render_widget(Paragraph::new(Span::styled(empty, theme.dim())), inner);
    } else {
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}
