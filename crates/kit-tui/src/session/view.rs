//! Drawing the session view: scrollback lines (printed once) and the live
//! region (spinner or approval, composer, status bar). Pure functions of the
//! state, so every state has a snapshot test at 80 and 60 columns.

use super::state::{Busy, Card, CardOutcome, Out, Session, Status};
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use std::time::Instant;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Rows the live region takes: activity (2), composer (3), status (1).
pub const LIVE_HEIGHT: u16 = 6;

const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];

/// One scrollback entry as terminal lines, hard-wrapped to `width`.
pub fn out_lines(out: &Out, theme: &Theme, width: u16) -> Vec<Line<'static>> {
    let dim = theme.dim();
    let line = match out {
        Out::You(text) => {
            let mut lines = vec![];
            for (i, l) in text.lines().enumerate() {
                let lead = if i == 0 { "you › " } else { "      " };
                lines.push(Line::from(vec![
                    Span::styled(format!("  {lead}"), dim),
                    Span::styled(l.to_owned(), theme.body().add_modifier(Modifier::BOLD)),
                ]));
            }
            return wrap(lines, width, 8);
        }
        Out::Kit { first, text } => {
            let lead = if *first {
                Span::styled("  kit › ", theme.title())
            } else {
                Span::raw("        ")
            };
            Line::from(vec![lead, Span::styled(text.clone(), theme.body())])
        }
        Out::Card(card) => card_line(card, theme, width),
        Out::Notice(text) => Line::from(vec![
            Span::styled("  ! ", theme.warn()),
            Span::styled(text.clone(), theme.warn()),
        ]),
        Out::Dim(text) => Line::from(Span::styled(format!("  {text}"), dim)),
        Out::Blank => Line::default(),
    };
    wrap(vec![line], width, 8)
}

/// `    ▸ Edit  crates/auth/src/token.rs        #3  ok` — one line; the target is cut, never wrapped.
fn card_line(card: &Card, theme: &Theme, width: u16) -> Line<'static> {
    let (mark, style) = match card.outcome {
        CardOutcome::Ok => ("▸", theme.dim()),
        CardOutcome::Failed => ("✗", theme.danger()),
        CardOutcome::Denied => ("⊘", theme.warn()),
    };
    let detail_style = match card.outcome {
        CardOutcome::Ok => theme.dim(),
        _ => style,
    };
    let number = format!("#{}", card.n);
    let verb = format!("{:<5}", card.verb);
    // "    ▸ " + verb + "  " + target + "  " + detail + "  " + number
    let fixed = 6 + verb.width() + 2 + 2 + 2 + number.width();
    let room = (width as usize).saturating_sub(fixed);
    let detail = cut(&card.detail, room.min(36).max(room / 3));
    let target_room = room.saturating_sub(detail.width());
    let target = cut(&card.target, target_room);
    let gap = target_room.saturating_sub(target.width());
    Line::from(vec![
        Span::styled(format!("    {mark} "), style),
        Span::styled(verb, theme.body().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(target, theme.body()),
        Span::raw(" ".repeat(gap + 2)),
        Span::styled(detail, detail_style),
        Span::raw("  "),
        Span::styled(number, theme.dim()),
    ])
}

/// Cut `s` to `max` columns with an ellipsis.
fn cut(s: &str, max: usize) -> String {
    if s.width() <= max {
        return s.to_owned();
    }
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = c.width().unwrap_or(0);
        if w + cw + 1 > max {
            break;
        }
        out.push(c);
        w += cw;
    }
    out.push('…');
    out
}

