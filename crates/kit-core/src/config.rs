//! `kit.toml` — per-repository configuration, checked into the project.
//!
//! Every field has a default, so Kit is useful before it is configured.
//! See `docs/dev/PRD-1.0.md` section 4.3.
//!
//! CONTRACT FILE. Changing anything here is a Claude-only operation; agents
//! file an issue instead of editing. See `docs/dev/BUILD-ASSIGNMENT.md`.

use serde::{Deserialize, Serialize};

/// Root of `kit.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct KitConfig {
    pub gate: GateConfig,
    pub firewall: FirewallConfig,
}

/// The definition of done. Commands run in the run's worktree, in declared order.
///
/// A `None` command is skipped, not failed — a repo without a type-checker is
/// not a broken repo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GateConfig {
    pub format: Option<String>,
    pub typecheck: Option<String>,
    pub test: Option<String>,
    /// Extra commands run after the named ones, in order.
    pub extra: Vec<String>,
    /// Ceiling for the whole gate, not per command.
    #[serde(with = "humantime_secs")]
    pub timeout: std::time::Duration,
    pub scope: ScopeConfig,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            format: None,
            typecheck: None,
            test: None,
            extra: Vec::new(),
            timeout: std::time::Duration::from_secs(300),
            scope: ScopeConfig::default(),
        }
    }
}

impl GateConfig {
    /// Commands in execution order, paired with the label shown in the UI.
    pub fn checks(&self) -> Vec<(&str, &str)> {
        let mut out = Vec::new();
        if let Some(c) = &self.format {
            out.push(("format", c.as_str()));
        }
        if let Some(c) = &self.typecheck {
            out.push(("typecheck", c.as_str()));
        }
        if let Some(c) = &self.test {
            out.push(("test", c.as_str()));
        }
        for c in &self.extra {
            out.push(("extra", c.as_str()));
        }
        out
    }

    /// True when nothing would run — the caller should infer defaults instead.
    pub fn is_empty(&self) -> bool {
        self.checks().is_empty()
    }
}

/// Paths an agent may touch. Deny is applied after allow.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScopeConfig {
    /// Globs the agent may write. Empty means the whole worktree.
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

/// Blast-radius firewall, ported from Guardian's `guard.js`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FirewallConfig {
    pub mode: FirewallMode,
}

impl Default for FirewallConfig {
    fn default() -> Self {
        Self {
            mode: FirewallMode::Block,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FirewallMode {
    /// Refuse the command. Default — Guardian's posture.
    #[default]
    Block,
    /// Log and allow. For users who find the firewall too strict.
    Warn,
    Off,
}

/// Serde helper: `timeout = "5m"` on the wire, `Duration` in Rust.
mod humantime_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{}s", d.as_secs()))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let raw = String::deserialize(d)?;
        parse(&raw).ok_or_else(|| serde::de::Error::custom(format!("bad duration: {raw}")))
    }

    /// Accepts `90`, `90s`, `5m`, `2h`.
    pub fn parse(raw: &str) -> Option<Duration> {
        let raw = raw.trim();
        let (num, mult) = match raw.chars().last()? {
            's' => (&raw[..raw.len() - 1], 1),
            'm' => (&raw[..raw.len() - 1], 60),
            'h' => (&raw[..raw.len() - 1], 3600),
            _ => (raw, 1),
        };
        num.trim()
            .parse::<u64>()
            .ok()
            .map(|n| Duration::from_secs(n * mult))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_is_valid_and_empty() {
        let cfg: KitConfig = toml::from_str("").unwrap();
        assert!(cfg.gate.is_empty());
        assert_eq!(cfg.firewall.mode, FirewallMode::Block);
    }

    #[test]
    fn parses_the_prd_example() {
        let cfg: KitConfig = toml::from_str(
            r#"
            [gate]
            format    = "pnpm format:check"
            typecheck = "pnpm typecheck"
            test      = "pnpm test"
            timeout   = "5m"

            [gate.scope]
            allow = ["src/**", "tests/**"]
            deny  = [".github/**"]

            [firewall]
            mode = "block"
            "#,
        )
        .unwrap();

        assert_eq!(cfg.gate.timeout, std::time::Duration::from_secs(300));
        assert_eq!(cfg.gate.checks().len(), 3);
        assert_eq!(cfg.gate.checks()[0], ("format", "pnpm format:check"));
        assert_eq!(cfg.gate.scope.allow, vec!["src/**", "tests/**"]);
    }

    #[test]
    fn skips_absent_checks_without_failing() {
        let cfg: KitConfig = toml::from_str("[gate]\ntest = \"cargo test\"\n").unwrap();
        assert_eq!(cfg.gate.checks(), vec![("test", "cargo test")]);
    }

    #[test]
    fn unknown_keys_are_rejected_loudly() {
        assert!(toml::from_str::<KitConfig>("[gate]\ntypo = \"x\"\n").is_err());
    }
}
