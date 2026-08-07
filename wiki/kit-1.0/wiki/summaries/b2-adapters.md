---
title: Summary — B2 agent adapters
type: summary
created: 2026-08-01
updated: 2026-08-01
sources: [b2-agent-adapters]
tags: [agents, source]
---

# Summary — B2 agent adapters

Source: `raw/notes/b2-agent-adapters.md`

## Intent

TUI Dispatch should feel like a multi-agent `codex exec` control room: pick agents × repos × task, isolate worktrees, inject skills, stream, gate, receipt.

## Invocation table

| Agent | Headless shape |
|-------|----------------|
| Codex | `codex exec -C wt -s workspace-write --json` |
| Claude | `claude -p` |
| Grok | `grok -p --cwd --always-approve` |
| Ollama | `ollama run $KIT_OLLAMA_MODEL` |

## Skills

Copy `.agents/skills` into worktree; prepend using-agent-skills routing. Env: `KIT_SKILLS_DIR`, `KIT_FULL_AUTO`.

## Defaults

Live when binary on PATH; `--dry-run` for CI. Auto-fallback to dry-run if missing.

## Remaining for “production adapters”

- Process handle registry (kill)
- PTY attach
- Approval UX without always full-auto
- Skill multi-select UI
