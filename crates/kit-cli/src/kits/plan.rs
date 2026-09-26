//! Plans: every change `kit add` would make, as data. The confirm screen,
//! `--print`, apply, the lock file and `kit remove` all read the same
//! actions. Each applied action is recorded with what it takes to undo it.
//! See `docs/dev/DESIGN-KITS.md` §3.

use super::fetch::SkillPayload;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Marker file in every skill folder Kit writes: `<hash> <kit>`.
pub const OWNED: &str = ".kit-owned";

/// One change to make.
#[derive(Debug, Clone)]
pub enum Action {
    /// Write a skill folder.
    Skill { dir: PathBuf, payload: SkillPayload },
    /// Add or replace this kit's block in an instruction file.
    Rules {
        file: PathBuf,
        kit: String,
        version: String,
        text: String,
    },
    /// Set `mcpServers.<name>` in a JSON file (`.mcp.json`).
    McpJson {
        file: PathBuf,
        name: String,
        value: serde_json::Value,
    },
    /// Set `[mcp_servers.<name>]` in a TOML file (Codex `config.toml`).
    McpToml {
        file: PathBuf,
        name: String,
        value: toml_edit::Table,
    },
    /// A user-scope Claude Code server, added through Claude Code's own CLI
    /// (`claude mcp add-json --scope user`), which owns `~/.claude.json`.
    ClaudeMcp {
        name: String,
        value: serde_json::Value,
    },
    /// Append one entry to `hooks.<event>` in a JSON settings file.
    HookJson {
        file: PathBuf,
        event: String,
        entry: serde_json::Value,
    },
    /// A piece this agent cannot take, said out loud.
    Skip { piece: String, why: String },
}

impl Action {
    /// One line for the plan screen.
    pub fn describe(&self) -> String {
        match self {
            Self::Skill { dir, .. } => format!("skill     {}", tilde(dir)),
            Self::Rules { file, kit, .. } => {
                format!("rules     {}  (block kit:{kit})", tilde(file))
            }
            Self::McpJson { file, name, .. } => format!("mcp       {name} → {}", tilde(file)),
            Self::McpToml { file, name, .. } => format!("mcp       {name} → {}", tilde(file)),
            Self::ClaudeMcp { name, .. } => {
                format!("mcp       {name} → claude mcp add-json --scope user")
            }
            Self::HookJson { file, event, .. } => format!("hook      {event} → {}", tilde(file)),
            Self::Skip { piece, why } => format!("skipped   {piece}: {why}"),
        }
    }
}

impl Action {
    /// The file or folder this action writes, if any.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Skill { dir, .. } => Some(dir),
            Self::Rules { file, .. }
            | Self::McpJson { file, .. }
            | Self::McpToml { file, .. }
            | Self::HookJson { file, .. } => Some(file),
            Self::ClaudeMcp { .. } | Self::Skip { .. } => None,
        }
    }

    /// The program an MCP action makes the agent start, as a command line
    /// (`npx -y pkg@1.2.3`). `None` for anything else, or a remote server.
    pub fn command(&self) -> Option<String> {
        let (cmd, args): (&str, Vec<String>) = match self {
            Self::McpJson { value, .. } | Self::ClaudeMcp { value, .. } => (
                value.get("command")?.as_str()?,
                value
                    .get("args")
                    .and_then(|a| a.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|a| a.as_str().map(str::to_string))
                    .collect(),
            ),
            Self::McpToml { value, .. } => (
                value.get("command")?.as_str()?,
                value
                    .get("args")
                    .and_then(|a| a.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|a| a.as_str().map(str::to_string))
                    .collect(),
            ),
            _ => return None,
        };
        Some(
            std::iter::once(cmd.to_string())
                .chain(args)
                .map(|a| shell_word(&a))
                .collect::<Vec<_>>()
                .join(" "),
        )
    }

    /// Identity of the thing this action changes. Two kits that change the
    /// same thing share it; it is undone only when neither needs it.
    pub fn key(&self) -> String {
        match self {
            Self::Skill { dir, .. } => format!("skill {}", key_path(dir)),
            Self::Rules { file, kit, .. } => format!("rules {} {kit}", key_path(file)),
            Self::McpJson { file, name, .. } | Self::McpToml { file, name, .. } => {
                format!("mcp {} {name}", key_path(file))
            }
            Self::ClaudeMcp { name, .. } => format!("claude-mcp user {name}"),
            Self::HookJson { file, event, entry } => {
                format!("hook {} {event} {entry}", key_path(file))
            }
            Self::Skip { piece, .. } => format!("skip {piece}"),
        }
    }

    /// Runs code on the user's machine (a local MCP server or a hook).
    pub fn runs_code(&self) -> bool {
        match self {
            Self::McpJson { value, .. } => value.get("command").is_some(),
            Self::McpToml { value, .. } => value.contains_key("command"),
            Self::ClaudeMcp { value, .. } => value.get("command").is_some(),
            Self::HookJson { .. } => true,
            _ => false,
        }
    }
}

