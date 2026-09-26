//! `KIT.toml`: one kit, the skills, rules, MCP servers, hooks, gate and
//! check that set an agent up for one job. See `docs/dev/DESIGN-KITS.md` §2.

use anyhow::{Result, bail};
use kit_core::GateConfig;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The only schema this build reads.
pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KitManifest {
    pub schema: u32,
    pub kit: KitMeta,
    #[serde(default)]
    pub skill: Vec<SkillRef>,
    pub rules: Option<Rules>,
    #[serde(default)]
    pub mcp: BTreeMap<String, McpServer>,
    #[serde(default)]
    pub hook: Vec<Hook>,
    pub gate: Option<GateConfig>,
    #[serde(default)]
    pub check: Check,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KitMeta {
    pub name: String,
    pub title: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub extends: Vec<String>,
    /// Agents the kit was written and tested for. A claim, not a filter.
    #[serde(default)]
    pub agents: Vec<String>,
    pub licence: Option<String>,
    /// A first task to try once the kit is installed.
    pub example: Option<String>,
}

/// A skill: upstream (`source` + `rev`) or shipped inside the kit.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillRef {
    /// Folder name the skill installs as.
    pub name: String,
    /// `github:owner/repo`; absent for a skill inside the kit.
    pub source: Option<String>,
    /// Path to the skill folder, in `source` or in the kit.
    pub path: String,
    /// Full commit sha; required with `source`.
    pub rev: Option<String>,
    pub licence: Option<String>,
}

impl SkillRef {
    /// `owner/repo@abcdef0`, or `None` for a skill inside the kit.
    pub fn origin(&self) -> Option<String> {
        let repo = self.source.as_deref()?.strip_prefix("github:")?;
        let rev = self.rev.as_deref().unwrap_or_default();
        Some(format!("{repo}@{}", rev.get(..7).unwrap_or(rev)))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpServer {
    /// A local server: the program to start (with `args`).
    pub command: Option<String>,
    /// A remote server: an `https://` URL. Exactly one of `command`, `url`.
    pub url: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    /// Values may only name the user's environment: `"$API_KEY"`.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

impl McpServer {
    /// What the plan shows: the command line, or the URL.
    pub fn command_line(&self) -> String {
        match (&self.command, &self.url) {
            (Some(cmd), _) => std::iter::once(cmd.as_str())
                .chain(self.args.iter().map(String::as_str))
                .collect::<Vec<_>>()
                .join(" "),
            (None, Some(url)) => url.clone(),
            (None, None) => String::new(),
        }
    }

    /// A local server runs code on the user's machine; a remote one does not.
    pub fn runs_code(&self) -> bool {
        self.command.is_some()
    }
}

/// Portable hook events. v0.1 has one; each agent writer maps it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    AfterEdit,
}

impl HookEvent {
    pub fn label(self) -> &'static str {
        match self {
            Self::AfterEdit => "after each edit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hook {
    pub on: HookEvent,
    /// Only files matching this glob trigger the hook.
    pub glob: Option<String>,
    /// Command to run; `$FILE` is the edited file.
    pub run: Option<String>,
    /// A hook Kit provides instead of a command: `use = "format-and-lint"`.
    #[serde(rename = "use")]
    pub builtin: Option<Builtin>,
}

/// Hooks that ship inside Kit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Builtin {
    /// Format the edited file, then lint it, with the tools the project
    /// already has (Prettier, ESLint, Ruff, rustfmt, gofmt, SwiftFormat,
    /// SwiftLint). Installs nothing.
    FormatAndLint,
}

impl Hook {
    /// What the plan and `kit show` print for this hook.
    pub fn describe(&self) -> String {
        match (&self.run, self.builtin) {
            (Some(run), _) => run.clone(),
            (None, Some(Builtin::FormatAndLint)) => {
                "format and lint with the project's own tools (kit:format-and-lint)".into()
            }
            (None, None) => String::new(),
        }
    }
}

/// How `kit doctor` proves the kit is live.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    #[serde(default)]
    pub mcp_starts: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
}

impl KitManifest {
    /// Parse and validate. Errors name the kit and the field.
    pub fn parse(raw: &str, origin: &str) -> Result<Self> {
        let m: Self = toml::from_str(raw)
            .map_err(|e| anyhow::anyhow!("{origin} is not a valid KIT.toml: {e}"))?;
        m.validate(origin)?;
        Ok(m)
    }

