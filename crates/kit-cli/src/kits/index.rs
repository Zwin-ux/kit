//! The kit index: `index.toml` in a Git repo (`github:Zwin-ux/kits` by
//! default), listing kits by name with a pinned source. Fetched with `git`
//! like skills, cached under `~/.kit/index/`, refreshed at most daily.
//! See `docs/dev/DESIGN-MARKETPLACE.md`.

use super::catalog::{Kit, Level};
use super::fetch;
use super::remote::{self, GithubSpec};
use crate::engine::paths::kit_home;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The index every install reads unless `KIT_INDEX` names another.
pub const DEFAULT: &str = "github:Zwin-ux/kits";

/// Refresh the cached index when it is older than this.
const MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// Only schema this build reads.
const SCHEMA: u32 = 1;

/// Where the index comes from: `KIT_INDEX` (a `github:owner/repo`, a folder
/// holding `index.toml`, a file, or `off`), else [`DEFAULT`]. Unit tests
/// default to `off` so they never touch the network.
pub fn source() -> String {
    match std::env::var("KIT_INDEX") {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ if cfg!(test) => "off".into(),
        _ => DEFAULT.into(),
    }
}

#[derive(Debug, Deserialize)]
struct RawIndex {
    schema: u32,
    #[serde(default)]
    kit: Vec<toml::Value>,
}

/// One listed kit.
#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    pub name: String,
    pub summary: String,
    pub title: Option<String>,
    /// `github:owner/repo`.
    pub source: String,
    /// Folder in `source` holding `KIT.toml`; empty or absent for the root.
    #[serde(default)]
    pub path: String,
    /// Full commit sha.
    pub rev: String,
    /// `official` or `index`.
    #[serde(default = "default_level")]
    pub level: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_level() -> String {
    "index".into()
}

impl Entry {
    pub fn spec(&self) -> GithubSpec {
        GithubSpec {
            repo: self.source.trim_start_matches("github:").to_string(),
            path: self.path.trim_matches('/').to_string(),
            rev: Some(self.rev.clone()),
        }
    }

    fn validate(&self) -> Result<()> {
        let slug = |s: &str| {
            !s.is_empty()
                && !s.starts_with('-')
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        };
        if !slug(&self.name) {
            bail!("name must be lowercase letters, digits and dashes");
        }
        let texts = [&self.summary, self.title.as_ref().unwrap_or(&self.summary)];
        if texts
            .into_iter()
            .chain(&self.tags)
            .any(|t| super::manifest::has_control(t))
        {
            bail!("title, summary and tags may not hold control characters");
        }
        let spec = format!(
            "{}/{}@{}",
            self.source,
            self.path.trim_matches('/'),
            self.rev
        );
        let spec = spec.replace("/@", "@");
        if remote::parse(&spec)?.is_none() {
            bail!("source must be github:owner/repo");
        }
        if self.rev.len() != 40 || !self.rev.bytes().all(|b| b.is_ascii_hexdigit()) {
            bail!("rev must be a full 40-character commit sha");
        }
        if !matches!(self.level.as_str(), "official" | "index") {
            bail!("level must be official or index");
        }
        Ok(())
    }
}

/// A loaded index.
#[derive(Debug, Clone)]
pub struct Index {
    pub entries: Vec<Entry>,
    /// Where it came from, for messages.
    pub from: String,
    /// Kit's own index: the only one whose `official` entries are Official.
    pub trusted: bool,
    /// Something to tell the user: a stale cache, entries left out.
    pub warnings: Vec<String>,
}

impl Index {
    pub fn get(&self, name: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.name == name)
    }

    pub fn level(&self, entry: &Entry) -> Level {
        if self.trusted && entry.level == "official" {
            Level::Official
        } else {
            Level::Index
        }
    }

    /// The listed kit, at its pinned commit.
    pub fn fetch(&self, entry: &Entry) -> Result<Kit> {
        let kit = remote::fetch_at(&entry.spec(), &entry.rev, self.level(entry))
            .with_context(|| format!("{} (listed in {})", entry.name, self.from))?;
        if kit.name() != entry.name {
            bail!(
                "{} lists '{}', but its KIT.toml is named '{}'. Kit will not guess",
                self.from,
                entry.name,
                kit.name()
            );
        }
        Ok(kit)
    }
}

