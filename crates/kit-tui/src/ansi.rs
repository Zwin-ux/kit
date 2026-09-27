//! Theme colours for line-by-line output (`kit setup`, `kit doctor`), so the
//! CLI and the Control Room speak one palette.
//!
//! Colour goes only to a terminal: piped output, `TERM=dumb` and tests get
//! plain text, and agent marks and tiles are left out there entirely, so
//! scripts and logs read exactly what they did before marks existed.
//! `NO_COLOR` keeps bold and dim and drops hue, as the TUI does.

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
    /// East Asian ambiguous-width glyphs take two cells here.
    wide: bool,
    /// The terminal's background is light.
    light: bool,
}

impl Paint {
    /// For stdout: on when it is a terminal.
    pub fn stdout() -> Self {
        let on = std::io::stdout().is_terminal()
            && std::env::var("TERM").map_or(true, |t| t != "dumb")
            && vt_ready();
        let light = on && light_background();
        let theme = match Theme::resolve() {
            t if light && t == Theme::kit() => Theme::kit_light(),
            t => t,
        };
        Self {
            theme,
            on,
            wide: ambiguous_is_wide(|k| std::env::var(k).ok()),
            light,
        }
    }

    /// Never paints.
    pub fn plain() -> Self {
        Self {
            theme: Theme::monochrome(),
            on: false,
            wide: false,
            light: false,
        }
    }

    /// Always paints with `theme` (tests, previews).
    pub fn with(theme: Theme) -> Self {
        Self {
            theme,
            on: true,
            wide: false,
            light: false,
        }
    }

    /// As if on a light background (tests, previews).
    pub fn light(mut self, light: bool) -> Self {
        self.light = light;
        self
    }

    /// As if ambiguous-width glyphs took two cells (a CJK locale).
    pub fn wide(mut self, wide: bool) -> Self {
        self.wide = wide;
        self
    }

    /// True where `●`, `○`, `⊘` and `◉` take two cells.
    pub fn is_wide(&self) -> bool {
        self.wide
    }

