//! Get a skill's files: from the kit itself, or from its upstream repo at
//! the pinned commit. Upstream repos are cached as bare git repos under
//! `~/.kit/cache/git/`, so a pinned commit downloads once.

use super::catalog::Kit;
use super::manifest::SkillRef;
use crate::engine::paths::kit_home;
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// One file of a skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillFile {
    /// Path inside the skill folder.
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub executable: bool,
}

/// A skill ready to write: its files and their content hash.
#[derive(Debug, Clone)]
pub struct SkillPayload {
    pub name: String,
    pub files: Vec<SkillFile>,
    /// `sha256:<hex>` over paths and contents; what kit.lock records.
    pub hash: String,
}

/// The files of `skill`, which belongs to `kit`.
pub fn skill_payload(kit: &Kit, skill: &SkillRef) -> Result<SkillPayload> {
    let files = match (&skill.source, &skill.rev) {
        (Some(source), Some(rev)) => upstream_files(source, rev, &skill.path)
            .with_context(|| format!("kit {}: skill {}", kit.name(), skill.name))?,
        _ => kit
            .files
            .files_under(&skill.path)?
            .into_iter()
            .map(|(path, bytes)| SkillFile {
                path,
                bytes,
                executable: false,
            })
            .collect(),
    };
    if !files.iter().any(|f| f.path == Path::new("SKILL.md")) {
        bail!(
            "kit {}: skill {} has no SKILL.md at {}",
            kit.name(),
            skill.name,
            skill.path
        );
    }
    Ok(SkillPayload {
        name: skill.name.clone(),
        hash: content_hash(&files),
        files,
    })
}

/// `sha256:` over each file's path and bytes, in path order.
pub fn content_hash(files: &[SkillFile]) -> String {
    let mut sorted: Vec<&SkillFile> = files.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    let mut h = Sha256::new();
    for f in sorted {
        h.update(f.path.to_string_lossy().replace('\\', "/").as_bytes());
        h.update([0]);
        h.update((f.bytes.len() as u64).to_le_bytes());
        h.update(&f.bytes);
    }
    let hex: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{hex}")
}

/// Where `github:owner/repo` is fetched from. Tests point this at a folder.
pub(crate) fn remote_url(repo: &str) -> String {
    let base = std::env::var("KIT_GIT_BASE").unwrap_or_else(|_| "https://github.com".into());
    format!("{}/{repo}", base.trim_end_matches('/'))
}

pub(crate) fn git(dir: &Path) -> Command {
    let mut c = Command::new("git");
    c.arg("-C").arg(dir);
    c.env("GIT_TERMINAL_PROMPT", "0");
    for var in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
    ] {
        c.env_remove(var);
    }
    c
}

pub(crate) fn run(cmd: &mut Command, what: &str) -> Result<Vec<u8>> {
    let out = cmd
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("cannot run git to {what}. Is git installed?"))?;
    if !out.status.success() {
        bail!(
            "git could not {what}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out.stdout)
}

/// The bare cache repo for `github:owner/repo`, with `rev` present.
pub(crate) fn cached_repo(source: &str, rev: &str) -> Result<PathBuf> {
    let repo = source
        .strip_prefix("github:")
        .context("only github: sources are supported")?;
    let dir = kit_home()
        .join("cache")
        .join("git")
        .join(format!("{repo}.git"));
    if !dir.join("HEAD").exists() {
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("cannot create {}", dir.display()))?;
        run(
            git(&dir).args(["init", "--bare", "-q"]),
            "create the skill cache",
        )?;
    }
    let have = git(&dir)
        .args(["cat-file", "-e", &format!("{rev}^{{commit}}")])
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !have {
        let url = remote_url(repo);
        run(
            git(&dir).args(["fetch", "-q", "--depth", "1", &url, rev]),
            &format!("download {repo} at {}", &rev[..rev.len().min(7)]),
        )?;
    }
    Ok(dir)
}

/// Files under `path` at `rev`, plus the repo's licence when the skill
/// folder has none of its own (a vendored skill keeps its licence text).
pub(crate) fn upstream_files(source: &str, rev: &str, path: &str) -> Result<Vec<SkillFile>> {
    let dir = cached_repo(source, rev)?;
    let prefix = format!("{}/", path.trim_end_matches('/'));
    let mut entries = ls_tree(&dir, rev, &[path])?;
    if entries.is_empty() {
        bail!("{source}@{} has no folder {path}", &rev[..7]);
    }
    for e in &mut entries {
        e.path = e.path.strip_prefix(&prefix).unwrap_or(&e.path).to_string();
    }
    let has_licence = entries.iter().any(|e| is_licence(&e.path));
    if !has_licence
        && let Some(root) = ls_tree(
            &dir,
            rev,
            &["LICENSE", "LICENSE.md", "LICENSE.txt", "COPYING"],
        )?
        .into_iter()
        .next()
    {
        entries.push(root);
    }
    let blobs = read_blobs(&dir, entries.iter().map(|e| e.sha.as_str()))?;
    Ok(entries
        .into_iter()
        .zip(blobs)
        .map(|(e, bytes)| SkillFile {
            path: PathBuf::from(e.path),
            bytes,
            executable: e.mode == "100755",
        })
        .collect())
}

fn is_licence(path: &str) -> bool {
    let upper = path.to_ascii_uppercase();
    !path.contains('/') && (upper.starts_with("LICENSE") || upper.starts_with("COPYING"))
}

struct TreeEntry {
    mode: String,
    sha: String,
    path: String,
}

