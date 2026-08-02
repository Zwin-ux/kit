---
title: Job to be Done
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0]
tags: [product]
status: planned
---

# Job to be Done

## Primary user

A developer who already runs **two or more** coding agents daily (Codex, Claude Code, Grok Build, sometimes Ollama), across real git repos. They are authenticated with those tools already. They do not want a new model or editor.

## Jobs

| Priority | Job | When it hurts |
|----------|-----|----------------|
| P0 | See all agent work in one place | 4–6 terminals, lost which branch is which |
| P0 | Stop a runaway agent | Agent loops or destroys files |
| P0 | Trust “done” | Agent claims done; `tsc` fails at review |
| P1 | Fan out one task to N repos | Same fix across monorepos / sister projects |
| P1 | Replay what changed | Diff archaeology after the fact |
| P2 | Self-correct agents | Retry with gate failure context |

## Status quo alternatives

- tmux + memory
- Manual `git worktree` juggling
- Hope + CI later
- Single-agent CLIs without orchestration

## Wedge (why Kit wins)

Session managers exist. **Nothing enforces definition of done** before the run reports success. Kit’s gate is the wedge; adapters and TUI are distribution and UX for that wedge.

## First user

The author (Zwin). Dogfood criterion: Kit must make **building Kit with grok/codex/claude** better within a week, or the product has failed.

## Acceptance for JTBD (v1.0)

- [ ] User can run ≥2 agents concurrently and name state of each without leaving Control Room
- [ ] User can kill a run without hunting PIDs
- [ ] A broken typecheck is shown as FAIL with first error line before “done”
- [ ] User can open a receipt folder and understand outcome without Kit UI
