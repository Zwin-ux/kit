//! The choice lists in `kit setup`: agents, focus and scope.
//!
//! One look for every question: a bold question, one row per option with
//! a cursor, a select dot (multi) and the agent's logo or mark, and a muted
//! key line. When answered, the list folds into one summary line
//! (`✓ Agents   Claude Code and Codex`), so the finished setup reads as a
//! short record of what was chosen.
//!
//! [`Picker`] is pure (keys in, lines out) and tested; [`run`] drives it on
//! the terminal in raw mode and draws logos where [`brand::graphics`] says
//! the terminal can.

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::{cursor, terminal};
use kit_core::AgentKind;
use kit_tui::ansi::Paint;
use kit_tui::brand;
use std::io::Write;

/// How a row's note reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Good,
    Warn,
    Muted,
}

/// One option.
#[derive(Debug, Clone)]
pub struct Item {
    pub label: String,
    /// Second column, muted.
    pub detail: String,
    /// Third column, in `tone`.
    pub note: String,
    pub tone: Tone,
    /// Shows this agent's logo or mark before the label.
    pub agent: Option<AgentKind>,
    /// The words for the summary line, when not the label.
    pub answer: Option<String>,
}

impl Item {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: String::new(),
            note: String::new(),
            tone: Tone::Plain,
            agent: None,
            answer: None,
        }
    }
    pub fn answer(mut self, a: impl Into<String>) -> Self {
        self.answer = Some(a.into());
        self
    }
    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = d.into();
        self
    }
    pub fn note(mut self, n: impl Into<String>, tone: Tone) -> Self {
        self.note = n.into();
        self.tone = tone;
        self
    }
    pub fn agent(mut self, a: AgentKind) -> Self {
        self.agent = Some(a);
        self
    }
}

/// A key the picker understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Toggle,
    Confirm,
    Cancel,
}

/// What a key did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Continue,
    Done(Vec<usize>),
    Cancelled,
}

/// A question and its options.
#[derive(Debug, Clone)]
pub struct Picker {
    pub question: String,
    /// The word for the summary line (`Agents`).
    pub summary: String,
    pub items: Vec<Item>,
    pub multi: bool,
    pub cursor: usize,
    pub chosen: Vec<bool>,
    pub error: Option<String>,
    /// Extra words for the key line (`kit show <kit> for details`).
    pub hint: Option<String>,
}

/// Cells before the mark in a multi-select: indent, cursor, dot.
const LOGO_COL: u16 = 6;

impl Picker {
    pub fn new(question: &str, summary: &str, items: Vec<Item>, multi: bool) -> Self {
        let n = items.len();
        Self {
            question: question.into(),
            summary: summary.into(),
            items,
            multi,
            cursor: 0,
            chosen: vec![false; n],
            error: None,
            hint: None,
        }
    }

    /// Pre-select these rows; the cursor starts on the first of them.
    pub fn select(mut self, rows: &[usize]) -> Self {
        for &i in rows {
            if i < self.chosen.len() {
                self.chosen[i] = true;
            }
        }
        if let Some(&first) = rows.iter().min() {
            self.cursor = first.min(self.items.len().saturating_sub(1));
        }
        self
    }

    pub fn hint(mut self, h: &str) -> Self {
        self.hint = Some(h.into());
        self
    }

    pub fn on_key(&mut self, key: Key) -> Step {
        let n = self.items.len();
        match key {
            Key::Up => self.cursor = (self.cursor + n - 1) % n,
            Key::Down => self.cursor = (self.cursor + 1) % n,
            Key::Toggle if self.multi => {
                self.chosen[self.cursor] = !self.chosen[self.cursor];
                self.error = None;
            }
            Key::Toggle => {}
            Key::Confirm if self.multi => {
                let picked: Vec<usize> = (0..n).filter(|&i| self.chosen[i]).collect();
                if picked.is_empty() {
                    self.error = Some("Pick at least one (space selects)".into());
                    return Step::Continue;
                }
                return Step::Done(picked);
            }
            Key::Confirm => return Step::Done(vec![self.cursor]),
            Key::Cancel => return Step::Cancelled,
        }
        Step::Continue
    }

    fn has_marks(&self) -> bool {
        self.items.iter().any(|i| i.agent.is_some())
    }

    /// Where the label starts, in cells. A single choice has no dot.
    fn label_col(&self) -> usize {
        let lead = if self.multi { LOGO_COL as usize } else { 4 };
        lead + if self.has_marks() { 3 } else { 0 }
    }

