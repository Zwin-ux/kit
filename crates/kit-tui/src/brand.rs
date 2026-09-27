//! Agent brands: the name, mark and colour each coding agent is shown
//! with, and its logo for terminals that can draw images.
//!
//! Every surface that lists agents (setup's picker, doctor, Dispatch, the
//! Control Room) takes them from here, so an agent looks the same
//! everywhere. The mark is two cells of text in the brand colour; the logo
//! is a 2×1-cell image drawn in its place where the terminal supports the
//! kitty or iTerm2 image protocol. Under `NO_COLOR` the mark keeps its
//! glyph and loses its colour.
//!
//! Logo art: `assets/logos/`, rendered from lobehub/lobe-icons (MIT). The
//! marks and logos belong to their owners and name the agent Kit sets up.

use crate::theme::Theme;
use kit_core::AgentKind;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

/// How one agent is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Brand {
    /// The product name, for prose and prompts (`Claude Code`).
    pub name: &'static str,
    /// Two cells of text that stand for the logo.
    pub mark: &'static str,
    /// The brand colour in truecolor terminals.
    pub rgb: (u8, u8, u8),
    /// The nearest of the 16 named colours.
    pub ansi: Color,
    /// The text logo for line output: three cells on the brand's own
    /// background, like a small app icon. Any terminal draws it.
    pub tile: Tile,
    logo: Option<&'static [u8]>,
}

/// An RGB colour.
pub type Rgb = (u8, u8, u8);

/// A three-cell text logo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    /// Exactly three cells.
    pub text: &'static str,
    pub fg: (u8, u8, u8),
    /// One background per cell, so a tile can carry a gradient.
    pub bg: [(u8, u8, u8); 3],
    /// Foreground and background in the 16 named colours.
    pub ansi: (Color, Color),
    /// Foreground and background on a light terminal, for a tile whose
    /// own background would vanish there.
    pub on_light: Option<(Rgb, Rgb)>,
}

const CLAUDE: Brand = Brand {
    name: "Claude Code",
    mark: "✻ ",
    rgb: (0xd9, 0x77, 0x57),
    ansi: Color::Red,
    tile: Tile {
        text: " ✻ ",
        fg: (0xff, 0xf6, 0xf0),
        bg: [(0xd9, 0x77, 0x57); 3],
        ansi: (Color::White, Color::Red),
        on_light: None,
    },
    logo: Some(include_bytes!("../assets/logos/claude.png")),
};
const CODEX: Brand = Brand {
    name: "Codex",
    mark: ">_",
    rgb: (0x7a, 0x9d, 0xff),
    ansi: Color::LightBlue,
    tile: Tile {
        text: " >_",
        fg: (0xff, 0xff, 0xff),
        bg: [(0xb1, 0xa7, 0xff), (0x7a, 0x9d, 0xff), (0x39, 0x41, 0xff)],
        ansi: (Color::White, Color::Blue),
        on_light: None,
    },
    logo: Some(include_bytes!("../assets/logos/codex.png")),
};
const GROK: Brand = Brand {
    name: "Grok",
    mark: "⊘ ",
    rgb: (0xf0, 0xf1, 0xe3),
    ansi: Color::White,
    tile: Tile {
        text: " ⊘ ",
        fg: (0x0a, 0x0a, 0x0a),
        bg: [(0xf0, 0xf1, 0xe3); 3],
        ansi: (Color::Black, Color::White),
        on_light: Some(((0xf0, 0xf1, 0xe3), (0x0a, 0x0a, 0x0a))),
    },
    logo: Some(include_bytes!("../assets/logos/grok.png")),
};
const OLLAMA: Brand = Brand {
    name: "Ollama",
    mark: "◉ ",
    rgb: (0xc8, 0xc8, 0xc8),
    ansi: Color::Gray,
    tile: Tile {
        text: " ◉ ",
        fg: (0x0a, 0x0a, 0x0a),
        bg: [(0xc8, 0xc8, 0xc8); 3],
        ansi: (Color::Black, Color::Gray),
        on_light: Some(((0xff, 0xff, 0xff), (0x55, 0x55, 0x55))),
    },
    logo: None,
};

/// The brand for an agent.
pub fn of(kind: AgentKind) -> &'static Brand {
    match kind {
        AgentKind::Claude => &CLAUDE,
        AgentKind::Codex => &CODEX,
        AgentKind::Grok => &GROK,
        AgentKind::Ollama => &OLLAMA,
    }
}

