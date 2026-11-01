//! The session view (`kit chat`, PRD §7.1): Kit's own agent in the terminal's
//! scrollback, with a small live region for what is still changing.
//!
//! - [`state`]: a pure reducer over agent events and keys
//! - [`view`]: scrollback lines and the live region, snapshot-tested
//! - [`term`]: the inline-viewport driver

pub mod state;
pub mod term;
pub mod view;

pub use state::{Answer, Approval, Cmd, Out, Session, Status};
pub use term::{Control, run};
