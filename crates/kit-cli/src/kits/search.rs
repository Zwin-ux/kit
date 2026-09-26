//! `kit search [WORDS]`: the starter kits and the kit index in one list,
//! best match first. See `docs/dev/DESIGN-MARKETPLACE.md`.

use super::catalog::{self, Level};
use super::index;
use super::install::{home_dir, repo_root};
use super::lock::Lock;
use super::writers::Scope;
use crate::cli::SearchArgs;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

/// One kit as search shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub name: String,
    pub title: String,
    pub description: String,
    pub level: &'static str,
    pub tags: Vec<String>,
}

fn short(level: Level) -> &'static str {
    match level {
        Level::Official => "Official",
        Level::Index => "Index",
        Level::Direct => "Direct",
    }
}

/// How well `row` matches every word; `None` when a word matches nothing.
pub fn score(row: &Row, words: &[String]) -> Option<u32> {
    let name = row.name.to_lowercase();
    let title = row.title.to_lowercase();
    let desc = row.description.to_lowercase();
    let mut total = 0;
    for w in words {
        let w = w.to_lowercase();
        let tag = |exact: bool| {
            row.tags.iter().any(|t| {
                let t = t.to_lowercase();
                if exact { t == w } else { t.contains(&w) }
            })
        };
        let best = if name == w {
            100
        } else if name.starts_with(&w) {
            60
        } else if name.contains(&w) {
            40
        } else if tag(true) {
            35
        } else if title.contains(&w) {
            30
        } else if tag(false) {
            25
        } else if desc.contains(&w) {
            10
        } else {
            return None;
        };
        total += best;
    }
    Some(total)
}

/// Rows matching `words`, best first; every row when `words` is empty.
pub fn rank(rows: &[Row], words: &[String]) -> Vec<Row> {
    let mut hits: Vec<(u32, &Row)> = rows
        .iter()
        .filter_map(|r| score(r, words).map(|s| (s, r)))
        .collect();
    hits.sort_by(|(a, ra), (b, rb)| b.cmp(a).then_with(|| ra.name.cmp(&rb.name)));
    hits.into_iter().map(|(_, r)| r.clone()).collect()
}

/// What search found: the rows, notes to print, where the index came from,
/// and why the index could not be read at all (only the bundled kits show).
struct Found {
    rows: Vec<Row>,
    notes: Vec<String>,
    from: String,
    failed: Option<String>,
}

/// Bundled kits, then index kits whose names are not bundled.
fn rows(refresh: bool) -> Result<Found> {
    let mut rows: Vec<Row> = catalog::bundled()?
        .iter()
        .map(|k| Row {
            name: k.name().to_string(),
            title: k.manifest.kit.title.clone(),
            description: k.manifest.kit.description.clone(),
            level: short(k.level),
            tags: Vec::new(),
        })
        .collect();
    let mut notes = Vec::new();
    let mut failed = None;
    let from = match index::load(refresh) {
        Ok(ix) => {
            notes.extend(ix.warnings.iter().cloned());
            for e in &ix.entries {
                if rows.iter().any(|r| r.name == e.name) {
                    continue;
                }
                rows.push(Row {
                    name: e.name.clone(),
                    title: e.title.clone().unwrap_or_else(|| e.name.clone()),
                    description: e.summary.clone(),
                    level: short(ix.level(e)),
                    tags: e.tags.clone(),
                });
            }
            ix.from
        }
        Err(err) => {
            failed = Some(err.to_string());
            String::new()
        }
    };
    Ok(Found {
        rows,
        notes,
        from,
        failed,
    })
}

/// Names installed for all projects or in this repo.
fn installed() -> BTreeSet<String> {
    let mut scopes = Vec::new();
    if let Ok(home) = home_dir() {
        scopes.push(Scope::Global { home });
    }
    if let Some(root) = repo_root(Path::new(".")) {
        scopes.push(Scope::Repo(root));
    }
    scopes
        .iter()
        .filter_map(|s| Lock::load(s).ok())
        .flat_map(|l| l.kits.into_iter().map(|e| e.name))
        .collect()
}

