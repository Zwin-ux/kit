//! `kit show [KIT]`: what a kit contains, before anything is installed.

use super::catalog::{self, Kit};
use anyhow::Result;
use std::fmt::Write as _;

pub fn cmd_show(spec: Option<&str>, json: bool) -> Result<()> {
    match spec {
        None => list(json),
        Some(spec) => one(spec, json),
    }
}

fn list(json: bool) -> Result<()> {
    let kits = catalog::bundled()?;
    if json {
        let items: Vec<_> = kits
            .iter()
            .map(|k| {
                let m = &k.manifest.kit;
                serde_json::json!({
                    "name": m.name, "title": m.title, "version": m.version,
                    "description": m.description, "extends": m.extends,
                    "level": k.level.label(),
                })
            })
            .collect();
        let env = crate::envelope(
            "show",
            true,
            serde_json::json!({ "kits": items }),
            None,
            vec![],
        );
        println!("{}", serde_json::to_string_pretty(&env)?);
        return Ok(());
    }
    let width = kits.iter().map(|k| k.name().len()).max().unwrap_or(0);
    for k in &kits {
        println!("{:width$}   {}", k.name(), k.manifest.kit.description);
    }
    println!();
    println!("next      kit show <kit>");
    Ok(())
}

fn one(spec: &str, json: bool) -> Result<()> {
    let chain = catalog::resolve(spec)?;
    if json {
        let env = crate::envelope("show", true, to_json(&chain), None, vec![]);
        println!("{}", serde_json::to_string_pretty(&env)?);
    } else {
        print!("{}", render(&chain)?);
    }
    Ok(())
}

/// The kit (last in `chain`) and what it brings from the kits it extends.
pub fn render(chain: &[Kit]) -> Result<String> {
    let top = chain.last().expect("resolve returns the kit itself");
    let meta = &top.manifest.kit;
    let mut s = String::new();
    let licence = meta.licence.as_deref().unwrap_or("no licence");
    writeln!(
        s,
        "{} {}   {}   {licence}",
        meta.title,
        meta.version,
        top.level.label()
    )?;
    writeln!(s, "{}", meta.description)?;
    if !meta.agents.is_empty() {
        writeln!(s, "for       {}", meta.agents.join(", "))?;
    }
    if !meta.extends.is_empty() {
        let bases: Vec<&str> = chain[..chain.len() - 1].iter().map(Kit::name).collect();
        writeln!(s, "extends   {}", bases.join(", "))?;
    }

    let skills: usize = chain.iter().map(|k| k.manifest.skill.len()).sum();
    writeln!(s)?;
    writeln!(s, "skills    {skills}")?;
    let width = chain
        .iter()
        .flat_map(|k| &k.manifest.skill)
        .map(|sk| sk.name.len())
        .max()
        .unwrap_or(0);
    let origin = |kit: &Kit, sk: &super::manifest::SkillRef| {
        sk.origin()
            .unwrap_or_else(|| format!("{} (in the kit)", kit.name()))
    };
    let from_width = chain
        .iter()
        .flat_map(|k| k.manifest.skill.iter().map(move |sk| origin(k, sk).len()))
        .max()
        .unwrap_or(0);
    for kit in chain {
        if kit.manifest.skill.is_empty() {
            continue;
        }
        if chain.len() > 1 {
            writeln!(s, "  from {}", kit.name())?;
        }
        for sk in &kit.manifest.skill {
            let licence = sk.licence.as_deref().unwrap_or("no licence");
            writeln!(
                s,
                "    {:width$}  {:from_width$}  {licence}",
                sk.name,
                origin(kit, sk)
            )?;
        }
    }

    let rules: Vec<String> = chain
        .iter()
        .filter_map(|k| {
            let r = k.manifest.rules.as_ref()?;
            let lines = k.files.read_to_string(&r.file).ok()?.lines().count();
            Some(format!("{} ({lines} lines)", k.name()))
        })
        .collect();
    if !rules.is_empty() {
        writeln!(s, "rules     {}", rules.join(", "))?;
    }
    for kit in chain {
        for (name, mcp) in &kit.manifest.mcp {
            let code = if mcp.runs_code() {
                "   RUNS CODE"
            } else {
                "   remote"
            };
            writeln!(s, "mcp       {name}   {}{code}", mcp.command_line())?;
        }
        for hook in &kit.manifest.hook {
            let only = hook
                .glob
                .as_deref()
                .map(|g| format!(" ({g})"))
                .unwrap_or_default();
            writeln!(s, "hook      {}{only}   RUNS CODE", hook.on.label())?;
            writeln!(s, "            runs  {}", hook.describe())?;
        }
        if let Some(gate) = &kit.manifest.gate {
            writeln!(
                s,
                "gate      every kit run in the repo (repo installs)   RUNS CODE"
            )?;
            for (_, cmd) in gate.checks() {
                writeln!(s, "            runs  {cmd}")?;
            }
        }
    }
    let checks: Vec<String> = chain
        .iter()
        .flat_map(|k| {
            let c = &k.manifest.check;
            c.mcp_starts
                .iter()
                .map(|m| format!("{m} MCP starts"))
                .chain(c.commands.iter().map(|cmd| format!("`{cmd}` runs")))
        })
        .collect();
    if !checks.is_empty() {
        writeln!(s, "check     {}", checks.join(", "))?;
    }
    let code: usize = chain.iter().map(|k| k.manifest.runs_code()).sum();
    writeln!(s)?;
    if code > 0 {
        writeln!(
            s,
            "Runs code on your machine: {code} (MCP servers, hooks and gate checks). Kit asks before installing them."
        )?;
    }
    writeln!(s, "next      kit add {} --global", top.name())?;
    Ok(s)
}

