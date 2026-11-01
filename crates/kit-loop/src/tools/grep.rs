//! `Grep`: regex search over files, honouring `.gitignore` (ripgrep's crates,
//! no external binary).

use super::{MAX_OUTPUT, ToolOutput, arg, truncate};
use crate::Workspace;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::{Searcher, sinks::UTF8};
use serde_json::Value;

const MAX_MATCHES: usize = 200;

pub fn run(ws: &Workspace, args: &Value) -> ToolOutput {
    let pattern = match arg(args, "pattern") {
        Ok(p) => p,
        Err(e) => return e,
    };
    let matcher = match RegexMatcherBuilder::new()
        .line_terminator(Some(b'\n'))
        .build(pattern)
    {
        Ok(m) => m,
        Err(e) => return ToolOutput::err(format!("invalid regex: {e}")),
    };
    let base = args["path"]
        .as_str()
        .map_or_else(|| ws.root.clone(), |p| ws.resolve(p));
    let filter = match args["glob"].as_str().map(globset::Glob::new).transpose() {
        Ok(g) => g.map(|g| g.compile_matcher()),
        Err(e) => return ToolOutput::err(format!("invalid glob: {e}")),
    };
    let mut lines = Vec::new();
    let mut searcher = Searcher::new();
    'files: for entry in ignore::WalkBuilder::new(&base)
        .hidden(false)
        .build()
        .flatten()
    {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let path = entry.path();
        if path.components().any(|c| c.as_os_str() == ".git") {
            continue;
        }
        if let Some(f) = &filter
            && !f.is_match(path.file_name().unwrap_or_default())
            && !f.is_match(ws.show(path))
        {
            continue;
        }
        let shown = ws.show(path);
        let _ = searcher.search_path(
            &matcher,
            path,
            UTF8(|n, text| {
                lines.push(format!("{shown}:{n}:{}", text.trim_end()));
                Ok(lines.len() < MAX_MATCHES)
            }),
        );
        if lines.len() >= MAX_MATCHES {
            lines.push(format!(
                "(stopped at {MAX_MATCHES} matches; narrow the pattern or path)"
            ));
            break 'files;
        }
    }
    if lines.is_empty() {
        return ToolOutput::ok("No matches");
    }
    ToolOutput::ok(truncate(&lines.join("\n"), MAX_OUTPUT))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn finds_matches_honours_gitignore_and_glob() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join("src")).unwrap();
        std::fs::create_dir_all(p.join("target")).unwrap();
        std::fs::write(p.join(".gitignore"), "target/\n").unwrap();
        std::fs::write(p.join("src/a.rs"), "fn parse() {}\nlet crlf = 1;\n").unwrap();
        std::fs::write(p.join("src/b.md"), "crlf note\n").unwrap();
        std::fs::write(p.join("target/x.rs"), "let crlf = 2;\n").unwrap();
        // .gitignore only applies inside a git repo.
        std::fs::create_dir(p.join(".git")).unwrap();
        let ws = Workspace::open(p).unwrap();
        let out = run(&ws, &json!({"pattern": "crlf", "glob": "*.rs"}));
        assert_eq!(out.content, "src/a.rs:2:let crlf = 1;");
        assert_eq!(run(&ws, &json!({"pattern": "zzz"})).content, "No matches");
        assert!(run(&ws, &json!({"pattern": "("})).is_error);
    }
}
