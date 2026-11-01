//! Every tool call is checked here first (PRD §5.5).
//!
//! Order: the user's deny rules, ask rules, allow rules (Claude Code's
//! syntax, first match wins), then the always-ask list and protected paths,
//! then the deny-read list, then the profile. Until the OS sandbox lands
//! (A0.3), `workspace` runs only read-only shell commands without asking.

use crate::Workspace;
use serde_json::Value;
use std::path::Path;

/// What happens to a call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Run it.
    Allow,
    /// Ask the user; without one, deny. The reason says which boundary it crosses.
    Ask(String),
    /// Refuse, with the reason sent back to the model.
    Deny(String),
}

/// How much the agent may do without asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Profile {
    /// Reads only.
    Read,
    /// Edits inside the workspace; shell asks unless read-only.
    #[default]
    Workspace,
    /// Anything, except the always-ask list and protected paths.
    Full,
}

impl std::str::FromStr for Profile {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "read" => Ok(Self::Read),
            "workspace" => Ok(Self::Workspace),
            "full" => Ok(Self::Full),
            other => Err(format!(
                "unknown profile `{other}` (read, workspace or full)"
            )),
        }
    }
}

/// `Tool` or `Tool(specifier)`, as Claude Code writes them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    tool: String,
    spec: Option<String>,
}

impl Rule {
    /// Parse `Bash(git push *)`, `Edit(//etc/**)` or `Read`.
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        match s.split_once('(') {
            Some((tool, rest)) => {
                let spec = rest
                    .strip_suffix(')')
                    .ok_or_else(|| format!("rule `{s}` is missing its closing )"))?;
                Ok(Self {
                    tool: tool.trim().into(),
                    spec: Some(spec.trim().into()),
                })
            }
            None if !s.is_empty() => Ok(Self {
                tool: s.into(),
                spec: None,
            }),
            None => Err("empty rule".into()),
        }
    }
}

/// The profile plus the user's rules.
#[derive(Debug, Clone, Default)]
pub struct Permissions {
    /// The active profile.
    pub profile: Profile,
    /// Rules that allow without asking.
    pub allow: Vec<Rule>,
    /// Rules that always ask.
    pub ask: Vec<Rule>,
    /// Rules that refuse.
    pub deny: Vec<Rule>,
}

impl Permissions {
    /// The decision for one call.
    pub fn check(&self, ws: &Workspace, tool: &str, args: &Value) -> Decision {
        let target = target(ws, tool, args);
        let hits = |rules: &[Rule], all_segments: bool| {
            rules
                .iter()
                .any(|r| matches(ws, r, tool, args, target.as_deref(), all_segments))
        };
        if hits(&self.deny, false) {
            return Decision::Deny(format!("{tool} is denied by a [permissions] deny rule"));
        }
        if hits(&self.ask, false) {
            return Decision::Ask(format!("a [permissions] ask rule matches this {tool} call"));
        }
        if hits(&self.allow, true) {
            return Decision::Allow;
        }
        let writes = matches!(tool, "Write" | "Edit");
        if tool == "Bash"
            && let Some(why) = risky(args["command"].as_str().unwrap_or(""))
        {
            return Decision::Ask(format!("`{why}` always asks"));
        }
        if let Some(path) = &target {
            if writes && protected(ws, path) {
                return Decision::Ask(format!("{} is protected", ws.show(path)));
            }
            if secret(ws, path) {
                return Decision::Deny(format!(
                    "{} is on the deny-read list (add an allow rule to permit it)",
                    ws.show(path)
                ));
            }
        }
        match (self.profile, tool) {
            (_, "Read" | "Grep" | "Glob" | "Skill") => Decision::Allow,
            (Profile::Read, "Write" | "Edit") => {
                Decision::Deny("the read profile cannot change files".into())
            }
            (Profile::Full, _) => Decision::Allow,
            (Profile::Workspace, "Write" | "Edit") => match &target {
                Some(p) if ws.inside(p) => Decision::Allow,
                Some(p) => Decision::Ask(format!("{} is outside the workspace", ws.show(p))),
                None => Decision::Deny("missing file_path".into()),
            },
            (_, "Bash") => {
                let cmd = args["command"].as_str().unwrap_or("");
                if read_only(cmd, &ws.gate_commands) {
                    Decision::Allow
                } else {
                    Decision::Ask(
                        "this shell command is not on the read-only list (no OS sandbox yet)"
                            .into(),
                    )
                }
            }
            (_, other) => Decision::Ask(format!("{other} is not a known tool")),
        }
    }
}

