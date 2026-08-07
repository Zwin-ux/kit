---
title: JSON Contract
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0]
tags: [cli, automation]
status: partial
---

# JSON Contract

Automation surface for scripts and CI.

## Principles

1. Stable field names (camelCase or snake_case — **pick one for 1.0 and document**; today `kit run --json` uses camelCase for some fields).
2. `schemaVersion` integer on every payload (adopt from 0.1 PR #7 spirit).
3. No breaking changes without version bump.

## kit run --json (today)

```json
{
  "id": "01…",
  "state": "pass",
  "receiptDir": "…",
  "worktreeRemoved": true,
  "gatePassed": true
}
```

## Target 1.0 envelope

```json
{
  "schemaVersion": 1,
  "command": "run",
  "ok": true,
  "data": { "...": "..." },
  "error": null
}
```

## Required before 1.0.0

- [ ] `schemaVersion` on all JSON outputs
- [ ] `doctor --json`
- [ ] Document in `docs/json-contract.md`
- [ ] Golden tests for JSON shapes
