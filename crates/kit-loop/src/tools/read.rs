//! `Read`: a text file with numbered lines.

use super::{MAX_OUTPUT, ToolOutput, arg, truncate};
use crate::Workspace;
use serde_json::Value;

const DEFAULT_LIMIT: usize = 2000;
const MAX_LINE: usize = 2000;

pub fn run(ws: &Workspace, args: &Value) -> ToolOutput {
    let path = match arg(args, "file_path") {
        Ok(p) => ws.resolve(p),
        Err(e) => return e,
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => return ToolOutput::err(format!("cannot read {}: {e}", ws.show(&path))),
    };
    if bytes[..bytes.len().min(8192)].contains(&0) {
        return ToolOutput::err(format!(
            "{} is a binary file ({} bytes)",
            ws.show(&path),
            bytes.len()
        ));
    }
    let lossy = std::str::from_utf8(&bytes).is_err();
    let text = String::from_utf8_lossy(&bytes);
    if text.is_empty() {
        return ToolOutput::ok(format!("{} is empty", ws.show(&path)));
    }
    let offset = args["offset"].as_u64().unwrap_or(1).max(1) as usize;
    let limit = args["limit"].as_u64().map_or(DEFAULT_LIMIT, |l| l as usize);
    let total = text.lines().count();
    let mut out = String::new();
    if lossy {
        out.push_str("(not valid UTF-8; invalid bytes shown as \u{fffd})\n");
    }
    for (i, line) in text.lines().enumerate().skip(offset - 1).take(limit) {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let shown = match line.char_indices().nth(MAX_LINE) {
            Some((cut, _)) => format!("{}…", &line[..cut]),
            None => line.to_owned(),
        };
        out.push_str(&format!("{:>6}\t{shown}\n", i + 1));
    }
    let end = (offset - 1 + limit).min(total);
    if end < total {
        out.push_str(&format!(
            "(lines {offset}-{end} of {total}; use offset to read more)\n"
        ));
    }
    ToolOutput::ok(truncate(&out, MAX_OUTPUT))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numbers_lines_strips_cr_and_pages() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "one\r\ntwo\r\nthree\r\n").unwrap();
        let ws = Workspace::open(dir.path()).unwrap();
        let out = run(&ws, &json!({"file_path": "a.txt", "offset": 2, "limit": 1}));
        assert_eq!(
            out.content,
            "     2\ttwo\n(lines 2-2 of 3; use offset to read more)\n"
        );
    }

    #[test]
    fn refuses_binary_and_reports_missing_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("b.bin"), [0u8, 1, 2]).unwrap();
        let ws = Workspace::open(dir.path()).unwrap();
        assert!(
            run(&ws, &json!({"file_path": "b.bin"}))
                .content
                .contains("binary file (3 bytes)")
        );
        assert!(run(&ws, &json!({"file_path": "nope.txt"})).is_error);
    }

    #[test]
    fn invalid_utf8_is_shown_with_a_note() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("l.txt"), b"caf\xe9\n").unwrap();
        let ws = Workspace::open(dir.path()).unwrap();
        let out = run(&ws, &json!({"file_path": "l.txt"}));
        assert!(out.content.starts_with("(not valid UTF-8") && out.content.contains("caf\u{fffd}"));
    }
}
