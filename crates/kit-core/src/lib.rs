//! Kit core — the shared contracts every other crate builds against.
//!
//! This crate is types and seams, not behaviour. `kit-gate` and `kit-agents`
//! provide implementations; `kit-tui` and `kit-cli` consume them.
//!
//! The modules re-exported here are CONTRACT FILES. See
//! `docs/dev/BUILD-ASSIGNMENT.md` for who may change them.

pub mod config;
pub mod gate;
pub mod run;

pub use config::{FirewallConfig, FirewallMode, GateConfig, KitConfig, ScopeConfig};
pub use gate::{CheckStatus, FirewallVerdict, Gate, GateCheck, GateOutcome};
pub use run::{AgentKind, Bounds, Receipt, Run, RunDelta, RunId, RunSpec, RunState};