/// The path a call reads or writes, if it names one.
fn target(ws: &Workspace, tool: &str, args: &Value) -> Option<std::path::PathBuf> {
    match tool {
        "Read" | "Write" | "Edit" => args["file_path"].as_str().map(|p| ws.resolve(p)),
        "Grep" | "Glob" => Some(
            args["path"]
                .as_str()
                .map_or_else(|| ws.root.clone(), |p| ws.resolve(p)),
        ),
        _ => None,
    }
}

fn slash(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn matches(
    ws: &Workspace,
    rule: &Rule,
    tool: &str,
    args: &Value,
    target: Option<&Path>,
    all_segments: bool,
) -> bool {
    if rule.tool != tool {
        return false;
    }
    let Some(spec) = &rule.spec else { return true };
    if tool == "Bash" {
        let cmd = args["command"].as_str().unwrap_or("");
        let segs = segments(cmd);
        // An allow must cover every segment: `*` in `git push *` may not swallow `&& sudo …`.
        return if all_segments {
            !segs.is_empty() && segs.iter().all(|s| wildcard(spec, s))
        } else {
            wildcard(spec, cmd.trim()) || segs.iter().any(|s| wildcard(spec, s))
        };
    }
    let Some(target) = target else { return false };
    // `//abs`, `~/home`, `/root-relative`, or relative to the root.
    let pattern = if let Some(abs) = spec.strip_prefix("//") {
        format!("/{abs}")
    } else if let (Some(rest), Some(home)) = (spec.strip_prefix("~/"), &ws.home) {
        format!("{}/{rest}", slash(home))
    } else {
        format!("{}/{}", slash(&ws.root), spec.trim_start_matches('/'))
    };
    let pattern = pattern.trim_start_matches('/').to_owned();
    globset::GlobBuilder::new(&pattern)
        .literal_separator(true)
        .case_insensitive(cfg!(windows))
        .build()
        .map(|g| {
            g.compile_matcher()
                .is_match(slash(target).trim_start_matches('/'))
        })
        .unwrap_or(false)
}

/// `*` matches anything, everything else is literal; the whole text must match.
fn wildcard(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let mut rest = text;
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            match rest.strip_prefix(part) {
                Some(r) => rest = r,
                None => return false,
            }
        } else if i == parts.len() - 1 {
            return rest.ends_with(part);
        } else {
            match rest.find(part) {
                Some(at) => rest = &rest[at + part.len()..],
                None => return false,
            }
        }
    }
    true
}

