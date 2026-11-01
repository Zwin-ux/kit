//! `~/.kit/config.toml`: what `kit setup` asked, so later commands default
//! to it. Written only by `kit setup`; safe to edit by hand.

use super::writers::Agent;
use crate::engine::paths::kit_home;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Agents Kit sets up: claude, codex, grok.
    #[serde(default)]
    pub agents: Vec<String>,
    /// Kits chosen in setup.
    #[serde(default)]
    pub kits: Vec<String>,
    /// "global" or "repo".
    #[serde(default)]
    pub scope: Option<String>,
    /// Kit's own agent: provider, model, profile, budget (written by hand).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentSection>,
    /// Permission rules for Kit's own agent, in Claude Code's syntax.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<PermissionsSection>,
}

/// `[agent]` in `~/.kit/config.toml`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSection {
    /// `anthropic` or `openai-compatible`.
    pub provider: Option<String>,
    /// Model id.
    pub model: Option<String>,
    /// Base URL for `openai-compatible` (ends in `/v1`).
    pub base_url: Option<String>,
    /// `read`, `workspace` or `full`.
    pub profile: Option<String>,
    /// Stop a session at this many dollars.
    pub budget_usd: Option<f64>,
    /// Replies per run.
    pub max_turns: Option<u32>,
    /// Extra environment variables tools may see, by name.
    #[serde(default)]
    pub env: Vec<String>,
}

/// `[permissions]` in `~/.kit/config.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionsSection {
    /// Rules that allow without asking.
    #[serde(default)]
    pub allow: Vec<String>,
    /// Rules that always ask.
    #[serde(default)]
    pub ask: Vec<String>,
    /// Rules that refuse.
    #[serde(default)]
    pub deny: Vec<String>,
}

pub fn path() -> PathBuf {
    kit_home().join("config.toml")
}

impl Config {
    pub fn load() -> Result<Option<Self>> {
        let file = path();
        match std::fs::read_to_string(&file) {
            Ok(raw) => toml::from_str(&raw).map(Some).with_context(|| {
                format!("{} is not valid. Fix it or run kit setup", file.display())
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("cannot read {}", file.display())),
        }
    }

    pub fn save(&self) -> Result<()> {
        let file = path();
        let body = toml::to_string(self)?;
        super::plan::write_file(
            &file,
            format!("# Written by kit setup. Edit it, or run kit setup again.\n{body}").as_bytes(),
        )
    }

    /// The saved agents Kit knows about.
    pub fn agents(&self) -> Vec<Agent> {
        self.agents
            .iter()
            .filter_map(|id| Agent::ALL.into_iter().find(|a| a.id() == id))
            .collect()
    }
}
