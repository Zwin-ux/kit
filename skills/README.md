# Skill catalog

Shared skills used by starter packs and direct installs.

Each skill is a folder with `SKILL.md`.  
See [docs/SKILL_SCHEMA.md](../docs/SKILL_SCHEMA.md).

## Catalog (v1)

| Skill | Purpose |
|-------|---------|
| `add-readme` | Clear project README |
| `project-setup` | Baseline repo setup |
| `code-review` | Structured change review |
| `write-tests` | Focused tests |
| `fix-bug` | Root-cause bug fixing |
| `pr-ready` | PR summary, test plan, risks |
| `ship-checklist` | Pre-ship checklist for apps |
| `a11y-pass` | Basic accessibility pass |
| `api-docs` | Public API documentation |
| `changelog` | Release notes |
| `cli-help` | CLI usage and help text |
| `data-check` | Data/notebook hygiene |
| `workspace-setup` | Monorepo / workspace map |

## Install

```sh
pnpm kit -- install ./skills/add-readme
pnpm kit -- pack install essentials
```
