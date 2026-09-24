//! Node: package manager from `packageManager` or the lockfile, and only
//! `package.json` scripts that exist and check without writing.

use super::is_writer;
use kit_core::GateConfig;
use serde_json::{Map, Value};
use std::path::Path;
use std::time::Duration;

/// Lockfile → package manager, in priority order.
const LOCKFILES: &[(&str, &str)] = &[
    ("pnpm-lock.yaml", "pnpm"),
    ("yarn.lock", "yarn"),
    ("bun.lockb", "bun"),
    ("bun.lock", "bun"),
    ("package-lock.json", "npm"),
    ("npm-shrinkwrap.json", "npm"),
];

/// Scripts whose name says "check": used unless the body writes files.
const FORMAT_CHECK: &[&str] = &[
    "format:check",
    "check:format",
    "fmt:check",
    "format-check",
    "prettier:check",
];
/// Scripts that may write: used only when the body has a check flag.
const FORMAT_PLAIN: &[&str] = &["format", "fmt", "prettier"];
const TYPECHECK: &[&str] = &[
    "typecheck",
    "type-check",
    "check-types",
    "types:check",
    "check:types",
    "tsc",
];
const LINT: &[&str] = &["lint:check", "lint"];

pub(super) fn detect(repo: &Path, gate: &mut GateConfig, notes: &mut Vec<String>) {
    gate.timeout = Duration::from_secs(10 * 60);
    let raw = std::fs::read_to_string(repo.join("package.json")).unwrap_or_default();
    let Ok(pkg) = serde_json::from_str::<Value>(&raw) else {
        notes.push("package.json is not valid JSON. Fix it, then run kit init again.".into());
        return;
    };
    let empty = Map::new();
    let scripts = pkg
        .get("scripts")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let script = |name: &str| scripts.get(name).and_then(Value::as_str);
    let pm = package_manager(repo, &pkg, notes);
    let run = |name: &str| format!("{pm} run {name}");

    if pkg.get("workspaces").is_some() || repo.join("pnpm-workspace.yaml").is_file() {
        notes.push("Workspaces found. The gate uses the root scripts only.".into());
    }

    // format
    for name in FORMAT_CHECK.iter().chain(FORMAT_PLAIN) {
        let Some(body) = script(name) else { continue };
        let plain = FORMAT_PLAIN.contains(name);
        let has_check = body.contains("--check") || body.contains("--list-different");
        if is_writer(body) || (plain && !has_check) {
            notes.push(format!(
                "Not used: script \"{name}\" ({body}) can change files. Add a \"format:check\" script."
            ));
            continue;
        }
        gate.format = Some(run(name));
        break;
    }

    // typecheck: a script, else tsc --noEmit when TypeScript is installed
    gate.typecheck = TYPECHECK
        .iter()
        .find(|n| script(n).is_some_and(|b| !is_writer(b)))
        .map(|n| run(n));
    if gate.typecheck.is_none() && repo.join("tsconfig.json").is_file() {
        if has_dep(&pkg, "typescript") {
            gate.typecheck = Some(format!("{} tsc --noEmit", exec(pm)));
        } else {
            notes.push(
                "tsconfig.json found, but typescript is not a dependency. No typecheck.".into(),
            );
        }
    }

    // lint goes to extra: it is not a type checker
    for name in LINT {
        let Some(body) = script(name) else { continue };
        if is_writer(body) {
            notes.push(format!(
                "Not used: script \"{name}\" ({body}) fixes files. Add a \"lint:check\" script."
            ));
            continue;
        }
        gate.extra.push(run(name));
        break;
    }

    // test
    match script("test") {
        Some(body) if body.contains("no test specified") => {
            notes.push("Not used: script \"test\" is the npm placeholder.".into());
        }
        Some(body) if is_watch(body) => notes.push(format!(
            "Not used: script \"test\" ({body}) runs in watch mode and does not stop."
        )),
        Some(_) => gate.test = Some(run("test")),
        None => notes.push("No \"test\" script in package.json.".into()),
    }
}

fn package_manager(repo: &Path, pkg: &Value, notes: &mut Vec<String>) -> &'static str {
    let declared = pkg
        .get("packageManager")
        .and_then(Value::as_str)
        .and_then(|s| s.split('@').next())
        .and_then(|name| {
            ["pnpm", "yarn", "bun", "npm"]
                .into_iter()
                .find(|p| *p == name)
        });
    let locks: Vec<&(&str, &str)> = LOCKFILES
        .iter()
        .filter(|(f, _)| repo.join(f).is_file())
        .collect();
    let chosen = declared.or(locks.first().map(|(_, pm)| *pm));
    match (declared, locks.first()) {
        (Some(pm), _) => notes.push(format!("Package manager: {pm} (packageManager field).")),
        (None, Some((file, pm))) => notes.push(format!("Package manager: {pm} ({file}).")),
        (None, None) => notes.push("No lockfile found. Using npm.".into()),
    }
    let pm = chosen.unwrap_or("npm");
    let other: Vec<&str> = locks
        .iter()
        .filter(|(_, p)| *p != pm)
        .map(|(f, _)| *f)
        .collect();
    if !other.is_empty() {
        notes.push(format!(
            "Using {pm}. Lockfile for another package manager: {}. Remove it if you do not use it.",
            other.join(", ")
        ));
    }
    pm
}

/// How to run a local binary with this package manager.
fn exec(pm: &str) -> &'static str {
    match pm {
        "pnpm" => "pnpm exec",
        "yarn" => "yarn",
        "bun" => "bunx",
        _ => "npx",
    }
}

fn has_dep(pkg: &Value, name: &str) -> bool {
    ["dependencies", "devDependencies"]
        .iter()
        .any(|k| pkg.get(k).and_then(|d| d.get(name)).is_some())
}

fn is_watch(body: &str) -> bool {
    body.split(['&', '|', ';']).any(|seg| {
        let t: Vec<&str> = seg.split_whitespace().collect();
        t.iter()
            .any(|w| w.starts_with("--watch") && !w.ends_with("=false"))
            || t.windows(2)
                .any(|w| w[0] == "vitest" && matches!(w[1], "watch" | "dev"))
    })
}
