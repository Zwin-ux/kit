# Harness Paths

## Goal
Make Kit skills reachable by real coding agents.
Show paths clearly. Write only when the user asks.

## Kit locations

| Scope | Path |
|-------|------|
| Global library | `~/.kit/skills/<name>/` |
| Project skills | `./.kit/skills/<name>/` (after `kit pack apply`) |

Override home with `KIT_HOME`.

## Agent harness locations (v0)

| Harness | Personal | Project |
|---------|----------|---------|
| **Claude Code** | `~/.claude/skills/<name>/` | `./.claude/skills/<name>/` |
| **Codex** | `~/.codex/skills/<name>/` | `./.codex/skills/<name>/` (optional) |
| **Grok Build** | `~/.grok/skills/<name>/` | `./.grok/skills/<name>/` (Kit convention) |

Claude Code and Codex paths follow public docs and community practice.  
Grok Build project/personal skill roots are Kit conventions until an official path is published.

## CLI

### Show paths (read-only)

```sh
kit paths
kit paths --dir ./my-app
kit paths --skill add-readme
```

### Link into a harness (Kit → agent)

Default is a **dry-run**. Nothing is written without `--write`.

```sh
# Plan only
kit link --to claude-code
kit link --to all --scope project

# Create links/copies
kit link --to claude-code --write
kit link --to codex --scope personal --write
kit link --to grok-build --mode copy --write --force
```

Source preference:

1. `./.kit/skills/<name>` when present  
2. Global library install path  

Modes:

- `symlink` (default) — falls back to copy if symlink fails  
- `copy` — full folder copy  

### Import from a harness (agent → Kit)

Capture skills already on disk for Claude Code / Codex / Grok into `~/.kit`.

Default is a **dry-run**. Nothing is written without `--write`.

```sh
# Plan only (personal harness dirs)
kit import --from claude-code
kit import --from all

# Install into Kit library
kit import --from claude-code --write
kit import --from codex --scope personal --write --force
kit import --from claude-code --skill my-skill --write
```

Invalid folders (no `SKILL.md` / schema fail) are skipped with a reason.  
Already-installed names skip unless `--force`.

## Recommended flow

```sh
npm i -g @kit-skills/cli
kit init --pack essentials
kit pack apply essentials --dir .
kit paths
kit link --to claude-code --write
# later, pull custom skills back into Kit:
kit import --from claude-code --write
```

## Core API

- `describePaths(options?)`
- `linkSkills(options?)`
- `importSkillsFromHarness(options?)`
- `resolveHarnessSkillsRoot(harness, scope, options)`

## Safety rules

- Never write harness folders without `write: true` / `--write` on **link**
- Never write the Kit library without `write: true` / `--write` on **import**
- Prefer project scope for team repos when linking
- Use `--force` only when replacing an existing target/install is intentional