    /// Anything that runs on the user's machine: hooks and MCP servers.
    pub fn runs_code(&self) -> usize {
        self.mcp.values().filter(|m| m.runs_code()).count() + self.hook.len()
    }

    fn validate(&self, origin: &str) -> Result<()> {
        let name = &self.kit.name;
        if self.schema != SCHEMA {
            bail!(
                "{origin} uses KIT.toml schema {}, this kit understands {SCHEMA}. Update kit",
                self.schema
            );
        }
        if !is_slug(name) {
            bail!("{origin}: kit name '{name}' must be lowercase letters, digits and dashes");
        }
        for s in &self.skill {
            if !is_slug(&s.name) {
                bail!(
                    "kit {name}: skill name '{}' must be lowercase letters, digits and dashes",
                    s.name
                );
            }
            if s.path.split(['/', '\\']).any(|part| part == "..") {
                bail!(
                    "kit {name}: skill {} path must stay inside its source",
                    s.name
                );
            }
            match (&s.source, &s.rev) {
                (Some(src), Some(rev)) => {
                    if src
                        .strip_prefix("github:")
                        .is_none_or(|r| r.split('/').count() != 2)
                    {
                        bail!(
                            "kit {name}: skill {} source '{src}' must be github:owner/repo",
                            s.name
                        );
                    }
                    if rev.len() != 40 || !rev.bytes().all(|b| b.is_ascii_hexdigit()) {
                        bail!(
                            "kit {name}: skill {} rev must be a full 40-character commit sha",
                            s.name
                        );
                    }
                }
                (Some(_), None) => {
                    bail!(
                        "kit {name}: skill {} has a source but no rev. Pin a commit",
                        s.name
                    )
                }
                (None, Some(_)) => bail!("kit {name}: skill {} has a rev but no source", s.name),
                (None, None) => {}
            }
        }
        for (server, mcp) in &self.mcp {
            for (key, value) in &mcp.env {
                if !value.starts_with('$') {
                    bail!(
                        "kit {name}: mcp {server} env {key} must name an environment variable (\"$NAME\"). Kits never carry secrets"
                    );
                }
            }
            match (&mcp.command, &mcp.url) {
                (Some(_), None) => {}
                (None, Some(url)) if url.starts_with("https://") => {}
                (None, Some(url)) => {
                    bail!("kit {name}: mcp {server} url '{url}' must start with https://")
                }
                _ => bail!("kit {name}: mcp {server} needs exactly one of command or url"),
            }
            if mcp.command.as_deref() == Some("npx") && !npx_is_pinned(&mcp.args) {
                bail!(
                    "kit {name}: mcp {server} must pin its package version (npx -y package@1.2.3)"
                );
            }
        }
        for h in &self.hook {
            if h.run.is_some() == h.builtin.is_some() {
                bail!("kit {name}: each [[hook]] needs exactly one of run or use");
            }
        }
        for c in &self.check.mcp_starts {
            if !self.mcp.contains_key(c) {
                bail!("kit {name}: check.mcp_starts names '{c}', which is not an [mcp] server");
            }
        }
        Ok(())
    }
}

fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !s.starts_with('-')
}

