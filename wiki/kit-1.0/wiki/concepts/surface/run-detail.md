---
title: Run Detail
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, current]
tags: [tui]
status: partial
---

# Run Detail

Deep view of one [[concepts/Run/index|Run]].

## Header (fixed 2 lines)

1. `KIT / RUN  repo · agent · task    STATE  GATE`
2. `worktree: …    id: …` (+ flash)

## Panes (one at a time)

| Pane | Content | Empty state |
|------|---------|-------------|
| **stream** | Agent output lines | `(no output yet)` |
| **gate** | Checklist + firewall/scope | “Gate has not run yet” / “No gate result” |
| **diff** | Unified diff text | Active: “when finishes”; terminal: “No file changes” |

Tabs: reverse highlight on active. Keys `1`/`2`/`3`, Tab/Shift-Tab, ←/→.

## Scroll model

- `stream_follow` default true → pin to tail
- ↑↓ / PgUp/PgDn disable follow
- End / `G` re-enable follow
- Home → top, follow off
- Detail does **not** change selected run with arrows

## Keys

| Key | Action |
|-----|--------|
| Esc | Control Room |
| a | Attach (stub → Attached screen + Action) |
| k / r | Kill / Retry seams |
| d | Jump to Dispatch |
| q | Quit |

## Missing for v1.0 quality

- [ ] Load diff from receipt when terminal
- [ ] Auto-scroll performance with multi-MB logs (cap already)
- [ ] Real attach (see [[concepts/surface/attach|Attach]])
- [ ] Gate pane live update during GATING (partial via RunUpdate)