fn to_json(chain: &[Kit]) -> serde_json::Value {
    let top = chain.last().expect("resolve returns the kit itself");
    let skills: Vec<_> = chain
        .iter()
        .flat_map(|k| {
            k.manifest.skill.iter().map(move |sk| {
                serde_json::json!({
                    "name": sk.name, "from": k.name(), "source": sk.source,
                    "path": sk.path, "rev": sk.rev, "licence": sk.licence,
                })
            })
        })
        .collect();
    let mcp: Vec<_> = chain
        .iter()
        .flat_map(|k| {
            k.manifest.mcp.iter().map(|(name, m)| {
                serde_json::json!({ "name": name, "command": m.command, "args": m.args, "url": m.url })
            })
        })
        .collect();
    let hooks: Vec<_> = chain
        .iter()
        .flat_map(|k| {
            k.manifest
                .hook
                .iter()
                .map(|h| serde_json::json!({ "on": "after_edit", "glob": h.glob, "run": h.run }))
        })
        .collect();
    serde_json::json!({
        "name": top.name(),
        "title": top.manifest.kit.title,
        "version": top.manifest.kit.version,
        "level": top.level.label(),
        "extends": chain[..chain.len() - 1].iter().map(Kit::name).collect::<Vec<_>>(),
        "skills": skills,
        "mcp": mcp,
        "hooks": hooks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_design_shows_its_bases_code_and_next_step() {
        let text = render(&catalog::resolve("frontend-design").unwrap()).unwrap();
        assert!(
            text.starts_with("Frontend Design 0.1.0   Official   MIT\n"),
            "{text}"
        );
        assert!(text.contains("extends   essentials"), "{text}");
        assert!(text.contains("  from essentials\n"), "{text}");
        // Every licence starts in the same column.
        let cols: Vec<usize> = text
            .lines()
            .filter(|l| l.starts_with("    ") && !l.starts_with("     "))
            .filter_map(|l| l.rfind("  ").map(|i| i + 2))
            .collect();
        assert!(
            cols.len() > 3 && cols.iter().all(|c| *c == cols[0]),
            "{text}"
        );
        assert!(
            text.contains("            runs  npx --no-install prettier"),
            "{text}"
        );
        assert!(text.contains("core-web-vitals"), "{text}");
        assert!(
            text.contains("npx -y chrome-devtools-mcp@1.10.1 --no-usage-statistics   RUNS CODE"),
            "{text}"
        );
        assert!(text.contains("Runs code on your machine: 2"), "{text}");
        assert!(
            text.trim_end()
                .ends_with("next      kit add frontend-design --global")
        );
    }

    #[test]
    fn a_kit_without_code_says_nothing_about_code() {
        let text = render(&catalog::resolve("essentials").unwrap()).unwrap();
        assert!(!text.contains("RUNS CODE"), "{text}");
        assert!(!text.contains("extends"), "{text}");
    }
}
