//! Kits that live in a GitHub repo: `github:owner/repo[/path/to/kit][@rev]`.
//! The kit folder is fetched with `git` at one commit (through the same
//! bare cache as upstream skills) and unpacked under
//! `~/.kit/cache/kits/<owner>/<repo>/<sha>/`, so a pinned kit is fetched
//! once and read like a folder after that.

use super::catalog::{Kit, KitFiles, Level};
use super::fetch;
use super::manifest::KitManifest;
use crate::engine::paths::kit_home;
use anyhow::{Context, Result, bail};
use std::path::{Component, Path, PathBuf};

/// A parsed `github:` kit spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubSpec {
    /// `owner/repo`.
    pub repo: String,
    /// Folder inside the repo holding `KIT.toml`; empty for the root.
    pub path: String,
    /// Commit, branch or tag; `None` for the default branch.
    pub rev: Option<String>,
}

impl GithubSpec {
    /// The spec pinned to `sha`: what `kit.lock` records.
    pub fn pinned(&self, sha: &str) -> String {
        let path = if self.path.is_empty() {
            String::new()
        } else {
            format!("/{}", self.path)
        };
        format!("github:{}{path}@{sha}", self.repo)
    }
}

/// `Some` for a `github:` spec, `None` for anything else.
pub fn parse(spec: &str) -> Result<Option<GithubSpec>> {
    let Some(body) = spec.strip_prefix("github:") else {
        return Ok(None);
    };
    let (body, rev) = match body.rsplit_once('@') {
        Some((b, r)) => (b, Some(r.to_string())),
        None => (body, None),
    };
    let mut parts = body.split('/');
    let (Some(owner), Some(repo)) = (parts.next(), parts.next()) else {
        bail!(
            "'{spec}' is not a kit source. Use github:owner/repo, github:owner/repo/path, or add @rev"
        );
    };
    let name_ok = |s: &str| {
        !s.is_empty()
            && !s.starts_with(['-', '.'])
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    };
    if !name_ok(owner) || !name_ok(repo) {
        bail!("'{spec}': owner and repo may only use letters, digits, '-', '_' and '.'");
    }
    let path: Vec<&str> = parts.collect();
    if path.iter().any(|p| !name_ok(p)) {
        bail!("'{spec}': the path inside the repo must be plain folder names");
    }
    if let Some(r) = &rev
        && (r.is_empty()
            || r.starts_with(['-', '/'])
            || r.contains("..")
            || !r
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'/')))
    {
        bail!("'{spec}': '@{r}' is not a commit, branch or tag name");
    }
    Ok(Some(GithubSpec {
        repo: format!("{owner}/{repo}"),
        path: path.join("/"),
        rev,
    }))
}

fn is_sha(rev: &str) -> bool {
    rev.len() == 40 && rev.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The commit a branch, tag or the default branch points at now.
fn resolve_rev(repo: &str, rev: Option<&str>) -> Result<String> {
    if let Some(r) = rev
        && is_sha(r)
    {
        return Ok(r.to_ascii_lowercase());
    }
    let home = kit_home();
    std::fs::create_dir_all(&home).with_context(|| format!("cannot create {}", home.display()))?;
    let want = rev.unwrap_or("HEAD");
    let out = fetch::run(
        fetch::git(&home).args(["ls-remote", "--", &fetch::remote_url(repo), want]),
        &format!("find {want} in {repo}"),
    )?;
    let out = String::from_utf8_lossy(&out);
    let refs: Vec<(&str, &str)> = out.lines().filter_map(|l| l.split_once('\t')).collect();
    let pick = |name: &str| refs.iter().find(|(_, r)| *r == name).map(|(s, _)| *s);
    let sha = if want == "HEAD" {
        pick("HEAD")
    } else {
        pick(&format!("refs/heads/{want}"))
            .or_else(|| pick(&format!("refs/tags/{want}^{{}}")))
            .or_else(|| pick(&format!("refs/tags/{want}")))
    };
    match sha {
        Some(s) if is_sha(s) => Ok(s.to_string()),
        _ if rev.is_some_and(|r| r.len() >= 7 && r.bytes().all(|b| b.is_ascii_hexdigit())) => {
            bail!("{repo}: use the full 40-character commit sha, not '{want}'")
        }
        _ => bail!("{repo} has no branch or tag '{want}'"),
    }
}

/// Fetch the kit at `spec` and read its `KIT.toml`. Always `Direct`,
/// unless the index lists this exact commit. A failure is one plain line
/// with a next step, never git's own output.
pub fn fetch(spec: &GithubSpec) -> Result<Kit> {
    let sha = resolve_rev(&spec.repo, spec.rev.as_deref()).map_err(|e| plain(e, spec))?;
    fetch_at(spec, &sha, Level::Direct).map_err(|e| plain(e, spec))
}

/// Why a fetch from GitHub failed, read from git's error text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Failure {
    /// The repo does not exist or is private: git asks for a password.
    Missing,
    /// The repo is there but has no such commit.
    Commit,
    Network,
    Other,
}

