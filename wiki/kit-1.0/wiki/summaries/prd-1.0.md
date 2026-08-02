---
title: Summary — PRD 1.0
type: summary
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0]
tags: [product, source]
---

# Summary — PRD 1.0

Source: `raw/notes/prd-1.0.md` (repo `docs/dev/PRD-1.0.md`)

## Takeaways

1. **Job:** Control room for parallel agent work — dispatch many, watch one place, nothing ships unproven.
2. **Primitive:** Everything is a view over [[concepts/Run/index|Run]].
3. **Differentiator:** [[concepts/Gate/index|Gate]] (`GATING` state) + immutable [[concepts/Receipt|Receipt]] — not another session manager.
4. **Principles:** Bounded by default; proof or it didn't happen; local-first; no credential custody; fail open on Kit bugs; terminal is the product.
5. **Cut for 1.0:** marketplace, registry API, social, Workshop editor, services/trading plugins, Ink TUI as ship vehicle.
6. **Milestones M0–M5** each have kill criteria (startup, one run, N concurrent, gate fixtures, board fan-out, clean-machine install).
7. **DoD:** install 3 OS, cold start &lt;100ms, 8 concurrent stable frames, gate fixtures green, reduced motion, `--json`, documented receipts, 60s demo of gate catch.

## What it implies for engineering

- Ship the **run loop** end-to-end before polish (mascot, mouse).
- Adapter isolation: one broken agent must not kill the room.
- Windows in CI from day one (PTY risk).
