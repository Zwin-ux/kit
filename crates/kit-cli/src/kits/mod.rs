//! Kits: bundles that set coding agents up for one job.
//! Design: `docs/dev/DESIGN-KITS.md`.

pub mod catalog;
pub mod config;
pub mod doctor;
pub mod fetch;
mod fmt;
pub mod hook;
pub mod install;
pub mod lock;
pub mod manifest;
pub mod plan;
pub mod setup;
mod show;
pub mod writers;

pub use show::cmd_show;
