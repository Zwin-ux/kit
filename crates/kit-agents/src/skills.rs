//! Inject addyosmani agent-skills into a run worktree and prompt.
//!
//! Production rule: agents get the same skill pack Kit uses, so TUI-dispatched
//! work follows define → plan → build → verify → review → ship.

use std::fs;
use std::path::{Path, PathBuf};

/// Resolve the skills directory (never panics).
///
/// Order: `KIT_SKILLS_DIR` → `<repo>/.agents/skills` → `<cwd>/.agents/skills`
/// → walk up from cwd looking for `.agents/skills`.
pub fn resolve_skills_dir(repo: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("KIT_SKILLS_DIR") {
        let p = PathBuf::from(p);
        if p.is_dir() {
            return Some(p);
        }
    }
    let candidates = [
        repo.join(".agents").join("skills"),
        std::env::current_dir()
            .ok()
            .map(|c| c.join(".agents").join("skills"))
            .unwrap_or_default(),
    ];
    for c in candidates {
        if c.is_dir() {
            return Some(c);
        }
    }
    // Walk up from cwd (monorepo / nested worktree cases).
    if let Ok(mut dir) = std::env::current_dir() {
        for _ in 0..6 {
            let c = dir.join(".agents").join("skills");
            if c.is_dir() {
                return Some(c);
            }
            if !dir.pop() {
                break;
            }
        }
    }
    None
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

Engineering skills live in `.agents/skills/` (addyosmani/agent-skills pack).
Start with `using-agent-skills`, then follow the lifecycle that fits the task.

## Rules

1. Surface assumptions before non-trivial work.
2. Small vertical slices; test after each change.
3. Do not invent credentials; use existing CLI logins only.
4. Prefer boring, reviewable diffs.

See the task prompt for the specific objective.
"#;
    fs::write(path, body)
}

/// Build the full prompt: skills preamble + user task.
pub fn build_prompt(user_task: &str, skills_installed: bool) -> String {
    let pack = if skills_installed {
        "Skills pack is installed at `.agents/skills/` (24 skills including using-agent-skills)."
    } else {
        "Skills pack was not found on the host; still follow the routing below."
    };

    format!(
        r#"# Kit Control Room — agent run

You are running under **Kit** (multi-agent control room). {pack}

## Skill routing (mandatory)

Before coding, apply the workflow from **using-agent-skills**:

| If the task is… | Use |
|-----------------|-----|
| Underspecified | interview yourself: list assumptions, then proceed carefully |
| New feature / change | spec-driven-development → planning-and-task-breakdown |
| Implementation | incremental-implementation + test-driven-development |
| UI | frontend-ui-engineering |
| Bug | debugging-and-error-recovery |
| Before claiming done | code-review-and-quality + code-simplification |

Core behaviors: surface assumptions · stop on confusion · simplicity first · scope discipline · verify with evidence.

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
        let p = build_prompt("fix the flaky test", true);
        assert!(p.contains("fix the flaky test"));
        assert!(p.contains("using-agent-skills"));
        assert!(p.contains("incremental-implementation"));
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
