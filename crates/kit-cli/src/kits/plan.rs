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
    /// Run an agent's own CLI (`claude mcp add`), with the command that undoes it.
    Command {
        argv: Vec<String>,
        undo: Vec<String>,
        what: String,
        /// Adds something that runs code (a local MCP server).
        code: bool,
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
            Self::Command { what, .. } => format!("mcp       {what}"),
            Self::HookJson { file, event, .. } => format!("hook      {event} → {}", tilde(file)),
            Self::Skip { piece, why } => format!("skipped   {piece}: {why}"),
        }
    }
}

impl Action {
    /// Identity of the thing this action changes. Two kits that change the
    /// same thing share it; it is undone only when neither needs it.
    pub fn key(&self) -> String {
        match self {
            Self::Skill { dir, .. } => format!("skill {}", dir.display()),
            Self::Rules { file, kit, .. } => format!("rules {} {kit}", file.display()),
            Self::McpJson { file, name, .. } | Self::McpToml { file, name, .. } => {
                format!("mcp {} {name}", file.display())
            }
            Self::Command { undo, .. } => format!("command {}", undo.join(" ")),
            Self::HookJson { file, event, entry } => {
                format!("hook {} {event} {entry}", file.display())
            }
            Self::Skip { piece, .. } => format!("skip {piece}"),
        }
    }

    /// Runs code on the user's machine (a local MCP server or a hook).
    pub fn runs_code(&self) -> bool {
        match self {
            Self::McpJson { value, .. } => value.get("command").is_some(),
            Self::McpToml { value, .. } => value.contains_key("command"),
            Self::Command { code, .. } => *code,
            Self::HookJson { .. } => true,
            _ => false,
        }
    }
}

/// What was done, with enough to undo it exactly. Stored in the lock file.
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
    },
    McpToml {
        file: PathBuf,
        name: String,
        created: bool,
    },
    Command {
        undo: Vec<String>,
    },
    HookJson {
        file: PathBuf,
        event: String,
        entry: serde_json::Value,
        created: bool,
    },
}

impl Applied {
    /// Same identity as [`Action::key`].
    pub fn key(&self) -> String {
        match self {
            Self::Skill { dir, .. } => format!("skill {}", dir.display()),
            Self::Rules { file, kit, .. } => format!("rules {} {kit}", file.display()),
            Self::McpJson { file, name, .. } | Self::McpToml { file, name, .. } => {
                format!("mcp {} {name}", file.display())
            }
            Self::Command { undo } => format!("command {}", undo.join(" ")),
            Self::HookJson {
                file, event, entry, ..
            } => format!("hook {} {event} {entry}", file.display()),
        }
    }
}

/// Apply every action in order. On a failure, undo what was applied and
/// return the error, so a failed `kit add` changes nothing.
pub fn apply_all(actions: &[Action], force: bool) -> Result<Vec<Applied>> {
    let mut done = Vec::new();
    for action in actions {
        match apply(action, force) {
            Ok(Some(a)) => done.push(a),
            Ok(None) => {}
            Err(err) => {
                for a in done.iter().rev() {
                    let _ = undo(a, true);
                }
                return Err(err.context("nothing was changed"));
            }
        }
    }
    Ok(done)
}

fn apply(action: &Action, force: bool) -> Result<Option<Applied>> {
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
            let mut doc = read_json(file)?;
            let servers = object_at(&mut doc, "mcpServers", file)?;
            if let Some(existing) = servers.get(name)
                && existing != value
                && !force
            {
                bail!(
                    "{} already has an MCP server '{name}' that Kit did not write. Rename or remove it, or use --force",
                    file.display()
                );
            }
            servers.insert(name.clone(), value.clone());
            write_json(file, &doc)?;
            Applied::McpJson {
                file: file.clone(),
                name: name.clone(),
                created,
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
            let same = servers
                .get(name)
                .and_then(|i| i.as_table())
                .is_some_and(|t| t.to_string() == value.to_string());
            if servers.contains_key(name) && !same && !force {
                bail!(
                    "{} already has [mcp_servers.{name}] that Kit did not write. Rename or remove it, or use --force",
                    file.display()
                );
            }
            servers.insert(name, toml_edit::Item::Table(value.clone()));
            write(file, &doc.to_string())?;
            Applied::McpToml {
                file: file.clone(),
                name: name.clone(),
                created,
            }
        }
        Action::Command { argv, undo, .. } => {
            run_argv(argv)?;
            Applied::Command { undo: undo.clone() }
        }
        Action::HookJson { file, event, entry } => {
            let created = !file.exists();
            let mut doc = read_json(file)?;
            let hooks = object_at(&mut doc, "hooks", file)?;
            let list = hooks
                .entry(event.clone())
                .or_insert_with(|| serde_json::Value::Array(Vec::new()))
                .as_array_mut()
                .with_context(|| format!("{}: hooks.{event} is not a list", file.display()))?;
            if !list.contains(entry) {
                list.push(entry.clone());
            }
            write_json(file, &doc)?;
            Applied::HookJson {
                file: file.clone(),
                event: event.clone(),
                entry: entry.clone(),
                created,
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
        } => {
            if file.exists() {
                let mut doc = read_json(file)?;
                if let Some(servers) = doc.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                    servers.remove(name);
                }
                finish_json(file, &doc, *created, "mcpServers")?;
            }
        }
        Applied::McpToml {
            file,
            name,
            created,
        } => {
            if file.exists() {
                let mut doc: toml_edit::DocumentMut = read_or_empty(file)?.parse()?;
                let empty = match doc.get_mut("mcp_servers").and_then(|t| t.as_table_mut()) {
                    Some(servers) => {
                        servers.remove(name);
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
        Applied::Command { undo } => run_argv(undo)?,
        Applied::HookJson {
            file,
            event,
            entry,
            created,
        } => {
            if file.exists() {
                let mut doc = read_json(file)?;
                if let Some(hooks) = doc.get_mut("hooks").and_then(|v| v.as_object_mut())
                    && let Some(list) = hooks.get_mut(event).and_then(|v| v.as_array_mut())
                {
                    list.retain(|e| e != entry);
                    if list.is_empty() {
                        hooks.remove(event);
                    }
                }
                finish_json(file, &doc, *created, "hooks")?;
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
        if disk_hash(dir)?.as_deref() == Some(payload.hash.as_str()) {
            return Ok(());
        }
        std::fs::remove_dir_all(dir)?;
    }
    for f in &payload.files {
        let path = dir.join(&f.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &f.bytes)
            .with_context(|| format!("cannot write {}", path.display()))?;
        #[cfg(unix)]
        if f.executable {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
        }
    }
    std::fs::write(dir.join(OWNED), format!("{}\n", payload.hash))?;
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
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file, text).with_context(|| format!("cannot write {}", file.display()))
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

fn write_json(file: &Path, doc: &serde_json::Value) -> Result<()> {
    write(file, &format!("{}\n", serde_json::to_string_pretty(doc)?))
}

/// Write `doc` back, deleting the file if Kit created it and nothing else is left.
fn finish_json(file: &Path, doc: &serde_json::Value, created: bool, key: &str) -> Result<()> {
    let mut doc = doc.clone();
    if let Some(obj) = doc.as_object_mut()
        && obj
            .get(key)
            .and_then(|v| v.as_object())
            .is_some_and(|o| o.is_empty())
    {
        obj.remove(key);
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
        )
        .unwrap();
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
        )
        .unwrap();
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
        )
        .unwrap_err();
        assert!(
            format!("{err:#}").starts_with("nothing was changed"),
            "{err:#}"
        );
        assert!(!rules.exists());
    }
}