/// Parse `index.toml`. Entries that fail validation are left out with a
/// warning, so one bad listing cannot break search for everyone.
pub fn parse(raw: &str, from: &str, trusted: bool) -> Result<Index> {
    let raw: RawIndex =
        toml::from_str(raw).with_context(|| format!("{from} is not a valid index.toml"))?;
    if raw.schema != SCHEMA {
        bail!(
            "{from} uses index schema {}, this kit understands {SCHEMA}. Update kit",
            raw.schema
        );
    }
    let mut entries: Vec<Entry> = Vec::new();
    let mut warnings = Vec::new();
    for (i, value) in raw.kit.into_iter().enumerate() {
        let parsed = value
            .try_into::<Entry>()
            .map_err(anyhow::Error::from)
            .and_then(|e| e.validate().map(|()| e));
        match parsed {
            Ok(e) if entries.iter().any(|x| x.name == e.name) => {
                warnings.push(format!(
                    "{from}: '{}' is listed twice; using the first",
                    e.name
                ));
            }
            Ok(e) => entries.push(e),
            Err(err) => warnings.push(format!("{from}: kit #{} left out: {err:#}", i + 1)),
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Index {
        entries,
        from: from.to_string(),
        trusted,
        warnings,
    })
}

/// The index from [`source`]. `refresh` fetches even when the cache is
/// fresh. A failed fetch falls back to the cache and says how old it is.
pub fn load(refresh: bool) -> Result<Index> {
    let src = source();
    if src == "off" {
        return Ok(Index {
            entries: Vec::new(),
            from: "no index (KIT_INDEX=off)".into(),
            trusted: false,
            warnings: Vec::new(),
        });
    }
    let trusted = src == DEFAULT;
    match remote::parse(&src)? {
        Some(spec) if spec.rev.is_some() => {
            bail!("KIT_INDEX={src}: an index is followed at its default branch; leave off the @rev")
        }
        Some(spec) => load_github(&spec, &src, refresh, trusted),
        None => {
            let path = PathBuf::from(&src);
            let file = if path.is_dir() {
                path.join("index.toml")
            } else {
                path
            };
            let raw = std::fs::read_to_string(&file)
                .with_context(|| format!("cannot read the kit index {}", file.display()))?;
            parse(&raw, &file.display().to_string(), trusted)
        }
    }
}

/// One cache per index source (repo and path), so a second index never
/// reads as, or overwrites, Kit's own trusted one.
fn cache_file(spec: &GithubSpec) -> PathBuf {
    let key = format!("{}/{}", spec.repo, spec.path).replace('/', "--");
    kit_home()
        .join("index")
        .join(key.trim_end_matches('-'))
        .join("index.toml")
}

fn load_github(spec: &GithubSpec, src: &str, refresh: bool, trusted: bool) -> Result<Index> {
    let file = cache_file(spec);
    let age = std::fs::metadata(&file)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok());
    let fresh = age.is_some_and(|a| a < MAX_AGE);
    if !refresh && fresh {
        let raw = std::fs::read_to_string(&file)?;
        return parse(&raw, src, trusted);
    }
    match download(spec, &file) {
        Ok(raw) => parse(&raw, src, trusted),
        Err(err) => {
            let Ok(raw) = std::fs::read_to_string(&file) else {
                return Err(err.context(format!("cannot fetch the kit index {src}")));
            };
            let mut index = parse(&raw, src, trusted)?;
            let hours = age.map_or(0, |a| a.as_secs() / 3600);
            index.warnings.push(format!(
                "could not refresh {src} ({}); using the copy from {} ago",
                first_line(&err),
                if hours < 48 {
                    format!("{hours}h")
                } else {
                    format!("{} days", hours / 24)
                }
            ));
            Ok(index)
        }
    }
}

fn first_line(err: &anyhow::Error) -> String {
    format!("{err:#}")
        .lines()
        .next()
        .unwrap_or_default()
        .to_string()
}

/// Fetch the default branch of the index repo and read `index.toml`.
fn download(spec: &GithubSpec, file: &Path) -> Result<String> {
    let dir = kit_home()
        .join("cache")
        .join("git")
        .join(format!("{}.git", spec.repo));
    if !dir.join("HEAD").exists() {
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("cannot create {}", dir.display()))?;
        fetch::run(
            fetch::git(&dir).args(["init", "--bare", "-q"]),
            "create the index cache",
        )?;
    }
    let url = fetch::remote_url(&spec.repo);
    fetch::run(
        fetch::git(&dir).args(["fetch", "-q", "--depth", "1", "--", &url, "HEAD"]),
        &format!("download {}", spec.repo),
    )?;
    let want = if spec.path.is_empty() {
        "FETCH_HEAD:index.toml".to_string()
    } else {
        format!("FETCH_HEAD:{}/index.toml", spec.path)
    };
    let raw = fetch::run(
        fetch::git(&dir).args(["cat-file", "blob", &want]),
        &format!("find index.toml in {}", spec.repo),
    )?;
    let raw = String::from_utf8(raw).context("index.toml is not UTF-8")?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = file.with_extension("toml.tmp");
    std::fs::write(&tmp, &raw)?;
    std::fs::rename(&tmp, file).with_context(|| format!("cannot write {}", file.display()))?;
    Ok(raw)
}

