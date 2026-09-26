//! Dispatch personas — role, not vendor.
//!
//! Codex/Claude/Grok/Ollama are CLIs. Product/Design/ENG/QA are who they act as.
//! The engine contract has no persona field; the TUI prepends a brief to the
//! job task and keeps the user-facing row task clean.

/// Role a run is acting as. Orthogonal to agent CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Persona {
    Product,
    Design,
    #[default]
    Eng,
    Qa,
}

impl Persona {
    pub const ALL: [Persona; 4] = [Self::Eng, Self::Product, Self::Design, Self::Qa];

    pub fn label(self) -> &'static str {
        match self {
            Self::Product => "product",
            Self::Design => "design",
            Self::Eng => "eng",
            Self::Qa => "qa",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "product" | "prod" | "pm" => Some(Self::Product),
            "design" | "des" | "ux" => Some(Self::Design),
            "eng" | "engineer" | "engineering" => Some(Self::Eng),
            "qa" | "quality" | "review" => Some(Self::Qa),
            _ => None,
        }
    }

    /// Short brief prepended to the engine task. Not shown in the table.
    pub fn brief(self) -> &'static str {
        match self {
            Self::Product => {
                "You are Product for this Kit run. Decide what should exist and why. \
                 Kill scope. Write acceptance in user terms. Do not implement unless \
                 the task says so. Skills: spec-driven-development, planning-and-task-breakdown. \
                 Voice: direct, no SaaS filler."
            }
            Self::Design => {
                "You are Design for this Kit run. Judge hierarchy, density, interaction \
                 language, empty states, and footer grammar. Do not invent product scope. \
                 Skills: frontend-ui-engineering. Kit keys: arrows move, k kills — never j/k nav."
            }
            Self::Eng => {
                "You are ENG for this Kit run. Smallest correct change. Test after each slice. \
                 Do not edit frozen contracts (kit-core run/config/gate, kit-agents trait, \
                 kit-tui event.rs). Skills: incremental-implementation, test-driven-development."
            }
            Self::Qa => {
                "You are QA for this Kit run. Prove claims with commands and receipts. \
                 Repro, then evidence. \"Looks right\" is not done. Skills: \
                 debugging-and-error-recovery, test-driven-development."
            }
        }
    }

    /// Engine-facing prompt: persona brief + the human task.
    pub fn wrap_task(self, user_task: &str) -> String {
        format!(
            "# Kit persona: {}\n\n{}\n\n## Task\n{}",
            self.label(),
            self.brief(),
            user_task.trim()
        )
    }
}

/// Default Dispatch persona toggles — ENG on so a single-agent submit stays 1×.
pub fn default_persona_toggles() -> Vec<(Persona, bool)> {
    Persona::ALL
        .into_iter()
        .map(|p| (p, p == Persona::Eng))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_keeps_user_task_and_names_role() {
        let wrapped = Persona::Design.wrap_task("fix the empty room");
        assert!(wrapped.contains("# Kit persona: design"));
        assert!(wrapped.contains("## Task\nfix the empty room"));
        assert!(wrapped.contains("frontend-ui-engineering"));
        assert!(!wrapped.contains("j/k nav") || wrapped.contains("never j/k"));
    }

    #[test]
    fn parse_aliases() {
        assert_eq!(Persona::parse("PM"), Some(Persona::Product));
        assert_eq!(Persona::parse("ux"), Some(Persona::Design));
        assert_eq!(Persona::parse("engineering"), Some(Persona::Eng));
        assert_eq!(Persona::parse("quality"), Some(Persona::Qa));
        assert_eq!(Persona::parse("codex"), None);
    }
}
