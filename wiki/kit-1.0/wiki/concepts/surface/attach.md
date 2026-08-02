---
title: Attach / Detach
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, current]
tags: [tui, pty]
status: stub
---

# Attach / Detach

PRD: `[a]ttach` takes over the terminal for interactive agents; **Esc detaches without killing**.

## Full concept

| Mode | Terminal ownership | Agent process |
|------|--------------------|---------------|
| Detached (default) | Kit Control Room event loop | Agent stdout piped into RunDelta |
| Attached | PTY connected to agent | Interactive; Kit is thin relay |
| Detach (Esc) | Restore Kit UI | Process **continues** |
| Kill | — | Process ends → Killed |

## Requirements (B2-pty)

1. Cross-platform PTY (Windows ConPTY, Unix pty)
2. Bounded output still captured for receipt while attached (or dual-write)
3. Resize events forwarded to child
4. `q` does **not** quit Kit while attached
5. Only one attached run at a time
6. Attach disabled if agent already exited

## Current stub

`Screen::Attached` shows honest empty state: “PTY not connected yet”.  
`Action::AttachSelected` fires; no process handoff.

## Priority for v1.0

**High for “feels like Codex”**, medium for “gate proof product”.  
Ship without attach if kill + stream + gate solid; mark attach as 1.0.1 if needed.  
Prefer attach for Claude/Grok interactive; Codex exec is often headless-only.