pub(crate) fn failure(err: &anyhow::Error) -> Failure {
    let text = format!("{err:#}").to_lowercase();
    let any = |marks: &[&str]| marks.iter().any(|m| text.contains(m));
    if any(&[
        "not our ref",
        "couldn't find remote ref",
        "no such remote ref",
    ]) {
        Failure::Commit
    } else if any(&[
        "could not resolve host",
        "could not resolve proxy",
        "connect tunnel failed",
        "failed to connect",
        "timed out",
        "network is unreachable",
        "connection refused",
        "connection reset",
    ]) {
        Failure::Network
    } else if any(&[
        "could not read username",
        "repository not found",
        "does not appear to be a git repository",
        "authentication failed",
        "returned error: 403",
        "returned error: 404",
    ]) {
        Failure::Missing
    } else {
        Failure::Other
    }
}

/// `err` from fetching `spec` as one line a person can act on. Errors Kit
/// wrote itself (no KIT.toml, an unpinned base, a bad manifest) pass through.
fn plain(err: anyhow::Error, spec: &GithubSpec) -> anyhow::Error {
    let repo = &spec.repo;
    let at = spec.rev.as_deref().unwrap_or("its default branch");
    match failure(&err) {
        Failure::Missing => anyhow::anyhow!(
            "github:{repo} was not found. Check the name at {}; Kit reads public repos only",
            fetch::remote_url(repo)
        ),
        Failure::Commit => anyhow::anyhow!(
            "github:{repo} has no commit {at}. Check the sha on GitHub, or leave @{at} off to take the default branch"
        ),
        Failure::Network => anyhow::anyhow!(
            "cannot reach {} to fetch github:{repo}. Check your connection, then run it again",
            fetch::remote_url(repo)
        ),
        Failure::Other if format!("{err:#}").contains(" has no folder ") => no_kit_toml(spec, at),
        Failure::Other => err,
    }
}

fn no_kit_toml(spec: &GithubSpec, at: &str) -> anyhow::Error {
    let folder = if spec.path.is_empty() {
        "the repo root".to_string()
    } else {
        spec.path.clone()
    };
    anyhow::anyhow!(
        "github:{} at {at} has no KIT.toml in {folder}. Name the folder that holds it: github:{}/<folder>",
        spec.repo,
        spec.repo
    )
}

/// The kit at `spec`, pinned to `sha`, with the trust level the caller
/// established.
pub fn fetch_at(spec: &GithubSpec, sha: &str, level: Level) -> Result<Kit> {
    let root = unpack(spec, sha)?;
    let toml = root.join("KIT.toml");
    let pinned = spec.pinned(sha);
    if !toml.is_file() {
        return Err(no_kit_toml(spec, &sha[..7]));
    }
    let raw = std::fs::read_to_string(&toml).with_context(|| format!("cannot read {pinned}"))?;
    let manifest = KitManifest::parse(&raw, &pinned)?;
    for base in &manifest.kit.extends {
        if !is_pinned_base(base)? {
            bail!(
                "{pinned} extends '{base}', which is not pinned. A kit from GitHub or the index may extend a kit that ships with Kit or github:owner/repo@<full commit sha>, so what installs is exactly what was reviewed"
            );
        }
    }
    let level = match level {
        Level::Direct => super::index::listed_level(spec, sha).unwrap_or(Level::Direct),
        other => other,
    };
    Ok(Kit {
        manifest,
        files: KitFiles::Path(root),
        level,
        pin: Some(pinned),
    })
}

/// A base a pinned kit may extend: bundled (pinned by Kit's version), or
/// a `github:` spec at a full commit sha.
fn is_pinned_base(base: &str) -> Result<bool> {
    if let Some(spec) = parse(base)? {
        return Ok(spec.rev.as_deref().is_some_and(is_sha));
    }
    Ok(super::catalog::bundled()?.iter().any(|k| k.name() == base))
}