    /// `s` laid out for this terminal: where ambiguous-width glyphs take two
    /// cells, the space after each is dropped, so columns still line up.
    pub fn fit(&self, s: &str) -> String {
        if !self.wide {
            return s.to_string();
        }
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            out.push(c);
            if AMBIGUOUS.contains(&c) && chars.peek() == Some(&' ') {
                chars.next();
            }
        }
        out
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
    /// Empty when not painting, so piped output has no marks.
    pub fn tile(&self, kind: AgentKind) -> String {
        let t = brand::of(kind).tile;
        if !self.on {
            return String::new();
        }
        if self.theme.monochrome {
            return self.paint(
                &self.fit(t.text),
                Style::default().add_modifier(Modifier::BOLD),
            );
        }
        let truecolor = matches!(self.theme.fg, Color::Rgb(..));
        let mut out = String::new();
        for (i, ch) in self.fit(t.text).chars().enumerate() {
            let style = if truecolor {
                let ((fr, fg, fb), (br, bg, bb)) = match t.on_light {
                    Some(pair) if self.light => pair,
                    _ => (t.fg, t.bg[i.min(2)]),
                };
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
        // Cream reads on dark; on a light background the muzzle is peach,
        // or it would vanish into the page.
        let cream = match (self.light, t.fg) {
            (true, Color::Rgb(..)) => Color::Rgb(0xf6, 0xc0, 0x9a),
            (true, _) => Color::Yellow,
            (false, fg) => fg,
        };
        pixels(self, &crate::fox::HEAD, |c| match c {
            'O' => Some(t.accent),
            'F' => Some(cream),
            'K' => Some(ink),
            _ => None,
        })
    }

    /// The agent's two-cell mark in its brand colour.
    /// Empty when not painting, so piped output has no marks.
    pub fn mark(&self, kind: AgentKind) -> String {
        if !self.on {
            return String::new();
        }
        let b = brand::of(kind);
        self.paint(&self.fit(b.mark), b.style(&self.theme))
    }
}

/// True when the terminal's background is light, asked once per process.
/// `KIT_BACKGROUND=light|dark` decides; otherwise `COLORFGBG`, then the
/// terminal itself (OSC 11); dark when nothing answers.
pub fn light_background() -> bool {
    static LIGHT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LIGHT.get_or_init(|| {
        match std::env::var("KIT_BACKGROUND").as_deref() {
            Ok("light") => return true,
            Ok("dark") => return false,
            _ => {}
        }
        if let Some(light) = std::env::var("COLORFGBG")
            .ok()
            .as_deref()
            .and_then(colorfgbg_light)
        {
            return light;
        }
        query_background().is_some_and(|rgb| luminance(rgb) > 0.5)
    })
}

/// `COLORFGBG` is `fg;bg` (sometimes `fg;x;bg`) in the 16 colours: 7 and
/// 15 are light backgrounds.
fn colorfgbg_light(v: &str) -> Option<bool> {
    let bg: u8 = v.rsplit(';').next()?.trim().parse().ok()?;
    Some(matches!(bg, 7 | 15))
}

fn luminance((r, g, b): (f32, f32, f32)) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// `rgb:RRRR/GGGG/BBBB` (1 to 4 hex digits each) from an OSC 11 reply.
#[cfg_attr(not(unix), allow(dead_code))]
fn parse_osc11(reply: &str) -> Option<(f32, f32, f32)> {
    let rest = &reply[reply.find("rgb:")? + 4..];
    let mut parts = rest.split('/').map(|p| {
        let hex: String = p.chars().take_while(char::is_ascii_hexdigit).collect();
        let max = 16f32.powi(hex.len() as i32) - 1.0;
        u32::from_str_radix(&hex, 16).ok().map(|v| v as f32 / max)
    });
    Some((parts.next()??, parts.next()??, parts.next()??))
}

/// Ask the terminal for its background colour. The OSC 11 query is
/// followed by a device-attributes query every terminal answers, so a
/// terminal that ignores OSC 11 costs one round trip, not a timeout.
#[cfg(unix)]
fn query_background() -> Option<(f32, f32, f32)> {
    use std::io::{IsTerminal, Read, Write};
    use std::os::fd::AsRawFd;
    let stdin = std::io::stdin();
    if !stdin.is_terminal() || !std::io::stdout().is_terminal() {
        return None;
    }
    let was_raw = crossterm::terminal::is_raw_mode_enabled().unwrap_or(false);
    if !was_raw {
        crossterm::terminal::enable_raw_mode().ok()?;
    }
    let mut out = std::io::stdout();
    let _ = out.write_all(b"\x1b]11;?\x1b\\\x1b[c");
    let _ = out.flush();
    let fd = stdin.as_raw_fd();
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(300);
    let mut reply = Vec::new();
    let mut lock = stdin.lock();
    // Read until the device-attributes answer (`ESC [ ? … c`) ends it.
    loop {
        let left = deadline.saturating_duration_since(std::time::Instant::now());
        if left.is_zero() {
            break;
        }
        let mut pfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one valid pollfd for the duration of the call.
        let ready = unsafe { libc::poll(&mut pfd, 1, left.as_millis() as libc::c_int) };
        if ready <= 0 {
            break;
        }
        let mut byte = [0u8; 64];
        let Ok(n) = lock.read(&mut byte) else { break };
        if n == 0 {
            break;
        }
        reply.extend_from_slice(&byte[..n]);
        let text = String::from_utf8_lossy(&reply);
        if let Some(i) = text.find("\x1b[?")
            && text[i..].contains('c')
        {
            break;
        }
    }
    if !was_raw {
        let _ = crossterm::terminal::disable_raw_mode();
    }
    parse_osc11(&String::from_utf8_lossy(&reply))
}

#[cfg(not(unix))]
fn query_background() -> Option<(f32, f32, f32)> {
    None
}

/// Glyphs Kit draws that are East Asian ambiguous width: one cell in most
/// terminals, two in CJK locales. (`❯` and `✻` are neutral, always one.)
const AMBIGUOUS: [char; 4] = ['●', '○', '⊘', '◉'];

/// A CJK locale, where terminals draw ambiguous-width glyphs two cells
/// wide. `KIT_WIDE=1|0` overrides.
pub fn ambiguous_is_wide(env: impl Fn(&str) -> Option<String>) -> bool {
    match env("KIT_WIDE").as_deref() {
        Some("1" | "on" | "true") => return true,
        Some("0" | "off" | "false") => return false,
        _ => {}
    }
    let locale = ["LC_ALL", "LC_CTYPE", "LANG"]
        .into_iter()
        .find_map(|k| env(k).filter(|v| !v.is_empty()))
        .unwrap_or_default();
    ["ja", "zh", "ko"].iter().any(|l| locale.starts_with(l))
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
    fn plain_passes_text_through_and_drops_marks() {
        let p = Paint::plain();
        assert_eq!(p.accent("kit"), "kit");
        assert_eq!(p.mark(AgentKind::Claude), "");
        assert_eq!(p.tile(AgentKind::Claude), "");
    }

    #[test]
    fn background_answers_parse() {
        let dark = parse_osc11("\x1b]11;rgb:0b0b/0e0e/1212\x1b\\").unwrap();
        assert!(luminance(dark) < 0.1);
        let light = parse_osc11("\x1b]11;rgb:ffff/ffff/ffff\x07\x1b[?62;22c").unwrap();
        assert!(luminance(light) > 0.9);
        assert!(parse_osc11("rgb:f/f/f").is_some_and(|c| luminance(c) > 0.9));
        assert_eq!(parse_osc11("\x1b[?62;22c"), None);
        assert_eq!(colorfgbg_light("0;15"), Some(true));
        assert_eq!(colorfgbg_light("15;default;0"), Some(false));
        assert_eq!(colorfgbg_light("x"), None);
    }

    #[test]
    fn light_backgrounds_keep_the_fox_and_grok_visible() {
        let dark = Paint::with(Theme::kit());
        let light = Paint::with(Theme::kit_light()).light(true);
        // The muzzle is cream on dark, peach on light.
        assert!(dark.fox_head()[4].contains("240;241;227"));
        assert!(light.fox_head()[4].contains("246;192;154"));
        // Grok's cream tile turns black on light.
        assert!(dark.tile(AgentKind::Grok).contains("48;2;240;241;227"));
        assert!(light.tile(AgentKind::Grok).contains("48;2;10;10;10"));
    }

    #[test]
    fn wide_locales_drop_the_space_after_ambiguous_glyphs() {
        let p = Paint::with(Theme::monochrome()).wide(true);
        assert_eq!(p.fit("● x ⊘ ✻ "), "●x ⊘✻ ");
        assert_eq!(p.mark(AgentKind::Grok), "\x1b[1m⊘\x1b[0m");
        assert_eq!(p.mark(AgentKind::Claude), "\x1b[1m✻ \x1b[0m");
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                pairs
                    .iter()
                    .find(|(n, _)| *n == k)
                    .map(|(_, v)| v.to_string())
            }
        };
        assert!(ambiguous_is_wide(env(&[("LANG", "ja_JP.UTF-8")])));
        assert!(ambiguous_is_wide(env(&[
            ("LC_ALL", "zh_CN.UTF-8"),
            ("LANG", "en_US")
        ])));
        assert!(!ambiguous_is_wide(env(&[("LANG", "en_US.UTF-8")])));
        assert!(!ambiguous_is_wide(env(&[
            ("LANG", "ko_KR"),
            ("KIT_WIDE", "0")
        ])));
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
