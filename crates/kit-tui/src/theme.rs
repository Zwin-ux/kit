//! Kit terminal theme — semantic tokens for the Control Room surface.
//!
//! Design source: `docs/dev/DESIGN-tui.md` + concept art under `docs/dev/assets/`.
//! Truecolor default; ANSI-16 when `COLORTERM` is not `truecolor`/`24bit`;
//! monochrome (`NO_COLOR`) must remain fully usable.

use kit_core::RunState;
use ratatui::style::{Color, Modifier, Style};

/// Semantic palette for one paint pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub muted: Color,
    pub accent: Color,
    pub success: Color,
    pub danger: Color,
    pub warn: Color,
    /// Background tint for FAIL rows (Reset in monochrome).
    pub fail_wash: Color,
    pub monochrome: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self::resolve()
    }
}

impl Theme {
    /// Default Kit palette (truecolor).
    pub fn kit() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::Rgb(0xf0, 0xf1, 0xe3),
            muted: Color::Rgb(0x6b, 0x72, 0x80),
            accent: Color::Rgb(0x00, 0xe6, 0xcc),
            success: Color::Rgb(0x39, 0xff, 0x9e),
            danger: Color::Rgb(0xff, 0x3b, 0x4e),
            warn: Color::Rgb(0xff, 0xba, 0x3d),
            fail_wash: Color::Rgb(0x2a, 0x12, 0x16),
            monochrome: false,
        }
    }

    /// Modifiers-only — usable when `NO_COLOR` is set.
    pub fn monochrome() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::Reset,
            muted: Color::Reset,
            accent: Color::Reset,
            success: Color::Reset,
            danger: Color::Reset,
            warn: Color::Reset,
            fail_wash: Color::Reset,
            monochrome: true,
        }
    }

    /// High-contrast a11y: pure ANSI brights on black.
    pub fn high_contrast() -> Self {
        Self {
            bg: Color::Black,
            fg: Color::White,
            muted: Color::Gray,
            accent: Color::Cyan,
            success: Color::LightGreen,
            danger: Color::LightRed,
            warn: Color::Yellow,
            fail_wash: Color::Reset,
            monochrome: false,
        }
    }

    /// Named ANSI-16 mapping of the Kit tokens (no RGB).
    pub fn ansi16() -> Self {
        Self {
            bg: Color::Reset,
            fg: Color::White,
            muted: Color::Gray,
            accent: Color::LightCyan,
            success: Color::LightGreen,
            danger: Color::LightRed,
            warn: Color::Yellow,
            fail_wash: Color::Black,
            monochrome: false,
        }
    }

    /// Pick theme from environment once per process (tests call variants directly).
    pub fn resolve() -> Self {
        if std::env::var_os("NO_COLOR").is_some() {
            return Self::monochrome();
        }
        match std::env::var("KIT_THEME").as_deref() {
            Ok("high") | Ok("high-contrast") | Ok("hc") => Self::high_contrast(),
            Ok("mono") | Ok("monochrome") => Self::monochrome(),
            _ if colorterm_is_truecolor() => Self::kit(),
            _ => Self::ansi16(),
        }
    }

    pub fn title(&self) -> Style {
        if self.monochrome {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(self.accent)
                .add_modifier(Modifier::BOLD)
        }
    }

    pub fn body(&self) -> Style {
        if self.monochrome {
            Style::default()
        } else {
            Style::default().fg(self.fg)
        }
    }

    pub fn dim(&self) -> Style {
        if self.monochrome {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            Style::default().fg(self.muted)
        }
    }

    pub fn footer(&self) -> Style {
        self.dim()
    }

    pub fn selected_row(&self) -> Style {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
    }

    pub fn border(&self, focused: bool) -> Style {
        if self.monochrome {
            if focused {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            }
        } else if focused {
            Style::default().fg(self.accent)
        } else {
            Style::default().fg(self.muted)
        }
    }

    /// STATE column color (always pair with text label).
    pub fn state_style(&self, state: RunState) -> Style {
        if self.monochrome {
            return match state {
                RunState::Fail | RunState::Error => Style::default().add_modifier(Modifier::BOLD),
                RunState::Running | RunState::Gating => {
                    Style::default().add_modifier(Modifier::BOLD)
                }
                _ => Style::default(),
            };
        }
        let fg = match state {
            RunState::Queued => self.warn,
            RunState::Running => self.accent,
            RunState::Gating => self.warn,
            RunState::Pass => self.success,
            RunState::Fail => self.danger,
            RunState::Killed => self.muted,
            RunState::Error => self.danger,
        };
        Style::default().fg(fg).add_modifier(Modifier::BOLD)
    }

    /// GATE column: PASS / FAIL / UNCONFIGURED / --
    pub fn gate_style(&self, label: &str) -> Style {
        if self.monochrome {
            return if label == "FAIL" || label == "UNCONFIGURED" {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
        }
        let fg = match label {
            "PASS" => self.success,
            "FAIL" => self.danger,
            "UNCONFIGURED" => self.warn,
            _ => self.muted,
        };
        Style::default().fg(fg).add_modifier(Modifier::BOLD)
    }

    pub fn fail_row(&self, selected: bool) -> Style {
        if self.monochrome {
            return if selected {
                Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            };
        }
        let style = Style::default().fg(self.fg).bg(self.fail_wash);
        if selected {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        }
    }

    pub fn annotation(&self) -> Style {
        if self.monochrome {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            Style::default().fg(self.danger).add_modifier(Modifier::DIM)
        }
    }

    pub fn success(&self) -> Style {
        if self.monochrome {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.success)
        }
    }

    pub fn danger(&self) -> Style {
        if self.monochrome {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.danger)
        }
    }

    pub fn warn(&self) -> Style {
        if self.monochrome {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.warn)
        }
    }

    pub fn accent(&self) -> Style {
        if self.monochrome {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.accent)
        }
    }
}

