//! `Skill`: load a skill's instructions by name.

use super::{ToolOutput, arg};
use crate::{Workspace, skills};
use serde_json::Value;

pub fn run(ws: &Workspace, args: &Value) -> ToolOutput {
    let name = match arg(args, "name") {
        Ok(n) => n,
        Err(e) => return e,
    };
    let Some(skill) = ws.skills.iter().find(|s| s.name == name) else {
        let known: Vec<&str> = ws.skills.iter().map(|s| s.name.as_str()).collect();
        return ToolOutput::err(format!(
            "no skill named `{name}`; available: {}",
            if known.is_empty() {
                "none".to_owned()
            } else {
                known.join(", ")
            }
        ));
    };
    match skills::body(skill) {
        Ok(body) => ToolOutput::ok(format!("# Skill: {name}\n{body}")),
        Err(e) => ToolOutput::err(format!("cannot read skill `{name}`: {e}")),
    }
}
