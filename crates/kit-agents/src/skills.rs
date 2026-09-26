//! The prompt every agent run gets.
//!
//! Runs no longer copy a skill pack or an `AGENTS.md` into the worktree.
//! Kits install skills into each agent's own config, and a repo's committed
//! skills come with the worktree through git. Copying them in made a run's
//! diff (and `kit land`) carry the user's global skills.

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
}
