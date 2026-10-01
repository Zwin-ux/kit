//! Native Claude Code entry. Separate from the headless worktree/gate adapter.
//! No prompts, authentication probes, settings edits, shell interpolation or model calls.

use crate::cli::ClaudeCommand;
use anyhow::{Context, Result, bail};
use include_dir::{Dir, include_dir};
use std::collections::BTreeMap;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;

const MIN_VERSION: (u32, u32, u32) = (2, 1, 287);
const MANIFEST: &str = include_str!("../claude-plugin/.claude-plugin/plugin.json");
static HOOKS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/claude-plugin/hooks");
static AGENTS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/claude-plugin/agents");
static SKILLS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/claude-plugin/skills");

fn binary() -> &'static str {
    if cfg!(windows) {
        "claude.exe"
    } else {
        "claude"
    }
}

fn supported(raw: &str) -> bool {
    let parts: Vec<_> = raw
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .split('.')
        .collect();
    if parts.len() != 3 {
        return false;
    }
    match (
        parts[0].parse::<u32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) {
        (Ok(major), Ok(minor), Ok(patch)) => (major, minor, patch) >= MIN_VERSION,
        _ => false,
    }
}

fn installed_version() -> Result<String> {
    let output = Command::new(binary()).arg("--version").output()
        .context("Claude Code is not on PATH. Install the native CLI from https://code.claude.com/docs/en/setup")?;
    let version = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || !supported(&version) {
        bail!(
            "native Kit Mods need Claude Code 2.1.287 or later; received {version:?}. See https://code.claude.com/docs/en/setup"
        );
    }
    Ok(version)
}

fn collect(dir: &Dir<'_>, prefix: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
    for file in dir.files() {
        files.insert(prefix.join(file.path()), file.contents().to_vec());
    }
    for child in dir.dirs() {
        collect(child, prefix, files);
    }
}

fn payload() -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::from([(
        PathBuf::from(".claude-plugin/plugin.json"),
        MANIFEST.as_bytes().to_vec(),
    )]);
    for (name, contents) in [
        ("README.md", include_str!("../claude-plugin/README.md")),
        ("LICENSE", include_str!("../claude-plugin/LICENSE")),
        (
            "provenance.json",
            include_str!("../claude-plugin/provenance.json"),
        ),
    ] {
        files.insert(PathBuf::from(name), contents.as_bytes().to_vec());
    }
    for (dir, prefix) in [(&HOOKS, "hooks"), (&AGENTS, "agents"), (&SKILLS, "skills")] {
        collect(dir, Path::new(prefix), &mut files);
    }
    files
}

fn refuse_links(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        if let Ok(meta) = std::fs::symlink_metadata(ancestor)
            && meta.file_type().is_symlink()
        {
            bail!(
                "{} is a link; choose a real local plugin directory",
                ancestor.display()
            );
        }
    }
    Ok(())
}

fn absolute(path: &Path) -> Result<PathBuf> {
    let path = std::path::absolute(path)?;
    refuse_links(&path)?;
    // A plugin directory is executable code. Network paths are not local installs.
    if path.to_string_lossy().starts_with("\\\\") {
        bail!("choose a local plugin directory, not a network path");
    }
    Ok(path)
}

fn install(path: &Path) -> Result<usize> {
    let path = absolute(path)?;
    if path.exists() {
        bail!(
            "{} already exists; choose a new directory to preserve its contents",
            path.display()
        );
    }
    let parent = path.parent().context("plugin directory needs a parent")?;
    std::fs::create_dir_all(parent)?;
    let stage = parent.join(format!(".kit-plugin-stage-{}", std::process::id()));
    // Only this newly created staging directory can be cleaned up below.
    std::fs::create_dir(&stage).context("cannot create a fresh plugin staging directory")?;
    let files = payload();
    let result = (|| -> Result<()> {
        for (relative, bytes) in &files {
            let file = stage.join(relative);
            std::fs::create_dir_all(file.parent().context("plugin file needs a parent")?)?;
            std::fs::write(file, bytes)?;
        }
        std::fs::rename(&stage, &path).context("cannot move the prepared plugin into place")?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&stage);
    }
    result?;
    Ok(files.len())
}

fn launch_args(path: &Path) -> Vec<std::ffi::OsString> {
    vec!["--plugin-dir".into(), path.as_os_str().to_owned()]
}