/// The brand for an agent id (`claude`), if Kit knows it.
pub fn by_id(id: &str) -> Option<&'static Brand> {
    [
        AgentKind::Claude,
        AgentKind::Codex,
        AgentKind::Grok,
        AgentKind::Ollama,
    ]
    .into_iter()
    .find(|k| k.label() == id)
    .map(of)
}

impl Brand {
    /// The brand colour for this theme: RGB, a named colour, or none.
    pub fn color(&self, theme: &Theme) -> Option<Color> {
        if theme.monochrome {
            None
        } else if matches!(theme.fg, Color::Rgb(..)) {
            let (r, g, b) = self.rgb;
            Some(Color::Rgb(r, g, b))
        } else {
            Some(self.ansi)
        }
    }

    /// The mark's style: bold, in the brand colour when there is colour.
    pub fn style(&self, theme: &Theme) -> Style {
        let bold = Style::default().add_modifier(Modifier::BOLD);
        match self.color(theme) {
            Some(c) => bold.fg(c),
            None => bold,
        }
    }

    /// The mark as a styled span, two cells wide.
    pub fn mark_span(&self, theme: &Theme) -> Span<'static> {
        Span::styled(self.mark, self.style(theme))
    }

    /// The escape sequence that draws this agent's logo over the two cells
    /// at the cursor, leaving the cursor where it was. `None` when the
    /// agent has no logo.
    pub fn logo(&self, graphics: Graphics, id: u32) -> Option<String> {
        let png = self.logo?;
        let data = base64(png);
        Some(match graphics {
            // a=T transmit and show, f=100 PNG, c/r the cell box, C=1 keep
            // the cursor, q=2 no replies (they would land on stdin).
            Graphics::Kitty => {
                let mut out = String::new();
                let chunks: Vec<&str> = chunk(&data, 4096);
                for (n, part) in chunks.iter().enumerate() {
                    let more = u8::from(n + 1 < chunks.len());
                    if n == 0 {
                        out.push_str(&format!(
                            "\x1b_Ga=T,f=100,i={id},c=2,r=1,C=1,q=2,m={more};{part}\x1b\\"
                        ));
                    } else {
                        out.push_str(&format!("\x1b_Gm={more},q=2;{part}\x1b\\"));
                    }
                }
                out
            }
            // Saved and restored around the image, because older iTerm2
            // builds move the cursor after one.
            Graphics::Iterm => format!(
                "\x1b7\x1b]1337;File=inline=1;size={};width=2;height=1;preserveAspectRatio=1;doNotMoveCursor=1:{data}\x07\x1b8",
                png.len()
            ),
        })
    }
}

/// The sequence that removes a logo drawn with [`Brand::logo`]. Kitty keeps
/// images until told; iTerm2's go with the text cells they sit in.
pub fn clear_logo(graphics: Graphics, id: u32) -> String {
    match graphics {
        Graphics::Kitty => format!("\x1b_Ga=d,d=I,i={id},q=2\x1b\\"),
        Graphics::Iterm => String::new(),
    }
}

/// An image protocol the terminal speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Graphics {
    Kitty,
    Iterm,
}

/// The image protocol for this terminal, from the environment. `None`
/// (text marks) unless the terminal is known to draw images, and always
/// inside tmux or screen, which do not pass images through by default.
/// `KIT_LOGOS=off` turns images off; `KIT_LOGOS=kitty|iterm` forces one.
pub fn graphics() -> Option<Graphics> {
    graphics_from(|k| std::env::var(k).ok())
}

/// [`graphics`] over any environment, for tests.
pub fn graphics_from(env: impl Fn(&str) -> Option<String>) -> Option<Graphics> {
    match env("KIT_LOGOS").as_deref() {
        Some("off" | "0" | "false" | "no") => return None,
        Some("kitty") => return Some(Graphics::Kitty),
        Some("iterm" | "iterm2") => return Some(Graphics::Iterm),
        _ => {}
    }
    if env("NO_COLOR").is_some() || env("TMUX").is_some() || env("STY").is_some() {
        return None;
    }
    if env("TERM").as_deref() == Some("dumb") {
        return None;
    }
    let program = env("TERM_PROGRAM").unwrap_or_default();
    // WezTerm speaks kitty's protocol only when a setting turns it on, and
    // iTerm2's always.
    match program.as_str() {
        "iTerm.app" | "WezTerm" => return Some(Graphics::Iterm),
        "ghostty" => return Some(Graphics::Kitty),
        _ => {}
    }
    if env("KITTY_WINDOW_ID").is_some() || env("TERM").as_deref() == Some("xterm-kitty") {
        return Some(Graphics::Kitty);
    }
    if env("TERM").as_deref() == Some("xterm-ghostty") {
        return Some(Graphics::Kitty);
    }
    None
}

