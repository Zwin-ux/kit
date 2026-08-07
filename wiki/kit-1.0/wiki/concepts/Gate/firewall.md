---
title: Firewall
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, current]
tags: [security]
status: shipped
---

# Firewall

Blast-radius screening for shell commands, ported from Guardian’s `guard.js`.

## Threat model

Catches **obvious catastrophic mistakes** from an agent (e.g. destructive paths, network exfil patterns), **not** a human deliberately bypassing Kit. Biases toward allow; **fails open** on Kit parse bugs.

## Modes

| Mode | Behavior |
|------|----------|
| `block` | Refuse command (default) |
| `warn` | Log + allow |
| `off` | No screening |

## Where it applies

- Gate implementation screens commands it runs
- Future: agent subprocess wrapper for all agent-invoked shells (deeper integration)

## What is not the firewall

- OS sandbox (Codex `-s workspace-write` is separate)
- Network policy full isolation
- Credential scanning of agent output (nice-to-have, not 1.0 core)

## Status

Implemented in kit-gate with fixture suite. Product docs for “why blocked” in UI **partial** (first failure line via gate checks; firewall blocks list on GateOutcome).