fn colorterm_is_truecolor() -> bool {
    std::env::var("COLORTERM").is_ok_and(|v| {
        let v = v.trim();
        v.eq_ignore_ascii_case("truecolor") || v.eq_ignore_ascii_case("24bit")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monochrome_has_no_rgb_wash() {
        let t = Theme::monochrome();
        assert!(t.monochrome);
        assert_eq!(t.fail_wash, Color::Reset);
    }

    #[test]
    fn monochrome_styles_use_modifiers_not_color() {
        let t = Theme::monochrome();
        let fail = t.state_style(RunState::Fail);
        assert!(fail.add_modifier.contains(Modifier::BOLD));
        let gate = t.gate_style("FAIL");
        assert!(gate.add_modifier.contains(Modifier::BOLD));
        let ann = t.annotation();
        assert!(ann.add_modifier.contains(Modifier::DIM));
        let selected = t.selected_row();
        assert!(selected.add_modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn kit_palette_has_accent() {
        let t = Theme::kit();
        assert!(!t.monochrome);
        assert_eq!(t.accent, Color::Rgb(0x00, 0xe6, 0xcc));
    }

    #[test]
    fn selected_fail_row_keeps_wash_not_reverse() {
        let t = Theme::kit();
        let selected = t.fail_row(true);
        assert_eq!(selected.bg, Some(t.fail_wash));
        assert!(selected.add_modifier.contains(Modifier::BOLD));
        assert!(!selected.add_modifier.contains(Modifier::REVERSED));
        assert_ne!(selected, t.selected_row());
    }

    #[test]
    fn high_contrast_uses_ansi_brights_not_kit_rgb() {
        let t = Theme::high_contrast();
        assert!(!t.monochrome);
        assert_eq!(t.bg, Color::Black);
        assert_eq!(t.fg, Color::White);
        assert_eq!(t.accent, Color::Cyan);
        assert_eq!(t.danger, Color::LightRed);
        assert_ne!(t.accent, Theme::kit().accent);
    }

    #[test]
    fn ansi16_maps_kit_tokens_to_named_ansi() {
        let t = Theme::ansi16();
        assert!(!t.monochrome);
        assert_eq!(t.bg, Color::Reset);
        assert_eq!(t.fg, Color::White);
        assert_eq!(t.muted, Color::Gray);
        assert_eq!(t.accent, Color::LightCyan);
        assert_eq!(t.success, Color::LightGreen);
        assert_eq!(t.danger, Color::LightRed);
        assert_eq!(t.warn, Color::Yellow);
        assert_eq!(t.fail_wash, Color::Black);
        for c in [
            t.bg, t.fg, t.muted, t.accent, t.success, t.danger, t.warn, t.fail_wash,
        ] {
            assert!(!matches!(c, Color::Rgb(_, _, _)), "{c:?}");
        }
        assert_ne!(t.accent, Theme::kit().accent);
        assert_ne!(t, Theme::high_contrast());
        assert_ne!(t, Theme::monochrome());
    }

    #[test]
    fn ansi16_fail_row_keeps_wash_not_reverse() {
        let t = Theme::ansi16();
        let selected = t.fail_row(true);
        assert_eq!(selected.bg, Some(t.fail_wash));
        assert!(selected.add_modifier.contains(Modifier::BOLD));
        assert!(!selected.add_modifier.contains(Modifier::REVERSED));
        assert_ne!(selected, t.selected_row());
    }

    #[test]
    fn state_styles_differ_in_color_mode() {
        let t = Theme::kit();
        assert_ne!(
            t.state_style(RunState::Running),
            t.state_style(RunState::Fail)
        );
        let a = Theme::ansi16();
        assert_ne!(
            a.state_style(RunState::Running),
            a.state_style(RunState::Fail)
        );
        assert_eq!(a.state_style(RunState::Fail).fg, Some(a.danger));
        let m = Theme::monochrome();
        assert_eq!(m.state_style(RunState::Fail).fg, None);
    }
}
