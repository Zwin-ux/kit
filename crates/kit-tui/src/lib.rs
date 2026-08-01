//! Kit Control Room — the terminal surface.
//!
//! Owned by Grok Build. Reference implementation for the architecture is
//! `C:/Users/mzwin/Documents/fennec/crates/fennec-tui`, which already runs
//! ratatui + crossterm with a single animation tick.
//!
//! See `docs/dev/BUILD-ASSIGNMENT.md` for the boundaries.
//!
//! ## Architecture
//!
//! - [`event`] — frozen contract: `AppEvent`, `Clock`, tick cadence
//! - [`app`] — pure reducer over `AppEvent` → `Action`
//! - [`event_loop`] — single `tokio::select!` merging terminal, tick, runs
//! - [`ui`] — Control Room placeholder frame

pub mod app;
pub mod event;
#[path = "loop.rs"]
pub mod event_loop;
pub mod ui;

pub use app::{Action, App, RunRow};
pub use event::{AppEvent, Clock, TICK_HZ, TICK_INTERVAL, motion_enabled};
pub use event_loop::run;