    /// The lines to draw: question, one per option, key line. `logos`
    /// leaves the mark cells blank for images to fill.
    pub fn lines(&self, width: usize, paint: &Paint, logos: bool) -> Vec<String> {
        let mut out = vec![paint.bold(&clip(&self.question, width.saturating_sub(1)))];
        let label_w = self
            .items
            .iter()
            .map(|i| i.label.chars().count())
            .max()
            .unwrap_or(0);
        let detail_w = self
            .items
            .iter()
            .filter(|i| !i.note.is_empty())
            .map(|i| i.detail.chars().count())
            .max()
            .unwrap_or(0);
        for (i, item) in self.items.iter().enumerate() {
            out.push(self.row(i, item, width, label_w, detail_w, paint, logos));
        }
        let keys = if self.multi {
            "↑↓ move · space select · enter confirm · esc cancel"
        } else {
            "↑↓ move · enter confirm · esc cancel"
        };
        let room = width.saturating_sub(3);
        let foot = match (&self.error, &self.hint) {
            (Some(e), _) => paint.warn(&clip(e, room)),
            (None, Some(h)) => paint.muted(&clip(&format!("{keys} · {h}"), room)),
            (None, None) => paint.muted(&clip(keys, room)),
        };
        out.push(format!("  {foot}"));
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn row(
        &self,
        i: usize,
        item: &Item,
        width: usize,
        label_w: usize,
        detail_w: usize,
        paint: &Paint,
        logos: bool,
    ) -> String {
        let here = i == self.cursor;
        let caret = if here {
            paint.accent("❯")
        } else {
            " ".into()
        };
        let dot = match (self.multi, self.chosen[i]) {
            (false, _) => String::new(),
            (true, true) => format!("{} ", paint.accent("●")),
            (true, false) => format!("{} ", paint.muted("○")),
        };
        let mark = match item.agent {
            _ if !self.has_marks() => String::new(),
            Some(_) if logos => "   ".into(),
            Some(a) => format!("{} ", paint.mark(a)),
            None => "   ".into(),
        };
        let mut s = format!("  {caret} {dot}{mark}");
        let room = width.saturating_sub(self.label_col() + 1);
        let label = clip(&format!("{:label_w$}", item.label), room);
        let used = label.chars().count();
        s.push_str(&if here {
            paint.bold(&label)
        } else {
            label.clone()
        });
        let mut room = room.saturating_sub(used);
        if !item.detail.is_empty() && room > 4 {
            let d = if item.note.is_empty() {
                item.detail.clone()
            } else {
                format!("{:detail_w$}", item.detail)
            };
            let d = clip(&format!("   {d}"), room);
            room = room.saturating_sub(d.chars().count());
            s.push_str(&paint.muted(&d));
        }
        if !item.note.is_empty() && room > 4 {
            let n = clip(&format!("   {}", item.note), room);
            s.push_str(&match item.tone {
                Tone::Plain => n,
                Tone::Good => paint.success(&n),
                Tone::Warn => paint.warn(&n),
                Tone::Muted => paint.muted(&n),
            });
        }
        s.trim_end().to_string()
    }

    /// The folded answer: `✓ Agents   Claude Code and Codex`.
    pub fn summary_line(&self, picked: &[usize], paint: &Paint) -> String {
        let names: Vec<&str> = picked
            .iter()
            .map(|&i| {
                let item = &self.items[i];
                item.answer.as_deref().unwrap_or(&item.label)
            })
            .collect();
        let answer = super::setup::and_list(&names);
        format!(
            "{} {}  {}",
            paint.success("✓"),
            paint.bold(&format!("{:11}", self.summary)),
            paint.accent(&answer)
        )
    }
}

/// `s` cut to `max` cells with `…`.
fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut out: String = s.chars().take(max - 1).collect();
    out = out.trim_end().to_string();
    out.push('…');
    out
}

/// Restores the terminal however the picker ends.
struct Raw;

impl Drop for Raw {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let mut out = std::io::stdout();
        let _ = crossterm::execute!(out, cursor::Show);
    }
}

