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

    /// The agent's three-cell text logo: its mark on the brand's own
    /// background. Monochrome keeps the glyph, bracketed by spaces.
    pub fn tile(&self, kind: AgentKind) -> String {
        let t = brand::of(kind).tile;
        if !self.on || self.theme.monochrome {
            return self.paint(t.text, Style::default().add_modifier(Modifier::BOLD));
        }
        let truecolor = matches!(self.theme.fg, Color::Rgb(..));
        let mut out = String::new();
        for (i, ch) in t.text.chars().enumerate() {
            let style = if truecolor {
                let (fr, fg, fb) = t.fg;
                let (br, bg, bb) = t.bg[i.min(2)];
                Style::default()
                    .fg(Color::Rgb(fr, fg, fb))
                    .bg(Color::Rgb(br, bg, bb))
            } else {
                Style::default().fg(t.ansi.0).bg(t.ansi.1)
            }
            .add_modifier(Modifier::BOLD);
            out.push_str(&format!("\x1b[{}m{ch}", sgr(style)));
        }
        out.push_str("\x1b[0m");
        out
    }

    /// The fox head in the theme's colours: fox red from the accent, cream
    /// from the text colour. Six lines, sixteen cells wide.
    pub fn fox_head(&self) -> Vec<String> {
        let t = self.theme;
        let ink = if matches!(t.fg, Color::Rgb(..)) {
            Color::Rgb(0x1c, 0x14, 0x12)
        } else {
            Color::Black
        };
        pixels(self, &crate::fox::HEAD, |c| match c {
            'O' => Some(t.accent),
            'F' => Some(t.fg),
            'K' => Some(ink),
            _ => None,
        })
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
    if let Some(bg) = style.bg.and_then(fg_code) {
        codes.push(bg_code(&bg));
    }
    codes.join(";")
}

/// The background code for a foreground code (`31` → `41`, `38;…` → `48;…`).
fn bg_code(fg: &str) -> String {
    if let Some(rest) = fg.strip_prefix("38;") {
        return format!("48;{rest}");
    }
    match fg.parse::<u8>() {
        Ok(n @ 30..=37) => (n + 10).to_string(),
        Ok(n @ 90..=97) => (n + 10).to_string(),
        _ => String::new(),
    }
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

/// Pixel art drawn two pixels per cell with half blocks, each pixel in its
/// own colour. `rows` are strings of palette keys, `.` transparent;
/// `palette` maps a key to its colour for this theme. Without colour
/// (plain or `NO_COLOR`) the art is one ink, drawn with the same blocks.
pub fn pixels(
    paint: &Paint,
    rows: &[&str],
    palette: impl Fn(char) -> Option<Color>,
) -> Vec<String> {
    let colour = paint.on && !paint.theme.monochrome;
    rows.chunks(2)
        .map(|pair| {
            let top: Vec<char> = pair[0].chars().collect();
            let bottom: Vec<char> = pair.get(1).map_or_else(Vec::new, |r| r.chars().collect());
            let mut line = String::new();
            for (x, &t) in top.iter().enumerate() {
                let b = bottom.get(x).copied().unwrap_or('.');
                let (t, b) = ((t != '.').then_some(t), (b != '.').then_some(b));
                if !colour {
                    line.push(match (t, b) {
                        (Some(_), Some(_)) => '█',
                        (Some(_), None) => '▀',
                        (None, Some(_)) => '▄',
                        (None, None) => ' ',
                    });
                    continue;
                }
                let (tc, bc) = (t.and_then(&palette), b.and_then(&palette));
                let cell = match (tc, bc) {
                    (Some(tc), Some(bc)) if tc == bc => ('█', Style::default().fg(tc)),
                    (Some(tc), Some(bc)) => ('▀', Style::default().fg(tc).bg(bc)),
                    (Some(tc), None) => ('▀', Style::default().fg(tc)),
                    (None, Some(bc)) => ('▄', Style::default().fg(bc)),
                    (None, None) => (' ', Style::default()),
                };
                if cell.0 == ' ' {
                    line.push(' ');
                } else {
                    line.push_str(&format!("\x1b[{}m{}\x1b[0m", sgr(cell.1), cell.0));
                }
            }
            line.trim_end().to_string()
        })
        .collect()
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
        assert_eq!(p.accent("kit"), "\x1b[38;2;255;90;31mkit\x1b[0m");
        assert_eq!(p.title("kit"), "\x1b[1;38;2;255;90;31mkit\x1b[0m");
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

    #[test]
    fn fox_head_is_six_rows_sixteen_cells() {
        for p in [
            Paint::plain(),
            Paint::with(Theme::kit()),
            Paint::with(Theme::ansi16()),
            Paint::with(Theme::monochrome()),
        ] {
            let head = p.fox_head();
            assert_eq!(head.len(), 6);
            assert!(head.iter().all(|l| visible_len(l) <= 16));
            assert_eq!(visible_len(&head[2]), 16, "{:?}", head[2]);
        }
        // Without colour it is one ink: no escape codes at all.
        assert!(
            Paint::plain()
                .fox_head()
                .iter()
                .all(|l| !l.contains('\x1b'))
        );
        // Two colours in one cell use the upper half block on a background.
        assert!(Paint::with(Theme::kit()).fox_head()[3].contains("48;2;"));
    }

    #[test]
    fn tiles_are_three_cells_in_every_theme() {
        for p in [
            Paint::plain(),
            Paint::with(Theme::kit()),
            Paint::with(Theme::ansi16()),
            Paint::with(Theme::monochrome()),
        ] {
            for k in [AgentKind::Claude, AgentKind::Codex, AgentKind::Grok] {
                assert_eq!(visible_len(&p.tile(k)), 3);
            }
        }
        assert_eq!(
            Paint::with(Theme::ansi16()).tile(AgentKind::Claude),
            "\x1b[1;97;41m \x1b[1;97;41m✻\x1b[1;97;41m \x1b[0m"
        );
    }
}
