//! Kit's built-in `format-and-lint` hook: after an agent edits a file, run
//! the formatter and linter the project already uses on that one file.
//!
//! Tools come from the project (`node_modules/.bin`, `.venv/bin`) or PATH.
//! Nothing is downloaded or installed; a missing Swift tool is named on
//! stderr and skipped. Formatter failures are reported but never block. Lint findings
//! exit 2 so Claude Code hands them to the model to fix.

use super::plan::find_program;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// How many lines of lint output reach the agent.
const MAX_LINES: usize = 40;

struct Tool {
    exe: PathBuf,
    args: Vec<String>,
    /// Where to run it, so the tool finds the project's config.
    cwd: PathBuf,
}

/// Exit code for the hook: 0, or 2 when the linter found problems.
pub fn format_and_lint(file: &Path) -> Result<i32> {
    if !file.is_file() {
        return Ok(0);
    }
    let (format, lint) = tools_for(file);
    if let Some(t) = format {
        let out = run(&t)?;
        if !out.0 {
            eprintln!(
                "kit: formatter could not format {}:\n{}",
                file.display(),
                out.1
            );
        }
    }
    if let Some(t) = lint {
        let (ok, text) = run(&t)?;
        if !ok {
            eprintln!("Lint findings in {} (fix these):\n{text}", file.display());
            return Ok(2);
        }
    }
    Ok(0)
}

fn run(t: &Tool) -> Result<(bool, String)> {
    let out = Command::new(&t.exe)
        .args(&t.args)
        .current_dir(&t.cwd)
        .stdin(Stdio::null())
        .output()?;
    let text: String = String::from_utf8_lossy(&out.stdout)
        .lines()
        .chain(String::from_utf8_lossy(&out.stderr).lines())
        .filter(|l| !l.trim().is_empty())
        .take(MAX_LINES)
        .collect::<Vec<_>>()
        .join("\n");
    Ok((out.status.success(), text))
}

/// The file as an absolute path, so it means the same file from any folder.
fn file_arg(file: &Path) -> String {
    let abs = if file.is_absolute() {
        file.to_path_buf()
    } else {
        std::env::current_dir().map_or_else(|_| file.to_path_buf(), |d| d.join(file))
    };
    abs.display().to_string()
}

/// The formatter and linter for this file's language, if installed.
fn tools_for(file: &Path) -> (Option<Tool>, Option<Tool>) {
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let dir = file.parent().unwrap_or(Path::new(".")).to_path_buf();
    // Run from the project root, so a root .swiftlint.yml or ruff config
    // applies, with the file's full path.
    let root = super::install::repo_root(&dir).unwrap_or_else(|| dir.clone());
    let f = file_arg(file);
    let tool = |exe: PathBuf, args: &[&str], cwd: &Path| Tool {
        exe,
        args: args.iter().map(|a| (*a).to_string()).collect(),
        cwd: cwd.to_path_buf(),
    };
    match ext.as_str() {
        "swift" => {
            let format = find_program("swiftformat")
                .map(|exe| tool(exe, &["--quiet", &f], &root))
                .or_else(|| {
                    find_program("swift-format")
                        .map(|exe| tool(exe, &["format", "--in-place", &f], &root))
                })
                .or_else(|| {
                    find_program("swift").map(|exe| tool(exe, &["format", "--in-place", &f], &root))
                });
            let lint = find_program("swiftlint")
                .map(|exe| tool(exe, &["lint", "--quiet", "--strict", &f], &root));
            if format.is_none() {
                eprintln!(
                    "kit: no Swift formatter found (swiftformat, swift-format or swift); {f} was not formatted"
                );
            }
            if lint.is_none() {
                eprintln!("kit: swiftlint is not installed; {f} was not linted");
            }
            (format, lint)
        }
        "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts" | "vue" | "svelte" => (
            node_tool("prettier", &dir)
                .map(|(exe, root)| tool(exe, &["--write", "--log-level", "warn", &f], &root)),
            node_tool("eslint", &dir).map(|(exe, root)| tool(exe, &[&f], &root)),
        ),
        "css" | "scss" | "less" | "html" | "json" | "md" | "mdx" | "yaml" | "yml" => (
            node_tool("prettier", &dir)
                .map(|(exe, root)| tool(exe, &["--write", "--log-level", "warn", &f], &root)),
            None,
        ),
        "py" | "pyi" => {
            let ruff = venv_tool("ruff", &dir).or_else(|| find_program("ruff"));
            (
                ruff.clone()
                    .map(|exe| tool(exe, &["format", "--quiet", &f], &root)),
                ruff.map(|exe| tool(exe, &["check", "--quiet", &f], &root)),
            )
        }
        "rs" => {
            let edition = rust_edition(&dir);
            (
                find_program("rustfmt").map(|exe| tool(exe, &["--edition", &edition, &f], &dir)),
                None,
            )
        }
        "go" => (
            find_program("gofmt").map(|exe| tool(exe, &["-w", &f], &dir)),
            None,
        ),
        _ => (None, None),
    }
}

