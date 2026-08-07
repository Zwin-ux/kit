---
title: Principles
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0]
tags: [product]
status: shipped
---

# Principles

These are **product law**. Features that violate them do not ship in 1.0.

## 1. Bounded by default

Every [[concepts/Run/index|Run]] has:

- wall-clock timeout
- output byte cap
- write scope (allow/deny globs)

No unbounded process. No infinite stream buffers in the TUI (display cap too).

**Implementation touchpoints:** `Bounds` contract, engine runner, TUI `OUTPUT_DISPLAY_CAP_BYTES`.

## 2. Proof or it didn't happen

No run reports success without a [[concepts/Gate/index|Gate]] result.  
Every terminal run writes an immutable [[concepts/Receipt|Receipt]] under `~/.kit/runs/<id>/`.

**Open policy:** vacuous gate (empty kit.toml) currently yields `passed: true` with an honest log line — see [[concepts/Gate/vacuous-vs-real|Vacuous vs real gate]].

## 3. Local-first

No Kit server. No required account. No telemetry.  
Auth for agents is **their** login (Codex/Claude/Grok/Ollama), not Kit’s.

## 4. No credential custody

Kit never reads, stores, or copies provider keys.  
Doctor may report “installed / missing CLI” but must not inspect secret files.

## 5. Fails open on Kit’s own bugs

Inherited from Guardian: a Kit defect must not brick real work.  
Firewall parsing errors → allow (or warn), not hard-block the universe.

## 6. The terminal is the product

Default entry is the Control Room TUI. CLI is first-class for automation (`kit run --json`), not a thin afterthought — but the **felt product** is the room.

## Interaction principle (surface)

One interaction language across screens: Esc back, stable selection, footer grammar, no layout thrash on selection or animation.