/// What was done, with enough to undo it exactly. Stored in the lock file.
///
/// Records are data, never commands: undoing one runs nothing but Kit's
/// own code, plus `claude mcp remove --scope user <name>` built here from a
/// validated name. Paths are stored relative to the scope's root (see
/// `lock.rs`), so a record can only ever point inside it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Applied {
    Skill {
        dir: PathBuf,
        hash: String,
    },
    Rules {
        file: PathBuf,
        kit: String,
        /// Kit created the file, so remove deletes it once empty.
        created: bool,
    },
    McpJson {
        file: PathBuf,
        name: String,
        created: bool,
        /// The user's own server of that name, replaced with `--force`;
        /// remove puts it back.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        previous: Option<serde_json::Value>,
        /// The file's text before Kit changed it, so remove can put back
        /// exactly those bytes. Never copied into the repo's kit.lock.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        original: Option<String>,
    },
    McpToml {
        file: PathBuf,
        name: String,
        created: bool,
        /// As for `McpJson`, the table's TOML text.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        previous: Option<String>,
    },
    ClaudeMcp {
        name: String,
        /// The server as added, so an upgrade can tell it changed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<serde_json::Value>,
    },
    HookJson {
        file: PathBuf,
        event: String,
        entry: serde_json::Value,
        created: bool,
        /// As for `McpJson`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        original: Option<String>,
    },
}

impl Applied {
    /// Drop the copy of the user's file text (for the repo's shared kit.lock).
    pub fn forget_original(&mut self) {
        if let Self::McpJson { original, .. } | Self::HookJson { original, .. } = self {
            *original = None;
        }
    }

    /// One line for the plan screen.
    pub fn describe(&self) -> String {
        match self {
            Self::Skill { dir, .. } => format!("skill {}", tilde(dir)),
            Self::Rules { file, kit, .. } => format!("rules {} (block kit:{kit})", tilde(file)),
            Self::McpJson { file, name, .. } | Self::McpToml { file, name, .. } => {
                format!("mcp {name} from {}", tilde(file))
            }
            Self::ClaudeMcp { name, .. } => format!("mcp {name} from Claude Code (user)"),
            Self::HookJson { file, event, .. } => format!("hook {event} from {}", tilde(file)),
        }
    }

    /// The file or folder this record changed, if any.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Skill { dir, .. } => Some(dir),
            Self::Rules { file, .. }
            | Self::McpJson { file, .. }
            | Self::McpToml { file, .. }
            | Self::HookJson { file, .. } => Some(file),
            Self::ClaudeMcp { .. } => None,
        }
    }

    pub fn path_mut(&mut self) -> Option<&mut PathBuf> {
        match self {
            Self::Skill { dir, .. } => Some(dir),
            Self::Rules { file, .. }
            | Self::McpJson { file, .. }
            | Self::McpToml { file, .. }
            | Self::HookJson { file, .. } => Some(file),
            Self::ClaudeMcp { .. } => None,
        }
    }

    /// Same identity as [`Action::key`].
    pub fn key(&self) -> String {
        match self {
            Self::Skill { dir, .. } => format!("skill {}", key_path(dir)),
            Self::Rules { file, kit, .. } => format!("rules {} {kit}", key_path(file)),
            Self::McpJson { file, name, .. } | Self::McpToml { file, name, .. } => {
                format!("mcp {} {name}", key_path(file))
            }
            Self::ClaudeMcp { name, .. } => format!("claude-mcp user {name}"),
            Self::HookJson {
                file, event, entry, ..
            } => format!("hook {} {event} {entry}", key_path(file)),
        }
    }
}

/// Is what `action` wants already exactly what `record` installed, and
/// still on disk? An upgrade or a changed server is not.
pub fn in_place(action: &Action, record: &Applied) -> bool {
    match (action, record) {
        (Action::Skill { payload, .. }, Applied::Skill { hash, .. }) => *hash == payload.hash,
        (
            Action::Rules {
                file,
                kit,
                version,
                text,
            },
            Applied::Rules { .. },
        ) => {
            let block = set_block("", kit, version, text);
            read_or_empty(file).is_ok_and(|t| t.contains(block.trim_end()))
        }
        (Action::McpJson { file, name, value }, Applied::McpJson { .. }) => read_json(file)
            .is_ok_and(|d| d.get("mcpServers").and_then(|s| s.get(name)) == Some(value)),
        (Action::McpToml { file, name, value }, Applied::McpToml { .. }) => read_or_empty(file)
            .ok()
            .and_then(|t| t.parse::<toml_edit::DocumentMut>().ok())
            .and_then(|d| {
                d.get("mcp_servers")
                    .and_then(|s| s.get(name))
                    .and_then(|i| i.as_table().map(ToString::to_string))
            })
            .is_some_and(|t| t == value.to_string()),
        (Action::ClaudeMcp { value, .. }, Applied::ClaudeMcp { value: had, .. }) => {
            had.as_ref().is_none_or(|h| h == value)
        }
        (Action::HookJson { file, event, entry }, Applied::HookJson { .. }) => read_json(file)
            .is_ok_and(|d| {
                d["hooks"][event]
                    .as_array()
                    .is_some_and(|l| l.contains(entry))
            }),
        (Action::Skip { .. }, _) => true,
        _ => false,
    }
}

