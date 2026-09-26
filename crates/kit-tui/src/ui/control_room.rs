//! Control Room frame — PRD §4.2 live table (1.0 craft remake).
//!
//! Visual bar: `docs/dev/DESIGN-tui.md` + concept-control-room.jpg

use super::common::{
    draw_empty_state, draw_footer, draw_header, draw_too_small, too_small, truncate,
};
use crate::app::{App, RunFilter, RunRow, format_gate_label, format_state_label};
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
        // The resting fox belongs to a ready room, not to the missing-agents error.
        let fox = if app.fox_on_screen() {
            crate::fox::lines(crate::fox::frame_at(&app.clock, app.motion_enabled()))
        } else {
            Vec::new()
        };
        draw_empty_state(frame, chunks[1], &theme, &fox, message, hint);
    } else if app.display_order().is_empty() {
        // A filter left on must say it is hiding runs.
        draw_empty_state(
            frame,
            chunks[1],
            &theme,
            &[],
            filtered_out_message(app.run_filter),
            "press f to change the filter",
        );
    } else {
        draw_table(frame, app, chunks[1], &theme);
    }

    draw_footer(frame, chunks[2], &theme, &footer_hints(app), "");
}

/// Footer keys for what is selected: no `[k]ill` on a past run, and
/// `[l]and` on a proven run with changes.
fn footer_hints(app: &App) -> String {
    let selected = app.selected_run();
    let mut hints =
        String::from(" [↑↓] select  [d]ispatch  [b]oard  [f]ilter  [enter] open  [g]ate");
    if !selected.is_some_and(|r| r.past) {
        hints.push_str("  [k]ill");
    }
    hints.push_str("  [r]etry");
    if selected.is_some_and(RunRow::landable) {
        hints.push_str("  [l]and");
    }
    hints.push_str("  [?]help");
    hints
}

fn filtered_out_message(filter: RunFilter) -> &'static str {
    match filter {
        RunFilter::Fail => "No failed runs",
        RunFilter::Running => "No runs in flight",
        RunFilter::Done => "No finished runs",
        RunFilter::All => "No runs",
    }
}

