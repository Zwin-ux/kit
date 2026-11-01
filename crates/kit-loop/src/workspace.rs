//! The folder the agent works in, and paths relative to it.

use crate::shell::Shell;
use crate::skills::SkillInfo;
use std::path::{Component, Path, PathBuf};

/// Where the agent works: the repo, or a run's worktree.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// The workspace root, canonical.
    pub root: PathBuf,
    /// The user's home folder, for `~` in rules and the deny-read list.
    pub home: Option<PathBuf>,
    /// The shell `Bash` runs.
    pub shell: Shell,
    /// Skills the agent may load.
    pub skills: Vec<SkillInfo>,
    /// The repo's own `kit.toml` gate commands: trusted, because the user wrote them.
    pub gate_commands: Vec<String>,
    /// Extra environment variables the user passes to tools by name.
    pub extra_env: Vec<String>,
}

impl Workspace {
    /// A workspace at `root`, with the shell and skills found on this machine.
    pub fn open(root: &Path) -> std::io::Result<Self> {
        let root = dunce(root.canonicalize()?);
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from);
        let skills = crate::skills::discover(&root, home.as_deref());
        Ok(Self {
            root,
            home,
            shell: Shell::detect(),
            skills,
            gate_commands: vec![],
            extra_env: vec![],
        })
    }

    /// `p` made absolute against the root, with `.` and `..` resolved
    /// lexically. Symlinks are resolved by [`Self::inside`].
    pub fn resolve(&self, p: &str) -> PathBuf {
        let path = Path::new(p);
        let joined = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        normalize(&joined)
    }

    /// Whether `path` is inside the workspace, after resolving symlinks of
    /// the longest part that exists (a new file's parent counts).
    pub fn inside(&self, path: &Path) -> bool {
        let mut existing = path.to_path_buf();
        let mut rest = Vec::new();
        while !existing.exists() {
            match (
                existing.file_name().map(|n| n.to_owned()),
                existing.parent(),
            ) {
                (Some(name), Some(parent)) => {
                    rest.push(name);
                    existing = parent.to_path_buf();
                }
                _ => return false,
            }
        }
        let Ok(real) = existing.canonicalize() else {
            return false;
        };
        let mut real = dunce(real);
        for name in rest.into_iter().rev() {
            real.push(name);
        }
        real.starts_with(&self.root)
    }

    /// A path for display: relative to the root when inside it.
    pub fn show(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .display()
            .to_string()
            .replace('\\', "/")
    }
}

/// Lexical normalisation: drop `.`, apply `..`.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// Windows `canonicalize` returns `\\?\C:\…`; Kit prints and compares plain paths.
pub fn dunce(p: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let s = p.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\")
            && !rest.starts_with("UNC\\")
        {
            return PathBuf::from(rest);
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inside_follows_symlinks_and_rejects_dot_dot() {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::open(dir.path()).unwrap();
        assert!(
            ws.inside(&ws.resolve("src/new/file.rs")),
            "a new file under the root is inside"
        );
        assert!(!ws.inside(&ws.resolve("../escape.txt")));
        assert!(!ws.inside(&ws.resolve(if cfg!(windows) {
            r"C:\Windows\x"
        } else {
            "/etc/passwd"
        })));
        #[cfg(unix)]
        {
            let outside = tempfile::tempdir().unwrap();
            std::os::unix::fs::symlink(outside.path(), dir.path().join("link")).unwrap();
            assert!(
                !ws.inside(&ws.resolve("link/secret")),
                "a symlink out of the root is outside"
            );
        }
    }
}
