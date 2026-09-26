//! `kit hook after-edit <KIT>`: what an agent's edit hook calls. Reads the
//! agent's hook JSON on stdin, finds the edited file, and runs the kit's
//! `after_edit` hooks whose glob matches, with `FILE` set.
//!
//! A failing hook is reported (exit 1) but never blocks the agent (exit 2
//! would feed the error back to Claude Code as an instruction).

use super::install::{home_dir, repo_root};
use super::lock::{Lock, LockedHook};
use super::writers::Scope;
use anyhow::Result;
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn after_edit(kit: &str) -> Result<i32> {
    let mut raw = String::new();
    std::io::stdin().take(4 << 20).read_to_string(&mut raw)?;
    let Some(file) = edited_file(&raw) else {
        return Ok(0);
    };
    let hooks = hooks_for(kit, &file)?;
    let mut code = 0;
    for h in hooks {
        if h.glob.as_deref().is_some_and(|g| !glob_match(g, &file)) {
            continue;
        }
        if !run(&h.run, &file)? {
            code = 1;
        }
    }
    Ok(code)
}

/// `tool_input.file_path` from the hook payload.
fn edited_file(raw: &str) -> Option<PathBuf> {
    let v: serde_json::Value = serde_json::from_str(raw).ok()?;
    let input = v.get("tool_input")?;
    let path = input
        .get("file_path")
        .or_else(|| input.get("path"))?
        .as_str()?;
    Some(PathBuf::from(path))
}

/// The kit's hooks: the repo install wins over the global one.
fn hooks_for(kit: &str, file: &Path) -> Result<Vec<LockedHook>> {
    let dir = file
        .parent()
        .filter(|d| d.is_dir())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let mut scopes = Vec::new();
    if let Some(root) = repo_root(&dir) {
        scopes.push(Scope::Repo(root));
    }
    scopes.push(Scope::Global { home: home_dir()? });
    for scope in scopes {
        if let Some(e) = Lock::load(&scope)?.get(kit) {
            return Ok(e.hooks.clone());
        }
    }
    Ok(Vec::new())
}

fn run(command: &str, file: &Path) -> Result<bool> {
    let shell = if cfg!(windows) { "bash" } else { "sh" };
    let Some(sh) = super::plan::find_program(shell) else {
        eprintln!("kit: hook skipped, {shell} is not on PATH: {command}");
        return Ok(true);
    };
    let out = std::process::Command::new(sh)
        .arg("-c")
        .arg(command)
        .env("FILE", file)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .output()?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let first = err.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        eprintln!("kit: hook failed on {}: {command}\n{first}", file.display());
    }
    Ok(out.status.success())
}

/// `*`, `?` and `{a,b}`. A pattern without `/` matches the file name;
/// with `/`, the path (`**` crosses folders).
pub fn glob_match(pattern: &str, file: &Path) -> bool {
    let path = file.to_string_lossy().replace('\\', "/");
    let subject = if pattern.contains('/') {
        path.as_str()
    } else {
        path.rsplit('/').next().unwrap_or(&path)
    };
    expand(pattern).iter().any(|p| {
        let p: Vec<char> = p.chars().collect();
        let s: Vec<char> = subject.chars().collect();
        if pattern.contains('/') {
            // Match any suffix at a folder boundary, so `src/*.rs` works
            // on an absolute path.
            (0..=s.len())
                .filter(|&i| i == 0 || s[i - 1] == '/')
                .any(|i| wild(&p, &s[i..]))
        } else {
            wild(&p, &s)
        }
    })
}

fn expand(pattern: &str) -> Vec<String> {
    let Some(open) = pattern.find('{') else {
        return vec![pattern.to_string()];
    };
    let Some(close) = pattern[open..].find('}').map(|c| open + c) else {
        return vec![pattern.to_string()];
    };
    let (head, tail) = (&pattern[..open], &pattern[close + 1..]);
    pattern[open + 1..close]
        .split(',')
        .flat_map(|alt| expand(&format!("{head}{alt}{tail}")))
        .collect()
}

fn wild(p: &[char], s: &[char]) -> bool {
    match p.first() {
        None => s.is_empty(),
        Some('*') if p.get(1) == Some(&'*') => {
            let rest = &p[2..];
            let rest = rest.strip_prefix(&['/']).unwrap_or(rest);
            (0..=s.len()).any(|i| wild(rest, &s[i..]))
        }
        Some('*') => (0..=s.len())
            .take_while(|&i| i == 0 || s[i - 1] != '/')
            .any(|i| wild(&p[1..], &s[i..])),
        Some('?') => !s.is_empty() && s[0] != '/' && wild(&p[1..], &s[1..]),
        Some(c) => s.first() == Some(c) && wild(&p[1..], &s[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globs_match_names_braces_and_paths() {
        let g = "*.{js,jsx,ts,tsx,css,scss,html,md}";
        assert!(glob_match(g, Path::new("/r/src/App.tsx")));
        assert!(glob_match(g, Path::new("README.md")));
        assert!(!glob_match(g, Path::new("/r/src/main.rs")));
        assert!(!glob_match(g, Path::new("/r/md")));
        assert!(glob_match("src/*.rs", Path::new("/r/src/main.rs")));
        assert!(!glob_match("src/*.rs", Path::new("/r/src/a/main.rs")));
        assert!(glob_match("src/**/*.rs", Path::new("/r/src/a/b/main.rs")));
        assert!(glob_match("src/**/*.rs", Path::new("/r/src/main.rs")));
        assert!(glob_match("?.md", Path::new("a.md")));
    }

    #[test]
    fn the_edited_file_comes_from_tool_input() {
        let raw = r#"{"tool_name":"Edit","tool_input":{"file_path":"/r/a.ts","old_string":"x"}}"#;
        assert_eq!(edited_file(raw), Some(PathBuf::from("/r/a.ts")));
        assert_eq!(edited_file(r#"{"tool_input":{}}"#), None);
        assert_eq!(edited_file("not json"), None);
    }
}
