//! `kit.lock`: what Kit installed, and exactly how to undo it.
//!
//! **Kit's own record is the only one Kit acts on.** Global installs live
//! in `~/.kit/kit.lock`. Repo installs are recorded in
//! `~/.kit/repos/<id>/kit.lock`, keyed by the repo's path, and written only
//! by `kit add` after the user approved the plan. Doctor, hooks and
//! `kit remove` read that record and nothing else.
//!
//! The repo's own `<repo>/kit.lock` is a copy for sharing (commit it so
//! teammates see which kits the repo uses). Kit writes it but never reads
//! it back: a cloned repo is untrusted input, and its lock must not be able
//! to choose what Kit runs or deletes.
//!
//! Paths in a record are relative to the scope's root (the repo, or home),
//! checked on load to stay inside it: nothing in a record can point Kit at
//! a file elsewhere, and the shared copy names no one's home folder.

use super::plan::Applied;
use super::writers::Scope;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

pub const SCHEMA: u32 = 1;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Lock {
    pub schema: u32,
    #[serde(default)]
    pub kits: Vec<Entry>,
}

/// One installed kit. A base pulled in by `extends` is its own entry, so
/// two kits that share it install it once.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub version: String,
    /// What was typed: a bundled name or a folder.
    pub source: String,
    /// Asked for by name, not only pulled in as a base.
    pub requested: bool,
    /// Installed kits that extend this one.
    #[serde(default)]
    pub required_by: Vec<String>,
    pub agents: Vec<String>,
    /// For `kit hook after-edit`: the hooks approved at install.
    #[serde(default)]
    pub hooks: Vec<LockedHook>,
    /// For `kit doctor`: the checks approved at install, so a KIT.toml that
    /// changes on disk later cannot change what doctor runs.
    #[serde(default)]
    pub checks: ApprovedChecks,
    pub applied: Vec<Applied>,
}

/// What `kit doctor` may run for a kit, fixed when it was installed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovedChecks {
    /// MCP servers that must answer `initialize`, as they were installed.
    #[serde(default)]
    pub mcp: std::collections::BTreeMap<String, super::manifest::McpServer>,
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedHook {
    pub glob: Option<String>,
    #[serde(default)]
    pub run: Option<String>,
    #[serde(default, rename = "use")]
    pub builtin: Option<super::manifest::Builtin>,
}

/// Kit's own record for this scope: the one Kit acts on.
pub fn path(scope: &Scope) -> PathBuf {
    let home = crate::engine::paths::kit_home();
    match scope {
        Scope::Global { .. } => home.join("kit.lock"),
        Scope::Repo(root) => home.join("repos").join(repo_id(root)).join("kit.lock"),
    }
}

/// The shareable copy committed in the repo. Written, never read.
pub fn shared_path(scope: &Scope) -> Option<PathBuf> {
    match scope {
        Scope::Global { .. } => None,
        Scope::Repo(root) => Some(root.join("kit.lock")),
    }
}

/// Where a scope's record paths are relative to.
pub fn base(scope: &Scope) -> &Path {
    match scope {
        Scope::Global { home } => home,
        Scope::Repo(root) => root,
    }
}

/// Top-level folders a global record may touch under home.
const GLOBAL_DIRS: [&str; 3] = [".claude", ".agents", ".codex"];

/// `<folder>-<hash>` of the repo's own root and its git folder, so every
/// clone and every worktree has its own record, and a folder whose `.git`
/// file points at another repo's git folder cannot take over its record.
fn repo_id(root: &Path) -> String {
    use sha2::{Digest, Sha256};
    let git = std::process::Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .current_dir(root)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()));
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let key = format!(
        "{}\n{}",
        canon(root).display(),
        git.as_deref().map(canon).unwrap_or_default().display()
    );
    let digest = Sha256::digest(key.as_bytes());
    let hex: String = digest.iter().take(8).map(|b| format!("{b:02x}")).collect();
    let name: String = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .take(40)
        .collect();
    format!("{name}-{hex}")
}

