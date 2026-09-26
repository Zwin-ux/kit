//! Translate a resolved kit into each agent's own files. One `KIT.toml`,
//! two writers (Claude Code, Codex); Grok reads what those write. See the
//! mapping table in `docs/dev/DESIGN-KITS.md` §3.

use super::catalog::Kit;
use super::fetch::{self, SkillPayload};
use super::manifest::McpServer;
use super::plan::{Action, find_program};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// The agents Kit can set up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Agent {
    Claude,
    Codex,
    Grok,
}

impl Agent {
    pub const ALL: [Self; 3] = [Self::Claude, Self::Codex, Self::Grok];

    pub fn id(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Grok => "grok",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::Codex => "Codex",
            Self::Grok => "Grok",
        }
    }

    /// Installed on this machine (its command is on PATH).
    pub fn installed(self) -> bool {
        find_program(self.id()).is_some()
    }
}

/// Where a kit is installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    /// Every project: the agent's config in the home directory.
    Global { home: PathBuf },
    /// One repo: files in the repo, shared with the team when committed.
    Repo(PathBuf),
}

impl Scope {
    /// The folder everything this scope writes stays inside.
    pub fn root(&self) -> &Path {
        match self {
            Self::Global { home } => home,
            Self::Repo(root) => root,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Global { .. } => "all projects".into(),
            Self::Repo(root) => format!(
                "this repo ({})",
                root.file_name().map_or_else(
                    || root.display().to_string(),
                    |n| n.to_string_lossy().into_owned()
                )
            ),
        }
    }
}

/// The pieces of one kit, fetched and ready to plan.
pub struct Resolved<'a> {
    pub kit: &'a Kit,
    pub skills: Vec<SkillPayload>,
    pub rules: Option<String>,
}

impl<'a> Resolved<'a> {
    /// Read the kit's rules and fetch its skills at their pins.
    pub fn load(kit: &'a Kit) -> Result<Self> {
        let skills = kit
            .manifest
            .skill
            .iter()
            .map(|s| fetch::skill_payload(kit, s))
            .collect::<Result<_>>()?;
        let rules = match &kit.manifest.rules {
            Some(r) => Some(kit.files.read_to_string(&r.file)?),
            None => None,
        };
        Ok(Self { kit, skills, rules })
    }
}

/// What to leave out.
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    /// Skills and rules only: no MCP servers, no hooks.
    pub no_code: bool,
}

/// Every action for one kit, for every agent, deduplicated in order.
pub fn plan(
    kit: &Resolved<'_>,
    agents: &[Agent],
    scope: &Scope,
    opts: Options,
    hook_program: &str,
) -> Vec<Action> {
    let mut out: Vec<Action> = Vec::new();
    for &agent in agents {
        let actions = match agent {
            Agent::Claude => claude(kit, scope, opts, hook_program),
            Agent::Codex => codex(kit, scope, opts),
            Agent::Grok => grok(kit, scope, opts, agents),
        };
        for a in actions {
            if !out.iter().any(|b| b.key() == a.key()) {
                out.push(a);
            }
        }
    }
    out.extend(gate(kit, scope, opts));
    out
}

/// The kit's `[gate]`: the checks every `kit run` in the repo must pass.
/// Agent-independent, so it is planned once.
fn gate(kit: &Resolved<'_>, scope: &Scope, opts: Options) -> Option<Action> {
    let m = &kit.kit.manifest;
    let commands: Vec<String> = m
        .gate
        .as_ref()?
        .checks()
        .iter()
        .map(|(_, cmd)| (*cmd).to_string())
        .collect();
    if commands.is_empty() {
        return None;
    }
    Some(match scope {
        Scope::Global { .. } => Action::Skip {
            piece: "gate".into(),
            why: "checks go into a repo's kit.toml; install into a repo to add them".into(),
        },
        Scope::Repo(_) if opts.no_code => Action::Skip {
            piece: "gate".into(),
            why: "left out (no code)".into(),
        },
        Scope::Repo(root) => Action::GateToml {
            file: root.join("kit.toml"),
            kit: m.kit.name.clone(),
            commands,
            previous: Vec::new(),
            shared: Vec::new(),
        },
    })
}