/// Two kits asking for the same key want the same thing.
pub fn same_content(a: &Action, b: &Action) -> bool {
    match (a, b) {
        (Action::Skill { payload: x, .. }, Action::Skill { payload: y, .. }) => x.hash == y.hash,
        (Action::McpJson { value: x, .. }, Action::McpJson { value: y, .. })
        | (Action::ClaudeMcp { value: x, .. }, Action::ClaudeMcp { value: y, .. }) => x == y,
        (Action::McpToml { value: x, .. }, Action::McpToml { value: y, .. }) => {
            x.to_string() == y.to_string()
        }
        _ => true,
    }
}

/// A record replaced by a newer apply keeps what only the first knew:
/// that Kit created the file, and the user's server it replaced.
pub fn merge(old: &Applied, new: Applied) -> Applied {
    match (old, new) {
        (Applied::Rules { created: c, .. }, Applied::Rules { file, kit, created }) => {
            Applied::Rules {
                file,
                kit,
                created: created || *c,
            }
        }
        (
            Applied::McpJson {
                created: c,
                previous: p,
                original: o,
                ..
            },
            Applied::McpJson {
                file,
                name,
                created,
                previous,
                original,
            },
        ) => Applied::McpJson {
            file,
            name,
            created: created || *c,
            previous: previous.or_else(|| p.clone()),
            original: o.clone().or(original),
        },
        (
            Applied::McpToml {
                created: c,
                previous: p,
                ..
            },
            Applied::McpToml {
                file,
                name,
                created,
                previous,
            },
        ) => Applied::McpToml {
            file,
            name,
            created: created || *c,
            previous: previous.or_else(|| p.clone()),
        },
        (
            Applied::HookJson {
                created: c,
                original: o,
                ..
            },
            Applied::HookJson {
                file,
                event,
                entry,
                created,
                original,
            },
        ) => Applied::HookJson {
            file,
            event,
            entry,
            created: created || *c,
            original: o.clone().or(original),
        },
        (_, new) => new,
    }
}

/// Apply every action in order. The result lines up with `actions`:
/// `None` where nothing needed doing (the user already had exactly that).
/// `ours` holds the keys of changes Kit already made (an upgrade may
/// replace those). On a failure every file and folder touched is put back
/// byte for byte, so a failed `kit add` really changes nothing.
pub fn apply_all(
    actions: &[Action],
    force: bool,
    ours: &std::collections::HashSet<String>,
) -> Result<Vec<Option<Applied>>> {
    let mut saved: Vec<Saved> = Vec::new();
    let mut done = Vec::new();
    let mut ours = ours.clone();
    for action in actions {
        let key = action.key();
        let result = save(action).and_then(|s| {
            saved.push(s);
            apply(action, force, ours.contains(&key))
        });
        ours.insert(key);
        match result {
            Ok(a) => done.push(a),
            Err(err) => {
                for (i, s) in saved.iter().enumerate().rev() {
                    s.restore(done.get(i).and_then(Option::as_ref));
                }
                return Err(err.context("nothing was changed"));
            }
        }
    }
    for s in saved {
        s.discard();
    }
    Ok(done)
}

/// What a path held before an action touched it.
enum Saved {
    File {
        path: PathBuf,
        bytes: Option<Vec<u8>>,
    },
    Dir {
        path: PathBuf,
        copy: Option<PathBuf>,
    },
    Nothing,
}

fn save(action: &Action) -> Result<Saved> {
    Ok(match action {
        Action::Skill { dir, .. } => {
            let copy = if dir.exists() {
                let copy = dir.with_file_name(format!(
                    ".{}.kit-rollback-{}",
                    dir.file_name().unwrap_or_default().to_string_lossy(),
                    std::process::id()
                ));
                let _ = std::fs::remove_dir_all(&copy);
                copy_dir(dir, &copy)?;
                Some(copy)
            } else {
                None
            };
            Saved::Dir {
                path: dir.clone(),
                copy,
            }
        }
        Action::ClaudeMcp { .. } | Action::Skip { .. } => Saved::Nothing,
        other => {
            let path = other
                .path()
                .expect("file actions have a path")
                .to_path_buf();
            let bytes = match std::fs::read(&path) {
                Ok(b) => Some(b),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e).with_context(|| format!("cannot read {}", path.display())),
            };
            Saved::File { path, bytes }
        }
    })
}

impl Saved {
    /// Put the path back as it was. `applied` undoes what cannot be saved
    /// as bytes (a server added through `claude mcp`).
    fn restore(&self, applied: Option<&Applied>) {
        match self {
            Self::File { path, bytes } => match bytes {
                Some(b) => {
                    let _ = write_file(path, b);
                }
                None => {
                    let _ = std::fs::remove_file(path);
                    prune(path, 2);
                }
            },
            Self::Dir { path, copy } => {
                let _ = std::fs::remove_dir_all(path);
                match copy {
                    Some(c) => {
                        let _ = std::fs::rename(c, path);
                    }
                    None => prune(path, 2),
                }
            }
            Self::Nothing => {
                if let Some(a @ Applied::ClaudeMcp { .. }) = applied {
                    let _ = undo(a, true);
                }
            }
        }
    }

