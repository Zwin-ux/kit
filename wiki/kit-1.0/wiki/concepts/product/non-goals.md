---
title: Non-goals (1.0)
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0]
tags: [product]
status: cut
---

# Non-goals (1.0)

Explicit cuts so they are not relitigated mid-build.

| Cut | Why |
|-----|-----|
| Skill marketplace / publish API | Not load-bearing for control room |
| Durable catalog / Postgres / Railway backend | Cloud product ≠ 1.0 wedge |
| Profiles, following, collections | Social |
| Teams / private registries | Enterprise 2.0 |
| Workshop skill editor | Substrate tooling, not product |
| Services / trading plugins | Trenchwire is separate |
| Ink TUI as ship vehicle | Rewritten in Rust/ratatui |
| Web dashboard as primary UI | Terminal is the product |

## Deferred but not “cut forever”

- Skill multi-select UI (nice; pack injection already works)
- Pixel mascot parity with 0.1 brand
- Full 0.1 JSON command surface parity (only keep what automation needs)

## Rule

If a proposal does not improve **dispatch / watch / gate / receipt** for multi-agent work, it is out of 1.0.