fn emit(json: bool, command: &str, data: serde_json::Value, text: &str) {
    if json {
        println!("{}", crate::json_envelope(command, true, data, None));
    } else {
        println!("{text}");
    }
}

pub fn run(action: ClaudeCommand, json: bool) -> Result<()> {
    match action {
        ClaudeCommand::Check => {
            let version = installed_version()?;
            emit(
                json,
                "claude.check",
                serde_json::json!({ "version": version, "mods": true, "loginChecked": false }),
                &format!("{version}: Mods supported. Login was not checked."),
            );
        }
        ClaudeCommand::Install { plugin_dir, print } => {
            let dir = absolute(&plugin_dir)?;
            let files: Vec<_> = payload()
                .keys()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
            if !print {
                install(&dir)?;
            }
            let verb = if print { "Would prepare" } else { "Prepared" };
            emit(
                json,
                "claude.install",
                serde_json::json!({ "directory": dir, "files": files, "written": !print, "enabled": false }),
                &format!(
                    "{verb} {} files in {}.\nLaunch explicitly: kit claude launch --plugin-dir \"{}\"",
                    files.len(),
                    dir.display(),
                    dir.display()
                ),
            );
        }
        ClaudeCommand::Launch { plugin_dir, print } => {
            let path = absolute(&plugin_dir)?;
            let manifest: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(path.join(".claude-plugin/plugin.json"))
                    .context("Kit plugin is missing; run `kit claude install` first")?,
            )?;
            if manifest.get("name").and_then(|n| n.as_str()) != Some("kit") {
                bail!("{} is not a Kit plugin", path.display());
            }
            if print {
                let args: Vec<_> = launch_args(&path)
                    .iter()
                    .map(|arg| arg.to_string_lossy().into_owned())
                    .collect();
                emit(
                    json,
                    "claude.launch",
                    serde_json::json!({ "program": binary(), "args": args, "launched": false }),
                    &format!("{} --plugin-dir \"{}\"", binary(), path.display()),
                );
                return Ok(());
            }
            if json || !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
                bail!(
                    "native Claude needs an interactive terminal; inspect with `kit claude launch --print`"
                );
            }
            installed_version()?;
            eprintln!(
                "kit: opening native Claude with Kit. Claude owns sign-in, prompts and permissions. Use /kit."
            );
            let status = Command::new(binary())
                .args(launch_args(&path))
                .status()
                .context("could not open native Claude")?;
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_version_is_checked_without_guessing() {
        for version in ["2.1.287 (Claude Code)", "2.2.0", "3.0.0"] {
            assert!(supported(version));
        }
        for version in ["2.1.286", "1.99.999", "2.1", "unknown", "2.1.287-canary"] {
            assert!(!supported(version));
        }
    }

    #[test]
    fn native_launch_has_no_prompt_or_permission_changes() {
        let path = Path::new("folder with spaces & symbols");
        assert_eq!(
            launch_args(path),
            vec![
                std::ffi::OsString::from("--plugin-dir"),
                path.as_os_str().to_owned()
            ]
        );
    }

    #[test]
    fn bundle_contains_declared_agents_and_knowledge() {
        let files = payload();
        assert_eq!(files.keys().filter(|p| p.starts_with("agents")).count(), 4);
        assert_eq!(files.keys().filter(|p| p.starts_with("skills")).count(), 8);
        assert!(files.contains_key(Path::new("agents/frontend-ui-builder.md")));
        assert!(!files.keys().any(|p| p.to_string_lossy().contains("types")));
        assert!(!files.keys().any(|p| p.to_string_lossy().contains(".mcp")));
        let manifest: serde_json::Value = serde_json::from_str(MANIFEST).unwrap();
        for agent in manifest["agents"].as_array().unwrap() {
            let path = agent.as_str().unwrap().trim_start_matches("./");
            assert!(
                files.contains_key(Path::new(path)),
                "missing declared agent {path}"
            );
        }
    }

    #[test]
    fn installation_preserves_existing_work_and_copies_the_bundle() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("kit-native-test-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let dest = root.join("plugin");
        install(&dest).unwrap();
        assert_eq!(
            std::fs::read_to_string(dest.join(".claude-plugin/plugin.json")).unwrap(),
            MANIFEST
        );
        std::fs::write(dest.join("notes.txt"), "user work").unwrap();
        assert!(install(&dest).is_err());
        assert_eq!(
            std::fs::read_to_string(dest.join("notes.txt")).unwrap(),
            "user work"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
