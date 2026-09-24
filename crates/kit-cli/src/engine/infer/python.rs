//! Python: ruff, mypy and pytest, each only when the repo configures it in
//! pyproject.toml, setup.cfg, a tool config file or a requirements file.

use kit_core::GateConfig;
use std::path::Path;
use std::time::Duration;

/// Lockfile → command prefix that runs tools in the project environment.
const RUNNERS: &[(&str, &str)] = &[
    ("uv.lock", "uv run "),
    ("poetry.lock", "poetry run "),
    ("pdm.lock", "pdm run "),
    ("Pipfile.lock", "pipenv run "),
];

/// Text Kit reads to find tool config and dependencies.
struct Sources {
    pyproject: String,
    setup_cfg: String,
    /// requirements*.txt, Pipfile and tox.ini joined.
    deps: String,
}

pub(super) fn detect(repo: &Path, gate: &mut GateConfig, notes: &mut Vec<String>) {
    gate.timeout = Duration::from_secs(10 * 60);
    let read = |name: &str| std::fs::read_to_string(repo.join(name)).unwrap_or_default();
    let mut deps = format!("{}\n{}", read("Pipfile"), read("tox.ini"));
    if let Ok(entries) = std::fs::read_dir(repo) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with("requirements") && name.ends_with(".txt") {
                deps.push('\n');
                deps.push_str(&read(&name));
            }
        }
    }
    let src = Sources {
        pyproject: read("pyproject.toml"),
        setup_cfg: read("setup.cfg"),
        deps,
    };
    let has_file = |names: &[&str]| names.iter().any(|n| repo.join(n).is_file());

    let prefix = RUNNERS
        .iter()
        .find(|(lock, _)| repo.join(lock).is_file())
        .map(|(lock, p)| {
            notes.push(format!("Tools run with `{}` ({lock}).", p.trim()));
            *p
        })
        .unwrap_or("");

    let ruff = has_file(&["ruff.toml", ".ruff.toml"])
        || src.pyproject.contains("[tool.ruff")
        || src.mentions("ruff");
    let black = src.pyproject.contains("[tool.black") || src.mentions("black");
    let mypy = has_file(&["mypy.ini", ".mypy.ini"])
        || src.pyproject.contains("[tool.mypy")
        || src.setup_cfg.contains("[mypy")
        || src.mentions("mypy");
    let pytest = has_file(&["pytest.ini", "conftest.py"])
        || src.pyproject.contains("[tool.pytest")
        || src.setup_cfg.contains("[tool:pytest")
        || src.deps.contains("[pytest]")
        || src.mentions("pytest");

    if ruff {
        gate.format = Some(format!("{prefix}ruff format --check"));
        gate.extra.push(format!("{prefix}ruff check"));
    } else if black {
        gate.format = Some(format!("{prefix}black --check ."));
    }
    if mypy {
        gate.typecheck = Some(format!("{prefix}mypy ."));
    }
    if pytest {
        gate.test = Some(format!("{prefix}pytest"));
    }
    if !(ruff || black || mypy || pytest) {
        notes.push(
            "No ruff, black, mypy or pytest config in pyproject.toml, setup.cfg or requirements files."
                .into(),
        );
    }
}

impl Sources {
    /// True when a requirements line or a pyproject dependency names `tool`.
    fn mentions(&self, tool: &str) -> bool {
        // "ruff" must not match "ruff-lsp" or "pytest" match "pytest-cov".
        let ends_name = |rest: &str| {
            !rest.starts_with(|c: char| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        };
        // requirements / Pipfile / Poetry style: the line starts with the name.
        let line_dep = |text: &str| {
            text.lines()
                .any(|l| l.trim().strip_prefix(tool).is_some_and(ends_name))
        };
        // PEP 621 / dependency-groups style: a quoted string starts with it.
        let quoted_dep = |text: &str| {
            ['"', '\''].iter().any(|q| {
                text.match_indices(&format!("{q}{tool}"))
                    .any(|(i, m)| ends_name(&text[i + m.len()..]))
            })
        };
        line_dep(&self.deps) || line_dep(&self.pyproject) || quoted_dep(&self.pyproject)
    }
}
