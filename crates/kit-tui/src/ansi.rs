//! Theme colours for line-by-line output (`kit setup`, `kit doctor`), so the
//! CLI and the Control Room speak one palette.
//!
//! Colour goes only to a terminal: piped output, `TERM=dumb` and tests get
//! plain text. `NO_COLOR` keeps bold and dim and drops hue, as the TUI does.

use crate::brand;
use crate::theme::Theme;
use kit_core::AgentKind;
use ratatui::style::{Color, Modifier, Style};
use std::io::IsTerminal;

/// Paints text with the theme, or passes it through.
#[derive(Debug, Clone, Copy)]
pub struct Paint {
    theme: Theme,
    on: bool,
}

impl Paint {
    /// For stdout: on when it is a terminal.
    pub fn stdout() -> Self {
        let on = std::io::stdout().is_terminal()
            && std::env::var("TERM").map_or(true, |t| t != "dumb")
            && vt_ready();
        Self {
            theme: Theme::resolve(),
            on,
        }
    }

    /// Never paints.
    pub fn plain() -> Self {
        Self {
            theme: Theme::monochrome(),
            on: false,
        }
    }

    /// Always paints with `theme` (tests, previews).
    pub fn with(theme: Theme) -> Self {
        Self { theme, on: true }
    }

    pub fn enabled(&self) -> bool {
        self.on
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// `text` wrapped in the SGR codes for `style`.
    pub fn paint(&self, text: &str, style: Style) -> String {
        if !self.on || text.is_empty() {
            return text.to_string();
        }
        let codes = sgr(style);
        if codes.is_empty() {
            text.to_string()
        } else {
            format!("\x1b[{codes}m{text}\x1b[0m")
        }
    }

    pub fn bold(&self, text: &str) -> String {
        self.paint(text, Style::default().add_modifier(Modifier::BOLD))
    }
    pub fn title(&self, text: &str) -> String {
        self.paint(text, self.theme.title())
    }
    pub fn accent(&self, text: &str) -> String {
        self.paint(text, self.theme.accent())
    }
    pub fn muted(&self, text: &str) -> String {
        self.paint(text, self.theme.dim())
    }
    pub fn success(&self, text: &str) -> String {
        self.paint(text, self.theme.success())
    }
    pub fn warn(&self, text: &str) -> String {
        self.paint(text, self.theme.warn())
    }
    pub fn danger(&self, text: &str) -> String {
        self.paint(text, self.theme.danger())
    }

    /// The agent's two-cell mark in its brand colour.
    pub fn mark(&self, kind: AgentKind) -> String {
        let b = brand::of(kind);
        self.paint(b.mark, b.style(&self.theme))
    }
}

/// Windows consoles draw escape codes only once virtual terminal
/// processing is on; this turns it on, and says no where it cannot be.
#[cfg(windows)]
pub fn vt_ready() -> bool {
    crossterm::ansi_support::supports_ansi()
}

/// Every other terminal Kit supports reads escape codes.
#[cfg(not(windows))]
pub fn vt_ready() -> bool {
    true
}

fn sgr(style: Style) -> String {
    let mut codes: Vec<String> = Vec::new();
    let m = style.add_modifier;
    if m.contains(Modifier::BOLD) {
        codes.push("1".into());
    }
    if m.contains(Modifier::DIM) {
        codes.push("2".into());
    }
    if m.contains(Modifier::ITALIC) {
        codes.push("3".into());
    }
    if m.contains(Modifier::UNDERLINED) {
        codes.push("4".into());
    }
    if m.contains(Modifier::REVERSED) {
        codes.push("7".into());
    }
    if let Some(fg) = style.fg.and_then(fg_code) {
        codes.push(fg);
    }
    codes.join(";")
}

fn fg_code(c: Color) -> Option<String> {
    Some(match c {
        Color::Reset => return None,
        Color::Black => "30".into(),
        Color::Red => "31".into(),
        Color::Green => "32".into(),
        Color::Yellow => "33".into(),
        Color::Blue => "34".into(),
        Color::Magenta => "35".into(),
        Color::Cyan => "36".into(),
        Color::Gray => "37".into(),
        Color::DarkGray => "90".into(),
        Color::LightRed => "91".into(),
        Color::LightGreen => "92".into(),
        Color::LightYellow => "93".into(),
        Color::LightBlue => "94".into(),
        Color::LightMagenta => "95".into(),
        Color::LightCyan => "96".into(),
        Color::White => "97".into(),
        Color::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
        Color::Indexed(i) => format!("38;5;{i}"),
    })
}

/// Display width of `s` with any SGR codes removed. Wide glyphs are not
/// measured; Kit's own strings do not use them.
pub fn visible_len(s: &str) -> usize {
    let mut n = 0;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            n += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_passes_text_through() {
        let p = Paint::plain();
        assert_eq!(p.accent("kit"), "kit");
        assert_eq!(p.mark(AgentKind::Claude), "✻ ");
    }

    #[test]
    fn truecolor_uses_rgb_and_brand_colours() {
        let p = Paint::with(Theme::kit());
        assert_eq!(p.accent("kit"), "\x1b[38;2;0;230;204mkit\x1b[0m");
        assert_eq!(p.title("kit"), "\x1b[1;38;2;0;230;204mkit\x1b[0m");
        assert_eq!(
            p.mark(AgentKind::Claude),
            "\x1b[1;38;2;217;119;87m✻ \x1b[0m"
        );
        assert_eq!(visible_len(&p.mark(AgentKind::Codex)), 2);
    }

    #[test]
    fn monochrome_keeps_weight_without_hue() {
        let p = Paint::with(Theme::monochrome());
        assert_eq!(p.mark(AgentKind::Codex), "\x1b[1m>_\x1b[0m");
        assert_eq!(p.muted("x"), "\x1b[2mx\x1b[0m");
    }
}