/// The package after npx's flags carries an exact version.
fn npx_is_pinned(args: &[String]) -> bool {
    let Some(pkg) = args.iter().find(|a| !a.starts_with('-')) else {
        return false;
    };
    // `@scope/name@1.2.3` or `name@1.2.3`; the version is after the last '@'
    // that is not the scope's leading one.
    let body = pkg.strip_prefix('@').unwrap_or(pkg);
    match body.rsplit_once('@') {
        Some((_, v)) => v.starts_with(|c: char| c.is_ascii_digit()),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: &str = "schema = 1\n[kit]\nname = \"x\"\ntitle = \"X\"\nversion = \"0.1.0\"\ndescription = \"d\"\n";

    fn parse(extra: &str) -> Result<KitManifest> {
        KitManifest::parse(&format!("{MIN}{extra}"), "test")
    }

    #[test]
    fn minimal_kit_parses() {
        let m = parse("").unwrap();
        assert_eq!(m.kit.name, "x");
        assert_eq!(m.runs_code(), 0);
    }

    #[test]
    fn a_hook_is_a_command_or_a_builtin_never_both() {
        let hook = "[[hook]]\non = \"after_edit\"\n";
        let m = parse(&format!("{hook}use = \"format-and-lint\"\n")).unwrap();
        assert_eq!(m.hook[0].builtin, Some(Builtin::FormatAndLint));
        assert_eq!(m.runs_code(), 1);
        for bad in ["", "use = \"format-and-lint\"\nrun = \"x\"\n"] {
            let err = parse(&format!("{hook}{bad}")).unwrap_err().to_string();
            assert!(err.contains("exactly one of run or use"), "{err}");
        }
        assert!(parse(&format!("{hook}use = \"reformat-everything\"\n")).is_err());
    }

    #[test]
    fn unknown_keys_are_errors() {
        let err = parse("[kit2]\n").unwrap_err().to_string();
        assert!(err.contains("kit2"), "{err}");
    }

    #[test]
    fn upstream_skills_must_be_pinned() {
        let base = "[[skill]]\nname = \"s\"\npath = \"skills/s\"\nsource = \"github:o/r\"\n";
        let err = parse(base).unwrap_err().to_string();
        assert!(err.contains("no rev"), "{err}");
        let err = parse(&format!("{base}rev = \"main\"\n"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("40-character"), "{err}");
        let sha = "2686b620fc1fed2e8f60c704839c766b8594c6b6";
        let m = parse(&format!("{base}rev = \"{sha}\"\n")).unwrap();
        assert_eq!(m.skill[0].origin().as_deref(), Some("o/r@2686b62"));
    }

    #[test]
    fn skill_paths_cannot_escape() {
        let err = parse("[[skill]]\nname = \"s\"\npath = \"../x\"\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("inside"), "{err}");
    }

    #[test]
    fn mcp_must_pin_and_carry_no_secrets() {
        let err =
            parse("[mcp.p]\ncommand = \"npx\"\nargs = [\"-y\", \"@playwright/mcp@latest\"]\n")
                .unwrap_err()
                .to_string();
        assert!(err.contains("pin"), "{err}");
        assert!(
            parse("[mcp.p]\ncommand = \"npx\"\nargs = [\"-y\", \"@playwright/mcp@0.0.82\"]\n")
                .is_ok()
        );
        assert!(parse("[mcp.p]\ncommand = \"npx\"\nargs = [\"-y\", \"pkg@1.0.0\"]\n").is_ok());
        let err = parse("[mcp.p]\ncommand = \"srv\"\nenv = { KEY = \"sk-live-123\" }\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("never carry secrets"), "{err}");
    }

    #[test]
    fn remote_mcp_is_https_and_runs_no_code() {
        let m = parse("[mcp.docs]\nurl = \"https://example.com/mcp\"\n").unwrap();
        assert_eq!(m.runs_code(), 0);
        let err = parse("[mcp.docs]\nurl = \"http://example.com/mcp\"\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("https://"), "{err}");
        let err = parse("[mcp.docs]\nargs = []\n").unwrap_err().to_string();
        assert!(err.contains("exactly one"), "{err}");
    }

    #[test]
    fn checks_name_real_servers() {
        let err = parse("[check]\nmcp_starts = [\"nope\"]\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("nope"), "{err}");
    }
}
