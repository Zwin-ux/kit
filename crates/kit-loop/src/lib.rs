//! Kit's own coding agent (docs/PRD.md §5): the workspace it works in, its
//! tools, the permission check every tool call passes first, and the loop.

#![warn(missing_docs)]

pub mod agent;
pub mod meter;
pub mod permissions;
pub mod prompt;
pub mod session;
pub mod shell;
pub mod skills;
pub mod textproto;
pub mod tools;
pub mod workspace;

pub use agent::{Agent, AgentEvent, Approver, Cancel, Config, DenyAll, Outcome};
pub use meter::{Meter, Prices};
pub use permissions::{Decision, Permissions, Profile, Rule};
pub use session::SessionLog;
pub use tools::{ToolOutput, run_tool, tool_specs};
pub use workspace::Workspace;