    fn discard(self) {
        if let Self::Dir { copy: Some(c), .. } = self {
            let _ = std::fs::remove_dir_all(c);
        }
    }
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if entry.file_type()?.is_dir() {
            copy_dir(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}

fn apply(action: &Action, force: bool, ours: bool) -> Result<Option<Applied>> {
    Ok(Some(match action {
        Action::Skill { dir, payload } => {
            write_skill(dir, payload, force)?;
            Applied::Skill {
                dir: dir.clone(),
                hash: payload.hash.clone(),
            }
        }
        Action::Rules {
            file,
            kit,
            version,
            text,
        } => {
            let created = !file.exists();
            let old = read_or_empty(file)?;
            write(file, &set_block(&old, kit, version, text))?;
            Applied::Rules {
                file: file.clone(),
                kit: kit.clone(),
                created,
            }
        }
        Action::McpJson { file, name, value } => {
            let created = !file.exists();
            let original = (!created).then(|| read_or_empty(file)).transpose()?;
            let mut doc = read_json(file)?;
            let servers = object_at(&mut doc, "mcpServers", file)?;
            let existing = servers.get(name).cloned();
            let previous = match existing {
                // The user already has exactly this: it stays theirs.
                Some(v) if v == *value && !ours => return Ok(None),
                Some(_) if ours => None,
                Some(v) if force => Some(v),
                Some(_) => bail!(
                    "{} already has an MCP server '{name}' that Kit did not write. Rename or remove it, or use --force",
                    file.display()
                ),
                None => None,
            };
            servers.insert(name.clone(), value.clone());
            write_json(file, &doc)?;
            Applied::McpJson {
                file: file.clone(),
                name: name.clone(),
                created,
                previous,
                original,
            }
        }
        Action::McpToml { file, name, value } => {
            let created = !file.exists();
            let raw = read_or_empty(file)?;
            let mut doc: toml_edit::DocumentMut = raw
                .parse()
                .with_context(|| format!("{} is not valid TOML", file.display()))?;
            let servers = doc
                .entry("mcp_servers")
                .or_insert(toml_edit::table())
                .as_table_mut()
                .with_context(|| format!("{}: mcp_servers is not a table", file.display()))?;
            servers.set_implicit(true);
            let existing = servers.get(name).map(|i| {
                i.as_table()
                    .map_or_else(|| i.to_string(), ToString::to_string)
            });
            let previous = match existing {
                Some(t) if t == value.to_string() && !ours => return Ok(None),
                Some(_) if ours => None,
                Some(t) if force => Some(t),
                Some(_) => bail!(
                    "{} already has [mcp_servers.{name}] that Kit did not write. Rename or remove it, or use --force",
                    file.display()
                ),
                None => None,
            };
            servers.insert(name, toml_edit::Item::Table(value.clone()));
            write(file, &doc.to_string())?;
            Applied::McpToml {
                file: file.clone(),
                name: name.clone(),
                created,
                previous,
            }
        }
        Action::ClaudeMcp { name, value } => {
            if ours {
                // An upgrade: Claude Code will not add over its own entry.
                run_argv(&claude_mcp_argv("remove", name, None)?)?;
            }
            run_argv(&claude_mcp_argv("add-json", name, Some(value))?)?;
            Applied::ClaudeMcp {
                name: name.clone(),
                value: Some(value.clone()),
            }
        }
        Action::HookJson { file, event, entry } => {
            let created = !file.exists();
            let original = (!created).then(|| read_or_empty(file)).transpose()?;
            let mut doc = read_json(file)?;
            let hooks = object_at(&mut doc, "hooks", file)?;
            let list = hooks
                .entry(event.clone())
                .or_insert_with(|| serde_json::Value::Array(Vec::new()))
                .as_array_mut()
                .with_context(|| format!("{}: hooks.{event} is not a list", file.display()))?;
            if list.contains(entry) {
                if !ours {
                    return Ok(None); // already there, and not Kit's to take out
                }
            } else {
                list.push(entry.clone());
            }
            write_json(file, &doc)?;
            Applied::HookJson {
                file: file.clone(),
                event: event.clone(),
                entry: entry.clone(),
                created,
                original,
            }
        }
        Action::Skip { .. } => return Ok(None),
    }))
}

/// Undo one applied change. Without `force`, a skill folder edited since
/// install is left in place and reported.
pub fn undo(applied: &Applied, force: bool) -> Result<Option<String>> {
    match applied {
        Applied::Skill { dir, hash } => {
            if !dir.exists() {
                return Ok(None);
            }
            if !force && disk_hash(dir)?.as_deref() != Some(hash.as_str()) {
                // It is the user's now: a later `kit add` must not treat it
                // as Kit's and overwrite it.
                let _ = std::fs::remove_file(dir.join(OWNED));
                return Ok(Some(format!(
                    "{} was changed after install; left in place (use --force to remove it)",
                    tilde(dir)
                )));
            }
            std::fs::remove_dir_all(dir)
                .with_context(|| format!("cannot remove {}", dir.display()))?;
            prune(dir, 2);
        }
        Applied::Rules { file, kit, created } => {
            if file.exists() {
                let text = remove_block(&read_or_empty(file)?, kit);
                if *created && text.trim().is_empty() {
                    std::fs::remove_file(file)?;
                    prune(file, 1);
                } else {
                    write(file, &text)?;
                }
            }
        }
        Applied::McpJson {
            file,
            name,
            created,
            previous,
            original,
        } => {
            if file.exists() {
                let mut doc = read_json(file)?;
                if let Some(servers) = doc.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                    match previous {
                        Some(v) => servers.insert(name.clone(), v.clone()),
                        None => servers.shift_remove(name),
                    };
                }
                finish_json(file, &doc, *created, "mcpServers", original.as_deref())?;
            }
        }
        Applied::McpToml {
            file,
            name,
            created,
            previous,
        } => {
            if file.exists() {
                let mut doc: toml_edit::DocumentMut = read_or_empty(file)?.parse()?;
                let empty = match doc.get_mut("mcp_servers").and_then(|t| t.as_table_mut()) {
                    Some(servers) => {
                        match previous
                            .as_deref()
                            .map(str::parse::<toml_edit::DocumentMut>)
                        {
                            Some(Ok(prev)) => {
                                servers
                                    .insert(name, toml_edit::Item::Table(prev.as_table().clone()));
                            }
                            _ => {
                                servers.remove(name);
                            }
                        }
                        servers.is_empty()
                    }
                    None => true,
                };
                if empty {
                    doc.remove("mcp_servers");
                }
                if *created && doc.to_string().trim().is_empty() {
                    std::fs::remove_file(file)?;
                    prune(file, 1);
                } else {
                    write(file, &doc.to_string())?;
                }
            }
        }
        Applied::ClaudeMcp { name, .. } => {
            run_argv(&claude_mcp_argv("remove", name, None)?)?;
        }
        Applied::HookJson {
            file,
            event,
            entry,
            created,
            original,
        } => {
            if file.exists() {
                let mut doc = read_json(file)?;
                if let Some(hooks) = doc.get_mut("hooks").and_then(|v| v.as_object_mut())
                    && let Some(list) = hooks.get_mut(event).and_then(|v| v.as_array_mut())
                {
                    list.retain(|e| e != entry);
                    if list.is_empty() {
                        hooks.shift_remove(event);
                    }
                }
                finish_json(file, &doc, *created, "hooks", original.as_deref())?;
            }
        }
    }
    Ok(None)
}

/// Is what Kit installed still as installed? `None` when it is gone.
pub fn drifted(applied: &Applied) -> Result<Option<String>> {
    if let Applied::Skill { dir, hash } = applied {
        return Ok(match disk_hash(dir)? {
            None => Some(format!("{} is missing", tilde(dir))),
            Some(h) if &h != hash => Some(format!("{} was changed by hand", tilde(dir))),
            Some(_) => None,
        });
    }
    Ok(None)
}

/// Does `path` resolve inside `base`, following any symlinks on the way?
/// A repo can make `.claude` a link to somewhere else; Kit then neither
/// writes through it nor deletes through it.
pub fn inside(path: &Path, base: &Path) -> bool {
    let Ok(base) = std::fs::canonicalize(base) else {
        return false;
    };
    // Kit never writes a link itself, so one in the final place is foreign.
    if std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        return false;
    }
    // The deepest part that exists decides where the rest would land.
    path.ancestors()
        .find(|p| p.exists())
        .and_then(|p| std::fs::canonicalize(p).ok())
        .is_some_and(|real| real.starts_with(&base))
}