/// Hard-wrap styled lines to `width`; continuation lines are indented by `indent`.
fn wrap(lines: Vec<Line<'static>>, width: u16, indent: usize) -> Vec<Line<'static>> {
    let width = (width as usize).max(indent + 8);
    let mut out = vec![];
    for line in lines {
        if line.width() <= width {
            out.push(line);
            continue;
        }
        let mut current: Vec<Span<'static>> = vec![];
        let mut w = 0;
        for span in line.spans {
            let mut buf = String::new();
            for c in span.content.chars() {
                let cw = c.width().unwrap_or(0);
                if w + cw > width {
                    if !buf.is_empty() {
                        current.push(Span::styled(std::mem::take(&mut buf), span.style));
                    }
                    out.push(Line::from(std::mem::take(&mut current)));
                    current.push(Span::raw(" ".repeat(indent)));
                    w = indent;
                }
                buf.push(c);
                w += cw;
            }
            if !buf.is_empty() {
                current.push(Span::styled(buf, span.style));
            }
        }
        out.push(Line::from(current));
    }
    out
}

/// The live region: activity rows, composer, status bar.
pub fn render_live(
    s: &Session,
    theme: &Theme,
    area: Rect,
    buf: &mut Buffer,
    now: Instant,
    motion: bool,
) {
    if area.height < LIVE_HEIGHT || area.width < 20 {
        Paragraph::new("kit: the terminal is too small")
            .style(theme.dim())
            .render(area, buf);
        return;
    }
    let activity = Rect { height: 2, ..area };
    let composer = Rect {
        y: area.y + 2,
        height: 3,
        ..area
    };
    let status = Rect {
        y: area.y + 5,
        height: 1,
        ..area
    };
    Paragraph::new(activity_lines(s, theme, area.width, now, motion)).render(activity, buf);
    render_composer(s, theme, composer, buf);
    Paragraph::new(status_line(&s.status, theme, area.width)).render(status, buf);
}

fn activity_lines(
    s: &Session,
    theme: &Theme,
    width: u16,
    now: Instant,
    motion: bool,
) -> Vec<Line<'static>> {
    let w = width as usize;
    if let Some(a) = &s.approval {
        let ask = format!("{} {}", a.tool, a.target);
        let mut first = vec![
            Span::styled("  ? ", theme.warn()),
            Span::styled(
                cut(&ask, w.saturating_sub(4)),
                theme.body().add_modifier(Modifier::BOLD),
            ),
        ];
        if first.iter().map(|s| s.width()).sum::<usize>() > w {
            first.truncate(2);
        }
        let mut choices = vec![
            Span::styled("    y", theme.accent()),
            Span::styled(" once  ", theme.dim()),
            Span::styled("a", theme.accent()),
            Span::styled(" always  ", theme.dim()),
            Span::styled("n", theme.accent()),
            Span::styled(" deny", theme.dim()),
        ];
        if !a.reason.is_empty() {
            let used: usize = choices.iter().map(|s| s.width()).sum();
            choices.push(Span::styled(
                format!("   {}", cut(&a.reason, w.saturating_sub(used + 3))),
                theme.dim(),
            ));
        }
        return vec![Line::from(first), Line::from(choices)];
    }
    let preview = if s.partial.trim().is_empty() {
        s.flash
            .clone()
            .map(|f| Line::from(Span::styled(format!("  {f}"), theme.warn())))
            .unwrap_or_default()
    } else {
        // The line still streaming: its tail, so the newest words are visible.
        let text = s.partial.replace('\t', "    ");
        let room = w.saturating_sub(8);
        let tail: String = if text.width() > room {
            let mut t: Vec<char> = vec![];
            let mut used = 1;
            for c in text.chars().rev() {
                used += c.width().unwrap_or(0);
                if used > room {
                    break;
                }
                t.push(c);
            }
            std::iter::once('…').chain(t.into_iter().rev()).collect()
        } else {
            text
        };
        let lead = if s.kit_started {
            Span::raw("        ")
        } else {
            Span::styled("  kit › ", theme.title())
        };
        Line::from(vec![lead, Span::styled(tail, theme.body())])
    };
    let busy = match &s.busy {
        Some(b) => {
            let since = s
                .busy_since
                .map(|t| now.saturating_duration_since(t))
                .unwrap_or_default();
            let frame = if motion {
                SPINNER[(since.as_millis() / 125) as usize % SPINNER.len()]
            } else {
                '•'
            };
            let style = match b {
                Busy::Asking => theme.warn(),
                _ => Style::default().fg(theme.live),
            };
            let style = if theme.monochrome {
                theme.body()
            } else {
                style
            };
            let word = format!(" {} · {}s", b.word(), since.as_secs());
            Line::from(vec![
                Span::styled(format!("  {frame}"), style),
                Span::styled(cut(&word, w.saturating_sub(3)), theme.dim()),
            ])
        }
        None if s.partial.trim().is_empty() && s.flash.is_some() => Line::default(),
        None => Line::default(),
    };
    vec![preview, busy]
}