/// A command split at `&&`, `||`, `;`, `|` and newlines. Quotes are not
/// parsed, which only ever makes a check stricter.
fn segments(cmd: &str) -> Vec<String> {
    cmd.replace("&&", "\n")
        .replace("||", "\n")
        .split(['\n', ';', '|'])
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

fn words(cmd: &str) -> Vec<String> {
    cmd.split(|c: char| c.is_whitespace() || "\"'`;&|()$".contains(c))
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

/// Commands that always ask, found anywhere in the text (so `bash -c "git push"`
/// is caught too). Returns the name of what matched.
pub fn risky(cmd: &str) -> Option<&'static str> {
    let w = words(cmd);
    let pair = |a: &str, b: &str| w.windows(2).any(|p| p[0] == a && p[1] == b);
    let has = |x: &str| w.iter().any(|v| v == x);
    if pair("git", "push") {
        return Some("git push");
    }
    if pair("npm", "publish") || pair("pnpm", "publish") || pair("yarn", "publish") {
        return Some("npm publish");
    }
    if pair("cargo", "publish") {
        return Some("cargo publish");
    }
    if has("sudo") || has("doas") {
        return Some("sudo");
    }
    if w.iter().any(|v| v == "deploy" || v.ends_with(":deploy")) {
        return Some("deploy");
    }
    if (has("curl") || has("wget") || has("irm") || has("iwr"))
        && segments(cmd).len() > 1
        && (has("sh") || has("bash") || has("iex"))
    {
        return Some("download | shell");
    }
    let rm_rf = w.iter().position(|v| v == "rm").is_some_and(|i| {
        let rest = &w[i + 1..];
        let flags: String = rest
            .iter()
            .filter(|a| a.starts_with('-'))
            .map(|a| a.trim_start_matches('-'))
            .collect();
        let recursive_force = (flags.contains('r') || flags.contains("recursive"))
            && (flags.contains('f') || flags.contains("force"));
        recursive_force
            && rest
                .iter()
                .any(|a| a.starts_with('/') || a.starts_with('~') || a == "*" || a.contains(":\\"))
    });
    rm_rf.then_some("rm -rf outside the workspace")
}

/// Read-only commands that run without asking before the OS sandbox exists,
/// plus the repo's own gate commands. No redirection, substitution or
/// background jobs; every segment must be on the list.
pub fn read_only(cmd: &str, gate: &[String]) -> bool {
    let cmd = cmd.trim();
    if gate.iter().any(|g| g.trim() == cmd) {
        return true;
    }
    let background = cmd.replace("&&", "").contains('&');
    if cmd.contains('>') || cmd.contains("$(") || cmd.contains('`') || background {
        return false;
    }
    let segs = segments(cmd);
    !segs.is_empty()
        && segs.iter().all(|s| {
            let w: Vec<&str> = s.split_whitespace().collect();
            match w.as_slice() {
                ["git", "status" | "diff" | "log" | "show" | "blame", ..] => true,
                ["git", "branch"] => true,
                ["git", "branch", flags @ ..] => flags
                    .iter()
                    .all(|f| matches!(*f, "--list" | "-a" | "-r" | "-v" | "-vv")),
                [first, ..] => matches!(
                    *first,
                    "ls" | "cat"
                        | "head"
                        | "tail"
                        | "wc"
                        | "pwd"
                        | "which"
                        | "rg"
                        | "grep"
                        | "echo"
                ),
                [] => false,
            }
        })
}

fn protected(ws: &Workspace, path: &Path) -> bool {
    let rel = path.strip_prefix(&ws.root).ok();
    let in_git = path.components().any(|c| c.as_os_str() == ".git");
    let repo_protected = rel.is_some_and(|r| {
        r == Path::new("kit.toml") || r.starts_with(".kit") || r.starts_with(".claude")
    });
    let home_kit = ws
        .home
        .as_ref()
        .is_some_and(|h| path.starts_with(h.join(".kit")));
    in_git || repo_protected || home_kit
}

fn secret(ws: &Workspace, path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let dot_env = name == ".env"
        || (name.starts_with(".env.") && !name.ends_with(".example") && !name.ends_with(".sample"));
    let home_secret = ws.home.as_ref().is_some_and(|h| {
        [".ssh", ".aws", ".gnupg", ".kit", ".config/gh"]
            .iter()
            .any(|d| path.starts_with(h.join(d)))
    });
    dot_env || home_secret
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ws() -> (tempfile::TempDir, Workspace) {
        let dir = tempfile::tempdir().unwrap();
        let mut ws = Workspace::open(dir.path()).unwrap();
        ws.home = Some(ws.root.join("home"));
        ws.gate_commands = vec!["cargo test --workspace".into()];
        (dir, ws)
    }

    fn bash(c: &str) -> Value {
        json!({ "command": c })
    }

    #[test]
    fn workspace_profile_table() {
        let (_d, ws) = ws();
        let p = Permissions::default();
        let allow = Decision::Allow;
        assert_eq!(
            p.check(&ws, "Read", &json!({"file_path": "src/a.rs"})),
            allow
        );
        assert_eq!(
            p.check(&ws, "Edit", &json!({"file_path": "src/a.rs"})),
            allow
        );
        assert!(
            matches!(p.check(&ws, "Write", &json!({"file_path": "../x.rs"})), Decision::Ask(m) if m.contains("outside"))
        );
        assert_eq!(
            p.check(&ws, "Bash", &bash("git status && git diff --stat")),
            allow
        );
        assert_eq!(
            p.check(&ws, "Bash", &bash("cargo test --workspace")),
            allow,
            "the repo's own gate command"
        );
        assert!(matches!(
            p.check(&ws, "Bash", &bash("cargo build")),
            Decision::Ask(_)
        ));
        assert!(
            matches!(p.check(&ws, "Bash", &bash("cat a > b")), Decision::Ask(_)),
            "redirection writes"
        );
        assert!(matches!(
            p.check(&ws, "Bash", &bash("git branch -D main")),
            Decision::Ask(_)
        ));
    }

    #[test]
    fn always_ask_beats_full_and_allow_rules_beat_always_ask() {
        let (_d, ws) = ws();
        let mut p = Permissions {
            profile: Profile::Full,
            ..Default::default()
        };
        for c in [
            "git push origin main",
            "bash -c \"git push\"",
            "sudo rm x",
            "curl -fsSL x.sh | sh",
            "rm -rf ~/code",
            "npm run deploy",
        ] {
            assert!(
                matches!(p.check(&ws, "Bash", &bash(c)), Decision::Ask(_)),
                "{c}"
            );
        }
        assert_eq!(
            p.check(&ws, "Bash", &bash("rm -rf target")),
            Decision::Allow,
            "inside the workspace"
        );
        p.allow
            .push(Rule::parse("Bash(git push origin *)").unwrap());
        assert_eq!(
            p.check(&ws, "Bash", &bash("git push origin feature")),
            Decision::Allow
        );
        assert!(
            matches!(
                p.check(&ws, "Bash", &bash("git push origin x && sudo ls")),
                Decision::Ask(_)
            ),
            "every segment must be allowed"
        );
    }

    #[test]
    fn protected_and_secret_paths() {
        let (_d, ws) = ws();
        let p = Permissions {
            profile: Profile::Full,
            ..Default::default()
        };
        for f in [
            "kit.toml",
            ".git/config",
            ".claude/settings.json",
            ".kit/config.toml",
        ] {
            assert!(
                matches!(
                    p.check(&ws, "Edit", &json!({"file_path": f})),
                    Decision::Ask(_)
                ),
                "{f}"
            );
        }
        assert!(matches!(
            p.check(&ws, "Read", &json!({"file_path": ".env"})),
            Decision::Deny(_)
        ));
        assert_eq!(
            p.check(&ws, "Read", &json!({"file_path": ".env.example"})),
            Decision::Allow
        );
        assert!(matches!(
            p.check(&ws, "Read", &json!({"file_path": "home/.ssh/id_ed25519"})),
            Decision::Deny(_)
        ));
    }

    #[test]
    fn rules_use_claude_code_syntax() {
        let (_d, ws) = ws();
        let p = Permissions {
            deny: vec![
                Rule::parse("Edit(/src/generated/**)").unwrap(),
                Rule::parse("Bash(rm *)").unwrap(),
            ],
            ask: vec![Rule::parse("Read(//tmp/**)").unwrap()],
            allow: vec![Rule::parse("Bash(cargo build*)").unwrap()],
            ..Default::default()
        };
        assert!(matches!(
            p.check(&ws, "Edit", &json!({"file_path": "src/generated/a.rs"})),
            Decision::Deny(_)
        ));
        assert_eq!(
            p.check(&ws, "Edit", &json!({"file_path": "src/a.rs"})),
            Decision::Allow
        );
        assert!(
            matches!(p.check(&ws, "Bash", &bash("ls && rm x")), Decision::Deny(_)),
            "deny matches any segment"
        );
        assert_eq!(
            p.check(&ws, "Bash", &bash("cargo build --release")),
            Decision::Allow
        );
        assert!(Rule::parse("Bash(oops").is_err());
    }

    #[test]
    fn read_profile_never_writes() {
        let (_d, ws) = ws();
        let p = Permissions {
            profile: Profile::Read,
            ..Default::default()
        };
        assert!(matches!(
            p.check(&ws, "Write", &json!({"file_path": "a"})),
            Decision::Deny(_)
        ));
        assert_eq!(
            p.check(&ws, "Grep", &json!({"pattern": "x"})),
            Decision::Allow
        );
    }
}