fn base(scope: &Scope, global: &str, repo: &str) -> PathBuf {
    match scope {
        Scope::Global { home } => home.join(global),
        Scope::Repo(root) => root.join(repo),
    }
}

fn skills(kit: &Resolved<'_>, dir: &Path) -> Vec<Action> {
    kit.skills
        .iter()
        .map(|p| Action::Skill {
            dir: dir.join(&p.name),
            payload: p.clone(),
        })
        .collect()
}

fn rules(kit: &Resolved<'_>, file: PathBuf) -> Option<Action> {
    let meta = &kit.kit.manifest.kit;
    kit.rules.as_ref().map(|text| Action::Rules {
        file,
        kit: meta.name.clone(),
        version: meta.version.clone(),
        text: text.clone(),
    })
}

fn left_out(kit: &Resolved<'_>) -> Vec<Action> {
    let m = &kit.kit.manifest;
    let mcp = m
        .mcp
        .iter()
        .filter(|(_, s)| s.runs_code())
        .map(|(n, _)| Action::Skip {
            piece: format!("mcp {n}"),
            why: "left out (no code)".into(),
        });
    let hooks = m.hook.iter().map(|h| Action::Skip {
        piece: format!("hook {}", h.on.label()),
        why: "left out (no code)".into(),
    });
    mcp.chain(hooks).collect()
}

// ---- Claude Code ----------------------------------------------------------

fn claude(kit: &Resolved<'_>, scope: &Scope, opts: Options, hook_program: &str) -> Vec<Action> {
    let m = &kit.kit.manifest;
    let mut out = skills(kit, &base(scope, ".claude/skills", ".claude/skills"));
    out.extend(rules(kit, base(scope, ".claude/CLAUDE.md", "CLAUDE.md")));
    for (name, server) in &m.mcp {
        if opts.no_code && server.runs_code() {
            continue;
        }
        let value = claude_mcp(server);
        out.push(match scope {
            // Claude Code owns ~/.claude.json and rewrites it while running,
            // so user-scope servers go through its own CLI.
            Scope::Global { .. } => {
                if find_program("claude").is_none() {
                    Action::Skip {
                        piece: format!("mcp {name}"),
                        why: "Claude Code is not installed; run kit add again once it is".into(),
                    }
                } else {
                    Action::ClaudeMcp {
                        name: name.clone(),
                        value,
                    }
                }
            }
            Scope::Repo(root) => Action::McpJson {
                file: root.join(".mcp.json"),
                name: name.clone(),
                value,
            },
        });
    }
    if opts.no_code {
        out.extend(left_out(kit));
        return out;
    }
    if !m.hook.is_empty() {
        out.push(Action::HookJson {
            file: base(scope, ".claude/settings.json", ".claude/settings.json"),
            event: "PostToolUse".into(),
            entry: serde_json::json!({
                "matcher": "Edit|Write|MultiEdit",
                "hooks": [{
                    "type": "command",
                    "command": format!(
                        "{hook_program} hook after-edit {} --scope {}",
                        m.kit.name,
                        match scope {
                            Scope::Global { .. } => "global",
                            Scope::Repo(_) => "repo",
                        }
                    ),
                }],
            }),
        });
    }
    out
}

/// `.mcp.json` / `claude mcp add-json` shape. `$VAR` becomes `${VAR}`,
/// which Claude Code expands from the environment when it starts.
fn claude_mcp(s: &McpServer) -> serde_json::Value {
    match (&s.command, &s.url) {
        (Some(cmd), _) => {
            let mut v = serde_json::json!({ "type": "stdio", "command": cmd, "args": s.args });
            if !s.env.is_empty() {
                let env: serde_json::Map<_, _> = s
                    .env
                    .iter()
                    .map(|(k, val)| (k.clone(), format!("${{{}}}", &val[1..]).into()))
                    .collect();
                v["env"] = env.into();
            }
            v
        }
        (None, url) => serde_json::json!({ "type": "http", "url": url }),
    }
}