/// Remove up to `levels` parent folders of `path` while they are empty
/// (`.claude/skills`, then `.claude`). A folder with anything in it stays.
fn prune(path: &Path, levels: usize) {
    for dir in path.ancestors().skip(1).take(levels) {
        if std::fs::remove_dir(dir).is_err() {
            break;
        }
    }
}

// ---- skills -------------------------------------------------------------

fn write_skill(dir: &Path, payload: &SkillPayload, force: bool) -> Result<()> {
    if dir.exists() {
        let owned = dir.join(OWNED).is_file();
        if !owned && !force {
            bail!(
                "{} exists and was not written by Kit. Move it, or use --force",
                tilde(dir)
            );
        }
        let on_disk = disk_hash(dir)?;
        if on_disk.as_deref() == Some(payload.hash.as_str()) {
            return Ok(());
        }
        // The marker holds the hash Kit wrote. A folder that no longer
        // matches it was edited by hand: never replace that silently.
        if owned && !force {
            let wrote = std::fs::read_to_string(dir.join(OWNED)).unwrap_or_default();
            if on_disk.as_deref() != Some(wrote.trim()) {
                bail!(
                    "{} was changed by hand since Kit installed it. Copy your changes, or use --force",
                    tilde(dir)
                );
            }
        }
        std::fs::remove_dir_all(dir)?;
    }
    for f in &payload.files {
        let path = dir.join(&f.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_file(&path, &f.bytes)?;
        #[cfg(unix)]
        if f.executable {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        }
    }
    write_file(&dir.join(OWNED), format!("{}\n", payload.hash).as_bytes())?;
    Ok(())
}

/// Hash of a skill folder on disk, skipping the marker. `None` if absent.
fn disk_hash(dir: &Path) -> Result<Option<String>> {
    if !dir.is_dir() {
        return Ok(None);
    }
    let mut files = Vec::new();
    collect(dir, dir, &mut files)?;
    Ok(Some(super::fetch::content_hash(&files)))
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<super::fetch::SkillFile>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(root, &path, out)?;
        } else if path.file_name().is_some_and(|n| n != OWNED) {
            out.push(super::fetch::SkillFile {
                path: path.strip_prefix(root).unwrap_or(&path).to_path_buf(),
                bytes: std::fs::read(&path)?,
                executable: false,
            });
        }
    }
    Ok(())
}

