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
    let msg = if area.width < 48 {
        format!(
            "need {MIN_WIDTH}×{MIN_HEIGHT} (now {}×{})",
            area.width, area.height
        )
    } else {
        format!(
            "need {MIN_WIDTH}×{MIN_HEIGHT} — maximize this terminal (now {}×{})",
            area.width, area.height
        )
    };
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
    let leftover = width.saturating_sub(title.chars().count() + stats.chars().count() + 2);
    if let Some(f) = flash
        && leftover >= 4
    {
        left.push_str(&truncate(&format!("  · {f}"), leftover));
    } else if let Some(e) = error
        && leftover >= 4
    {
        left.push_str(&truncate(&format!("  ! {e}"), leftover));
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
///
/// Prefer complete short-form tokens over mid-token truncation when width is tight.
pub fn draw_footer(frame: &mut Frame, area: Rect, theme: &Theme, hints: &str, status: &str) {
    let width = area.width as usize;
    let fitted = fit_footer_hints(
        hints,
        width.saturating_sub(if status.is_empty() {
            0
        } else {
            status.chars().count() + 1
        }),
    );
    let hints_n = fitted.chars().count();
    let status_n = status.chars().count();
    let text = if status.is_empty() {
        fitted
    } else if hints_n + 1 + status_n <= width {
        let spaces = width.saturating_sub(hints_n + status_n);
        format!("{fitted}{}{status}", " ".repeat(spaces))
    } else {
        truncate(&fitted, width)
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(text, theme.footer()))),
        area,
    );
}

/// Drop lower-priority tokens so the footer stays readable at 80 and 60 cols.
/// Tokens are split on `"  ["` boundaries (kit footer grammar).
///
/// Keep `[?]help`, `[r]etry`, `[k]ill`, and `[d]ispatch` before dropping
/// `[b]oard` / `[f]ilter` / `[enter] open`. Never drop mid-token.
pub fn fit_footer_hints(hints: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if hints.chars().count() <= max_chars {
        return hints.to_string();
    }
    let parts: Vec<&str> = hints.split("  [").collect();
    if parts.is_empty() {
        return truncate(hints, max_chars);
    }
    let mut tokens: Vec<String> = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        if i == 0 {
            if !p.is_empty() {
                tokens.push((*p).to_string());
            }
        } else {
            tokens.push(format!("[{p}"));
        }
    }
    while tokens.len() > 1 {
        let candidate = tokens.join("  ");
        if candidate.chars().count() <= max_chars {
            return candidate;
        }
        // Drop board / filter / enter first so help, retry, kill, dispatch survive.
        // Other screens keep the original drop-from-the-end behavior.
        if let Some(i) = drop_footer_index(&tokens) {
            tokens.remove(i);
        } else {
            tokens.pop();
        }
    }
    truncate(&tokens.join("  "), max_chars)
}

/// Drop filter/enter/panes before Board, so 80-col CR keeps `[b]oard`.
fn drop_footer_index(tokens: &[String]) -> Option<usize> {
    const ORDER: &[&str] = &[
        "[f]ilter",
        "[enter]",
        "[a]ttach",
        "[3]diff",
        "[2]gate",
        "[1]stream",
        "[b]oard",
    ];
    for prefix in ORDER {
        if let Some(i) = tokens.iter().position(|t| t.trim().starts_with(prefix)) {
            return Some(i);
        }
    }
    None
}

/// Empty-state body: one primary message + one action hint, under `art`
/// (every line the same width) when the panel has room for all of it.
pub fn draw_empty_state(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    art: &[String],
    message: &str,
    hint: &str,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border(false));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let art_width = art.first().map_or(0, |l| l.chars().count());
    let text_rows = 3; // gap, message, hint
    let need = art.len() + text_rows;
    let fits = !art.is_empty()
        && usize::from(inner.height) >= need
        && usize::from(inner.width) >= art_width;
    let mut lines = Vec::new();
    if fits {
        let pad = (usize::from(inner.height) - need) / 2;
        lines.extend(std::iter::repeat_n(Line::from(""), pad));
        lines.extend(
            art.iter()
                .map(|l| Line::from(Span::styled(l.clone(), theme.body()))),
        );
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(message, theme.body())));
    lines.push(Line::from(Span::styled(
        hint,
        theme.dim().add_modifier(Modifier::BOLD),
    )));
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

