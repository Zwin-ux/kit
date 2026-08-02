---
title: kit.toml
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, kit-core-config-contract]
tags: [config]
status: shipped
---

# kit.toml

Per-repository configuration, checked into the project. Loaded from **repo root** (not worktree-only copy unless present there).

## Schema (frozen)

```toml
[gate]
format    = "pnpm format:check"   # optional
typecheck = "pnpm typecheck"      # optional
test      = "pnpm test"           # optional
extra     = ["cargo clippy ..."]  # list
timeout   = "5m"                  # whole gate budget

[gate.scope]
allow = ["src/**", "tests/**", "docs/**"]
deny  = [".github/**", "*.lock"]

[firewall]
mode = "block"   # block | warn | off
```

Unknown keys: **rejected** (`deny_unknown_fields`).

## Semantics

| Rule | Behavior |
|------|----------|
| Missing command key | Skipped, not failed |
| Empty gate (all none, no extra) | Vacuous outcome — see [[concepts/Gate/vacuous-vs-real]] |
| Timeout | Shared across all checks |
| Scope | Allow then deny; used for post-run / firewall context |

## Defaults

Useful with **no file**: Kit still runs; gate is vacuous today.  
PRD wants inferred defaults (e.g. detect package.json → npm test) — **planned** for quality 1.0.

## Authorship

Contract changes: Claude-only.