/// `node_modules/.bin/<name>` in this folder or a parent, with that
/// project root. Only the project's own install counts; never npx.
fn node_tool(name: &str, from: &Path) -> Option<(PathBuf, PathBuf)> {
    let names: Vec<String> = if cfg!(windows) {
        vec![format!("{name}.cmd"), format!("{name}.exe")]
    } else {
        vec![name.to_string()]
    };
    from.ancestors().find_map(|dir| {
        let bin = dir.join("node_modules").join(".bin");
        names
            .iter()
            .map(|n| bin.join(n))
            .find(|p| p.is_file())
            .map(|p| (p, dir.to_path_buf()))
    })
}

/// A tool in the nearest `.venv`.
fn venv_tool(name: &str, from: &Path) -> Option<PathBuf> {
    let rel = if cfg!(windows) {
        PathBuf::from(".venv")
            .join("Scripts")
            .join(format!("{name}.exe"))
    } else {
        PathBuf::from(".venv").join("bin").join(name)
    };
    from.ancestors().map(|d| d.join(&rel)).find(|p| p.is_file())
}

/// The edition in the nearest Cargo.toml, so rustfmt parses the file the
/// way cargo would. Workspace-inherited editions fall back to the root's.
fn rust_edition(from: &Path) -> String {
    for dir in from.ancestors() {
        let Ok(raw) = std::fs::read_to_string(dir.join("Cargo.toml")) else {
            continue;
        };
        let Ok(doc) = raw.parse::<toml::Table>() else {
            continue;
        };
        let edition = doc
            .get("package")
            .and_then(|p| p.get("edition"))
            .and_then(|e| e.as_str())
            .or_else(|| {
                doc.get("workspace")
                    .and_then(|w| w.get("package"))
                    .and_then(|p| p.get("edition"))
                    .and_then(|e| e.as_str())
            });
        if let Some(e) = edition {
            return e.to_string();
        }
    }
    "2021".into()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("kit-fmt-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn script(path: &Path, body: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn project_tools_format_then_lint_findings_exit_2() {
        let root = scratch("node");
        let bin = root.join("node_modules/.bin");
        script(&bin.join("prettier"), r#"echo formatted >> "$PWD/calls""#);
        script(
            &bin.join("eslint"),
            r#"echo "a.ts:1:1 no-unused-vars"; exit 1"#,
        );
        let file = root.join("src/a.ts");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "let a = 1\n").unwrap();

        assert_eq!(format_and_lint(&file).unwrap(), 2);
        let calls = std::fs::read_to_string(root.join("calls")).unwrap();
        assert_eq!(calls, "formatted\n", "prettier ran from the project root");

        script(&bin.join("eslint"), "exit 0");
        assert_eq!(format_and_lint(&file).unwrap(), 0);
    }

    #[test]
    fn unknown_files_and_missing_tools_do_nothing() {
        let root = scratch("none");
        let file = root.join("notes.txt");
        std::fs::write(&file, "x").unwrap();
        assert_eq!(format_and_lint(&file).unwrap(), 0);
        let css = root.join("a.css");
        std::fs::write(&css, "a{}").unwrap();
        assert_eq!(format_and_lint(&css).unwrap(), 0, "no prettier installed");
    }

    #[test]
    fn rust_edition_comes_from_the_nearest_manifest() {
        let root = scratch("rs");
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace.package]\nedition = \"2024\"\n",
        )
        .unwrap();
        let krate = root.join("crates/a");
        std::fs::create_dir_all(&krate).unwrap();
        std::fs::write(
            krate.join("Cargo.toml"),
            "[package]\nname = \"a\"\nedition.workspace = true\n",
        )
        .unwrap();
        assert_eq!(rust_edition(&krate), "2024");
    }
}
