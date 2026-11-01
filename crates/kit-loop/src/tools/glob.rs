//! `Glob`: file paths matching a pattern, newest first.

use super::{ToolOutput, arg};
use crate::Workspace;
use serde_json::Value;
use std::time::SystemTime;

const MAX_PATHS: usize = 500;

pub fn run(ws: &Workspace, args: &Value) -> ToolOutput {
    let pattern = match arg(args, "pattern") {
        Ok(p) => p,
        Err(e) => return e,
    };
    let glob = match globset::GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
    {
        Ok(g) => g.compile_matcher(),
        Err(e) => return ToolOutput::err(format!("invalid glob: {e}")),
    };
    let base = args["path"]
        .as_str()
        .map_or_else(|| ws.root.clone(), |p| ws.resolve(p));
    let mut found: Vec<(SystemTime, String)> = ignore::WalkBuilder::new(&base)
        .hidden(false)
        .build()
        .flatten()
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
        .filter(|e| !e.path().components().any(|c| c.as_os_str() == ".git"))
        .filter_map(|e| {
            let rel = e
                .path()
                .strip_prefix(&base)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");
            glob.is_match(&rel).then(|| {
                let mtime = e
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                (mtime, ws.show(e.path()))
            })
        })
        .collect();
    if found.is_empty() {
        return ToolOutput::ok("No files match");
    }
    found.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let total = found.len();
    let mut out: Vec<String> = found.into_iter().take(MAX_PATHS).map(|(_, p)| p).collect();
    if total > MAX_PATHS {
        out.push(format!("({total} files; showing the newest {MAX_PATHS})"));
    }
    ToolOutput::ok(out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn matches_relative_paths_with_double_star() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join("src/deep")).unwrap();
        std::fs::write(p.join("src/deep/a.rs"), "").unwrap();
        std::fs::write(p.join("top.rs"), "").unwrap();
        std::fs::write(p.join("notes.md"), "").unwrap();
        let ws = Workspace::open(p).unwrap();
        let mut got: Vec<String> = run(&ws, &json!({"pattern": "**/*.rs"}))
            .content
            .lines()
            .map(String::from)
            .collect();
        got.sort();
        assert_eq!(got, ["src/deep/a.rs", "top.rs"]);
        assert_eq!(
            run(&ws, &json!({"pattern": "*.rs"})).content,
            "top.rs",
            "* stops at /"
        );
        assert_eq!(
            run(&ws, &json!({"pattern": "*.py"})).content,
            "No files match"
        );
    }
}
