//! Kits: bundles that set coding agents up for one job.
//! Design: `docs/dev/DESIGN-KITS.md`.

pub mod catalog;
pub mod manifest;
mod show;

pub use show::cmd_show;
