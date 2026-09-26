//! Kits: bundles that set coding agents up for one job.
//! Design: `docs/dev/DESIGN-KITS.md`.

pub mod catalog;
pub mod config;
pub mod doctor;
pub mod fetch;
pub mod hook;
pub mod index;
pub mod install;
pub mod lock;
pub mod manifest;
pub mod new;
pub mod plan;
pub mod remote;
pub mod search;
pub mod setup;
mod show;
pub mod sync;
pub mod writers;

pub use show::cmd_show;
