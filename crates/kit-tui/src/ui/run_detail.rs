//! Run detail — stream, gate log, diff panes (PRD §4.2).

use super::common::{truncate, viewport_start};
use crate::app::{
    App, DetailPane, RunRow, Screen, format_gate_label, format_state_label, gate_log_lines,
};
use kit_core::RunState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn draw(frame: &mut Frame, app: &App, pane: DetailPane) {
    let area = frame.area();
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
        frame.render_widget(Paragraph::new("No run selected."), area);
        return;
    };

    draw_header(frame, app, run, chunks[0]);
    draw_tabs(frame, pane, chunks[1]);
    draw_body(frame, app, run, pane, chunks[2]);
    draw_footer(frame, app, chunks[3]);
}

pub fn draw_attached(frame: &mut Frame, app: &App) {
    let area = frame.area();
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

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ))),
        chunks[0],
    );

    let body = [
        "PTY not connected yet — Esc detaches without killing.".into(),
        String::new(),
        "[ waiting for agent PTY supervision ]".into(),
        String::new(),
        app.flash_message().unwrap_or("").to_string(),
    ]
    .join("\n");

    frame.render_widget(
        Paragraph::new(body).block(Block::default().borders(Borders::ALL).title(" attach ")),
        chunks[1],
    );
    frame.render_widget(Paragraph::new(" [esc] detach (without kill)"), chunks[2]);
}

fn draw_header(frame: &mut Frame, app: &App, run: &RunRow, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let state = format_state_label(run, &app.clock);
    let gate = format_gate_label(run);
    let l1 = format!(
        "KIT / RUN  {} · {} · {}    {state}  GATE {gate}",
        run.repo,
        run.agent,
        truncate(&run.task, 28)
    );

    let wt = run
        .worktree
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(no worktree)".into());
    let mut l2 = format!(
        "worktree: {}    id: {}",
        truncate(&wt, 40),
        truncate(&run.id.0, 12)
    );
    if let Some(flash) = app.flash_message() {
        l2.push_str("  · ");
        l2.push_str(flash);
    }

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            l1,
            Style::default().add_modifier(Modifier::BOLD),
        ))),
        chunks[0],
    );
    frame.render_widget(Paragraph::new(l2), chunks[1]);
}

fn draw_tabs(frame: &mut Frame, pane: DetailPane, area: Rect) {
    let tabs = [DetailPane::Stream, DetailPane::Gate, DetailPane::Diff];
    let mut spans = Vec::new();
    for (i, t) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        let label = format!(" {} ", t.label());
        if *t == pane {
            spans.push(Span::styled(
                label,
                Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD),
            ));
        } else {
            spans.push(Span::raw(label));
        }
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_body(frame: &mut Frame, app: &App, run: &RunRow, pane: DetailPane, area: Rect) {
    let title = match pane {
        DetailPane::Stream if run.output_truncated => " stream (truncated) ",
        DetailPane::Stream => " stream ",
        DetailPane::Gate => " gate ",
        DetailPane::Diff => " diff ",
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = body_text(run, pane);
    let lines: Vec<&str> = text.lines().collect();
    let total = lines.len().max(1);
    let start = viewport_start(total, inner.height, app.detail_scroll, app.stream_follow);
    let end = (start + inner.height as usize).min(total);
    let visible: Vec<Line> = lines[start..end]
        .iter()
        .map(|l| Line::from((*l).to_string()))
        .collect();

    frame.render_widget(Paragraph::new(visible).wrap(Wrap { trim: false }), inner);
}

fn body_text(run: &RunRow, pane: DetailPane) -> String {
    match pane {
        DetailPane::Stream => {
            if run.output.is_empty() {
                "(no output yet)".into()
            } else {
                run.output.clone()
            }
        }
        DetailPane::Gate => gate_log_lines(run).join("\n"),
        DetailPane::Diff => {
            if run.diff.is_empty() {
                if matches!(
                    run.state,
                    RunState::Queued | RunState::Running | RunState::Gating
                ) {
                    "Diff available when the run finishes (receipt).".into()
                } else {
                    "No file changes in this run.".into()
                }
            } else {
                run.diff.clone()
            }
        }
    }
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let follow = if matches!(
        app.screen,
        Screen::RunDetail {
            pane: DetailPane::Stream | DetailPane::Diff
        }
    ) {
        if app.stream_follow {
            " follow"
        } else {
            " scrolled"
        }
    } else {
        ""
    };
    let hints =
        format!(" [esc] back  [1]stream [2]gate [3]diff  [a]ttach  [k]ill  [r]etry{follow}");
    frame.render_widget(Paragraph::new(hints), area);
}
