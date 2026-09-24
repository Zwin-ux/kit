//! Isolated git worktrees for each run (PRD principle: no shared dirty tree).

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Create a detached worktree at `dest` from `repo` HEAD.
///
/// Branch name is local-only (`kit/run-<short-id>`). Fails if `repo` is not a git
/// checkout — production Kit does not invent a VCS.
pub fn create_worktree(repo: &Path, dest: &Path, branch: &str) -> Result<()> {
    if dest.exists() {
        bail!("worktree path already exists: {}", dest.display());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create worktree parent {}", parent.display()))?;
    }

    // git narrates on stdout ("HEAD is now at …") and kit's stdout carries
    // `--json`, so git's stdout goes to our stderr, where people still see it.
    let status = git(repo)
        .args([
            "worktree",
            "add",
            "--quiet",
            "--detach",
            dest.to_str().context("worktree path utf-8")?,
            "HEAD",
        ])
        .stdout(std::io::stderr())
        .status()
        .context("git worktree add")?;

    if !status.success() {
        // Fallback: branch-based add when detach is rejected (older git).
        let status = git(repo)
            .args([
                "worktree",
                "add",
                "--quiet",
                "-B",
                branch,
                dest.to_str().context("worktree path utf-8")?,
                "HEAD",
            ])
            .stdout(std::io::stderr())
            .status()
            .context("git worktree add -B")?;
        if !status.success() {
            bail!("git worktree add failed for {}", dest.display());
        }
    }

    Ok(())
}

/// Unified diff of uncommitted changes in the worktree (empty if clean).
pub fn worktree_diff(worktree: &Path) -> Result<String> {
    let output = git(worktree)
        .args(["diff", "HEAD"])
        .output()
        .context("git diff")?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// True when the worktree has no staged or unstaged changes vs HEAD.
pub fn is_clean(worktree: &Path) -> Result<bool> {
    let output = git(worktree)
        .args(["status", "--porcelain"])
        .output()
        .context("git status")?;
    Ok(output.stdout.is_empty())
}

/// Remove the worktree registration and directory when unchanged.
pub fn remove_if_clean(repo: &Path, worktree: &Path) -> Result<bool> {
    if !is_clean(worktree)? {
        return Ok(false);
    }
    // Same rule as create_worktree: nothing from git on kit's stdout.
    let _ = git(repo)
        .args([
            "worktree",
            "remove",
            "--force",
            worktree.to_str().context("worktree path utf-8")?,
        ])
        .stdout(std::io::stderr())
        .status();
    if worktree.exists() {
        let _ = std::fs::remove_dir_all(worktree);
    }
    Ok(true)
}

fn git(cwd: &Path) -> Command {
    let mut c = Command::new("git");
    c.current_dir(cwd);
    // Avoid interactive prompts in CI/agent environments.
    c.env("GIT_TERMINAL_PROMPT", "0");
    // Discover .git from `cwd`. Inherited GIT_DIR would operate on the parent
    // checkout (this workspace's kit.toml) instead of the run's repo.
    c.env_remove("GIT_DIR");
    c.env_remove("GIT_WORK_TREE");
    c.env_remove("GIT_INDEX_FILE");
    c.env_remove("GIT_OBJECT_DIRECTORY");
    c.env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES");
    c.env_remove("GIT_COMMON_DIR");
    c.env_remove("GIT_PREFIX");
    c
}

/// `C:\x` instead of `\\?\C:\x`. Windows `canonicalize` returns verbatim
/// paths, which leak into errors and receipts and confuse other tools.
/// UNC (`\\?\UNC\…`) and non-Windows paths are unchanged.
pub(crate) fn strip_verbatim(path: PathBuf) -> PathBuf {
    let Some(text) = path.to_str() else {
        return path;
    };
    match text.strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => path,
    }
}

/// Sanitize a run id into a short branch segment.
pub fn branch_name(run_id: &str) -> String {
    let short: String = run_id.chars().take(12).collect();
    format!("kit/run-{short}")
}

/// Resolve a user-facing repo token to an absolute path.
///
/// `.` / empty → cwd. Existing path → that path. Short name matching cwd
/// basename (e.g. form label `kit` while cwd is `.../kit`) → cwd. Else join cwd.
pub fn resolve_repo(token: &str) -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("cwd")?;
    let abs = if token.is_empty() || token == "." {
        cwd
    } else {
        let raw = PathBuf::from(token);
        if raw.is_absolute() && raw.exists() {
            raw
        } else if cwd.join(&raw).exists() {
            cwd.join(raw)
        } else if cwd.file_name().and_then(|n| n.to_str()) == Some(token) {
            cwd
        } else if raw.exists() {
            raw
        } else {
            // Last resort: cwd (dispatch form labels may not be paths).
            cwd
        }
    };
    let abs = strip_verbatim(abs.canonicalize().unwrap_or(abs));
    let git_ok = abs.join(".git").exists()
        || git(&abs)
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    if !git_ok {
        bail!(
            "not a git repository: {} (Kit runs require git for isolation)",
            abs.display()
        );
    }
    Ok(abs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbatim_drive_paths_lose_the_prefix() {
        assert_eq!(
            strip_verbatim(PathBuf::from(r"\\?\C:\work\repo")),
            PathBuf::from(r"C:\work\repo")
        );
        let unc = PathBuf::from(r"\\?\UNC\server\share");
        assert_eq!(strip_verbatim(unc.clone()), unc);
        assert_eq!(
            strip_verbatim(PathBuf::from("/home/me/repo")),
            PathBuf::from("/home/me/repo")
        );
    }
}
