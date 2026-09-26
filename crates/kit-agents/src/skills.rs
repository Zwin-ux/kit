//! The prompt every agent run gets, and where a repo's skill pack lives.
//!
//! Runs no longer copy a skill pack or an `AGENTS.md` into the worktree.
//! Kits install skills into each agent's own config, and a repo's committed
//! skills come with the worktree through git. Copying them in made a run's
//! diff (and `kit land`) carry the user's global skills.

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

fn count_skills(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().join("SKILL.md").is_file())
                .count()
        })
        .unwrap_or(0)
}

/// The prompt for one run: the user's task, then how to deliver.
///
/// The task comes first and is passed as written. Nothing here may be about
/// Kit's own code or a particular skill pack.
pub fn build_prompt(user_task: &str) -> String {
    format!(
        "{}\n\n## Delivery\n\n\
         - Work only in this repository worktree.\n\
         - Make the smallest change that satisfies the task.\n\
         - Summarize what you did and how to verify it.\n",
        user_task.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn prompt_is_the_task_then_delivery_and_nothing_about_kit() {
        let p = build_prompt("  fix the flaky test\n");
        assert!(p.starts_with("fix the flaky test\n\n## Delivery"), "{p}");
        for leftover in [
            "Kit",
            "completeness-qa",
            "Harness",
            "Skills pack",
            ".agents/skills",
            "using-agent-skills",
        ] {
            assert!(!p.contains(leftover), "{leftover} in the prompt: {p}");
        }
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
}
