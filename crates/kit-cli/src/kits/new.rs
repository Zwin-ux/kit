//! `kit new NAME`: a folder with a `KIT.toml`, rules and one skill, valid
//! as written, so saving your own kit is an edit and a `git push`.
//! See `docs/dev/DESIGN-MARKETPLACE.md`.

use super::catalog::{self, Kit};
use super::plan::tilde;
use crate::cli::NewArgs;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// `ios-team` → `Ios Team`.
fn title_of(name: &str) -> String {
    name.split('-')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && !s.ends_with('-')
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn toml_str(s: &str) -> String {
    toml_edit::Value::from(s).to_string()
}

/// The files of a new kit from scratch, as (path, contents).
pub fn scaffold(name: &str, description: &str, extends: &[String]) -> Vec<(PathBuf, String)> {
    let title = title_of(name);
    let skill = format!("{name}-house-style");
    let extends_line = if extends.is_empty() {
        "# extends   = [\"essentials\"]        # build on other kits; their pieces install first\n"
            .to_string()
    } else {
        let list: Vec<String> = extends.iter().map(|e| toml_str(e)).collect();
        format!("extends     = [{}]\n", list.join(", "))
    };
    let kit_toml = format!(
        r#"# A kit sets coding agents up for one job. Edit this file, then:
#   kit show .            what it installs
#   kit add . --print     the exact plan, nothing written
schema = 1

[kit]
name        = {name_s}
title       = {title_s}
version     = "0.1.0"
description = {desc_s}
agents      = ["claude", "codex", "grok"]
licence     = "MIT"
{extends_line}# example   = "a first task to try once the kit is installed"

# Rules: added to CLAUDE.md and AGENTS.md between Kit's own markers.
[rules]
file = "RULES.md"

# A skill that lives in this kit.
[[skill]]
name    = {skill_s}
path    = "skills/{skill}"
licence = "MIT"

# A skill from someone else's repo, pinned to a full commit sha:
# [[skill]]
# name    = "frontend-ui-engineering"
# source  = "github:addyosmani/agent-skills"
# path    = "skills/frontend-ui-engineering"
# rev     = "2686b620fc1fed2e8f60c704839c766b8594c6b6"
# licence = "MIT"

# An MCP server. It runs code, so Kit shows it and asks first. Pin the version:
# [mcp.playwright]
# command = "npx"
# args    = ["-y", "@playwright/mcp@0.0.41"]

# A hook after each edit (runs code; Claude Code only for now):
# [[hook]]
# on   = "after_edit"
# glob = "*.{{ts,tsx}}"
# run  = "npx --no-install prettier --write \"$FILE\""

# How kit doctor proves the kit is live:
# [check]
# commands = ["npx --no-install prettier --version"]
"#,
        name_s = toml_str(name),
        title_s = toml_str(&title),
        desc_s = toml_str(description),
        skill_s = toml_str(&skill),
    );
    let rules = format!(
        "When working on {title} tasks:\n\n\
         - Say what you will change before changing it.\n\
         - Keep each change small and run the project's checks after it.\n\n\
         Replace these lines with how your team wants the agent to work.\n"
    );
    let skill_md = format!(
        "---\nname: {skill}\ndescription: How {title} work is done here. Use when starting or reviewing {title} work.\n---\n\n\
         # {title} house style\n\n\
         Agents read the description above to decide when to load this skill,\n\
         so keep it to one specific line.\n\n\
         ## Steps\n\n\
         1. Replace these steps with what your team does.\n\
         2. Point at real files and commands, not general advice.\n"
    );
    vec![
        (PathBuf::from("KIT.toml"), kit_toml),
        (PathBuf::from("RULES.md"), rules),
        (PathBuf::from(format!("skills/{skill}/SKILL.md")), skill_md),
        (
            PathBuf::from("README.md"),
            readme(name, &title, description),
        ),
    ]
}

fn readme(name: &str, title: &str, description: &str) -> String {
    format!(
        "# {title}\n\n{description}\n\n\
         A [Kit](https://github.com/Zwin-ux/kit) kit: skills, rules and tools that set\n\
         Claude Code, Codex and Grok up for one job.\n\n\
         ## Install\n\n\
         ```console\n\
         kit add github:<you>/{name} --global   # every project\n\
         kit add github:<you>/{name}            # this repo only\n\
         ```\n\n\
         Kit shows every file, key and command before writing, and\n\
         `kit remove {name}` undoes it exactly.\n\n\
         ## Work on it\n\n\
         ```console\n\
         kit show .           # what it installs\n\
         kit add . --print    # the exact plan, nothing written\n\
         ```\n\n\
         Everything is in `KIT.toml`.\n"
    )
}

/// The files of `kit`, renamed to `name` at version 0.1.0.
fn copy_of(kit: &Kit, name: &str, description: Option<&str>) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let raw = kit.files.read_to_string("KIT.toml")?;
    let mut doc: toml_edit::DocumentMut = raw.parse().context("KIT.toml is not valid TOML")?;
    let title = title_of(name);
    let meta = doc["kit"]
        .as_table_mut()
        .context("KIT.toml has no [kit] table")?;
    meta["name"] = toml_edit::value(name);
    meta["title"] = toml_edit::value(title.as_str());
    meta["version"] = toml_edit::value("0.1.0");
    if let Some(d) = description {
        meta["description"] = toml_edit::value(d);
    }
    let mut out = vec![(PathBuf::from("KIT.toml"), doc.to_string().into_bytes())];
    let m = &kit.manifest;
    if let Some(r) = &m.rules {
        out.push((
            PathBuf::from(&r.file),
            kit.files.read_to_string(&r.file)?.into_bytes(),
        ));
    }
    for s in m.skill.iter().filter(|s| s.source.is_none()) {
        for (rel, bytes) in kit.files.files_under(&s.path)? {
            out.push((Path::new(&s.path).join(rel), bytes));
        }
    }
    let desc = description.unwrap_or(&m.kit.description);
    out.push((
        PathBuf::from("README.md"),
        readme(name, &title, desc).into_bytes(),
    ));
    Ok(out)
}

