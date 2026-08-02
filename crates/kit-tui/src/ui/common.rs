//! Shared rendering helpers — one interaction voice across screens.
//!
//! Design: `docs/dev/DESIGN-tui.md` header / body / footer grammar.

use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Soft floor: below this, show a clear message instead of a crushed layout.
pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 12;

/// Truncate to `max` display characters, appending `…` when cut.
pub fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Compute the first line index to show for a scrollable body.
pub fn viewport_start(total_lines: usize, height: u16, scroll: u16, follow: bool) -> usize {
    let height = height as usize;
    if total_lines <= height {
        return 0;
    }
    if follow {
        return total_lines - height;
    }
    let max_start = total_lines - height;
    (scroll as usize).min(max_start)
}

/// True when the frame is below the usable floor.
pub fn too_small(area: Rect) -> bool {
    area.width < MIN_WIDTH || area.height < MIN_HEIGHT
}

/// Full-area message when the terminal is unusably small.
pub fn draw_too_small(frame: &mut Frame, area: Rect, theme: &Theme) {
    let msg = format!(
        "terminal too small — need {MIN_WIDTH}×{MIN_HEIGHT} (now {}×{})",
        area.width, area.height
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(msg, theme.warn()))).centered(),
        area,
    );
}

/// Header: brand/title left, stats right, optional flash/error.
pub fn draw_header(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    title: &str,
    stats: &str,
    flash: Option<&str>,
    error: Option<&str>,
) {
    let width = area.width as usize;
    let mut left = title.to_string();
    if let Some(f) = flash {
        let tag = format!("  · {f}");
        if left.chars().count() + tag.chars().count()
            < width.saturating_sub(stats.chars().count() + 2)
        {
            left.push_str(&tag);
        }
    } else if let Some(e) = error {
        let tag = format!("  ! {e}");
        if left.chars().count() + tag.chars().count()
            < width.saturating_sub(stats.chars().count() + 2)
        {
            left.push_str(&tag);
        }
    }

    let left_n = left.chars().count();
    let right_n = stats.chars().count();
    let line = if left_n + 1 + right_n <= width {
        let spaces = width.saturating_sub(left_n + right_n);
        format!("{left}{}{stats}", " ".repeat(spaces))
    } else {
        truncate(&left, width)
    };

    // Paint title portion bold/accent; rest dim stats if we can split simply.
    let spans = if line.starts_with(title) && !stats.is_empty() && line.ends_with(stats) {
        let mid = line.len().saturating_sub(stats.len());
        vec![
            Span::styled(line[..mid].to_string(), theme.title()),
            Span::styled(line[mid..].to_string(), theme.dim()),
        ]
    } else {
        vec![Span::styled(line, theme.title())]
    };

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Footer hints (left) + optional status (right).
pub fn draw_footer(frame: &mut Frame, area: Rect, theme: &Theme, hints: &str, status: &str) {
    let width = area.width as usize;
    let hints_n = hints.chars().count();
    let status_n = status.chars().count();
    let text = if status.is_empty() {
        truncate(hints, width)
    } else if hints_n + 1 + status_n <= width {
        let spaces = width.saturating_sub(hints_n + status_n);
        format!("{hints}{}{status}", " ".repeat(spaces))
    } else {
        truncate(hints, width)
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(text, theme.footer()))),
        area,
    );
}

/// Empty-state body: one primary message + one action hint.
pub fn draw_empty_state(frame: &mut Frame, area: Rect, theme: &Theme, message: &str, hint: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border(false));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(message, theme.body())),
        Line::from(Span::styled(hint, theme.dim().add_modifier(Modifier::BOLD))),
    ];
    frame.render_widget(Paragraph::new(lines).centered(), inner);
}

/// Centered help overlay (clear + bordered panel).
pub fn draw_help_overlay(frame: &mut Frame, area: Rect, theme: &Theme, lines: Vec<Line<'static>>) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" help ")
        .border_style(theme.border(true))
        .title_style(theme.title());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(lines).style(theme.body()), inner);
}

/// Style a single stream/log line with cheap severity heuristics.
pub fn style_log_line(theme: &Theme, line: &str) -> Style {
    let lower = line.to_ascii_lowercase();
    if lower.contains("fail")
        || lower.contains("error")
        || lower.contains("panic")
        || lower.contains("fatal")
        || (line.starts_with('-') && !line.starts_with("---"))
    {
        theme.danger()
    } else if lower.contains("warn") {
        theme.warn()
    } else if lower.contains("pass")
        || lower.contains("ok")
        || (line.starts_with('+') && !line.starts_with("+++"))
    {
        theme.success()
    } else {
        theme.body()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_unchanged() {
        assert_eq!(truncate("hi", 5), "hi");
    }

    #[test]
    fn truncate_long_ellipsis() {
        let s = truncate("abcdef", 4);
        assert_eq!(s.chars().count(), 4);
        assert!(s.ends_with('…'));
    }

    #[test]
    fn too_small_detects_floor() {
        assert!(too_small(Rect::new(0, 0, 40, 20)));
        assert!(too_small(Rect::new(0, 0, 80, 8)));
        assert!(!too_small(Rect::new(0, 0, 80, 24)));
    }
}
