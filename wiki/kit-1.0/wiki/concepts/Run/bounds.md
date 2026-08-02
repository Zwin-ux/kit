---
title: Bounds
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, kit-core-run-contract]
tags: [core, security]
status: partial
---

# Bounds

Limits applied to every [[concepts/Run/index|Run]]. Principle: **bounded by default**.

## Fields (contract)

| Field | Meaning | Default (today) |
|-------|---------|-----------------|
| `timeout` | Wall-clock ceiling for agent process | 30m |
| `output_cap_bytes` | Max captured agent output | 8 MiB |
| `write_allow` | Globs agent may write (empty = whole worktree) | empty |
| `write_deny` | Globs never writable (after allow) | empty |

## Behaviors

### Timeout

- Engine must kill agent when exceeded → `Error` or `Killed` with message.
- Gate has its **own** timeout in `kit.toml` (`gate.timeout`).

**Status:** Bounds field exists; hard kill-on-timeout in engine **planned**.

### Output cap

- Engine stops appending when cap hit; `receipt.output_truncated = true`.
- TUI has separate **display** cap (512 KiB) for list scrolling.

### Write scope

- Enforced by firewall/scope checks and (future) agent sandbox flags.
- Codex default sandbox `workspace-write` approximates worktree-only writes.

## CLI / config exposure (v1.0)

Planned flags / kit.toml keys:

```
kit run --timeout 10m --output-cap 4m
```

Per-repo defaults may live under a future `[bounds]` table — **not in frozen config yet** (would be Claude contract change).