pub fn cmd_new(args: &NewArgs, json: bool) -> Result<()> {
    let name = args.name.as_str();
    if !is_slug(name) {
        bail!(
            "'{name}' cannot be a kit name. Use lowercase letters, digits and dashes, like ios-team"
        );
    }
    if catalog::bundled()?.iter().any(|k| k.name() == name) {
        bail!(
            "{name} is a kit that ships with Kit. Pick another name, or start from it: kit new my-{name} --from {name}"
        );
    }
    let dir = args.dir.clone().unwrap_or_else(|| PathBuf::from(name));
    if dir.exists() && std::fs::read_dir(&dir).is_ok_and(|mut d| d.next().is_some()) {
        bail!(
            "{} already exists and is not empty. Pick another name or --dir",
            dir.display()
        );
    }

    let files: Vec<(PathBuf, Vec<u8>)> = match &args.from {
        Some(spec) => copy_of(&catalog::find(spec)?, name, args.description.as_deref())?,
        None => {
            let desc = args
                .description
                .clone()
                .unwrap_or_else(|| format!("Sets coding agents up for {} work", title_of(name)));
            scaffold(name, &desc, &args.extends)
                .into_iter()
                .map(|(p, s)| (p, s.into_bytes()))
                .collect()
        }
    };
    for (rel, bytes) in &files {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        std::fs::write(&path, bytes).with_context(|| format!("cannot write {}", path.display()))?;
    }
    // What we wrote must be a kit Kit can install; fail loudly if not.
    let spec = if dir.is_absolute() || dir.starts_with(".") {
        dir.display().to_string()
    } else {
        format!(".{}{}", std::path::MAIN_SEPARATOR, dir.display())
    };
    catalog::resolve(&spec)
        .with_context(|| format!("the new kit in {} does not load", dir.display()))?;

    if json {
        let data = serde_json::json!({
            "name": name,
            "dir": dir.display().to_string(),
            "files": files.iter().map(|(p, _)| p.to_string_lossy().replace('\\', "/")).collect::<Vec<_>>(),
            "from": args.from,
        });
        let env = crate::envelope("new", true, data, None, vec![]);
        println!("{}", serde_json::to_string_pretty(&env)?);
        return Ok(());
    }
    let shown = tilde(&dir);
    println!("Created {}/", shown.trim_end_matches(['/', '\\']));
    let rows: Vec<(String, &str)> = files
        .iter()
        .map(|(p, _)| {
            let p = p.to_string_lossy().replace('\\', "/");
            let what = match p.as_str() {
                "KIT.toml" => "what the kit installs (edit this)",
                "README.md" => "how others install it",
                _ if args.from.is_none() && p.starts_with("skills/") => {
                    "a skill that lives in the kit"
                }
                _ if args.from.is_none() => "added to CLAUDE.md and AGENTS.md",
                _ => "",
            };
            (p, what)
        })
        .collect();
    let w = rows.iter().map(|(p, _)| p.len()).max().unwrap_or(0);
    for (p, what) in rows {
        println!("  {}", format!("{p:w$}   {what}").trim_end());
    }
    println!();
    println!("Try it    kit show {spec}");
    println!("          kit add {spec} --print");
    println!(
        "Share it  push {} to GitHub, then: kit add github:<you>/{name}",
        shown
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_come_from_names() {
        assert_eq!(title_of("ios-team"), "Ios Team");
        assert_eq!(title_of("llm"), "Llm");
    }

    #[test]
    fn a_scaffold_is_a_valid_kit_and_every_example_is_valid_too() {
        let files = scaffold("ios-team", "iOS \"house\" rules", &["essentials".into()]);
        let toml = &files[0].1;
        let m = super::super::manifest::KitManifest::parse(toml, "scaffold").unwrap();
        assert_eq!(m.kit.name, "ios-team");
        assert_eq!(m.kit.description, "iOS \"house\" rules");
        assert_eq!(m.kit.extends, ["essentials"]);
        assert_eq!(m.skill[0].path, "skills/ios-team-house-style");
        // Uncommenting every example must still parse: the template teaches.
        let uncommented: String = toml
            .lines()
            .map(|l| match l.strip_prefix("# ") {
                Some(rest)
                    if rest.starts_with('[') || rest.contains(" = ") || rest.starts_with("on ") =>
                {
                    rest
                }
                _ => l,
            })
            .map(|l| format!("{l}\n"))
            .collect();
        let m = super::super::manifest::KitManifest::parse(&uncommented, "uncommented")
            .unwrap_or_else(|e| panic!("{e}\n{uncommented}"));
        assert_eq!(m.skill.len(), 2);
        assert_eq!(m.mcp.len(), 1);
        assert_eq!(m.hook.len(), 1);
    }
}
