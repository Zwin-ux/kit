//! Where kits come from: the starter kits bundled in the binary, or a
//! folder on disk. The Git index comes later (DESIGN-KITS.md §6).

use super::manifest::KitManifest;
use anyhow::{Context, Result, bail};
use include_dir::{Dir, include_dir};
use std::path::{Path, PathBuf};

static BUNDLED: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/kits");

/// How much Kit vouches for a kit's source. Shown above every plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Ships with Kit, reviewed by its maintainers.
    Official,
    /// A folder or repo the user named. Not reviewed by Kit.
    Direct,
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Self::Official => "Official",
            Self::Direct => "Direct source, not reviewed by Kit",
        }
    }
}

/// A kit's own files (rules, local skills).
#[derive(Debug, Clone)]
pub enum KitFiles {
    Bundled(&'static Dir<'static>),
    Path(PathBuf),
}

impl KitFiles {
    pub fn read_to_string(&self, rel: &str) -> Result<String> {
        match self {
            Self::Bundled(dir) => dir
                .get_file(dir.path().join(rel))
                .and_then(|f| f.contents_utf8())
                .map(str::to_owned)
                .with_context(|| format!("bundled kit has no {rel}")),
            Self::Path(root) => std::fs::read_to_string(root.join(rel))
                .with_context(|| format!("cannot read {}", root.join(rel).display())),
        }
    }

    /// Every file under `rel`, as (path relative to `rel`, bytes).
    #[allow(dead_code)] // the installer uses it; the next slice lands it
    pub fn files_under(&self, rel: &str) -> Result<Vec<(PathBuf, Vec<u8>)>> {
        let mut out = Vec::new();
        match self {
            Self::Bundled(dir) => {
                let base = dir.path().join(rel);
                let sub = dir
                    .get_dir(&base)
                    .with_context(|| format!("bundled kit has no folder {rel}"))?;
                collect_bundled(sub, &base, &mut out);
            }
            Self::Path(root) => {
                let base = root.join(rel);
                if !base.is_dir() {
                    bail!("no folder {}", base.display());
                }
                collect_path(&base, &base, &mut out)?;
            }
        }
        out.sort();
        Ok(out)
    }
}

#[allow(dead_code)]
fn collect_bundled(dir: &'static Dir<'static>, base: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
    for f in dir.files() {
        let rel = f.path().strip_prefix(base).unwrap_or(f.path());
        out.push((rel.to_path_buf(), f.contents().to_vec()));
    }
    for d in dir.dirs() {
        collect_bundled(d, base, out);
    }
}

#[allow(dead_code)]
fn collect_path(dir: &Path, base: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("cannot read {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_path(&path, base, out)?;
        } else {
            let rel = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
            out.push((rel, std::fs::read(&path)?));
        }
    }
    Ok(())
}

/// One kit with its files and trust level.
#[derive(Debug, Clone)]
pub struct Kit {
    pub manifest: KitManifest,
    pub files: KitFiles,
    pub level: Level,
}

impl Kit {
    pub fn name(&self) -> &str {
        &self.manifest.kit.name
    }
}

/// The starter kits, sorted by name.
pub fn bundled() -> Result<Vec<Kit>> {
    let mut kits = Vec::new();
    for dir in BUNDLED.dirs() {
        let file = dir
            .get_file(dir.path().join("KIT.toml"))
            .with_context(|| format!("bundled kit {} has no KIT.toml", dir.path().display()))?;
        let raw = file.contents_utf8().context("KIT.toml is not UTF-8")?;
        let manifest = KitManifest::parse(raw, &format!("bundled {}", dir.path().display()))?;
        kits.push(Kit {
            manifest,
            files: KitFiles::Bundled(dir),
            level: Level::Official,
        });
    }
    kits.sort_by(|a, b| a.name().cmp(b.name()));
    Ok(kits)
}

/// A kit by name (bundled) or by folder path (`./my-kit`, `/abs/kit`).
pub fn find(spec: &str) -> Result<Kit> {
    if looks_like_path(spec) {
        let root = PathBuf::from(spec);
        let toml = root.join("KIT.toml");
        let raw = std::fs::read_to_string(&toml)
            .with_context(|| format!("no KIT.toml in {}", root.display()))?;
        let manifest = KitManifest::parse(&raw, &toml.display().to_string())?;
        return Ok(Kit {
            manifest,
            files: KitFiles::Path(root),
            level: Level::Direct,
        });
    }
    let kits = bundled()?;
    if let Some(k) = kits.iter().find(|k| k.name() == spec) {
        return Ok(k.clone());
    }
    let names: Vec<&str> = kits.iter().map(Kit::name).collect();
    match closest(spec, &names) {
        Some(near) => bail!("no kit named '{spec}'. Did you mean '{near}'? See kit show"),
        None => bail!("no kit named '{spec}'. Kits: {}", names.join(", ")),
    }
}