fn render_composer(s: &Session, theme: &Theme, area: Rect, buf: &mut Buffer) {
    let focused = s.approval.is_none();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border(focused));
    let inner = block.inner(area);
    block.render(area, buf);
    let room = inner.width.saturating_sub(2) as usize;
    let line = if s.composer.text.is_empty() {
        let hint = match (&s.approval, &s.busy) {
            (Some(_), _) => "Answer above: y, a or n",
            (None, Some(_)) => "Esc to interrupt",
            (None, None) => "Ask Kit to do something · ? for help",
        };
        Line::from(vec![
            Span::styled("› ", theme.accent()),
            Span::styled(cut(hint, room), theme.dim()),
        ])
    } else {
        // Show the line the cursor is on, scrolled so the cursor stays visible.
        let before = &s.composer.text[..s.composer.cursor];
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = s.composer.text[s.composer.cursor..]
            .find('\n')
            .map(|i| s.composer.cursor + i)
            .unwrap_or(s.composer.text.len());
        let current = &s.composer.text[line_start..line_end];
        let col = s.composer.text[line_start..s.composer.cursor].width();
        let skip = col.saturating_sub(room.saturating_sub(1));
        let mut shown = String::new();
        let mut w = 0;
        for c in current.chars() {
            let cw = c.width().unwrap_or(0);
            if w >= skip && w - skip + cw <= room {
                shown.push(c);
            }
            w += cw;
        }
        let lines = s.composer.text.matches('\n').count() + 1;
        let lead = if lines > 1 {
            format!("{lines}")
        } else {
            "›".into()
        };
        Line::from(vec![
            Span::styled(format!("{lead} "), theme.accent()),
            Span::styled(shown, theme.body()),
        ])
    };
    Paragraph::new(line).render(inner, buf);
}

/// `claude-sonnet-5-5 · 18.2k ctx · ≈$0.041 · workspace unsandboxed · main · ? help`,
/// dropping fields from the right until it fits.
pub fn status_line(st: &Status, theme: &Theme, width: u16) -> Line<'static> {
    let mut fields: Vec<String> = vec![st.model.clone()];
    if st.ctx > 0 {
        fields.push(format!("{} ctx", tokens(st.ctx)));
    }
    if !st.cost.is_empty() {
        fields.push(st.cost.clone());
    }
    fields.push(format!("{} unsandboxed", st.profile));
    if let Some(b) = &st.branch {
        fields.push(b.clone());
    }
    fields.push("? help".into());
    let fits = |f: &[String]| {
        2 + f.iter().map(|s| s.width()).sum::<usize>() + 3 * f.len().saturating_sub(1)
            <= width as usize
    };
    while fields.len() > 1 && !fits(&fields) {
        fields.pop();
    }
    let mut spans = vec![Span::raw("  ")];
    for (i, f) in fields.into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" · ", theme.dim()));
        }
        spans.push(Span::styled(f, theme.dim()));
    }
    Line::from(spans)
}