/// A path with the home folder written `~`, as the rest of Kit prints paths.
pub fn tilde(path: &std::path::Path) -> String {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    match home.map(std::path::PathBuf::from) {
        Some(home) if !home.as_os_str().is_empty() => match path.strip_prefix(&home) {
            Ok(rest) if rest.as_os_str().is_empty() => "~".into(),
            Ok(rest) => format!("~{}{}", std::path::MAIN_SEPARATOR, rest.display()),
            Err(_) => path.display().to_string(),
        },
        _ => path.display().to_string(),
    }
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
    } else if lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|w| matches!(w, "ok" | "pass" | "passed" | "passing"))
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

    /// "ok" and "pass" count only as whole words: grok, token and bypass
    /// are not success.
    #[test]
    fn log_success_matches_whole_words() {
        let theme = Theme::monochrome();
        let body = theme.body();
        for line in [
            "grok: running kit.toml gate",
            "refreshing token",
            "reading the book",
            "bypass the cache",
        ] {
            assert_eq!(style_log_line(&theme, line), body, "{line}");
        }
        for line in ["ok", "tests: ok (3)", "PASS  format", "check passed"] {
            assert_ne!(style_log_line(&theme, line), body, "{line}");
        }
    }

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

    const CR_FOOTER: &str = " [↑↓] select  [d]ispatch  [b]oard  [f]ilter  [enter] open  [g]ate  [k]ill  [r]etry  [?]help";

    #[test]
    fn fit_footer_drops_trailing_tokens_not_mid_token() {
        let fitted = fit_footer_hints(CR_FOOTER, 50);
        assert!(fitted.chars().count() <= 50);
        assert!(
            !fitted.ends_with('…') || fitted.contains('['),
            "prefer whole tokens: {fitted}"
        );
        // Still starts with select / primary grammar when possible.
        assert!(fitted.contains("select") || fitted.contains("dispatch"));
    }

    #[test]
    fn fit_footer_keeps_retry_help_kill_dispatch_at_80() {
        let fitted = fit_footer_hints(CR_FOOTER, 80);
        assert!(fitted.chars().count() <= 80, "{fitted}");
        assert!(
            fitted.contains("[r]etry") || fitted.contains("[r]"),
            "{fitted}"
        );
        assert!(
            fitted.contains("[?]help") || fitted.contains("[?]"),
            "{fitted}"
        );
        assert!(
            fitted.contains("[k]ill") || fitted.contains("[k]"),
            "{fitted}"
        );
        assert!(
            fitted.contains("[d]ispatch") || fitted.contains("[d]"),
            "{fitted}"
        );
        assert!(
            fitted.contains("[b]oard") || fitted.contains("[b]"),
            "80-col Control Room must keep Board: {fitted}"
        );
    }

    #[test]
    fn fit_footer_keeps_retry_help_kill_dispatch_at_60() {
        let fitted = fit_footer_hints(CR_FOOTER, 60);
        assert!(fitted.chars().count() <= 60, "{fitted}");
        assert!(
            fitted.contains("[r]etry") || fitted.contains("[r]"),
            "{fitted}"
        );
        assert!(
            fitted.contains("[?]help") || fitted.contains("[?]"),
            "{fitted}"
        );
        assert!(
            fitted.contains("[k]ill") || fitted.contains("[k]"),
            "{fitted}"
        );
        assert!(
            fitted.contains("[d]ispatch") || fitted.contains("[d]"),
            "{fitted}"
        );
    }
}
