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
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

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
    let filter = app.run_filter.label();
    let stats = if agents.is_empty() {
        format!(
            "[{filter}]  {} RUNNING  {} GATING  {} FAIL",
            app.running_count(),
            app.gated_count(),
            app.fail_count()
        )
    } else if app.runs.is_empty() {
        agents
    } else {
        format!(
            "[{filter}]  {}  ·  {}R {}G {}F",
            agents,
            app.running_count(),
            app.gated_count(),
            app.fail_count()
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
        " [↑↓] select  [d]ispatch  [b]oard  [f]ilter  [enter] open  [g]ate  [k]ill  [r]etry  [?]help",
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
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border(true))
        .title(" runs ")
        .title_style(theme.dim());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let widths = column_widths(inner.width);
    let mut lines: Vec<Line> = vec![header_line(widths, theme)];
    let order = app.display_order();
    let selected_id = app.selected_id.as_ref();
    for &idx in &order {
        let run = &app.runs[idx];
        let selected = selected_id.is_some_and(|id| *id == run.id);
        lines.push(data_line(run, selected, app, theme, widths));
        if let Some(summary) = run.gate_summary() {
            lines.push(annotation_line(
                &summary,
                selected,
                theme,
                inner.width as usize,
            ));
        }
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Five Control Room columns. FAIL annotations are *not* a table cell —
/// they paint full inner width so `^ tsc: 3 errors` survives 60 cols.
fn column_widths(total: u16) -> [usize; 5] {
    let usable = total.saturating_sub(4) as usize; // 4 gaps between 5 cols
    let pct = if total < 70 {
        [18usize, 16, 30, 18, 18]
    } else {
        [20, 14, 36, 15, 15]
    };
    let mut w = [0usize; 5];
    let mut used = 0;
    for i in 0..4 {
        w[i] = (usable * pct[i] / 100).max(1);
        used += w[i];
    }
    w[4] = usable.saturating_sub(used).max(1);
    // `codex·eng` and `GATING 2m` must not collapse before TASK does.
    if w[1] < 12 && w[2] > 20 {
        let need = 12 - w[1];
        w[2] -= need;
        w[1] += need;
    }
    if w[3] < 12 && w[2] > 16 {
        let need = 12 - w[3];
        w[2] -= need;
        w[3] += need;
    }
    w
}

fn pad_cell(s: &str, width: usize) -> String {
    let n = s.chars().count();
    if n >= width {
        truncate(s, width)
    } else {
        format!("{s}{}", " ".repeat(width - n))
    }
}

fn header_line(widths: [usize; 5], theme: &Theme) -> Line<'static> {
    let text = format!(
        "{} {} {} {} {}",
        pad_cell("REPO", widths[0]),
        pad_cell("AGENT", widths[1]),
        pad_cell("TASK", widths[2]),
        pad_cell("STATE", widths[3]),
        pad_cell("GATE", widths[4]),
    );
    Line::from(Span::styled(text, theme.dim().add_modifier(Modifier::BOLD)))
}

fn data_line(
    run: &RunRow,
    selected: bool,
    app: &App,
    theme: &Theme,
    widths: [usize; 5],
) -> Line<'static> {
    let fail = matches!(run.state, RunState::Fail | RunState::Error);
    let marker = if selected { "▶ " } else { "  " };
    let repo = format!("{marker}{}", run.repo);
    let state_label = format_state_label(run, &app.clock, app.motion_enabled());
    let gate_label = format_gate_label(run);
    let base = if fail {
        theme.fail_row(selected)
    } else if selected {
        theme.selected_row()
    } else {
        theme.body()
    };
    let state_style = if fail {
        theme.state_style(run.state)
    } else if selected {
        theme.selected_row()
    } else {
        theme.state_style(run.state)
    };
    let gate_style = if fail {
        theme.gate_style(&gate_label)
    } else if selected {
        theme.selected_row()
    } else {
        theme.gate_style(&gate_label)
    };
    let repo_cell = pad_cell(&repo, widths[0]);
    let mut spans = Vec::with_capacity(11);
    if selected {
        // Cyan focus rail: the caret, then the rest of the row.
        let rail: String = repo_cell.chars().take(2).collect();
        let rest: String = repo_cell.chars().skip(2).collect();
        spans.push(Span::styled(
            rail,
            theme.accent().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(rest, base));
    } else {
        spans.push(Span::styled(repo_cell, base));
    }
    let parts = [
        (pad_cell(&run.agent_cell(), widths[1]), base),
        (pad_cell(&run.task, widths[2]), base),
        (pad_cell(&state_label, widths[3]), state_style),
        (pad_cell(&gate_label, widths[4]), gate_style),
    ];
    for (cell, style) in parts {
        spans.push(Span::styled(" ", base));
        spans.push(Span::styled(cell, style));
    }
    Line::from(spans)
}

fn annotation_line(
    summary: &str,
    selected: bool,
    theme: &Theme,
    inner_width: usize,
) -> Line<'static> {
    let budget = inner_width.saturating_sub(2); // "^ "
    let text = format!("^ {}", truncate(summary, budget.saturating_sub(2)));
    let style = if selected {
        theme.fail_row(true).add_modifier(Modifier::DIM)
    } else {
        theme.annotation()
    };
    Line::from(Span::styled(text, style))
}