fn tokens(n: u64) -> String {
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::state::{Approval, Status};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use kit_loop::{AgentEvent, Outcome};
    use serde_json::json;
    use std::time::Duration;

    fn status() -> Status {
        Status {
            model: "claude-sonnet-5-5".into(),
            ctx: 18_200,
            cost: "≈$0.041".into(),
            profile: "workspace".into(),
            branch: Some("main".into()),
        }
    }

    /// The scrollback entries plus the live region, as plain text.
    fn frame(s: &mut Session, width: u16, now: Instant) -> String {
        let theme = Theme::monochrome();
        let mut text = String::new();
        for out in s.drain_out() {
            for line in out_lines(&out, &theme, width) {
                let l: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
                text.push_str(l.trim_end());
                text.push('\n');
            }
        }
        text.push_str(&"─".repeat(width as usize));
        text.push('\n');
        let area = Rect::new(0, 0, width, LIVE_HEIGHT);
        let mut buf = Buffer::empty(area);
        render_live(s, &theme, area, &mut buf, now, true);
        for y in 0..LIVE_HEIGHT {
            let row: String = (0..width)
                .map(|x| buf[(x, y)].symbol().to_owned())
                .collect();
            text.push_str(row.trim_end());
            text.push('\n');
        }
        text
    }

    fn type_and_send(s: &mut Session, text: &str, now: Instant) {
        for c in text.chars() {
            s.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE), now);
        }
        s.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), now);
    }

    fn working(width: u16) -> String {
        let t0 = Instant::now();
        let mut s = Session::new(status());
        type_and_send(&mut s, "the login test fails on Windows", t0);
        s.on_agent(
            AgentEvent::Text {
                text: "Reading the test first.\n".into(),
            },
            t0,
        );
        s.on_agent(
            AgentEvent::ToolStart {
                id: "1".into(),
                name: "Read".into(),
                args: json!({"file_path": "tests/login.rs"}),
            },
            t0,
        );
        s.on_agent(
            AgentEvent::ToolEnd {
                id: "1".into(),
                name: "Read".into(),
                content: "1\tuse auth;\n2\t\n3\tfn login() {}".into(),
                is_error: false,
                denied: false,
            },
            t0,
        );
        s.on_agent(
            AgentEvent::ToolStart {
                id: "2".into(),
                name: "Edit".into(),
                args: json!({"file_path": "crates/auth/src/token.rs"}),
            },
            t0,
        );
        s.on_agent(
            AgentEvent::ToolEnd {
                id: "2".into(),
                name: "Edit".into(),
                content: "Edited crates/auth/src/token.rs".into(),
                is_error: false,
                denied: false,
            },
            t0,
        );
        s.on_agent(
            AgentEvent::ToolStart {
                id: "3".into(),
                name: "Bash".into(),
                args: json!({"command": "cargo test -p auth login"}),
            },
            t0,
        );
        frame(&mut s, width, t0 + Duration::from_secs(4))
    }

    #[test]
    fn empty_at_80_and_60() {
        let now = Instant::now();
        insta::assert_snapshot!("empty_80", frame(&mut Session::new(status()), 80, now));
        insta::assert_snapshot!("empty_60", frame(&mut Session::new(status()), 60, now));
    }

    #[test]
    fn working_at_80_and_60() {
        insta::assert_snapshot!("working_80", working(80));
        insta::assert_snapshot!("working_60", working(60));
    }

    #[test]
    fn approval_at_80_and_60() {
        for width in [80, 60] {
            let now = Instant::now();
            let mut s = Session::new(status());
            type_and_send(&mut s, "ship it", now);
            s.ask(
                Approval {
                    tool: "Bash".into(),
                    target: "git push origin main".into(),
                    reason: "network and a remote write".into(),
                },
                now,
            );
            insta::assert_snapshot!(format!("approval_{width}"), frame(&mut s, width, now));
        }
    }

    #[test]
    fn streaming_and_errors_at_80() {
        let now = Instant::now();
        let mut s = Session::new(status());
        type_and_send(&mut s, "why is CI red", now);
        s.on_agent(
            AgentEvent::ToolEnd {
                id: "9".into(),
                name: "Bash".into(),
                content: "error[E0425]: cannot find value `tokn`\n  --> src/lib.rs:4:5".into(),
                is_error: true,
                denied: false,
            },
            now,
        );
        s.on_agent(
            AgentEvent::Notice {
                text: "rate limited, retrying in 8s".into(),
            },
            now,
        );
        s.on_agent(
            AgentEvent::Text {
                text: "The build fails because a variable is misspelled in src/lib.rs and".into(),
            },
            now,
        );
        insta::assert_snapshot!("streaming_80", frame(&mut s, 80, now));
        s.turn_ended(&Outcome::Interrupted);
        insta::assert_snapshot!("interrupted_80", frame(&mut s, 80, now));
    }

    #[test]
    fn the_status_bar_drops_fields_from_the_right() {
        let theme = Theme::monochrome();
        let text = |w| {
            status_line(&status(), &theme, w)
                .spans
                .iter()
                .map(|s| s.content.to_string())
                .collect::<String>()
        };
        assert_eq!(
            text(100),
            "  claude-sonnet-5-5 · 18.2k ctx · ≈$0.041 · workspace unsandboxed · main · ? help"
        );
        assert_eq!(text(60), "  claude-sonnet-5-5 · 18.2k ctx · ≈$0.041");
        assert_eq!(text(10), "  claude-sonnet-5-5");
    }

    #[test]
    fn long_prompts_wrap_with_an_indent() {
        let theme = Theme::monochrome();
        let lines = out_lines(&Out::You("a ".repeat(40)), &theme, 40);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| l.width() <= 40));
        let second: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(second.starts_with("        "));
    }
}