// ---- rules blocks -------------------------------------------------------

fn open_marker(kit: &str) -> String {
    format!("<!-- kit:{kit} ")
}

fn close_marker(kit: &str) -> String {
    format!("<!-- /kit:{kit} -->")
}

/// `text` with this kit's block added, or replaced if present.
pub fn set_block(text: &str, kit: &str, version: &str, body: &str) -> String {
    let block = format!(
        "{}{version} (managed by kit; edits inside are replaced) -->\n{}\n{}\n",
        open_marker(kit),
        body.trim_end(),
        close_marker(kit)
    );
    let stripped = remove_block(text, kit);
    let base = stripped.trim_end();
    if base.is_empty() {
        block
    } else {
        format!("{base}\n\n{block}")
    }
}

/// `text` without this kit's block. Text outside the markers is untouched.
pub fn remove_block(text: &str, kit: &str) -> String {
    let (open, close) = (open_marker(kit), close_marker(kit));
    let Some(start) = text.find(&open) else {
        return text.to_string();
    };
    let Some(rel_end) = text[start..].find(&close) else {
        return text.to_string();
    };
    let mut end = start + rel_end + close.len();
    if text[end..].starts_with('\n') {
        end += 1;
    }
    let before = text[..start].trim_end_matches('\n');
    let after = text[end..].trim_start_matches('\n');
    match (before.is_empty(), after.is_empty()) {
        (true, _) => after.to_string(),
        (false, true) => format!("{before}\n"),
        (false, false) => format!("{before}\n\n{after}"),
    }
}

// ---- files --------------------------------------------------------------

fn read_or_empty(file: &Path) -> Result<String> {
    match std::fs::read_to_string(file) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e).with_context(|| format!("cannot read {}", file.display())),
    }
}

fn write(file: &Path, text: &str) -> Result<()> {
    write_file(file, text.as_bytes())
}

/// Write `bytes` to `file` without ever following a link: the file must not
/// be a link, the data goes to a new temp file opened with `create_new`
/// (O_EXCL) under a fresh name in the same folder, and it is renamed over
/// the file only after checking again. A planted link, at the file or at a
/// guessable temp name, cannot redirect the write elsewhere.
pub fn write_file(file: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write as _;
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let is_link = |p: &Path| std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink());
    if is_link(file) {
        bail!(
            "{} is a link. Kit will not write through it",
            file.display()
        );
    }
    let parent = file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    let tmp = parent.join(format!(
        ".{}.kit-{}-{nanos:x}-{}.tmp",
        file.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let written = (|| -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()
    })();
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        return Err(e).with_context(|| format!("cannot write {}", file.display()));
    }
    if is_link(file) {
        let _ = std::fs::remove_file(&tmp);
        bail!(
            "{} became a link. Kit will not write through it",
            file.display()
        );
    }
    std::fs::rename(&tmp, file).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        anyhow::Error::new(e).context(format!("cannot write {}", file.display()))
    })
}

fn read_json(file: &Path) -> Result<serde_json::Value> {
    let raw = read_or_empty(file)?;
    if raw.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(&raw).with_context(|| {
        format!(
            "{} is not valid JSON, so Kit will not edit it. Fix it and run again",
            file.display()
        )
    })
}

/// Write `doc` laid out the way the file already is: compact stays
/// compact, an indent of 4 or a tab is kept, and so are CRLF line ends
/// and the final newline. Keys keep their order (`preserve_order`).
fn write_json(file: &Path, doc: &serde_json::Value) -> Result<()> {
    write(file, &render_json(doc, &read_or_empty(file)?)?)
}

