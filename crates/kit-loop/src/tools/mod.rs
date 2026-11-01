//! The tools the model sees (PRD §5.2). Names and parameter names are Claude
//! Code's, so hooks and permission rules written for Claude Code see the same
//! shapes. One file per tool.

use crate::Workspace;
use kit_providers::ToolSpec;
use serde_json::{Value, json};

mod bash;
mod edit;
mod glob;
mod grep;
mod read;
mod skill;
mod write;

/// What a tool returns to the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutput {
    /// Text for the model.
    pub content: String,
    /// True when the call failed; the model sees it as an error.
    pub is_error: bool,
}

impl ToolOutput {
    pub(crate) fn ok(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
        }
    }
    pub(crate) fn err(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
        }
    }
}

/// Tools that never write: safe to run side by side.
pub const READ_ONLY: &[&str] = &["Read", "Grep", "Glob", "Skill"];

/// Output beyond this is cut, keeping the head and the tail.
pub const MAX_OUTPUT: usize = 30_000;

/// The tool definitions sent to the model.
pub fn tool_specs() -> Vec<ToolSpec> {
    let spec = |name: &str, description: &str, schema: Value| ToolSpec {
        name: name.into(),
        description: description.into(),
        schema,
    };
    let s = |d: &str| json!({ "type": "string", "description": d });
    vec![
        spec(
            "Read",
            "Read a text file. Lines are numbered. Use offset/limit for long files.",
            json!({
                "type": "object", "required": ["file_path"],
                "properties": { "file_path": s("Path, absolute or relative to the workspace"),
                    "offset": { "type": "integer", "description": "First line, 1-based" },
                    "limit": { "type": "integer", "description": "Number of lines (default 2000)" } }
            }),
        ),
        spec(
            "Write",
            "Create or overwrite a file with exactly this content.",
            json!({
                "type": "object", "required": ["file_path", "content"],
                "properties": { "file_path": s("Path"), "content": s("The whole file") }
            }),
        ),
        spec(
            "Edit",
            "Replace old_string with new_string in a file. old_string must match exactly once unless replace_all. Read the file first.",
            json!({
                "type": "object", "required": ["file_path", "old_string", "new_string"],
                "properties": { "file_path": s("Path"), "old_string": s("Exact text to replace"),
                    "new_string": s("Replacement"), "replace_all": { "type": "boolean" } }
            }),
        ),
        spec(
            "Bash",
            "Run a shell command in the workspace. Returns output and exit code.",
            json!({
                "type": "object", "required": ["command"],
                "properties": { "command": s("The command"),
                    "timeout": { "type": "integer", "description": "Milliseconds, default 120000, max 600000" } }
            }),
        ),
        spec(
            "Grep",
            "Search file contents with a regex. Honours .gitignore. Returns path:line:text.",
            json!({
                "type": "object", "required": ["pattern"],
                "properties": { "pattern": s("Regex"), "path": s("File or folder (default: workspace)"),
                    "glob": s("Only files matching this glob, e.g. *.rs") }
            }),
        ),
        spec(
            "Glob",
            "Find files by glob pattern, newest first. Honours .gitignore.",
            json!({
                "type": "object", "required": ["pattern"],
                "properties": { "pattern": s("Glob, e.g. src/**/*.ts"), "path": s("Folder (default: workspace)") }
            }),
        ),
        spec(
            "Skill",
            "Load a skill's instructions by name (see the skills list).",
            json!({
                "type": "object", "required": ["name"], "properties": { "name": s("Skill name") }
            }),
        ),
    ]
}

/// Run one tool call. Permission checks happen before this.
pub async fn run_tool(ws: &Workspace, name: &str, args: &Value) -> ToolOutput {
    match name {
        "Read" => read::run(ws, args),
        "Write" => write::run(ws, args),
        "Edit" => edit::run(ws, args),
        "Bash" => bash::run(ws, args).await,
        "Grep" => grep::run(ws, args),
        "Glob" => glob::run(ws, args),
        "Skill" => skill::run(ws, args),
        other => ToolOutput::err(format!("unknown tool `{other}`")),
    }
}

/// A required string argument, or the error to send back.
pub(crate) fn arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, ToolOutput> {
    args[key]
        .as_str()
        .ok_or_else(|| ToolOutput::err(format!("missing string argument `{key}`")))
}

/// Keep the head and tail of long output, with a marker saying what was cut.
pub fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_owned();
    }
    let floor = |mut i: usize| {
        while !text.is_char_boundary(i) {
            i -= 1;
        }
        i
    };
    let head = floor(max / 2);
    let tail = floor(text.len() - max / 2);
    format!(
        "{}\n…[{} bytes cut]…\n{}",
        &text[..head],
        tail - head,
        &text[tail..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_keeps_head_and_tail_on_char_boundaries() {
        let text = "é".repeat(100);
        let cut = truncate(&text, 51);
        assert!(cut.starts_with('é') && cut.ends_with('é'));
        assert!(cut.contains("bytes cut"));
        assert_eq!(truncate("short", 10), "short");
    }

    #[test]
    fn every_spec_has_an_object_schema() {
        let specs = tool_specs();
        assert_eq!(
            specs.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            ["Read", "Write", "Edit", "Bash", "Grep", "Glob", "Skill"]
        );
        assert!(specs.iter().all(|s| s.schema["type"] == "object"));
    }
}
