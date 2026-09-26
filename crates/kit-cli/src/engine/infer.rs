//! Gate detection: one source of truth for `kit init` and live-run inference.
//!
//! [`detect`] reads files only (no PATH, no processes) and proposes checkers,
//! never writers. `kit init` writes the proposal to `kit.toml`; a live run with
//! no gate uses [`infer_gate`], which is the same proposal minus commands whose
//! program is not on PATH. A wrong inferred command is worse than no check.

mod node;
mod python;
#[cfg(test)]
pub(crate) mod tests;

use kit_core::GateConfig;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// The root toolchain the gate was built for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toolchain {
    Rust,
    Go,
    Node,
    Python,
}

impl Toolchain {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Node => "node",
            Self::Python => "python",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Go => "Go",
            Self::Node => "Node",
            Self::Python => "Python",
        }
    }
}

/// What [`detect`] found in a repo.
#[derive(Debug, Clone)]
pub struct Detection {
    pub toolchain: Option<Toolchain>,
    /// The file that selected the toolchain, e.g. `Cargo.toml`.
    pub marker: Option<String>,
    pub gate: GateConfig,
    /// Why a candidate was used or left out, in plain words.
    pub notes: Vec<String>,
    /// Other projects in the repo that are not in the gate.
    pub skipped: Vec<String>,
}

/// Root markers in priority order: the first match is the root toolchain.
const MARKERS: &[(Toolchain, &[&str])] = &[
    (Toolchain::Rust, &["Cargo.toml"]),
    (Toolchain::Go, &["go.mod"]),
    (Toolchain::Node, &["package.json"]),
    (
        Toolchain::Python,
        &[
            "pyproject.toml",
            "setup.cfg",
            "setup.py",
            "requirements.txt",
            "Pipfile",
        ],
    ),
];

/// Folders never searched for nested projects.
const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "vendor",
    "dist",
    "build",
    "venv",
    "__pycache__",
];

/// Propose a gate from repo files. Reads files only; runs nothing.
pub fn detect(repo: &Path) -> Detection {
    let mut det = Detection {
        toolchain: None,
        marker: None,
        gate: GateConfig::default(),
        notes: Vec::new(),
        skipped: Vec::new(),
    };
    for (tc, files) in MARKERS {
        let Some(found) = files.iter().find(|f| repo.join(f).is_file()) else {
            continue;
        };
        if det.toolchain.is_some() {
            det.skipped
                .push(format!("{found} ({}) at the repo root", tc.title()));
            continue;
        }
        det.toolchain = Some(*tc);
        det.marker = Some((*found).to_string());
    }
    nested_projects(repo, &mut det.skipped);

    let (gate, notes) = (&mut det.gate, &mut det.notes);
    match det.toolchain {
        Some(Toolchain::Rust) => {
            gate.format = Some("cargo fmt --all --check".into());
            gate.typecheck = Some("cargo clippy --workspace --all-targets -- -D warnings".into());
            gate.test = Some("cargo test --workspace".into());
            gate.timeout = Duration::from_secs(15 * 60);
        }
        Some(Toolchain::Go) => {
            // `gofmt -l` exits 0 even when it lists files; `-d` exits 1 on a diff
            // and needs no shell pipe, so it works under cmd and sh.
            gate.format = Some("gofmt -d .".into());
            gate.typecheck = Some("go vet ./...".into());
            gate.test = Some("go test ./...".into());
            gate.timeout = Duration::from_secs(10 * 60);
        }
        Some(Toolchain::Node) => node::detect(repo, gate, notes),
        Some(Toolchain::Python) => python::detect(repo, gate, notes),
        None => {}
    }
    det
}

/// Live-run inference: the [`detect`] proposal, minus commands whose program
/// is not on PATH. Returns an empty gate when nothing is safe.
pub fn infer_gate(repo: &Path) -> GateConfig {
    let mut gate = detect(repo).gate;
    let runnable = |c: &String| on_path(program(c));
    gate.format = gate.format.filter(runnable);
    gate.typecheck = gate.typecheck.filter(runnable);
    gate.test = gate.test.filter(runnable);
    gate.extra.retain(runnable);
    gate
}

/// The gate a live run uses when kit.toml names no format, typecheck or
/// test check of its own and every `extra` command is one a kit added
/// (`kit_added`, from Kit's own record): the inferred checks first, then
/// the kits' commands. A kit's checks add to the inferred ones, never
/// replace them; a gate the user wrote runs exactly as written. `None`
/// when that gate stands as it is or nothing can be inferred.
pub fn with_inferred(gate: &GateConfig, repo: &Path, kit_added: &[String]) -> Option<GateConfig> {
    if gate.format.is_some() || gate.typecheck.is_some() || gate.test.is_some() {
        return None;
    }
    if !gate.extra.iter().all(|c| kit_added.contains(c)) {
        return None;
    }
    let inferred = infer_gate(repo);
    if inferred.is_empty() {
        return None;
    }
    let mut out = gate.clone();
    out.format = inferred.format;
    out.typecheck = inferred.typecheck;
    out.test = inferred.test;
    out.extra = inferred.extra;
    for c in &gate.extra {
        if !out.extra.contains(c) {
            out.extra.push(c.clone());
        }
    }
    if gate.timeout == GateConfig::default().timeout {
        out.timeout = inferred.timeout;
    }
    Some(out)
}

