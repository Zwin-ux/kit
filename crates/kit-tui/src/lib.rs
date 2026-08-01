//! Kit Control Room — the terminal surface.
//!
//! Owned by Grok Build. Reference implementation for the architecture is
//! `C:/Users/mzwin/Documents/fennec/crates/fennec-tui`, which already runs
//! ratatui + crossterm with a single animation tick.
//!
//! See `docs/dev/BUILD-ASSIGNMENT.md` for the boundaries.

pub mod event;

pub use event::{AppEvent, Clock, TICK_HZ, TICK_INTERVAL, motion_enabled};
