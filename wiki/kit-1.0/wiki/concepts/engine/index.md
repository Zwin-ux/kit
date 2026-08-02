---
title: Engine
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [current, b2-agent-adapters]
tags: [engine]
status: partial
---

# Engine

The execution path that turns a task into a [[concepts/Receipt|Receipt]].

## Location today

`crates/kit-cli/src/engine/` — production-shaped modules living in CLI until extraction:

| Module | Responsibility |
|--------|----------------|
| `paths` | `KIT_HOME`, runs/, worktrees/ |
| `worktree` | create / diff / clean remove |
| `store` | write receipt + logs; load kit.toml |
| `runner` | orchestrate one run |

## Sub-pages

- [[concepts/engine/pipeline|Pipeline]] — step-by-step execute
- [[concepts/engine/adapters|Adapters]] — kit-agents
- [[concepts/engine/handle-registry|Handle registry]] — kill/retry (planned)

## Invariants

1. Only engine sets Running/Gating/terminal states.
2. Always write receipt before claiming done to user.
3. Gate always produces an outcome object on the success path.
4. Clean worktrees removed; dirty kept.

## Extraction plan (post-1.0 or late 1.0)

Move engine to `kit-core` or `kit-engine` crate when multiple binaries need it — not blocking 1.0 if CLI+TUI share current modules.