/// The level of a `github:` kit when the cached index lists that exact
/// commit, without touching the network.
pub fn listed_level(spec: &GithubSpec, sha: &str) -> Option<Level> {
    let src = source();
    let cached = match remote::parse(&src).ok()? {
        Some(ix) => std::fs::read_to_string(cache_file(&ix)).ok()?,
        None if src == "off" => return None,
        None => {
            let p = PathBuf::from(&src);
            std::fs::read_to_string(if p.is_dir() { p.join("index.toml") } else { p }).ok()?
        }
    };
    let index = parse(&cached, &src, src == DEFAULT).ok()?;
    index
        .entries
        .iter()
        .find(|e| {
            let s = e.spec();
            s.repo.eq_ignore_ascii_case(&spec.repo)
                && s.path == spec.path
                && e.rev.eq_ignore_ascii_case(sha)
        })
        .map(|e| index.level(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA: &str = "2686b620fc1fed2e8f60c704839c766b8594c6b6";

    fn entry(name: &str, extra: &str) -> String {
        format!(
            "[[kit]]\nname = \"{name}\"\nsummary = \"s\"\nsource = \"github:o/r\"\npath = \"kits/{name}\"\nrev = \"{SHA}\"\n{extra}"
        )
    }

    #[test]
    fn bad_entries_are_left_out_with_a_reason() {
        let raw = format!(
            "schema = 1\n{}{}{}{}{}",
            entry(
                "good",
                "level = \"official\"\ntags = [\"ios\"]\nfuture_field = 1\n"
            ),
            entry("Bad Name", ""),
            entry("good", ""),
            "[[kit]]\nname = \"unpinned\"\nsummary = \"s\"\nsource = \"github:o/r\"\nrev = \"main\"\n",
            "[[kit]]\nname = \"escape\"\nsummary = \"s\"\nsource = \"github:o/r\"\npath = \"../x\"\nrev = \"2686b620fc1fed2e8f60c704839c766b8594c6b6\"\n",
        );
        let ix = parse(&raw, "test", true).unwrap();
        let names: Vec<&str> = ix.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["good"]);
        assert_eq!(ix.warnings.len(), 4, "{:?}", ix.warnings);
        assert!(ix.warnings.iter().any(|w| w.contains("twice")));
        assert!(ix.warnings.iter().any(|w| w.contains("40-character")));
        assert_eq!(ix.level(&ix.entries[0]), Level::Official);
    }

    #[test]
    fn only_kits_own_index_can_say_official() {
        let raw = format!("schema = 1\n{}", entry("x", "level = \"official\"\n"));
        let ix = parse(&raw, "someone/else", false).unwrap();
        assert_eq!(ix.level(&ix.entries[0]), Level::Index);
    }

    #[test]
    fn a_newer_schema_is_refused() {
        let err = parse("schema = 2\n", "t", true).unwrap_err().to_string();
        assert!(err.contains("Update kit"), "{err}");
    }

    /// A local index repo: `owner/kits` with index.toml listing one kit.
    pub(crate) fn fake_index(base: &Path) -> String {
        let sha = crate::kits::remote::tests::fake_kit_repo(base, "kits", "kits/tipper", "tipper");
        let dir = base.join("owner/kits");
        std::fs::write(
            dir.join("index.toml"),
            format!(
                "schema = 1\n[[kit]]\nname = \"tipper\"\nsummary = \"Tips for iOS design\"\nsource = \"github:owner/kits\"\npath = \"kits/tipper\"\nrev = \"{sha}\"\nlevel = \"official\"\ntags = [\"ios\"]\n"
            ),
        )
        .unwrap();
        for args in [&["add", "index.toml"][..], &["commit", "-q", "-m", "index"]] {
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
        }
        sha
    }

    #[test]
    fn the_index_is_fetched_cached_and_used_stale_when_offline() {
        let _lock = crate::engine::paths::kit_home_test_lock();
        let base = std::env::temp_dir().join(format!("kit-index-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let sha = fake_index(&base);
        // SAFETY: test lock held; no other thread reads these vars.
        unsafe {
            std::env::set_var("KIT_HOME", base.join("home"));
            std::env::set_var("KIT_GIT_BASE", &base);
            std::env::set_var("KIT_INDEX", "github:owner/kits");
        }
        let ix = load(false).unwrap();
        assert!(!ix.trusted, "only the default index is trusted");
        assert_eq!(ix.entries[0].name, "tipper");
        let kit = crate::kits::catalog::find("tipper").unwrap();
        assert_eq!(kit.level, Level::Index);
        assert_eq!(
            kit.pin.as_deref(),
            Some(format!("github:owner/kits/kits/tipper@{sha}").as_str())
        );
        // The same commit named directly is recognised as listed.
        let direct =
            crate::kits::catalog::find(&format!("github:owner/kits/kits/tipper@{sha}")).unwrap();
        assert_eq!(direct.level, Level::Index);

        // Offline: a forced refresh falls back to the cache and says so.
        std::fs::remove_dir_all(base.join("owner")).unwrap();
        let ix = load(true).unwrap();
        assert_eq!(ix.entries.len(), 1);
        assert!(
            ix.warnings[0].contains("could not refresh"),
            "{:?}",
            ix.warnings
        );

        // No cache and no network: a clear error.
        std::fs::remove_dir_all(base.join("home/index")).unwrap();
        let err = load(false).unwrap_err();
        assert!(
            format!("{err:#}").contains("cannot fetch the kit index"),
            "{err:#}"
        );
        unsafe {
            std::env::remove_var("KIT_HOME");
            std::env::remove_var("KIT_GIT_BASE");
            std::env::remove_var("KIT_INDEX");
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}
