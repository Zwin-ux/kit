//! Kit's own coding agent (docs/PRD.md §5): the workspace it works in, its
//! tools, and the permission check every tool call passes first.

#![warn(missing_docs)]

pub mod permissions;
pub mod shell;
pub mod skills;
pub mod tools;
pub mod workspace;

pub use permissions::{Decision, Permissions, Profile};
pub use tools::{ToolOutput, run_tool, tool_specs};
pub use workspace::Workspace;