/// Ask on the terminal. `None` when cancelled (Esc, Ctrl-C).
pub fn run(mut picker: Picker) -> Result<Option<Vec<usize>>> {
    if !kit_tui::ansi::vt_ready() {
        anyhow::bail!(
            "this console cannot draw kit setup's questions. Answer with flags instead:\n  \
             kit setup --agent claude --kit frontend-design --global --yes"
        );
    }
    let paint = Paint::stdout();
    let graphics = brand::graphics().filter(|_| picker.has_marks());
    let mut out = std::io::stdout();
    terminal::enable_raw_mode()?;
    let _raw = Raw;
    crossterm::execute!(out, cursor::Hide)?;

    let width = || terminal::size().map_or(80, |(w, _)| w as usize).max(20);
    let draw = |p: &Picker, out: &mut std::io::Stdout, first: bool| -> Result<usize> {
        let lines = p.lines(width(), &paint, graphics.is_some());
        let mut buf = String::new();
        if !first {
            // Back to the question line; logo cells are skipped, not
            // rewritten, so the images stay.
            buf.push_str(&format!("\r\x1b[{}A", lines.len() - 1));
        }
        for (n, line) in lines.iter().enumerate() {
            buf.push('\r');
            let row = n.checked_sub(1).and_then(|i| p.items.get(i));
            match (graphics, row.and_then(|r| r.agent)) {
                (Some(_), Some(_)) if !first => {
                    // Prefix, skip the logo cells, then the rest.
                    let (head, tail) = split_at_cells(line, LOGO_COL as usize);
                    buf.push_str(&head);
                    buf.push_str("\x1b[2C");
                    buf.push_str(&skip_cells(&tail, 2));
                }
                _ => buf.push_str(line),
            }
            buf.push_str("\x1b[K");
            if n + 1 < lines.len() {
                buf.push_str("\r\n");
            }
        }
        if first && let Some(g) = graphics {
            // Draw each logo once, from the bottom line upward.
            let last = lines.len() - 1;
            for (i, item) in p.items.iter().enumerate() {
                let Some(agent) = item.agent else { continue };
                let Some(logo) = brand::of(agent).logo(g, logo_id(i)) else {
                    continue;
                };
                let up = last - (i + 1);
                buf.push_str("\x1b7");
                if up > 0 {
                    buf.push_str(&format!("\x1b[{up}A"));
                }
                buf.push_str(&format!("\r\x1b[{LOGO_COL}C"));
                buf.push_str(&logo);
                buf.push_str("\x1b8");
            }
        }
        out.write_all(buf.as_bytes())?;
        out.flush()?;
        Ok(lines.len())
    };

    let height = draw(&picker, &mut out, true)?;
    let step = loop {
        let Event::Key(k) = event::read()? else {
            continue;
        };
        if k.kind != KeyEventKind::Press {
            continue;
        }
        let key = match k.code {
            KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => Key::Cancel,
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => Key::Up,
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => Key::Down,
            KeyCode::Char(' ') => Key::Toggle,
            KeyCode::Enter => Key::Confirm,
            KeyCode::Esc => Key::Cancel,
            _ => continue,
        };
        match picker.on_key(key) {
            Step::Continue => {
                draw(&picker, &mut out, false)?;
            }
            done => break done,
        }
    };

    // Fold the list into its summary line.
    let mut buf = String::new();
    if let Some(g) = graphics {
        for i in 0..picker.items.len() {
            buf.push_str(&brand::clear_logo(g, logo_id(i)));
        }
    }
    buf.push_str(&format!("\r\x1b[{}A\x1b[J", height - 1));
    let result = match step {
        Step::Done(picked) => {
            buf.push_str(&picker.summary_line(&picked, &paint));
            Some(picked)
        }
        _ => {
            buf.push_str(&format!(
                "{} {}",
                paint.muted("·"),
                paint.muted(&format!("{:11}  cancelled", picker.summary))
            ));
            None
        }
    };
    buf.push_str("\r\n");
    out.write_all(buf.as_bytes())?;
    out.flush()?;
    Ok(result)
}

/// A kitty image id per row, clear of ids other programs tend to use.
fn logo_id(row: usize) -> u32 {
    0x4b49_5400 + row as u32
}

/// Split a painted line after `cells` visible cells.
fn split_at_cells(s: &str, cells: usize) -> (String, String) {
    let mut seen = 0;
    let mut in_esc = false;
    for (i, c) in s.char_indices() {
        if in_esc {
            if c.is_ascii_alphabetic() {
                in_esc = false;
            }
            continue;
        }
        if c == '\x1b' {
            in_esc = true;
            continue;
        }
        if seen == cells {
            return (s[..i].to_string(), s[i..].to_string());
        }
        seen += 1;
    }
    (s.to_string(), String::new())
}

