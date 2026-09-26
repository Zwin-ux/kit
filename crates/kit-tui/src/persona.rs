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

    /// The role in a few lines, added after the user's task. It speaks about
    /// the user's project only: nothing in it may be about Kit's own code.
    pub fn brief(self) -> &'static str {
        match self {
            Self::Product => {
                "Act as the product owner. Decide what should exist and why, keep the \
                 scope small, and write acceptance criteria in the user's terms. Do not \
                 implement anything unless the task asks for it."
            }
            Self::Design => {
                "Act as the designer. Judge hierarchy, density, interaction and empty \
                 states against the conventions this project already uses. Do not add \
                 product scope."
            }
            Self::Eng => {
                "Act as the engineer. Make the smallest correct change, follow this \
                 project's conventions, and run its tests after each step."
            }
            Self::Qa => {
                "Act as QA. Prove each claim with a command and its output: reproduce \
                 first, then show the evidence. \"Looks right\" is not done."
            }
        }
    }

    /// Engine-facing prompt: the user's task first, then the role. The task
    /// leads because its first line titles the receipt and the `kit land`
    /// commit.
    pub fn wrap_task(self, user_task: &str) -> String {
        format!(
            "{}\n\n---\nRole ({}): {}",
            user_task.trim(),
            self.label(),
            self.brief()
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
    fn wrap_keeps_user_task_first_and_names_role() {
        let wrapped = Persona::Design.wrap_task("  fix the empty room\n");
        assert_eq!(wrapped.lines().next(), Some("fix the empty room"));
        assert!(wrapped.contains("Role (design): Act as the designer."));
    }

    /// Briefs go into every user's prompt: none may carry Kit's own dev notes.
    #[test]
    fn briefs_say_nothing_about_kit_itself() {
        for p in Persona::ALL {
            let b = p.brief().to_ascii_lowercase();
            for word in ["kit", "contract", "j/k", "skills:"] {
                assert!(!b.contains(word), "{}: {word}", p.label());
            }
        }
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
