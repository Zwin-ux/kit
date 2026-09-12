---
title: Status Matrix
type: concept
created: 2026-08-01
updated: 2026-08-13
sources: [current, prd-1.0]
tags: [roadmap]
status: partial
---

# Status Matrix

Legend: **S** shipped · **P** partial · **L** planned · **C** cut

## Core concepts

| Concept | Status | Notes |
|---------|--------|-------|
| Run model | S | contract + engine |
| Run lifecycle states | P | kill/retry wired; live kill unproven |
| Bounds timeout enforce | P | timeout maps to Killed on the agent; gate has its own ceiling |
| Worktree isolation | S | |
| Receipt write | S | `kit receipt list/show` |
| Gate engine | S | ~50 firewall fixtures, not 855 |
| Vacuous gate policy | S | UNCONFIGURED in TUI; live exit 1; dry-run exempt unless `kit.toml` |
| Firewall | S | |
| Skills injection | S | multi-select L |
| Agent adapters | S | auth probe is “binary exists” |
| Handle registry | S | in-memory; kill uses it |
| **kit.toml (this repo)** | **S** | 2026-08-13 — fmt/clippy/test |

## Surfaces

| Surface | Status | Notes |
|---------|--------|-------|
| Control Room | S | |
| Run Detail | P | attach stub |
| Dispatch | P | labels + hardcoded siblings, not a path browser |
| Board | P | prefill only |
| Attach/PTY | L | stub screen |
| Doctor CLI | S | `--json` shipped |
| Filter `f` | S | ALL → FAIL → RUN → DONE |
| Doctor TUI | L | |
| Library | L / optional | |

## Platform

| Item | Status |
|------|--------|
| Event loop / clock | S |
| 3-OS Rust CI | S (last merge #13 green) |
| Startup budget CI | S (`kit --version`, not TUI paint) |
| npm platform packages | L |
| curl installer | L |
| README 1.0-first | P (0.1 pages still below the fold) |
| Demo recording | L |

## Quality gates

| Item | Status |
|------|--------|
| insta snapshots TUI | S |
| kit-gate fixtures | S |
| Engine dry-run test | S |
| Live adapter integration test | L (mock binary) |
| 8-concurrent proof | S (dry-run / bare fixture) |
| Reduced motion | P |
| Dogfood | P |
