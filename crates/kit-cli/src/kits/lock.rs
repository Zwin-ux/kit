//! `kit.lock`: what Kit installed, and exactly how to undo it.
//! Global installs: `~/.kit/kit.lock`. Repo installs: `<repo>/kit.lock`,
//! committed so teammates get the same kits.

use super::plan::Applied;
use super::writers::Scope;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    /// What fetches exactly this kit again (`github:o/r/path@sha`, or a
    /// folder); `kit sync` installs from it. Absent for bundled kits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin: Option<String>,
    /// Asked for by name, not only pulled in as a base.
    pub requested: bool,
    /// Installed kits that extend this one.
    #[serde(default)]
    pub required_by: Vec<String>,
    pub agents: Vec<String>,
    /// For `kit hook after-edit`.
    #[serde(default)]
    pub hooks: Vec<LockedHook>,
    pub applied: Vec<Applied>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedHook {
    pub glob: Option<String>,
    pub run: String,
}

pub fn path(scope: &Scope) -> PathBuf {
    match scope {
        Scope::Global { .. } => crate::engine::paths::kit_home().join("kit.lock"),
        Scope::Repo(root) => root.join("kit.lock"),
    }
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
        let lock: Self = serde_json::from_str(&raw)
            .with_context(|| format!("{} is damaged. Kit will not guess", file.display()))?;
        if lock.schema != SCHEMA {
            bail!(
                "{} was written by a newer kit (schema {}). Update kit",
                file.display(),
                lock.schema
            );
        }
        Ok(lock)
    }

    pub fn save(&self, scope: &Scope) -> Result<()> {
        let file = path(scope);
        if self.kits.is_empty() {
            if file.exists() {
                std::fs::remove_file(&file)?;
            }
            return Ok(());
        }
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = file.with_extension("lock.tmp");
        std::fs::write(&tmp, format!("{}\n", serde_json::to_string_pretty(self)?))?;
        std::fs::rename(&tmp, &file).with_context(|| format!("cannot write {}", file.display()))
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