fn render_json(doc: &serde_json::Value, like: &str) -> Result<String> {
    use serde::Serialize;
    let body = like.trim_end();
    let mut text = if body.is_empty() || body.contains('\n') {
        let indent = body
            .lines()
            .skip(1)
            .map(|l| &l[..l.len() - l.trim_start_matches([' ', '\t']).len()])
            .find(|w| !w.is_empty())
            .unwrap_or("  ");
        let mut out = Vec::new();
        let fmt = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
        doc.serialize(&mut serde_json::Serializer::with_formatter(&mut out, fmt))?;
        String::from_utf8(out)?
    } else {
        serde_json::to_string(doc)?
    };
    if like.is_empty() || like.ends_with('\n') {
        text.push('\n');
    }
    if like.contains("\r\n") {
        text = text.replace('\n', "\r\n");
    }
    Ok(text)
}

/// Write `doc` back, deleting the file if Kit created it and nothing else is
/// left. When what is left is what the file held before Kit, its original
/// bytes go back exactly.
fn finish_json(
    file: &Path,
    doc: &serde_json::Value,
    created: bool,
    key: &str,
    original: Option<&str>,
) -> Result<()> {
    let before = original.and_then(|t| serde_json::from_str::<serde_json::Value>(t).ok());
    if let (Some(text), Some(v)) = (original, &before)
        && v == doc
    {
        return write(file, text);
    }
    let mut doc = doc.clone();
    if let Some(obj) = doc.as_object_mut()
        && obj
            .get(key)
            .and_then(|v| v.as_object())
            .is_some_and(|o| o.is_empty())
    {
        obj.shift_remove(key);
    }
    if let (Some(text), Some(v)) = (original, &before)
        && *v == doc
    {
        return write(file, text);
    }
    if created && doc.as_object().is_some_and(|o| o.is_empty()) {
        std::fs::remove_file(file)?;
        prune(file, 1);
        Ok(())
    } else {
        write_json(file, &doc)
    }
}

fn object_at<'a>(
    doc: &'a mut serde_json::Value,
    key: &str,
    file: &Path,
) -> Result<&'a mut serde_json::Map<String, serde_json::Value>> {
    let root = doc
        .as_object_mut()
        .with_context(|| format!("{} is not a JSON object", file.display()))?;
    root.entry(key)
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .with_context(|| format!("{}: {key} is not an object", file.display()))
}

/// `claude mcp <verb> --scope user <name> [json]`. The name must be a
/// plain name, so it can never be read as a flag (`--scope=project`).
fn claude_mcp_argv(
    verb: &str,
    name: &str,
    json: Option<&serde_json::Value>,
) -> Result<Vec<String>> {
    if !super::manifest::is_slug(name) {
        bail!("'{name}' is not a valid MCP server name");
    }
    let mut argv: Vec<String> = ["claude", "mcp", verb, "--scope", "user", name]
        .map(String::from)
        .to_vec();
    argv.extend(json.map(ToString::to_string));
    Ok(argv)
}