// ---- Codex ------------------------------------------------------------------

fn codex(kit: &Resolved<'_>, scope: &Scope, opts: Options) -> Vec<Action> {
    let m = &kit.kit.manifest;
    let mut out = skills(kit, &base(scope, ".agents/skills", ".agents/skills"));
    out.extend(rules(kit, base(scope, ".codex/AGENTS.md", "AGENTS.md")));
    let config = base(scope, ".codex/config.toml", ".codex/config.toml");
    for (name, server) in &m.mcp {
        if opts.no_code && server.runs_code() {
            continue;
        }
        out.push(match codex_mcp(server) {
            Ok(value) => Action::McpToml {
                file: config.clone(),
                name: name.clone(),
                value,
            },
            Err(why) => Action::Skip {
                piece: format!("mcp {name} for Codex"),
                why,
            },
        });
    }
    if opts.no_code {
        out.extend(left_out(kit));
    } else if !m.hook.is_empty() {
        out.push(Action::Skip {
            piece: "hooks for Codex".into(),
            why: "not supported by Kit yet".into(),
        });
    }
    out
}

/// Codex `[mcp_servers.<name>]`. Codex passes environment variables through
/// by name (`env_vars`), so `KEY = "$KEY"` works and a rename does not.
fn codex_mcp(s: &McpServer) -> std::result::Result<toml_edit::Table, String> {
    let mut t = toml_edit::Table::new();
    match (&s.command, &s.url) {
        (Some(cmd), _) => {
            t.insert("command", toml_edit::value(cmd.as_str()));
            let args: toml_edit::Array = s.args.iter().map(String::as_str).collect();
            t.insert("args", toml_edit::value(args));
        }
        (None, Some(url)) => {
            t.insert("url", toml_edit::value(url.as_str()));
        }
        (None, None) => return Err("has no command or url".into()),
    }
    if !s.env.is_empty() {
        let mut names = toml_edit::Array::new();
        for (k, v) in &s.env {
            if v[1..] != **k {
                return Err(format!(
                    "Codex cannot rename environment variables ({k} = {v})"
                ));
            }
            names.push(k.as_str());
        }
        t.insert("env_vars", toml_edit::value(names));
    }
    Ok(t)
}

// ---- Grok -------------------------------------------------------------------

