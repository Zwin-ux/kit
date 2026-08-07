//! M1 run engine — production-shaped path for one isolated run.
//!
//! Lives in kit-cli until it graduates to a dedicated crate. UI and headless
//! CLI share [`runner::execute`].

pub mod cancel;
pub mod infer;
pub mod paths;
pub mod registry;
pub mod runner;
pub mod store;
pub mod supervisor;
pub mod worktree;

#[allow(unused_imports)] // re-exports for tests / future CLI
pub use cancel::CancelHandle;
#[allow(unused_imports)]
pub use registry::{MAX_CONCURRENT_RUNS, RunRegistry, concurrency_limiter};
#[allow(unused_imports)]
pub use runner::{RunOptions, execute, execute_cancellable, parse_agent};
#[allow(unused_imports)]
pub use supervisor::{ConcurrencyProbe, proof_dispatch_n, run_supervisor, spawn_production};
