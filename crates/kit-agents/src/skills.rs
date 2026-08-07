//! Inject a skill pack into a run worktree and prompt.
//!
//! Default pack is [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills)
//! under `.agents/skills`. Any directory of `*/SKILL.md` folders works — including
//! [Harness skills](https://github.com/harness/harness-skills) via `KIT_SKILLS_DIR`
//! or a repo-root `skills/` tree (Harness layout).
//!
//! Harness skills need the Harness MCP server; they are optional domain packs,
//! not Kit's default coding pack.

use std::fs;
use std::path::{Path, PathBuf};

/// Resolve the skills directory (never panics).
///
/// Order:
/// 1. `KIT_SKILLS_DIR` (explicit override — use for harness-skills or custom packs)
/// 2. `<repo>/.agents/skills` then `<repo>/skills` (if it looks like a skill pack)
/// 3. same under cwd
/// 4. walk up from cwd looking for `.agents/skills` or `skills/`
pub fn resolve_skills_dir(repo: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("KIT_SKILLS_DIR") {
        let p = PathBuf::from(p);
        if looks_like_skill_pack(&p) {
            return Some(p);
        }
    }

    let roots: Vec<PathBuf> = {
        let mut v = vec![repo.to_path_buf()];
        if let Ok(cwd) = std::env::current_dir()
            && cwd != repo
        {
            v.push(cwd);
        }
        v
    };

    for root in &roots {
        if let Some(p) = pack_under(root) {
            return Some(p);
        }
    }

    // Walk up from cwd (monorepo / nested worktree cases).
    if let Ok(mut dir) = std::env::current_dir() {
        for _ in 0..6 {
            if let Some(p) = pack_under(&dir) {
                return Some(p);
            }
            if !dir.pop() {
                break;
            }
        }
    }
    None
}

/// Prefer `.agents/skills`, then a repo-root `skills/` pack (Harness layout).
fn pack_under(root: &Path) -> Option<PathBuf> {
    let agents = root.join(".agents").join("skills");
    if looks_like_skill_pack(&agents) {
        return Some(agents);
    }
    let plain = root.join("skills");
    if looks_like_skill_pack(&plain) {
        return Some(plain);
    }
    None
}

/// True when `dir` exists and contains at least one `*/SKILL.md` skill folder.
pub fn looks_like_skill_pack(dir: &Path) -> bool {
    dir.is_dir() && count_skills(dir) > 0
}

/// Copy skill pack into the worktree when absent. Returns how many skill dirs linked/copied.
pub fn install_into_worktree(worktree: &Path, skills_src: &Path) -> std::io::Result<usize> {
    let dest = worktree.join(".agents").join("skills");
    if dest.is_dir() {
        // Already present (e.g. repo already vendors skills).
        return Ok(count_skills(&dest));
    }
    fs::create_dir_all(dest.parent().unwrap_or(worktree))?;
    copy_dir_recursive(skills_src, &dest)?;
    Ok(count_skills(&dest))
}

/// Ensure a minimal AGENTS.md exists so agents discover Kit conventions.
pub fn ensure_agents_md(worktree: &Path) -> std::io::Result<()> {
    let path = worktree.join("AGENTS.md");
    if path.exists() {
        return Ok(());
    }
    let body = r#"# Kit run workspace

You are executing a task under **Kit**, the control room for parallel agent work.

## Skills

Skill folders live in `.agents/skills/` (each skill is `name/SKILL.md`).
If a router skill such as `using-agent-skills` is present, start there.
Otherwise read skill descriptions and pick the one that matches the task.

## Rules

1. Surface assumptions before non-trivial work.
2. Small vertical slices; test after each change.
3. Do not invent credentials; use existing CLI logins only.
4. Prefer boring, reviewable diffs.
5. Domain packs (e.g. Harness) may require their own MCP/tools — use them when available.

See the task prompt for the specific objective.
"#;
    fs::write(path, body)
}

/// Kind of pack for prompt routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillPackKind {
    /// addyosmani-style engineering pack with `using-agent-skills`.
    CodingRouter,
    /// Any other SKILL.md pack (Harness, custom).
    Generic,
    /// No pack on disk.
    Missing,
}