/// Unpack the kit folder at `sha` once; later calls reuse it.
fn unpack(spec: &GithubSpec, sha: &str) -> Result<PathBuf> {
    // One flat folder per (commit, kit path), so two kits in one repo at
    // one commit (every kit in an index repo) never share or clear a folder.
    let key = if spec.path.is_empty() {
        "+root".to_string()
    } else {
        spec.path.replace('/', "+")
    };
    let dir = kit_home()
        .join("cache")
        .join("kits")
        .join(&spec.repo)
        .join(sha);
    let root = dir.join(&key);
    let done = dir.join(format!("{key}.kit-complete"));
    if done.is_file() && root.join("KIT.toml").is_file() {
        return Ok(root);
    }
    let source = format!("github:{}", spec.repo);
    let folder = if spec.path.is_empty() {
        "."
    } else {
        &spec.path
    };
    let files = fetch::upstream_files(&source, sha, folder)
        .with_context(|| format!("cannot fetch the kit {}", spec.pinned(sha)))?;
    if root.exists() {
        std::fs::remove_dir_all(&root)
            .with_context(|| format!("cannot clear {}", root.display()))?;
    }
    for f in &files {
        if !plain_relative(&f.path) {
            bail!(
                "{}: refusing a file outside the kit folder: {}",
                spec.pinned(sha),
                f.path.display()
            );
        }
        let path = root.join(&f.path);
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
    std::fs::write(&done, format!("{}\n", spec.pinned(sha)))?;
    Ok(root)
}

fn plain_relative(p: &Path) -> bool {
    p.components().all(|c| matches!(c, Component::Normal(_)))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A local "github" (for `KIT_GIT_BASE`) with `owner/<repo>` holding a
    /// kit at `path` (empty for the root). Returns the commit sha.
    pub(crate) fn fake_kit_repo(base: &Path, repo: &str, path: &str, name: &str) -> String {
        let dir = base.join("owner").join(repo);
        let kit = if path.is_empty() {
            dir.clone()
        } else {
            dir.join(path)
        };
        std::fs::create_dir_all(kit.join("skills/tip")).unwrap();
        std::fs::write(
            kit.join("KIT.toml"),
            format!(
                "schema = 1\n[kit]\nname = \"{name}\"\ntitle = \"T\"\nversion = \"0.2.0\"\ndescription = \"a remote kit\"\n[rules]\nfile = \"RULES.md\"\n[[skill]]\nname = \"tip\"\npath = \"skills/tip\"\nlicence = \"MIT\"\n"
            ),
        )
        .unwrap();
        std::fs::write(kit.join("RULES.md"), "Remote rules.\n").unwrap();
        std::fs::write(
            kit.join("skills/tip/SKILL.md"),
            "---\nname: tip\n---\nTip.\n",
        )
        .unwrap();
        let git = |args: &[&str]| {
            let out = fetch::git(&dir)
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
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["add", "."]);
        git(&["commit", "-q", "-m", "kit"]);
        git(&["tag", "v1"]);
        git(&["rev-parse", "HEAD"])
    }

    #[test]
    fn specs_parse_and_refuse_anything_odd() {
        let s = parse("github:ana/ios-kit").unwrap().unwrap();
        assert_eq!(
            (s.repo.as_str(), s.path.as_str(), s.rev),
            ("ana/ios-kit", "", None)
        );
        let s = parse("github:ana/kits/kits/ios@v1.2").unwrap().unwrap();
        assert_eq!(s.path, "kits/ios");
        assert_eq!(s.rev.as_deref(), Some("v1.2"));
        assert_eq!(s.pinned("abc"), "github:ana/kits/kits/ios@abc");
        assert!(parse("frontend-design").unwrap().is_none());
        assert!(parse("./github:x").unwrap().is_none());
        for bad in [
            "github:ana",
            "github:-x/y",
            "github:a/b/../c",
            "github:a/b//c",
            "github:a/b@--upload-pack=evil",
            "github:a/b@",
            "github:a b/c",
            "github:a/b/c:d",
            "github:a/b/.git",
        ] {
            assert!(parse(bad).is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn a_github_kit_is_fetched_pinned_and_unpacked_once() {
        let _lock = crate::engine::paths::kit_home_test_lock();
        let base = std::env::temp_dir().join(format!("kit-remote-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let sha = fake_kit_repo(&base, "kits", "kits/tipper", "tipper");
        // SAFETY: test lock held; no other thread reads these vars.
        unsafe {
            std::env::set_var("KIT_HOME", base.join("home"));
            std::env::set_var("KIT_GIT_BASE", &base);
        }
        for spec in [
            "github:owner/kits/kits/tipper",
            "github:owner/kits/kits/tipper@main",
            "github:owner/kits/kits/tipper@v1",
        ] {
            let kit = crate::kits::catalog::find(spec).unwrap();
            assert_eq!(kit.name(), "tipper");
            assert_eq!(kit.level, Level::Direct);
            assert_eq!(
                kit.pin.as_deref(),
                Some(format!("github:owner/kits/kits/tipper@{sha}").as_str())
            );
            assert_eq!(
                kit.files.read_to_string("RULES.md").unwrap(),
                "Remote rules.\n"
            );
        }
        // Two kits in one repo at one commit each get their own folder.
        let root_kit = crate::kits::catalog::find(&format!("github:owner/kits@{sha}"));
        assert!(root_kit.is_err(), "the repo root has no KIT.toml");
        let again =
            crate::kits::catalog::find(&format!("github:owner/kits/kits/tipper@{sha}")).unwrap();
        assert_eq!(
            again.files.read_to_string("RULES.md").unwrap(),
            "Remote rules.\n"
        );

        // Pinned by sha: served from the cache with the remote gone.
        std::fs::remove_dir_all(base.join("owner")).unwrap();
        let kit =
            crate::kits::catalog::find(&format!("github:owner/kits/kits/tipper@{sha}")).unwrap();
        assert_eq!(kit.manifest.skill[0].name, "tip");
        let err = crate::kits::catalog::find("github:owner/gone").unwrap_err();
        let err = format!("{err:#}");
        assert!(err.contains("github:owner/gone was not found"), "{err}");
        assert!(!err.contains("git could not"), "no raw git output: {err}");
        unsafe {
            std::env::remove_var("KIT_HOME");
            std::env::remove_var("KIT_GIT_BASE");
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}