impl Lock {
    pub fn load(scope: &Scope) -> Result<Self> {
        let file = path(scope);
        let raw = match std::fs::read_to_string(&file) {
            Ok(raw) => raw,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self {
                    schema: SCHEMA,
                    kits: Vec::new(),
                });
            }
            Err(e) => return Err(e).with_context(|| format!("cannot read {}", file.display())),
        };
        let mut lock: Self = serde_json::from_str(&raw)
            .with_context(|| format!("{} is damaged. Kit will not guess", file.display()))?;
        if lock.schema != SCHEMA {
            bail!(
                "{} was written by a newer kit (schema {}). Update kit",
                file.display(),
                lock.schema
            );
        }
        for a in lock.kits.iter_mut().flat_map(|e| &mut e.applied) {
            if let Some(path) = a.path_mut() {
                *path = anchor(path, scope).with_context(|| {
                    format!("{} is damaged. Kit will not guess", file.display())
                })?;
            }
        }
        Ok(lock)
    }

    pub fn save(&self, scope: &Scope) -> Result<()> {
        let mut stored = Self {
            schema: self.schema,
            kits: self.kits.clone(),
        };
        for a in stored.kits.iter_mut().flat_map(|e| &mut e.applied) {
            if let Some(path) = a.path_mut() {
                *path = relative(path, scope)?;
            }
        }
        let body = format!("{}\n", serde_json::to_string_pretty(&stored)?);
        write_or_remove(&path(scope), (!self.kits.is_empty()).then_some(&body))?;
        if let Some(shared) = shared_path(scope) {
            // A kit from a folder is named by where it sits in the repo, or
            // only by its folder name: never a path on this machine.
            for e in &mut stored.kits {
                // What the user's files held before Kit stays on this machine.
                for a in &mut e.applied {
                    a.forget_original();
                }
                let src = Path::new(&e.source);
                if src.is_absolute() {
                    e.source = match relative(src, scope) {
                        Ok(rel) => format!("./{}", rel.display()),
                        Err(_) => format!(
                            "folder {}",
                            src.file_name().unwrap_or_default().to_string_lossy()
                        ),
                    };
                }
            }
            let body = format!("{}\n", serde_json::to_string_pretty(&stored)?);
            write_or_remove(&shared, (!self.kits.is_empty()).then_some(&body))?;
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Entry> {
        self.kits.iter().find(|e| e.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Entry> {
        self.kits.iter_mut().find(|e| e.name == name)
    }

    /// Everything installed by any kit, by key.
    pub fn applied(&self) -> impl Iterator<Item = &Applied> {
        self.kits.iter().flat_map(|e| &e.applied)
    }
}

/// `path` relative to the scope's root, with `/` separators.
fn relative(path: &Path, scope: &Scope) -> Result<PathBuf> {
    let rel = path.strip_prefix(base(scope)).with_context(|| {
        format!(
            "{} is outside {}; Kit will not record it",
            path.display(),
            base(scope).display()
        )
    })?;
    Ok(PathBuf::from(rel.to_string_lossy().replace('\\', "/")))
}

/// A recorded relative path, back under the scope's root. Anything that
/// could leave it (absolute, `..`, or for home a folder that is not an
/// agent's) is refused.
fn anchor(rel: &Path, scope: &Scope) -> Result<PathBuf> {
    let parts: Vec<Component<'_>> = rel.components().collect();
    let plain = !parts.is_empty() && parts.iter().all(|c| matches!(c, Component::Normal(_)));
    let allowed = match scope {
        Scope::Repo(_) => true,
        Scope::Global { .. } => parts
            .first()
            .is_some_and(|c| GLOBAL_DIRS.iter().any(|d| c.as_os_str() == *d)),
    };
    if !plain || !allowed {
        bail!("it names {}, outside Kit's folders", rel.display());
    }
    Ok(base(scope).join(rel))
}

/// Kit's lock files are never links, not even to a file in the same repo.
pub fn check_not_linked(scope: &Scope) -> Result<()> {
    for file in std::iter::once(path(scope)).chain(shared_path(scope)) {
        if std::fs::symlink_metadata(&file).is_ok_and(|m| m.file_type().is_symlink()) {
            bail!(
                "{} is a link. Kit keeps its record there and will not write through it. Remove the link and run again",
                file.display()
            );
        }
    }
    Ok(())
}

fn write_or_remove(file: &std::path::Path, body: Option<&String>) -> Result<()> {
    if std::fs::symlink_metadata(file).is_ok_and(|m| m.file_type().is_symlink()) {
        bail!(
            "{} is a link. Kit will not write through it",
            file.display()
        );
    }
    let Some(body) = body else {
        if file.exists() {
            std::fs::remove_file(file)?;
        }
        return Ok(());
    };
    super::plan::write_file(file, body.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorded_paths_stay_inside_their_scope() {
        let repo = Scope::Repo(PathBuf::from("/r"));
        let global = Scope::Global {
            home: PathBuf::from("/h"),
        };
        assert_eq!(
            anchor(Path::new(".claude/skills/a"), &repo).unwrap(),
            PathBuf::from("/r/.claude/skills/a")
        );
        assert_eq!(
            anchor(Path::new(".codex/config.toml"), &global).unwrap(),
            PathBuf::from("/h/.codex/config.toml")
        );
        for bad in ["/etc/passwd", "../x", ".claude/../../x", ""] {
            assert!(anchor(Path::new(bad), &repo).is_err(), "{bad}");
        }
        for bad in [".ssh/id_ed25519", ".bashrc", "code/proj/CLAUDE.md"] {
            assert!(anchor(Path::new(bad), &global).is_err(), "{bad}");
        }
        assert_eq!(
            relative(Path::new("/r/.claude/skills/a"), &repo).unwrap(),
            PathBuf::from(".claude/skills/a")
        );
        assert!(relative(Path::new("/elsewhere/x"), &repo).is_err());
    }
}
