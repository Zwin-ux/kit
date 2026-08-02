//! M1 run engine — production-shaped path for one isolated run.
//!
//! Lives in kit-cli until it graduates to a dedicated crate. UI and headless
//! CLI share [`runner::execute`].

pub mod paths;
pub mod runner;
pub mod store;
pub mod worktree;

pub use runner::{RunOptions, execute, parse_agent};
