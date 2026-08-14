# SWE quality ladder

Short loop. Quality first, flavor second.

| Order | Skill | When |
|---|---|---|
| current | `completeness-qa` | always (you are here) |
| 1 | `write-tests` | any stub or untested public symbol |
| 2 | `fix-bug` | task/commit looks like a bug, or a stub sits on a symbol that already has tests |
| 3 | `code-review` | any public symbol |
| 4a | `a11y-pass` | react / vue / svelte / next in package.json |
| 4b | `cli-help` | package.json `bin` or Cargo `[[bin]]` |
| 4c | `api-docs` | library-shaped (no web framework, or Cargo lib) |
| 4d | `data-check` | `notebooks/` or `pyproject.toml` + `data/` |

Cap: four rows total including `completeness-qa`.

Do not recommend `add-readme` or `project-setup` here — those belong to `kit recommend` / `kit ready`.

## Ranking test-next

Keep: stubs, names in the user task/spec, files in the diff, user-facing handlers/commands/exports.  
Drop: tiny `Display`/`Debug`/mapper impls that are not stubs.  
Never add a symbol the inventory did not emit. Print at most five.
