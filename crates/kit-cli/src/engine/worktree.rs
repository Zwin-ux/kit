//! Isolated git worktrees for each run (PRD principle: no shared dirty tree).

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Create a detached worktree at `dest` from `repo` HEAD. Returns the base
/// commit: the sha the worktree starts at. The receipt diff is taken against
/// it, so an agent that commits in the worktree cannot hide its changes.
///
/// Branch name is local-only (`kit/run-<short-id>`). Fails if `repo` is not a git
/// checkout — production Kit does not invent a VCS.
pub fn create_worktree(repo: &Path, dest: &Path, branch: &str) -> Result<String> {
    if dest.exists() {
        bail!("worktree path already exists: {}", dest.display());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create worktree parent {}", parent.display()))?;
    }
    // Pin the sha first: HEAD can move while the worktree is being added.
    let base = head_commit(repo)?;

    // git narrates on stdout ("HEAD is now at …") and kit's stdout carries
    // `--json`, so git's stdout goes to our stderr, where people still see it.
    let dest_str = dest.to_str().context("worktree path utf-8")?;
    let status = git(repo)
        .args(["worktree", "add", "--quiet", "--detach", dest_str, &base])
        .stdout(std::io::stderr())
        .status()
        .with_context(|| format!("cannot run git worktree add {dest_str}"))?;

    if !status.success() {
        // Fallback: branch-based add when detach is rejected (older git).
        let status = git(repo)
            .args(["worktree", "add", "--quiet", "-B", branch, dest_str, &base])
            .stdout(std::io::stderr())
            .status()
            .context("git worktree add -B")?;
        if !status.success() {
            bail!("git worktree add failed for {}", dest.display());
        }
    }

    Ok(base)
}

/// The commit `HEAD` names in `dir`.
pub fn head_commit(dir: &Path) -> Result<String> {
    let out = git(dir)
        .args(["rev-parse", "--verify", "--quiet", "HEAD^{commit}"])
        .output()
        .context("git rev-parse HEAD")?;
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if !out.status.success() || sha.is_empty() {
        bail!(
            "{} has no commits yet. Make a first commit, then run again",
            dir.display()
        );
    }
    Ok(sha)
}

/// Binary-safe diff of everything the run changed since `base`: commits the
/// agent made, staged and unstaged edits, and new (untracked, not ignored)
/// files. Empty when nothing changed. Applies with `git apply` to `base`.
///
/// Stages all changes in the run's own worktree (`git add -A`); the user's
/// checkout is never touched.
pub fn worktree_diff(worktree: &Path, base: &str) -> Result<String> {
    let add = git(worktree)
        .args(["add", "-A"])
        .output()
        .context("git add -A")?;
    if !add.status.success() {
        bail!(
            "git add -A failed in {}: {}",
            worktree.display(),
            String::from_utf8_lossy(&add.stderr).trim()
        );
    }
    let output = git(worktree)
        .args([
            "diff",
            "--cached",
            "--binary",
            "--no-color",
            "--no-ext-diff",
            base,
        ])
        .output()
        .context("git diff")?;
    if !output.status.success() {
        bail!(
            "git diff {base} failed in {}: {}",
            worktree.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// True when the run left nothing behind: HEAD is still `base` and no file
/// differs from it. A moved HEAD (the agent committed) is never clean, even
/// when the tree matches, so no commit is lost with the worktree.
pub fn is_clean(worktree: &Path, base: &str) -> Result<bool> {
    if head_commit(worktree)? != base {
        return Ok(false);
    }
    let output = git(worktree)
        .args(["status", "--porcelain"])
        .output()
        .context("git status")?;
    if !output.status.success() {
        bail!("git status failed in {}", worktree.display());
    }
    Ok(output.stdout.is_empty())
}

/// Remove the worktree registration and directory when unchanged since `base`.
pub fn remove_if_clean(repo: &Path, worktree: &Path, base: &str) -> Result<bool> {
    if !is_clean(worktree, base)? {
        return Ok(false);
    }
    remove_worktree(repo, worktree);
    Ok(true)
}

/// Remove a run worktree: its registration and its directory. Best effort.
pub fn remove_worktree(repo: &Path, worktree: &Path) {
    // Same rule as create_worktree: nothing from git on kit's stdout.
    if let Some(path) = worktree.to_str() {
        let _ = git(repo)
            .args(["worktree", "remove", "--force", path])
            .stdout(std::io::stderr())
            .status();
    }
    if worktree.exists() {
        let _ = std::fs::remove_dir_all(worktree);
    }
}

pub(crate) fn git(cwd: &Path) -> Command {
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
    if abs.join(".git").exists() {
        return Ok(abs);
    }
    // From a subdirectory, use the repo root: the worktree is the whole repo,
    // the gate runs at its root, so kit.toml must be read there too.
    let top = git(&abs)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .filter(|s| !s.is_empty());
    match top {
        Some(top) => {
            let top = PathBuf::from(top);
            Ok(strip_verbatim(top.canonicalize().unwrap_or(top)))
        }
        None => bail!(
            "not a git repository: {} (Kit runs require git for isolation)",
            abs.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `kit run` from `repo/src` must use `repo`: the gate and kit.toml live at the root.
    #[test]
    fn subdirectory_resolves_to_repo_root() {
        let root = std::env::temp_dir().join(format!(
            "kit-subdir-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let sub = root.join("src").join("deep");
        std::fs::create_dir_all(&sub).unwrap();
        assert!(git(&root).args(["init", "-q"]).status().unwrap().success());
        let resolved = resolve_repo(sub.to_str().unwrap()).expect("inside a repo");
        assert_eq!(resolved, strip_verbatim(root.canonicalize().unwrap()));
        let _ = std::fs::remove_dir_all(&root);
    }

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