fn looks_like_path(spec: &str) -> bool {
    spec.starts_with('.') || spec.starts_with('/') || spec.contains(['/', '\\'])
}

/// A kit and everything it extends, bases first, each kit once.
pub fn resolve(spec: &str) -> Result<Vec<Kit>> {
    let mut out: Vec<Kit> = Vec::new();
    let mut stack = Vec::new();
    visit(find(spec)?, &mut out, &mut stack)?;
    Ok(out)
}

fn visit(kit: Kit, out: &mut Vec<Kit>, stack: &mut Vec<String>) -> Result<()> {
    let name = kit.name().to_string();
    if stack.contains(&name) {
        stack.push(name);
        bail!("kits extend each other in a loop: {}", stack.join(" → "));
    }
    if out.iter().any(|k| k.name() == name) {
        return Ok(());
    }
    stack.push(name);
    for base in kit.manifest.kit.extends.clone() {
        visit(find(&base)?, out, stack)?;
    }
    stack.pop();
    out.push(kit);
    Ok(())
}

/// Nearest name by edit distance, if it is close enough to be a typo.
fn closest<'a>(input: &str, names: &[&'a str]) -> Option<&'a str> {
    names
        .iter()
        .map(|n| (strsim(input, n), *n))
        .filter(|(d, n)| *d <= 2.max(n.len() / 4))
        .min_by_key(|(d, _)| *d)
        .map(|(_, n)| n)
}

/// Levenshtein distance.
fn strsim(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cur = row[j + 1];
            row[j + 1] = (prev + usize::from(ca != *cb))
                .min(row[j] + 1)
                .min(row[j + 1] + 1);
            prev = cur;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bundled_kit_is_valid_and_complete() {
        let kits = bundled().unwrap();
        let names: Vec<&str> = kits.iter().map(Kit::name).collect();
        assert_eq!(
            names,
            [
                "backend-engineer",
                "essentials",
                "frontend-design",
                "fullstack-design",
                "llm-engineer"
            ]
        );
        for kit in &kits {
            let m = &kit.manifest;
            if let Some(r) = &m.rules {
                assert!(!kit.files.read_to_string(&r.file).unwrap().trim().is_empty());
            }
            for s in m.skill.iter().filter(|s| s.source.is_none()) {
                let files = kit.files.files_under(&s.path).unwrap();
                assert!(
                    files.iter().any(|(p, _)| p == Path::new("SKILL.md")),
                    "{} skill {} has no SKILL.md",
                    m.kit.name,
                    s.name
                );
            }
            for s in &m.skill {
                assert!(
                    s.licence.is_some(),
                    "{} skill {} has no licence",
                    m.kit.name,
                    s.name
                );
            }
            let chain = resolve(kit.name()).unwrap();
            let mut seen = std::collections::HashSet::new();
            for sk in chain.iter().flat_map(|k| &k.manifest.skill) {
                assert!(
                    seen.insert(&sk.name),
                    "{}: skill {} twice",
                    kit.name(),
                    sk.name
                );
            }
        }
    }

    #[test]
    fn extends_resolve_bases_first_once() {
        let names: Vec<String> = resolve("fullstack-design")
            .unwrap()
            .iter()
            .map(|k| k.name().to_string())
            .collect();
        assert_eq!(names, ["essentials", "frontend-design", "fullstack-design"]);
    }

    #[test]
    fn typos_get_a_suggestion() {
        let err = find("frontend-desing").unwrap_err().to_string();
        assert!(err.contains("Did you mean 'frontend-design'"), "{err}");
        let err = find("zzz").unwrap_err().to_string();
        assert!(err.contains("Kits: backend-engineer"), "{err}");
    }

    #[test]
    fn a_folder_is_a_direct_kit() {
        let dir = std::env::temp_dir().join(format!("kit-folder-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("KIT.toml"),
            "schema = 1\n[kit]\nname = \"mine\"\ntitle = \"Mine\"\nversion = \"0.1.0\"\ndescription = \"d\"\nextends = [\"essentials\"]\n",
        )
        .unwrap();
        let kits = resolve(dir.to_str().unwrap()).unwrap();
        assert_eq!(kits[0].name(), "essentials");
        assert_eq!(kits[1].level, Level::Direct);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