/// Regular files (not symlinks or submodules) at `paths` in `rev`.
fn ls_tree(dir: &Path, rev: &str, paths: &[&str]) -> Result<Vec<TreeEntry>> {
    let out = run(
        git(dir)
            .args(["ls-tree", "-r", "-z", "--full-tree", rev, "--"])
            .args(paths),
        "list the skill's files",
    )?;
    let mut entries = Vec::new();
    for rec in out.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let rec = String::from_utf8_lossy(rec);
        let Some((meta, path)) = rec.split_once('\t') else {
            continue;
        };
        let mut parts = meta.split(' ');
        let (Some(mode), Some(kind), Some(sha)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        if kind == "blob" && (mode == "100644" || mode == "100755") {
            entries.push(TreeEntry {
                mode: mode.into(),
                sha: sha.into(),
                path: path.into(),
            });
        }
    }
    Ok(entries)
}

/// Contents of each blob, in order, through one `git cat-file --batch`.
fn read_blobs<'a>(dir: &Path, shas: impl Iterator<Item = &'a str>) -> Result<Vec<Vec<u8>>> {
    let mut child = git(dir)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("cannot run git cat-file")?;
    let shas: Vec<String> = shas.map(str::to_owned).collect();
    let mut stdin = child.stdin.take().context("git stdin")?;
    let writer = std::thread::spawn(move || -> std::io::Result<()> {
        for sha in shas {
            writeln!(stdin, "{sha}")?;
        }
        Ok(())
    });
    let mut reader = BufReader::new(child.stdout.take().context("git stdout")?);
    let mut out = Vec::new();
    let mut header = String::new();
    loop {
        header.clear();
        if reader.read_line(&mut header)? == 0 {
            break;
        }
        let size: usize = header
            .trim_end()
            .rsplit(' ')
            .next()
            .and_then(|s| s.parse().ok())
            .with_context(|| format!("unexpected git output: {}", header.trim_end()))?;
        let mut buf = vec![0; size];
        reader.read_exact(&mut buf)?;
        let mut nl = [0u8; 1];
        reader.read_exact(&mut nl)?;
        out.push(buf);
    }
    writer
        .join()
        .map_err(|_| anyhow::anyhow!("git writer panicked"))??;
    child.wait()?;
    Ok(out)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::kits::catalog;

    fn sh(dir: &Path, args: &[&str]) -> String {
        let out = git(dir)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// A local "github" with `owner/repo` holding one skill; returns
    /// (base dir for KIT_GIT_BASE, commit sha).
    pub(crate) fn fake_upstream(tag: &str) -> (PathBuf, String) {
        let base = std::env::temp_dir().join(format!("kit-upstream-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let repo = base.join("owner").join("repo");
        std::fs::create_dir_all(repo.join("skills/demo/scripts")).unwrap();
        std::fs::write(
            repo.join("skills/demo/SKILL.md"),
            "---\nname: demo\n---\nDo it.\n",
        )
        .unwrap();
        std::fs::write(repo.join("skills/demo/scripts/run.sh"), "echo hi\n").unwrap();
        std::fs::write(repo.join("LICENSE"), "MIT License\n").unwrap();
        std::fs::write(repo.join("README.md"), "not part of the skill\n").unwrap();
        sh(&repo, &["init", "-q"]);
        sh(&repo, &["add", "."]);
        sh(&repo, &["commit", "-q", "-m", "init"]);
        let rev = sh(&repo, &["rev-parse", "HEAD"]);
        (base, rev)
    }

    #[test]
    fn upstream_skill_is_fetched_at_its_pin_with_the_repo_licence() {
        let _lock = crate::engine::paths::kit_home_test_lock();
        let (base, rev) = fake_upstream("fetch");
        let home = base.join("home");
        // SAFETY: test lock held; no other thread reads these vars.
        unsafe {
            std::env::set_var("KIT_HOME", &home);
            std::env::set_var("KIT_GIT_BASE", &base);
        }
        let files = upstream_files("github:owner/repo", &rev, "skills/demo").unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|f| f.path.to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["SKILL.md", "scripts/run.sh", "LICENSE"]);
        // Second read is served from the cache, even with the remote gone.
        std::fs::remove_dir_all(base.join("owner")).unwrap();
        let again = upstream_files("github:owner/repo", &rev, "skills/demo").unwrap();
        assert_eq!(content_hash(&again), content_hash(&files));
        let err = upstream_files("github:owner/repo", &rev, "skills/none").unwrap_err();
        assert!(err.to_string().contains("no folder skills/none"), "{err}");
        unsafe {
            std::env::remove_var("KIT_HOME");
            std::env::remove_var("KIT_GIT_BASE");
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn local_skills_come_from_the_kit() {
        let kit = catalog::find("llm-engineer").unwrap();
        let skill = kit
            .manifest
            .skill
            .iter()
            .find(|s| s.name == "llm-evals")
            .unwrap();
        let p = skill_payload(&kit, skill).unwrap();
        assert_eq!(p.files.len(), 1);
        assert!(p.hash.starts_with("sha256:"));
    }

    #[test]
    fn hash_depends_on_content_and_paths_not_order() {
        let f = |p: &str, b: &str| SkillFile {
            path: p.into(),
            bytes: b.into(),
            executable: false,
        };
        let a = content_hash(&[f("a", "1"), f("b", "2")]);
        assert_eq!(a, content_hash(&[f("b", "2"), f("a", "1")]));
        assert_ne!(a, content_hash(&[f("a", "1"), f("b", "3")]));
        assert_ne!(a, content_hash(&[f("a", "1"), f("c", "2")]));
    }
}