/// Header stats: the agent strip (when it fits), then the counts. The
/// counts are right-aligned and their form depends on the width alone, so a
/// flash that fits beside them comes and goes without moving them; only the
/// strip gives way to it.
///
/// `draw_header` drops the stats whole when title + stats overflow, so this
/// picks a form that fits. Queued runs are counted whenever there are any
/// (16 dispatched never reads as 8); counts are this session's runs, with
/// past runs from receipts as `· N past`. In `--demo` the strip is hidden:
/// it describes this machine, not the fixture rows.
fn header_stats(app: &App, width: usize) -> String {
    let title = TITLE.chars().count() + 2;
    // A live flash carries the next action.
    let flash = app
        .flash_message()
        .map_or(0, |f| format!("  · {f}").chars().count());
    let strips: Vec<String> = if app.demo {
        Vec::new()
    } else {
        [app.agents_strip(), app.agents_strip_short()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect()
    };
    if app.runs.is_empty()
        && let Some(strip) = strips
            .iter()
            .find(|s| title + s.chars().count() + flash <= width)
    {
        return strip.clone();
    }
    let filter = app.run_filter.label();
    let (r, q, g, f) = (
        app.running_count(),
        app.queued_count(),
        app.gated_count(),
        app.fail_count(),
    );
    let (queued_wide, queued_short) = if q > 0 {
        (format!("{q} QUEUED  "), format!("{q}Q "))
    } else {
        (String::new(), String::new())
    };
    let past = if app.past_total > 0 {
        format!("  ·  {} past", app.past_total)
    } else {
        String::new()
    };
    let wide = format!("[{filter}]  {r} RUNNING  {queued_wide}{g} GATING  {f} FAIL{past}");
    let short = format!("[{filter}] {r}R {queued_short}{g}G {f}F{past}");
    let counts = if title + wide.chars().count() <= width {
        wide
    } else {
        short
    };
    let counts_n = counts.chars().count();
    if flash > 0 {
        // Narrow terminals: a flash that cannot show whole beside the counts
        // takes the line for its 2 s.
        return if title + counts_n + flash <= width {
            counts
        } else {
            String::new()
        };
    }
    strips
        .iter()
        .map(|strip| format!("{strip}  ·  {counts}"))
        .find(|s| title + s.chars().count() <= width)
        .unwrap_or(counts)
}

/// Empty Control Room copy — cold-start cockpit, not a blank form.
fn empty_room_copy(app: &App) -> (&'static str, &'static str) {
    let ready = app.agents_ready_count();
    if ready == 0 && !app.agents_probe.is_empty() {
        (
            "No coding agents on PATH",
            "install claude, codex or grok  ·  kit doctor  ·  or try kit --demo",
        )
    } else {
        (
            "No runs yet",
            "press d to give your agents a task  ·  ? help  ·  or try kit --demo",
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
    let mut groups: Vec<Vec<Line>> = Vec::with_capacity(order.len() + 2);
    let mut selected_group = 0;
    let mut past_shown = false;
    for &idx in &order {
        let run = &app.runs[idx];
        if run.past && !past_shown {
            past_shown = true;
            groups.push(vec![divider_line(app, theme, inner.width as usize)]);
        }
        let selected = selected_id.is_some_and(|id| *id == run.id);
        if selected {
            selected_group = groups.len();
        }
        let mut group = vec![data_line(run, selected, app, theme, widths)];
        if let Some(summary) = run.failure_summary() {
            group.push(annotation_line(
                &summary,
                selected,
                run.past,
                theme,
                inner.width as usize,
            ));
        }
        groups.push(group);
    }
    let hidden = app.past_hidden();
    if past_shown && hidden > 0 {
        groups.push(vec![Line::from(Span::styled(
            format!("  ↓ {hidden} more · kit receipt"),
            theme.dim(),
        ))]);
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

/// The muted rule between this session's runs and past runs from receipts.
/// With nothing live above it, it also says what to do next.
fn divider_line(app: &App, theme: &Theme, width: usize) -> Line<'static> {
    let next = if app.no_live_runs() {
        "  ·  d  give your agents a task"
    } else {
        ""
    };
    let text = truncate(&format!("── earlier · from receipts{next} "), width);
    let fill = width.saturating_sub(text.chars().count());
    Line::from(Span::styled(
        format!("{text}{}", "─".repeat(fill)),
        theme.dim(),
    ))
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
    if total < 70 {
        return narrow_widths(usable);
    }
    let pct = [20usize, 14, 36, 15, 15];
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

/// STATE (`⠋ GATING 12m`) and GATE (`UNCONFIGURED`) never clip: they take
/// their full width first. REPO, AGENT and TASK share the rest, and AGENT
/// (`codex·eng`) is the first to give way.
fn narrow_widths(usable: usize) -> [usize; 5] {
    const STATE: usize = 12;
    const GATE: usize = 12;
    let rest = usable.saturating_sub(STATE + GATE);
    let repo = (rest / 4).max(1);
    let agent = (rest * 3 / 10).max(1);
    let task = rest.saturating_sub(repo + agent).max(1);
    [repo, agent, task, STATE, GATE]
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
    let failed = matches!(run.state, RunState::Fail | RunState::Error);
    // The wash marks what needs attention now; a past failure keeps its red
    // words but not the band.
    let fail = failed && !run.past;
    let marker = if selected { "▶ " } else { "  " };
    let repo = format!("{marker}{}", run.repo_name());
    let mut state_label = format_state_label(run, &app.clock, app.motion_enabled());
    if run.past
        && let Some(ended) = run.ended_at
    {
        // A past run shows its age where a live one shows elapsed time.
        let aged = format!(
            "{state_label} {}",
            crate::past::format_age(ended, std::time::SystemTime::now())
        );
        if aged.chars().count() <= widths[3] {
            state_label = aged;
        }
    }
    let gate_label = format_gate_label(run);
    let base = if fail {
        theme.fail_row(selected)
    } else if selected {
        theme.selected_row()
    } else if run.past {
        theme.dim()
    } else {
        theme.body()
    };
    // NO CHANGES is amber: the checks passed, but proved nothing was done.
    let own_state = if run.no_changes() {
        theme.warn().add_modifier(Modifier::BOLD)
    } else if run.past && !failed {
        theme.dim()
    } else {
        theme.state_style(run.state)
    };
    let own_gate = if run.past && gate_label != "FAIL" {
        theme.dim()
    } else {
        theme.gate_style(&gate_label)
    };
    // FAIL rows are one solid wash: the state and gate colours sit on it.
    let state_style = if fail {
        base.patch(own_state)
    } else if selected {
        theme.selected_row()
    } else {
        own_state
    };
    let gate_style = if fail {
        base.patch(own_gate)
    } else if selected {
        theme.selected_row()
    } else {
        own_gate
    };
    let repo_cell = pad_cell(&repo, widths[0]);
    let mut spans = Vec::with_capacity(11);
    if selected {
        // Cyan focus rail: the caret, then the rest of the row.
        let rail: String = repo_cell.chars().take(2).collect();
        let rest: String = repo_cell.chars().skip(2).collect();
        // On a FAIL row the caret sits on the wash; elsewhere it is the rail.
        let rail_style = if fail {
            base.patch(theme.accent())
        } else {
            theme.accent()
        };
        spans.push(Span::styled(rail, rail_style.add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(rest, base));
    } else {
        spans.push(Span::styled(repo_cell, base));
    }
    let parts = [
        (pad_cell(&run.agent_cell(), widths[1]), base),
        (pad_cell(run.task_line(), widths[2]), base),
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
    past: bool,
    theme: &Theme,
    inner_width: usize,
) -> Line<'static> {
    let budget = inner_width.saturating_sub(2); // "^ "
    let text = format!(
        "^ {}",
        truncate(
            summary.lines().next().unwrap_or(""),
            budget.saturating_sub(2)
        )
    );
    // Pad to the border so the wash is a band, not a highlight on the text.
    let text = format!("{text:<inner_width$}");
    let style = if past {
        // No wash under a past failure: the red words are enough.
        theme.annotation()
    } else if selected {
        theme.fail_row(true).add_modifier(Modifier::DIM)
    } else {
        // The wash's colour only: BOLD from the row plus DIM reads as neither.
        match theme.fail_row(false).bg {
            Some(bg) => theme.annotation().bg(bg),
            None => theme.annotation(),
        }
    };
    Line::from(Span::styled(text, style))
}