/// Classify a skills directory (or missing).
pub fn pack_kind(skills_dir: Option<&Path>) -> SkillPackKind {
    let Some(dir) = skills_dir else {
        return SkillPackKind::Missing;
    };
    if !looks_like_skill_pack(dir) {
        return SkillPackKind::Missing;
    }
    if dir.join("using-agent-skills").join("SKILL.md").is_file() {
        SkillPackKind::CodingRouter
    } else {
        SkillPackKind::Generic
    }
}

/// Build the full prompt: skills preamble + user task.
///
/// Pass the same `skills_src` directory that was installed into the worktree
/// so routing matches the pack (coding router vs generic / Harness).
pub fn build_prompt(user_task: &str, skills_src: Option<&Path>) -> String {
    let kind = pack_kind(skills_src);
    let installed = !matches!(kind, SkillPackKind::Missing);
    build_prompt_for(user_task, kind, installed)
}

/// Testable prompt builder with an explicit pack kind.
pub fn build_prompt_for(user_task: &str, kind: SkillPackKind, skills_installed: bool) -> String {
    let routing = match kind {
        SkillPackKind::CodingRouter => {
            r#"## Skill routing (mandatory)

Before coding, apply the workflow from **using-agent-skills**:

| If the task is… | Use |
|-----------------|-----|
| Underspecified | interview yourself: list assumptions, then proceed carefully |
| New feature / change | spec-driven-development → planning-and-task-breakdown |
| Implementation | incremental-implementation + test-driven-development |
| UI | frontend-ui-engineering |
| Bug | debugging-and-error-recovery |
| Before claiming done | code-review-and-quality + code-simplification |

Core behaviors: surface assumptions · stop on confusion · simplicity first · scope discipline · verify with evidence."#
        }
        SkillPackKind::Generic => {
            r#"## Skill routing (mandatory)

A skill pack is installed at `.agents/skills/` (each skill has `SKILL.md`).

1. Scan skill names/descriptions for the best match to the user task.
2. Follow that skill's instructions fully (tools, MCP, YAML, policies).
3. If nothing matches, do the smallest correct change and say which skills you considered.
4. Domain packs (e.g. Harness) may require MCP/API credentials already configured on the host — do not invent secrets."#
        }
        SkillPackKind::Missing => {
            r#"## Skill routing

No skill pack was found on the host. Still:

- Surface assumptions · stop on confusion · simplicity first · scope discipline · verify with evidence."#
        }
    };

    let pack = if skills_installed {
        "Skills pack is installed at `.agents/skills/`."
    } else {
        "Skills pack was not found on the host."
    };

    format!(
        r#"# Kit Control Room — agent run

You are running under **Kit** (multi-agent control room). {pack}

{routing}

## User task

{user_task}

## Delivery

- Work only in this repository worktree.
- Make the smallest change that satisfies the task.
- Summarize what you did and how to verify it.
"#
    )
}

fn count_skills(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().join("SKILL.md").is_file())
                .count()
        })
        .unwrap_or(0)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn preamble_includes_task_and_routing() {
        let p = build_prompt_for("fix the flaky test", SkillPackKind::CodingRouter, true);
        assert!(p.contains("fix the flaky test"));
        assert!(p.contains("using-agent-skills"));
        assert!(p.contains("incremental-implementation"));
    }

    #[test]
    fn generic_pack_prompt_mentions_skill_md() {
        let p = build_prompt_for("debug pipeline", SkillPackKind::Generic, true);
        assert!(p.contains("debug pipeline"));
        assert!(p.contains("SKILL.md"));
        assert!(!p.contains("using-agent-skills"));
    }

    #[test]
    fn looks_like_skill_pack_requires_skill_md() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!("kit-pack-detect-{stamp}"));
        let empty = root.join("empty");
        fs::create_dir_all(&empty).unwrap();
        assert!(!looks_like_skill_pack(&empty));
        let skill = root.join("skills").join("debug-pipeline");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "---\nname: debug-pipeline\n---\n").unwrap();
        assert!(looks_like_skill_pack(&root.join("skills")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn install_copies_skill_tree() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!("kit-skills-test-{stamp}"));
        let src = root.join("src");
        let skill = src.join("using-agent-skills");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# test\n").unwrap();
        let wt = root.join("wt");
        fs::create_dir_all(&wt).unwrap();
        let n = install_into_worktree(&wt, &src).unwrap();
        assert_eq!(n, 1);
        assert!(
            wt.join(".agents")
                .join("skills")
                .join("using-agent-skills")
                .join("SKILL.md")
                .is_file()
        );
        let _ = fs::remove_dir_all(root);
    }
}