fn chunk(s: &str, size: usize) -> Vec<&str> {
    // Base64 is ASCII, so byte offsets are char boundaries.
    (0..s.len())
        .step_by(size)
        .map(|i| &s[i..(i + size).min(s.len())])
        .collect()
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for group in bytes.chunks(3) {
        let n = group
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, b)| acc | u32::from(*b) << (16 - 8 * i));
        for i in 0..4 {
            if i <= group.len() {
                out.push(TABLE[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |k| pairs.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone())
    }

    #[test]
    fn base64_matches_the_standard_alphabet() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64(&[0xff, 0xfe]), "//4=");
    }

    #[test]
    fn marks_are_two_cells() {
        for kind in [
            AgentKind::Claude,
            AgentKind::Codex,
            AgentKind::Grok,
            AgentKind::Ollama,
        ] {
            assert_eq!(of(kind).mark.chars().count(), 2, "{kind:?}");
            assert_eq!(of(kind).tile.text.chars().count(), 3, "{kind:?}");
        }
        assert_eq!(by_id("codex").map(|b| b.name), Some("Codex"));
        assert_eq!(by_id("nope"), None);
    }

    #[test]
    fn colour_follows_the_theme() {
        let b = of(AgentKind::Claude);
        assert_eq!(b.color(&Theme::kit()), Some(Color::Rgb(0xd9, 0x77, 0x57)));
        assert_eq!(b.color(&Theme::ansi16()), Some(Color::Red));
        assert_eq!(b.color(&Theme::monochrome()), None);
    }

    #[test]
    fn images_only_where_the_terminal_draws_them() {
        assert_eq!(graphics_from(env(&[])), None);
        assert_eq!(
            graphics_from(env(&[("TERM_PROGRAM", "Apple_Terminal")])),
            None
        );
        assert_eq!(
            graphics_from(env(&[("TERM_PROGRAM", "iTerm.app")])),
            Some(Graphics::Iterm)
        );
        assert_eq!(
            graphics_from(env(&[("TERM_PROGRAM", "WezTerm")])),
            Some(Graphics::Iterm)
        );
        assert_eq!(
            graphics_from(env(&[("TERM_PROGRAM", "ghostty")])),
            Some(Graphics::Kitty)
        );
        assert_eq!(
            graphics_from(env(&[("TERM", "xterm-kitty")])),
            Some(Graphics::Kitty)
        );
        // tmux, NO_COLOR and the opt-out win over the terminal.
        assert_eq!(
            graphics_from(env(&[("TERM", "xterm-kitty"), ("TMUX", "/tmp/t")])),
            None
        );
        assert_eq!(
            graphics_from(env(&[("TERM_PROGRAM", "iTerm.app"), ("NO_COLOR", "1")])),
            None
        );
        assert_eq!(
            graphics_from(env(&[("TERM_PROGRAM", "ghostty"), ("KIT_LOGOS", "off")])),
            None
        );
        assert_eq!(
            graphics_from(env(&[("KIT_LOGOS", "kitty")])),
            Some(Graphics::Kitty)
        );
    }

    #[test]
    fn kitty_logo_keeps_the_cursor_and_asks_for_no_reply() {
        let s = of(AgentKind::Claude).logo(Graphics::Kitty, 7).unwrap();
        assert!(s.starts_with("\x1b_Ga=T,f=100,i=7,c=2,r=1,C=1,q=2,"));
        assert!(s.ends_with("\x1b\\"));
        assert_eq!(
            clear_logo(Graphics::Kitty, 7),
            "\x1b_Ga=d,d=I,i=7,q=2\x1b\\"
        );
        let s = of(AgentKind::Codex).logo(Graphics::Iterm, 1).unwrap();
        assert!(s.starts_with("\x1b7\x1b]1337;File=inline=1;"));
        assert!(s.ends_with("\x07\x1b8"));
        assert!(of(AgentKind::Ollama).logo(Graphics::Kitty, 1).is_none());
    }
}
