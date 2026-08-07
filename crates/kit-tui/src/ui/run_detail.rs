//! Run detail — stream, gate log, diff panes (1.0 craft remake).

use super::common::{
    draw_footer, draw_header, draw_too_small, style_log_line, too_small, truncate, viewport_start,
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
    let follow = if app.stream_follow { " follow" } else { "" };
    draw_footer(
        frame,
        chunks[3],
        &theme,
        &format!(" [esc] back  [1]stream [2]gate [3]diff  [a]ttach  [k]ill  [r]etry{follow}"),
        "",
    );
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
            r.repo,
            r.agent,
            truncate(&r.task, 24)
        ),
        None => "KIT / ATTACHED".into(),
    };

    draw_header(
        frame,
        chunks[0],
        &theme,
        &title,
        "PTY 1.0.1",
        app.flash_message(),
        None,
    );

    let body: Vec<Line> = vec![
        Line::from(Span::styled("PTY attach ships in 1.0.1", theme.title())),
        Line::from(""),
        Line::from(Span::styled(
            "Esc detaches without killing the run.",
            theme.body(),
        )),
        Line::from(Span::styled("q is disabled while attached.", theme.dim())),
        Line::from(""),
        Line::from(Span::styled(
            app.flash_message()
                .unwrap_or("[ waiting for agent PTY supervision ]"),
            theme.warn(),
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
    draw_footer(frame, chunks[2], &theme, " [esc] detach (without kill)", "");
}

fn draw_run_header(frame: &mut Frame, app: &App, run: &RunRow, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let state = format_state_label(run, &app.clock);
    let gate = format_gate_label(run);
    let l1 = format!(
        "KIT / RUN  {} · {} · {}",
        run.repo,
        run.agent,
        truncate(&run.task, 28)
    );

    let wt = run
        .worktree
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "—".into());
    let l2 = format!(
        "worktree  {}",
        truncate(&wt, area.width.saturating_sub(12) as usize)
    );

    let header_line = Line::from(vec![
        Span::styled(l1, theme.title()),
        Span::raw("  "),
        Span::styled(state, theme.state_style(run.state)),
        Span::raw("  GATE "),
        Span::styled(gate.clone(), theme.gate_style(&gate)),
    ]);
    frame.render_widget(Paragraph::new(header_line), chunks[0]);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(l2, theme.dim()))),
        chunks[1],
    );
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
                    let style = if l.contains("UNCONFIGURED") {
                        theme.warn()
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
