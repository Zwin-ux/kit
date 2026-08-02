//! Minimal theme helpers for the Control Room surface (F5 light).
//!
//! Full mascot / motion port is a later slice. This module centralizes styles so
//! screens do not invent ad-hoc colors. Reduced motion remains in
//! [`crate::event::motion_enabled`] (`NO_COLOR`, `KIT_MOTION=off`).

use ratatui::style::{Modifier, Style};

/// Kit terminal theme — high contrast, monochrome-friendly.
#[derive(Debug, Clone, Copy, Default)]
pub struct Theme {
    pub bold: Style,
    pub dim: Style,
    pub reverse: Style,
    pub reverse_dim: Style,
}

impl Theme {
    pub fn default_theme() -> Self {
        Self {
            bold: Style::default().add_modifier(Modifier::BOLD),
            dim: Style::default().add_modifier(Modifier::DIM),
            reverse: Style::default().add_modifier(Modifier::REVERSED),
            reverse_dim: Style::default().add_modifier(Modifier::REVERSED | Modifier::DIM),
        }
    }

    /// Higher contrast: same modifiers (terminals already inverted); reserved
    /// for future color pairs without changing layout.
    pub fn high_contrast() -> Self {
        Self::default_theme()
    }
}
