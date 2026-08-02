//! Shared rendering helpers for Control Room and Run Detail.

/// Truncate to `max` display characters, appending `…` when cut.
pub fn truncate(s: &str, max: usize) -> String {
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