/// Run an agent CLI directly, never through a shell, so kit-supplied
/// arguments (JSON, names) cannot be reinterpreted. On Windows the npm
/// `.cmd` shim is found on PATH and std escapes its arguments.
fn run_argv(argv: &[String]) -> Result<()> {
    let (program, args) = argv.split_first().context("empty command")?;
    let exe = find_program(program)
        .with_context(|| format!("{program} is not on PATH, so Kit cannot run `{program}`"))?;
    let out = std::process::Command::new(exe)
        .args(args)
        .stdin(std::process::Stdio::null())
        .output()
        .with_context(|| format!("cannot run {program}"))?;
    if !out.status.success() {
        bail!(
            "`{}` failed: {}",
            argv.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

/// Full path of `program` on PATH, trying Windows executable extensions.
pub fn find_program(program: &str) -> Option<PathBuf> {
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT".into())
            .split(';')
            .map(str::to_ascii_lowercase)
            .collect()
    } else {
        vec![String::new()]
    };
    std::env::split_paths(&std::env::var_os("PATH")?).find_map(|dir| {
        exts.iter()
            .map(|ext| dir.join(format!("{program}{ext}")))
            .find(|p| p.is_file())
    })
}

/// A path as it appears in a key: one separator, so a path rebuilt from a
/// record (`root` + `.claude/skills/x`) matches the one a writer made
/// (`root/.claude/skills` + `x`) on Windows too.
fn key_path(p: &Path) -> String {
    p.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// One argument as it would be typed: quoted when it has spaces or quotes.
pub fn shell_word(a: &str) -> String {
    if !a.is_empty()
        && a.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./@=:,+%^~".contains(c))
    {
        a.to_string()
    } else {
        format!("'{}'", a.replace('\'', "'\\''"))
    }
}

/// For display: relative inside the current folder, `~/…` under home.
pub fn tilde(path: &Path) -> String {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    if let Ok(cwd) = std::env::current_dir()
        && !home
            .as_ref()
            .is_some_and(|h| Path::new(h).starts_with(&cwd))
        && let Ok(rest) = path.strip_prefix(&cwd)
        && !rest.as_os_str().is_empty()
    {
        return rest.display().to_string();
    }
    if let Some(home) = home
        && let Ok(rest) = path.strip_prefix(&home)
    {
        return format!("~/{}", rest.display());
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kits::fetch::SkillFile;

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("kit-plan-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn payload(body: &str) -> SkillPayload {
        let files = vec![SkillFile {
            path: "SKILL.md".into(),
            bytes: body.into(),
            executable: false,
        }];
        SkillPayload {
            name: "s".into(),
            hash: crate::kits::fetch::content_hash(&files),
            files,
        }
    }

    #[test]
    fn blocks_are_added_replaced_and_removed_without_touching_the_rest() {
        let mine = "# My rules\n\nBe nice.\n";
        let one = set_block(mine, "a", "0.1.0", "Rule A");
        assert!(one.starts_with(mine.trim_end()));
        assert!(one.contains("<!-- kit:a 0.1.0 "));
        let two = set_block(&one, "b", "0.1.0", "Rule B");
        let again = set_block(&two, "a", "0.2.0", "Rule A2");
        assert!(again.contains("Rule A2") && !again.contains("Rule A\n"));
        assert_eq!(again.matches("<!-- kit:a ").count(), 1);
        let back = remove_block(&remove_block(&again, "a"), "b");
        assert_eq!(back, mine);
        assert_eq!(remove_block(&set_block("", "a", "1", "x"), "a"), "");
    }

    #[test]
    fn skill_folders_round_trip_and_refuse_foreign_folders() {
        let d = scratch("skill");
        let dir = d.join("skills/s");
        let applied = apply_all(
            &[Action::Skill {
                dir: dir.clone(),
                payload: payload("v1"),
            }],
            false,
            &Default::default(),
        )
        .unwrap()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        assert!(dir.join(OWNED).is_file());
        assert_eq!(drifted(&applied[0]).unwrap(), None);
        std::fs::write(dir.join("SKILL.md"), "edited").unwrap();
        assert!(
            drifted(&applied[0])
                .unwrap()
                .unwrap()
                .contains("changed by hand")
        );
        let kept = undo(&applied[0], false).unwrap();
        assert!(kept.unwrap().contains("left in place"));
        undo(&applied[0], true).unwrap();
        assert!(!dir.exists());

        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("SKILL.md"), "mine").unwrap();
        let err = apply_all(
            &[Action::Skill {
                dir: dir.clone(),
                payload: payload("v1"),
            }],
            false,
            &Default::default(),
        )
        .unwrap_err();
        assert!(
            format!("{err:#}").contains("was not written by Kit"),
            "{err:#}"
        );
    }

    #[test]
    fn mcp_and_hooks_round_trip_leaving_user_config_alone() {
        let d = scratch("json");
        let mcp = d.join(".mcp.json");
        std::fs::write(&mcp, r#"{"mcpServers":{"mine":{"command":"x"}}}"#).unwrap();
        let settings = d.join("settings.json");
        let toml = d.join("config.toml");
        std::fs::write(&toml, "model = \"o3\" # keep me\n").unwrap();
        let entry = serde_json::json!({"matcher": "Edit", "hooks": []});
        let mut table = toml_edit::Table::new();
        table.insert("command", toml_edit::value("npx"));
        let applied = apply_all(
            &[
                Action::McpJson {
                    file: mcp.clone(),
                    name: "k".into(),
                    value: serde_json::json!({"command": "npx"}),
                },
                Action::HookJson {
                    file: settings.clone(),
                    event: "PostToolUse".into(),
                    entry: entry.clone(),
                },
                Action::McpToml {
                    file: toml.clone(),
                    name: "k".into(),
                    value: table,
                },
            ],
            false,
            &Default::default(),
        )
        .unwrap()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        assert!(
            std::fs::read_to_string(&toml)
                .unwrap()
                .contains("[mcp_servers.k]")
        );
        for a in applied.iter().rev() {
            undo(a, false).unwrap();
        }
        let left: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&mcp).unwrap()).unwrap();
        assert_eq!(
            left,
            serde_json::json!({"mcpServers": {"mine": {"command": "x"}}})
        );
        assert!(!settings.exists(), "Kit created it, and it is empty again");
        assert_eq!(
            std::fs::read_to_string(&toml).unwrap(),
            "model = \"o3\" # keep me\n"
        );
    }

    #[test]
    fn a_failed_apply_undoes_what_it_did() {
        let d = scratch("rollback");
        let rules = d.join("CLAUDE.md");
        let bad = d.join("bad.json");
        std::fs::write(&bad, "{ not json").unwrap();
        let err = apply_all(
            &[
                Action::Rules {
                    file: rules.clone(),
                    kit: "a".into(),
                    version: "1".into(),
                    text: "x".into(),
                },
                Action::McpJson {
                    file: bad,
                    name: "k".into(),
                    value: serde_json::json!({}),
                },
            ],
            false,
            &Default::default(),
        )
        .unwrap_err();
        assert!(
            format!("{err:#}").starts_with("nothing was changed"),
            "{err:#}"
        );
        assert!(!rules.exists());
    }
}