/// Grok reads Claude Code's files and `.agents/skills`. With Claude Code in
/// the same install there is nothing extra to write.
fn grok(kit: &Resolved<'_>, scope: &Scope, opts: Options, agents: &[Agent]) -> Vec<Action> {
    let m = &kit.kit.manifest;
    if agents.contains(&Agent::Claude) {
        return vec![Action::Skip {
            piece: "Grok".into(),
            why: "reads what Kit writes for Claude Code (kit doctor confirms)".into(),
        }];
    }
    let mut out = skills(kit, &base(scope, ".agents/skills", ".agents/skills"));
    match scope {
        Scope::Repo(root) => out.extend(rules(kit, root.join("AGENTS.md"))),
        Scope::Global { .. } if kit.rules.is_some() => out.push(Action::Skip {
            piece: "rules for Grok".into(),
            why: "no confirmed global rules file yet; add Claude Code or install into a repo"
                .into(),
        }),
        Scope::Global { .. } => {}
    }
    let code = m.mcp.len() + m.hook.len();
    if code > 0 && !opts.no_code {
        out.push(Action::Skip {
            piece: "mcp and hooks for Grok".into(),
            why: "not confirmed yet; add Claude Code and Grok picks them up".into(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kits::catalog;

    fn fixture_kit(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("kit-writers-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("skills/hello")).unwrap();
        std::fs::write(
            d.join("skills/hello/SKILL.md"),
            "---\nname: hello\n---\nHi\n",
        )
        .unwrap();
        std::fs::write(d.join("RULES.md"), "Be kind.\n").unwrap();
        std::fs::write(
            d.join("KIT.toml"),
            r#"schema = 1
[kit]
name = "demo"
title = "Demo"
version = "0.1.0"
description = "d"
[rules]
file = "RULES.md"
[[skill]]
name = "hello"
path = "skills/hello"
[mcp.docs]
url = "https://example.com/mcp"
[mcp.local]
command = "npx"
args = ["-y", "thing@1.0.0"]
env = { API_KEY = "$API_KEY" }
[[hook]]
on = "after_edit"
glob = "*.md"
run = "true"
"#,
        )
        .unwrap();
        d
    }

    /// Plan lines with `/` separators, so the expectations hold on Windows.
    fn lines(actions: &[Action]) -> Vec<String> {
        actions
            .iter()
            .map(|a| a.describe().replace('\\', "/"))
            .collect()
    }

    #[test]
    fn repo_install_for_claude_and_codex_writes_each_native_file() {
        let dir = fixture_kit("repo");
        let kit = catalog::find(dir.to_str().unwrap()).unwrap();
        let r = Resolved::load(&kit).unwrap();
        let root = PathBuf::from("/r");
        let actions = plan(
            &r,
            &[Agent::Claude, Agent::Codex],
            &Scope::Repo(root.clone()),
            Options::default(),
            "kit",
        );
        let text = lines(&actions).join("\n");
        for want in [
            "skill     /r/.claude/skills/hello",
            "rules     /r/CLAUDE.md",
            "mcp       docs → /r/.mcp.json",
            "hook      PostToolUse → /r/.claude/settings.json",
            "skill     /r/.agents/skills/hello",
            "rules     /r/AGENTS.md",
            "mcp       local → /r/.codex/config.toml",
            "skipped   hooks for Codex",
        ] {
            assert!(text.contains(want), "missing {want:?} in\n{text}");
        }
        let local = actions.iter().find_map(|a| match a {
            Action::McpJson { name, value, .. } if name == "local" => Some(value.clone()),
            _ => None,
        });
        assert_eq!(local.unwrap()["env"]["API_KEY"], "${API_KEY}");
        let toml = actions.iter().find_map(|a| match a {
            Action::McpToml { name, value, .. } if name == "local" => Some(value.to_string()),
            _ => None,
        });
        assert!(toml.unwrap().contains(r#"env_vars = ["API_KEY"]"#));
    }

    #[test]
    fn no_code_keeps_remote_mcp_and_names_what_it_left_out() {
        let dir = fixture_kit("nocode");
        let kit = catalog::find(dir.to_str().unwrap()).unwrap();
        let r = Resolved::load(&kit).unwrap();
        let actions = plan(
            &r,
            &[Agent::Claude],
            &Scope::Repo("/r".into()),
            Options { no_code: true },
            "kit",
        );
        let text = lines(&actions).join("\n");
        assert!(text.contains("mcp       docs"), "{text}");
        assert!(!text.contains("mcp       local"), "{text}");
        assert!(
            text.contains("skipped   mcp local: left out (no code)"),
            "{text}"
        );
        assert!(!actions.iter().any(|a| matches!(a, Action::HookJson { .. })));
    }

    #[test]
    fn grok_with_codex_shares_agents_skills_once() {
        let dir = fixture_kit("grok");
        let kit = catalog::find(dir.to_str().unwrap()).unwrap();
        let r = Resolved::load(&kit).unwrap();
        let actions = plan(
            &r,
            &[Agent::Codex, Agent::Grok],
            &Scope::Repo("/r".into()),
            Options::default(),
            "kit",
        );
        let skills = actions
            .iter()
            .filter(|a| matches!(a, Action::Skill { .. }))
            .count();
        assert_eq!(skills, 1);
    }
}
