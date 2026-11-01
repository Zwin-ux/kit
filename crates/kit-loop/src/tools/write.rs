//! `Write`: create or overwrite a file.

use super::{ToolOutput, arg};
use crate::Workspace;
use serde_json::Value;

pub fn run(ws: &Workspace, args: &Value) -> ToolOutput {
    let (path, content) = match (arg(args, "file_path"), arg(args, "content")) {
        (Ok(p), Ok(c)) => (ws.resolve(p), c),
        (Err(e), _) | (_, Err(e)) => return e,
    };
    match path.parent() {
        Some(parent) if parent.is_dir() => {}
        _ => return ToolOutput::err(format!("the folder for {} does not exist", ws.show(&path))),
    }
    let existed = path.exists();
    match std::fs::write(&path, content) {
        Ok(()) => ToolOutput::ok(format!(
            "{} {} ({} bytes)",
            if existed { "Overwrote" } else { "Created" },
            ws.show(&path),
            content.len()
        )),
        Err(e) => ToolOutput::err(format!("cannot write {}: {e}", ws.show(&path))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn creates_then_overwrites_and_needs_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::open(dir.path()).unwrap();
        assert_eq!(
            run(&ws, &json!({"file_path": "n.txt", "content": "hi"})).content,
            "Created n.txt (2 bytes)"
        );
        assert!(
            run(&ws, &json!({"file_path": "n.txt", "content": "yo"}))
                .content
                .starts_with("Overwrote")
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("n.txt")).unwrap(),
            "yo"
        );
        assert!(run(&ws, &json!({"file_path": "no/such/dir.txt", "content": ""})).is_error);
    }
}
