//! `Edit`: exact replacement. When the exact text is not found, one retry
//! ignores line endings and trailing whitespace, and says so. Nothing fuzzier.

use super::{ToolOutput, arg};
use crate::Workspace;
use serde_json::Value;

pub fn run(ws: &Workspace, args: &Value) -> ToolOutput {
    let (path, old, new) = match (
        arg(args, "file_path"),
        arg(args, "old_string"),
        arg(args, "new_string"),
    ) {
        (Ok(p), Ok(o), Ok(n)) => (ws.resolve(p), o, n),
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => return e,
    };
    let all = args["replace_all"].as_bool().unwrap_or(false);
    let shown = ws.show(&path);
    if old.is_empty() {
        return ToolOutput::err("old_string is empty; use Write to create a file");
    }
    if old == new {
        return ToolOutput::err("old_string and new_string are the same");
    }
    let text = match std::fs::read(&path) {
        Ok(b) => match String::from_utf8(b) {
            Ok(t) => t,
            Err(_) => {
                return ToolOutput::err(format!(
                    "{shown} is not UTF-8 text; Edit cannot change it safely"
                ));
            }
        },
        Err(e) => return ToolOutput::err(format!("cannot read {shown}: {e}")),
    };
    let crlf = text.contains("\r\n");
    let (updated, count, loose) = match replace(&text, old, new, all, crlf) {
        Ok(r) => r,
        Err(e) => return ToolOutput::err(format!("{shown}: {e}")),
    };
    if let Err(e) = std::fs::write(&path, &updated) {
        return ToolOutput::err(format!("cannot write {shown}: {e}"));
    }
    let note = if loose {
        " (matched ignoring line endings and trailing whitespace)"
    } else {
        ""
    };
    ToolOutput::ok(format!(
        "Edited {shown}: {count} replacement{}{note}",
        if count == 1 { "" } else { "s" }
    ))
}

/// Returns the new text, the number of replacements, and whether the loose match was used.
fn replace(
    text: &str,
    old: &str,
    new: &str,
    all: bool,
    crlf: bool,
) -> Result<(String, usize, bool), String> {
    let exact = text.matches(old).count();
    if exact > 0 {
        if exact > 1 && !all {
            return Err(format!(
                "old_string matches {exact} times; add context or set replace_all"
            ));
        }
        let n = if all { exact } else { 1 };
        return Ok((text.replacen(old, new, n), n, false));
    }
    // Loose: compare lines with trailing whitespace and CR removed.
    let want: Vec<&str> = old.lines().map(str::trim_end).collect();
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let starts: Vec<usize> = (0..lines.len().saturating_sub(want.len() - 1))
        .filter(|&i| {
            want.iter()
                .enumerate()
                .all(|(j, w)| lines[i + j].trim_end() == *w)
        })
        .collect();
    match starts.len() {
        0 => return Err("old_string not found (Read the file again: it may have changed)".into()),
        n if n > 1 && !all => {
            return Err(format!(
                "old_string matches {n} times; add context or set replace_all"
            ));
        }
        _ => {}
    }
    // The replacement takes the file's line ending.
    let eol = if crlf { "\r\n" } else { "\n" };
    let body = new.lines().collect::<Vec<_>>().join(eol);
    let mut out = String::new();
    let mut i = 0;
    let mut used = 0;
    while i < lines.len() {
        if starts.contains(&i) && (all || used == 0) {
            out.push_str(&body);
            if !body.is_empty() && lines[i + want.len() - 1].ends_with('\n') {
                out.push_str(eol);
            }
            i += want.len();
            used += 1;
        } else {
            out.push_str(lines[i]);
            i += 1;
        }
    }
    Ok((out, used, true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn edit(text: &str, args: Value) -> (ToolOutput, String) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("f.rs"), text).unwrap();
        let ws = Workspace::open(dir.path()).unwrap();
        let mut a = args;
        a["file_path"] = json!("f.rs");
        let out = run(&ws, &a);
        (
            out,
            std::fs::read_to_string(dir.path().join("f.rs")).unwrap(),
        )
    }

    #[test]
    fn exact_once_and_refuses_ambiguity() {
        let (out, text) = edit("a\nb\na\n", json!({"old_string": "b", "new_string": "B"}));
        assert_eq!(
            (out.content.as_str(), text.as_str()),
            ("Edited f.rs: 1 replacement", "a\nB\na\n")
        );
        let (out, text) = edit("a\nb\na\n", json!({"old_string": "a", "new_string": "x"}));
        assert!(out.is_error && out.content.contains("matches 2 times") && text == "a\nb\na\n");
        let (_, text) = edit(
            "a\nb\na\n",
            json!({"old_string": "a", "new_string": "x", "replace_all": true}),
        );
        assert_eq!(text, "x\nb\nx\n");
    }

    #[test]
    fn lf_old_string_matches_a_crlf_file_and_keeps_crlf() {
        let (out, text) = edit(
            "fn a() {\r\n    1\r\n}\r\n",
            json!({"old_string": "    1\n}", "new_string": "    2\n}"}),
        );
        assert!(
            out.content.contains("ignoring line endings"),
            "{}",
            out.content
        );
        assert_eq!(text, "fn a() {\r\n    2\r\n}\r\n");
    }

    #[test]
    fn trailing_whitespace_is_ignored_but_nothing_fuzzier() {
        let (out, text) = edit(
            "let x = 1;   \nlet y = 2;\n",
            json!({"old_string": "let x = 1;", "new_string": "let x = 3;"}),
        );
        assert!(!out.is_error, "{}", out.content);
        assert_eq!(
            text, "let x = 3;   \nlet y = 2;\n",
            "exact substring wins first"
        );
        let (out, _) = edit(
            "let  x = 1;\n",
            json!({"old_string": "let x = 1;", "new_string": "z"}),
        );
        assert!(out.is_error && out.content.contains("not found"));
    }

    #[test]
    fn non_utf8_and_no_op_edits_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("l.txt"), b"caf\xe9").unwrap();
        let ws = Workspace::open(dir.path()).unwrap();
        assert!(
            run(
                &ws,
                &json!({"file_path": "l.txt", "old_string": "caf", "new_string": "x"})
            )
            .content
            .contains("not UTF-8")
        );
        let (out, _) = edit("a", json!({"old_string": "a", "new_string": "a"}));
        assert!(out.is_error);
    }
}