pub fn cmd_search(args: &SearchArgs, json: bool) -> Result<()> {
    let Found {
        rows: all,
        notes,
        from,
        failed,
    } = rows(args.refresh)?;
    let hits = rank(&all, &args.words);
    let have = installed();
    let query = args.words.join(" ");

    if json {
        let kits: Vec<_> = hits
            .iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.name, "title": r.title, "description": r.description,
                    "level": r.level, "tags": r.tags, "installed": have.contains(&r.name),
                })
            })
            .collect();
        let data = serde_json::json!({ "query": query, "index": from, "kits": kits });
        let env = crate::envelope("search", failed.is_none(), data, failed.clone(), notes);
        println!("{}", serde_json::to_string_pretty(&env)?);
        if failed.is_some() {
            std::process::exit(2);
        }
        return Ok(());
    }

    if hits.is_empty() {
        println!(
            "No kits match \"{query}\". kit search lists all {}.",
            all.len()
        );
        let slug: String = args
            .words
            .first()
            .map(|w| {
                w.to_lowercase()
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                    .collect()
            })
            .unwrap_or_default();
        println!("Make your own: kit new {}", slug.trim_matches('-'));
    } else {
        let nw = hits.iter().map(|r| r.name.len()).max().unwrap_or(0);
        let lw = hits.iter().map(|r| r.level.len()).max().unwrap_or(0);
        let dw = hits.iter().map(|r| r.description.len()).max().unwrap_or(0);
        for r in &hits {
            let mark = if have.contains(&r.name) {
                "   installed"
            } else {
                ""
            };
            let line = format!(
                "{:nw$}   {:lw$}   {:dw$}{mark}",
                r.name, r.level, r.description
            );
            println!("{}", line.trim_end());
        }
        println!();
        println!("next      kit show <kit>   ·   kit add <kit> --global");
    }
    for n in &notes {
        println!("note      {n}");
    }
    if let Some(err) = failed {
        // Exit 2 (could not run): the index was not read; the list above is
        // only the kits that ship with Kit.
        eprintln!("kit: {err}");
        eprintln!("     Only the kits that ship with Kit are listed.");
        std::process::exit(2);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, title: &str, desc: &str, tags: &[&str]) -> Row {
        Row {
            name: name.into(),
            title: title.into(),
            description: desc.into(),
            level: "Index",
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
        }
    }

    fn names(rows: &[Row]) -> Vec<&str> {
        rows.iter().map(|r| r.name.as_str()).collect()
    }

    fn words(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn name_beats_title_beats_tags_beats_description() {
        let rows = [
            row("a-writer", "Docs", "mentions design once", &[]),
            row("apple-design", "Apple Design", "HIG review", &["ios"]),
            row("design", "Design", "exact", &[]),
            row("ui-kit", "Design systems", "x", &[]),
            row("zzz", "Z", "nothing", &["design"]),
        ];
        assert_eq!(
            names(&rank(&rows, &words("design"))),
            ["design", "apple-design", "zzz", "ui-kit", "a-writer"]
        );
    }

    #[test]
    fn every_word_must_match_and_case_is_ignored() {
        let rows = [
            row("apple-design", "Apple Design", "HIG review", &["iOS"]),
            row("android", "Android", "Kotlin", &["mobile"]),
        ];
        assert_eq!(names(&rank(&rows, &words("IOS design"))), ["apple-design"]);
        assert!(rank(&rows, &words("ios kotlin")).is_empty());
        assert_eq!(rank(&rows, &[]).len(), 2, "no words lists everything");
    }

    #[test]
    fn the_starter_kits_are_always_searchable_offline() {
        let found = rows(false).unwrap();
        assert!(found.notes.is_empty(), "{:?}", found.notes);
        assert!(found.failed.is_none());
        let hits = rank(&found.rows, &words("frontend"));
        assert_eq!(names(&hits)[0], "frontend-design");
        assert_eq!(hits[0].level, "Official");
    }
}