/// Drop the first `cells` visible cells of `s`, keeping its escapes.
fn skip_cells(s: &str, cells: usize) -> String {
    let (_, tail) = split_at_cells(s, cells);
    // Escapes before the cut still style the tail.
    let head_escapes: String = {
        let (head, _) = split_at_cells(s, cells);
        let mut keep = String::new();
        let mut esc = String::new();
        for c in head.chars() {
            if c == '\x1b' || !esc.is_empty() {
                esc.push(c);
                if c.is_ascii_alphabetic() {
                    keep.push_str(&esc);
                    esc.clear();
                }
            }
        }
        keep
    };
    format!("{head_escapes}{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use kit_tui::ansi::visible_len;
    use kit_tui::theme::Theme;

    fn agents() -> Picker {
        Picker::new(
            "Which agents should Kit set up?",
            "Agents",
            vec![
                Item::new("Claude Code")
                    .detail("2.1.283")
                    .note("logged in", Tone::Good)
                    .agent(AgentKind::Claude),
                Item::new("Codex")
                    .detail("codex-cli 0.155.0")
                    .note("not logged in", Tone::Warn)
                    .agent(AgentKind::Codex),
                Item::new("Grok")
                    .detail("not installed")
                    .note("Kit can still write its files", Tone::Muted)
                    .agent(AgentKind::Grok),
            ],
            true,
        )
        .select(&[0])
    }

    #[test]
    fn rows_line_up_with_marks() {
        let lines = agents().lines(100, &Paint::plain(), false);
        assert_eq!(
            lines,
            vec![
                "Which agents should Kit set up?",
                "  ❯ ● ✻  Claude Code   2.1.283             logged in",
                "    ○ >_ Codex         codex-cli 0.155.0   not logged in",
                "    ○ ⊘  Grok          not installed       Kit can still write its files",
                "  ↑↓ move · space select · enter confirm · esc cancel",
            ]
        );
    }

    #[test]
    fn logo_cells_are_left_blank_for_images() {
        let lines = agents().lines(100, &Paint::plain(), true);
        assert!(lines[1].starts_with("  ❯ ●    Claude Code"));
        let (head, _) = split_at_cells(&lines[1], LOGO_COL as usize);
        assert_eq!(head, "  ❯ ● ");
    }

    #[test]
    fn narrow_terminals_cut_with_an_ellipsis_never_wrap() {
        for width in [30, 44, 60] {
            for line in agents().lines(width, &Paint::plain(), false) {
                assert!(
                    visible_len(&line) < width,
                    "{width}: {line:?} is {} cells",
                    visible_len(&line)
                );
            }
        }
        let lines = agents().lines(60, &Paint::plain(), false);
        assert!(lines[3].ends_with("still w…"), "{:?}", lines[3]);
        let lines = agents().lines(44, &Paint::plain(), false);
        assert_eq!(lines[3], "    ○ ⊘  Grok          not installed");
    }

    #[test]
    fn painted_rows_have_the_same_width_as_plain_ones() {
        let plain = agents().lines(100, &Paint::plain(), false);
        let painted = agents().lines(100, &Paint::with(Theme::kit()), false);
        for (a, b) in plain.iter().zip(&painted) {
            assert_eq!(visible_len(a), visible_len(b));
        }
    }

    #[test]
    fn multi_select_needs_one_choice() {
        let mut p = agents();
        assert_eq!(p.on_key(Key::Toggle), Step::Continue);
        assert_eq!(p.on_key(Key::Confirm), Step::Continue);
        assert!(p.error.is_some());
        p.on_key(Key::Down);
        p.on_key(Key::Toggle);
        assert!(p.error.is_none());
        p.on_key(Key::Up);
        p.on_key(Key::Up);
        assert_eq!(p.cursor, 2, "wraps");
        p.on_key(Key::Toggle);
        assert_eq!(p.on_key(Key::Confirm), Step::Done(vec![1, 2]));
        assert_eq!(
            p.summary_line(&[1, 2], &Paint::plain()),
            "✓ Agents       Codex and Grok"
        );
        assert_eq!(p.on_key(Key::Cancel), Step::Cancelled);
    }

    #[test]
    fn single_select_takes_the_cursor_row() {
        let mut p = Picker::new(
            "Install for",
            "Install for",
            vec![
                Item::new("All my projects").detail("your agents use it everywhere"),
                Item::new("This repo only").detail("~/code/shop · commit it to share"),
            ],
            false,
        )
        .select(&[1]);
        let lines = p.lines(80, &Paint::plain(), false);
        assert_eq!(
            lines[1],
            "    All my projects   your agents use it everywhere"
        );
        assert_eq!(
            lines[2],
            "  ❯ This repo only    ~/code/shop · commit it to share"
        );
        assert_eq!(p.on_key(Key::Toggle), Step::Continue);
        assert_eq!(p.on_key(Key::Confirm), Step::Done(vec![1]));
    }

    #[test]
    fn skipping_logo_cells_keeps_the_styles_after_them() {
        let painted = agents().lines(100, &Paint::with(Theme::kit()), true);
        let (head, tail) = split_at_cells(&painted[1], LOGO_COL as usize);
        assert_eq!(visible_len(&head), LOGO_COL as usize);
        let rest = skip_cells(&tail, 2);
        assert_eq!(
            visible_len(&rest),
            visible_len(&painted[1]) - LOGO_COL as usize - 2
        );
    }
}
