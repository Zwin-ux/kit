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

const TITLE: &str = "KIT / CONTROL ROOM";

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

    let stats = header_stats(app, area.width as usize);
    draw_header(
        frame,
        chunks[0],
        &theme,
        TITLE,
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

/// Header counts first, then the agent strip, longest form that fits.
///
/// `draw_header` drops the stats whole when title + stats overflow, so this
/// picks a form that fits: queued runs are counted whenever there are any
/// (16 dispatched never reads as 8), and the counts outlive the strip.
fn header_stats(app: &App, width: usize) -> String {
    let filter = app.run_filter.label();
    let (r, q, g, f) = (
        app.running_count(),
        app.queued_count(),
        app.gated_count(),
        app.fail_count(),
    );
    let fits = |s: &str| TITLE.chars().count() + 2 + s.chars().count() <= width;
    let strips = [app.agents_strip(), app.agents_strip_short()];
    if app.runs.is_empty()
        && let Some(strip) = strips.iter().find(|s| !s.is_empty() && fits(s))
    {
        return strip.clone();
    }
    let queued_wide = if q > 0 {
        format!("{q} QUEUED  ")
    } else {
        String::new()
    };
    let queued_short = if q > 0 {
        format!("{q}Q ")
    } else {
        String::new()
    };
    let wide = format!("[{filter}]  {r} RUNNING  {queued_wide}{g} GATING  {f} FAIL");
    let short = format!("[{filter}] {r}R {queued_short}{g}G {f}F");
    let mut candidates = Vec::new();
    for counts in [&wide, &short] {
        for strip in &strips {
            if !strip.is_empty() {
                candidates.push(format!("{counts}  ·  {strip}"));
            }
        }
        candidates.push(counts.clone());
    }
    candidates.into_iter().find(|s| fits(s)).unwrap_or(short)
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
    let order = app.display_order();
    let selected_id = app.selected_id.as_ref();
    // One group per run: its row, then its FAIL annotation if any.
    let mut groups: Vec<Vec<Line>> = Vec::with_capacity(order.len());
    let mut selected_group = 0;
    for &idx in &order {
        let run = &app.runs[idx];
        let selected = selected_id.is_some_and(|id| *id == run.id);
        if selected {
            selected_group = groups.len();
        }
        let mut group = vec![data_line(run, selected, app, theme, widths)];
        if let Some(summary) = run.gate_summary() {
            group.push(annotation_line(
                &summary,
                selected,
                theme,
                inner.width as usize,
            ));
        }
        groups.push(group);
    }

    let budget = inner.height.saturating_sub(1) as usize; // column header
    let (start, end) = visible_groups(&groups, selected_group, budget);
    let mut lines: Vec<Line> = vec![header_line(widths, theme)];
    lines.extend(groups[start..end].iter().flatten().cloned());
    if start > 0 || end < groups.len() {
        lines.push(overflow_line(start, groups.len() - end, theme));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

/// The runs that fit in `budget` lines, as a range of `groups`. When they do
/// not all fit, one line is kept for the overflow count and the range is the
/// first page that holds the selected run.
fn visible_groups(groups: &[Vec<Line>], selected: usize, budget: usize) -> (usize, usize) {
    let total: usize = groups.iter().map(Vec::len).sum();
    if total <= budget {
        return (0, groups.len());
    }
    let budget = budget.saturating_sub(1); // overflow line
    let fits =
        |start: usize, end: usize| groups[start..end].iter().map(Vec::len).sum::<usize>() <= budget;
    // Earliest start that still shows the selected run.
    let mut start = 0;
    while start < selected && !fits(start, selected + 1) {
        start += 1;
    }
    let mut end = start;
    while end < groups.len() && fits(start, end + 1) {
        end += 1;
    }
    (start, end.max((start + 1).min(groups.len())))
}

fn overflow_line(above: usize, below: usize, theme: &Theme) -> Line<'static> {
    let mut parts = Vec::with_capacity(2);
    if above > 0 {
        parts.push(format!("↑ {above} more above"));
    }
    if below > 0 {
        parts.push(format!("↓ {below} more below"));
    }
    Line::from(Span::styled(
        format!("  {}", parts.join("  ·  ")),
        theme.dim(),
    ))
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

/// The table's one line of a task. Retry tasks carry the gate failure on
/// later lines; a newline inside a cell would shift every column after it.
fn first_line(s: &str) -> &str {
    s.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim_end()
}

/// REPO shows the folder name, as Dispatch does; the engine keeps the path.
fn repo_label(repo: &str) -> &str {
    std::path::Path::new(repo)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(repo)
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
    let repo = format!("{marker}{}", repo_label(&run.repo));
    let state_label = format_state_label(run, &app.clock, app.motion_enabled());
    let gate_label = format_gate_label(run);
    let base = if fail {
        theme.fail_row(selected)
    } else if selected {
        theme.selected_row()
    } else {
        theme.body()
    };
    // FAIL rows are one solid wash: the state and gate colours sit on it.
    let state_style = if fail {
        base.patch(theme.state_style(run.state))
    } else if selected {
        theme.selected_row()
    } else {
        theme.state_style(run.state)
    };
    let gate_style = if fail {
        base.patch(theme.gate_style(&gate_label))
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
            base.patch(theme.accent()).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(rest, base));
    } else {
        spans.push(Span::styled(repo_cell, base));
    }
    let parts = [
        (pad_cell(&run.agent_cell(), widths[1]), base),
        (pad_cell(first_line(&run.task), widths[2]), base),
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
    let text = format!(
        "^ {}",
        truncate(first_line(summary), budget.saturating_sub(2))
    );
    // Pad to the border so the wash is a band, not a highlight on the text.
    let text = format!("{text:<inner_width$}");
    let style = if selected {
        theme.fail_row(true).add_modifier(Modifier::DIM)
    } else {
        theme.fail_row(false).patch(theme.annotation())
    };
    Line::from(Span::styled(text, style))
}