/// The line a live run prints when kit.toml's `extra` lines are ones the
/// repo's committed kit.lock (`team_added`) says a kit added, but this
/// machine's record (`kit_added`) does not have: a teammate's clone or CI.
/// A gate the user wrote gets no line. The team copy only picks the hint.
pub fn inference_note(
    gate: &GateConfig,
    repo: &Path,
    kit_added: &[String],
    team_added: &[String],
) -> Option<String> {
    let named = gate.format.is_some() || gate.typecheck.is_some() || gate.test.is_some();
    let unrecorded: Vec<&String> = gate
        .extra
        .iter()
        .filter(|c| !kit_added.contains(c))
        .collect();
    if named
        || unrecorded.is_empty()
        || !unrecorded.iter().all(|c| team_added.contains(c))
        || infer_gate(repo).is_empty()
    {
        return None;
    }
    Some(
        "gate: runs kit.toml as written, with no inferred format/typecheck/test checks (no Kit record of these extras on this machine; `kit add <kit>` restores them)\n"
            .into(),
    )
}

/// Programs the gate needs that are not on PATH, in check order, no repeats.
pub fn missing_programs(gate: &GateConfig) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (_, cmd) in gate.checks() {
        let p = program(cmd);
        if !out.iter().any(|o| o == p) && !on_path(p) {
            out.push(p.to_string());
        }
    }
    out
}

/// True when outcome is a vacuous pass (no checks, no violations) — CEO: render UNCONFIGURED.
pub fn is_vacuous(outcome: &kit_core::GateOutcome) -> bool {
    outcome.passed
        && outcome.checks.is_empty()
        && outcome.scope_violations.is_empty()
        && outcome.firewall_blocks.is_empty()
}

/// True when a command changes files instead of checking them: a write/fix
/// flag, or a formatter called without a check flag.
pub fn is_writer(body: &str) -> bool {
    const WRITE_FLAGS: &[&str] = &["--write", "-w", "--fix", "--apply", "--apply-unsafe"];
    const CHECK_FLAGS: &[&str] = &["--check", "-c", "--list-different", "-l", "--diff", "-d"];
    const FORMATTERS: &[&[&str]] = &[
        &["prettier"],
        &["black"],
        &["rustfmt"],
        &["gofmt"],
        &["cargo", "fmt"],
        &["go", "fmt"],
        &["ruff", "format"],
        &["biome", "format"],
        &["dprint", "fmt"],
        &["deno", "fmt"],
    ];
    body.split(['&', '|', ';']).any(|segment| {
        let toks: Vec<&str> = segment.split_whitespace().collect();
        if toks
            .iter()
            .any(|t| WRITE_FLAGS.contains(t) || t.starts_with("--fix="))
        {
            return true;
        }
        let formats = FORMATTERS
            .iter()
            .any(|f| toks.windows(f.len()).any(|w| w == *f));
        formats && !toks.iter().any(|t| CHECK_FLAGS.contains(t))
    })
}

/// First word of a command: the program the gate starts.
pub fn program(command: &str) -> &str {
    command.split_whitespace().next().unwrap_or("")
}

/// Projects one folder down that the root gate does not cover.
fn nested_projects(repo: &Path, skipped: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(repo) else {
        return;
    };
    let mut dirs: Vec<_> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('.') && !IGNORED_DIRS.contains(&n.as_str()))
        .collect();
    dirs.sort();
    for dir in dirs {
        for (tc, files) in MARKERS {
            if let Some(f) = files.iter().find(|f| repo.join(&dir).join(f).is_file()) {
                skipped.push(format!("{dir}/{f} ({})", tc.title()));
                break;
            }
        }
        if skipped.len() >= 10 {
            break;
        }
    }
}

pub fn on_path(bin: &str) -> bool {
    if bin.is_empty() {
        return false;
    }
    // Prefer a real lookup; avoid shelling for speed and sandbox friendliness.
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            if dir.join(bin).is_file() {
                return true;
            }
            #[cfg(windows)]
            for ext in ["exe", "cmd", "bat", "com"] {
                if dir.join(format!("{bin}.{ext}")).is_file() {
                    return true;
                }
            }
        }
    }
    // Fallback: try spawning (covers Windows App Paths / PATHEXT edge cases).
    Command::new(bin)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
