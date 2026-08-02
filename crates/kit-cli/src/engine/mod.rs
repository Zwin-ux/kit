//! M1 run engine — production-shaped path for one isolated run.
//!
//! Lives in kit-cli until it graduates to a dedicated crate. UI and headless
//! CLI share [`runner::execute`].

pub mod cancel;
pub mod paths;
pub mod registry;
pub mod runner;
pub mod store;
pub mod worktree;

pub use cancel::CancelHandle;
pub use registry::{RunRegistry, concurrency_limiter};
pub use runner::{RunOptions, execute, execute_cancellable, parse_agent};
