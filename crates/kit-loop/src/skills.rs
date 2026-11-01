//! Skills: `SKILL.md` folders (agentskills.io format). Only the name and
//! description go into the prompt; the body loads when asked for.

use std::path::{Path, PathBuf};

/// One skill found on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInfo {
    /// `name` from the frontmatter, else the folder name.
    pub name: String,
    /// `description` from the frontmatter.
    pub description: String,
    /// The `SKILL.md` file.
    pub path: PathBuf,
}

/// Skills in the repo's `.claude/skills` and `.agents/skills`, then the
/// home folder's. A repo skill wins over a home skill of the same name.
pub fn discover(root: &Path, home: Option<&Path>) -> Vec<SkillInfo> {
    let mut dirs = vec![root.join(".claude/skills"), root.join(".agents/skills")];
    if let Some(h) = home {
        dirs.push(h.join(".claude/skills"));
        dirs.push(h.join(".agents/skills"));
    }
    let mut out: Vec<SkillInfo> = Vec::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut found: Vec<_> = entries
            .flatten()
            .map(|e| e.path().join("SKILL.md"))
            .filter(|p| p.is_file())
            .collect();
        found.sort();
        for path in found {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let folder = path
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned());
            let (name, description) = frontmatter(&text);
            let Some(name) = name.or(folder) else {
                continue;
            };
            if !out.iter().any(|s| s.name == name) {
                out.push(SkillInfo {
                    name,
                    description: description.unwrap_or_default(),
                    path,
                });
            }
        }
    }
    out
}

/// `name` and `description` from YAML frontmatter, read line by line (no
/// YAML parser: the two fields are plain scalars in practice).
fn frontmatter(text: &str) -> (Option<String>, Option<String>) {
    let text = text.replace("\r\n", "\n");
    let Some(rest) = text.strip_prefix("---\n") else {
        return (None, None);
    };
    let Some(end) = rest.find("\n---") else {
        return (None, None);
    };
    let mut name = None;
    let mut description = None;
    for line in rest[..end].lines() {
        let unquote = |v: &str| v.trim().trim_matches('"').trim_matches('\'').to_owned();
        if let Some(v) = line.strip_prefix("name:") {
            name = Some(unquote(v));
        } else if let Some(v) = line.strip_prefix("description:") {
            description = Some(unquote(v));
        }
    }
    (name.filter(|n| !n.is_empty()), description)
}

/// The skill's body, frontmatter removed.
pub fn body(skill: &SkillInfo) -> std::io::Result<String> {
    let text = std::fs::read_to_string(&skill.path)?.replace("\r\n", "\n");
    Ok(
        match text
            .strip_prefix("---\n")
            .and_then(|r| r.find("\n---").map(|e| (r, e)))
        {
            Some((r, e)) => r[e + 4..].trim_start_matches(['-', '\n']).to_owned(),
            None => text,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_skills_win_and_bodies_drop_frontmatter() {
        let repo = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let write = |base: &Path, dir: &str, text: &str| {
            let d = base.join(dir);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("SKILL.md"), text).unwrap();
        };
        write(
            repo.path(),
            ".claude/skills/review",
            "---\r\nname: review\r\ndescription: \"Review code\"\r\n---\r\n# Steps\r\nRead it.\r\n",
        );
        write(
            home.path(),
            ".claude/skills/review",
            "---\nname: review\ndescription: home copy\n---\nx",
        );
        write(home.path(), ".agents/skills/untitled", "no frontmatter");
        let skills = discover(repo.path(), Some(home.path()));
        assert_eq!(skills.len(), 2);
        assert_eq!(skills[0].description, "Review code");
        assert_eq!(skills[1].name, "untitled");
        assert_eq!(body(&skills[0]).unwrap(), "# Steps\nRead it.\n");
    }
}
