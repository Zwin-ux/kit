---
title: Status Matrix
type: concept
created: 2026-08-01
updated: 2026-08-01
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
| Run lifecycle states | P | kill/retry incomplete |
| Bounds timeout enforce | L | field exists |
| Worktree isolation | S | |
| Receipt write | S | docs incomplete |
| Gate engine | S | fixtures green |
| Vacuous gate policy | P | needs product decision |
| Firewall | S | |
| Skills injection | S | multi-select L |
| Agent adapters | S | auth probe weak |
| Handle registry | L | blocks kill |

## Surfaces

| Surface | Status | Notes |
|---------|--------|-------|
| Control Room | S | |
| Run Detail | P | attach stub |
| Dispatch | P | labels not path browser |
| Board | P | prefill only |
| Attach/PTY | L | stub screen |
| Doctor CLI | S | --json L |
| Doctor TUI | L | |
| Library | L / optional | |

## Platform

| Item | Status |
|------|--------|
| Event loop / clock | S |
| 3-OS Rust CI | P (PR checks failing — fix P0) |
| Startup budget CI | S (ubuntu) |
| npm platform packages | L |
| curl installer | L |
| README 1.0-first | P |
| Demo recording | L |

## Quality gates

| Item | Status |
|------|--------|
| insta snapshots TUI | S |
| kit-gate fixtures | S |
| Engine dry-run test | S |
| Live adapter integration test | L (mock binary) |
| 8-concurrent proof | L |
| Reduced motion | P |
