---
title: Query — What next for high-quality Kit 1.0?
created: 2026-08-01
type: query
---

# What should be done next for a high-quality v1.0?

Grounded in the wiki: [[concepts/roadmap/workstreams]], [[concepts/roadmap/status-matrix]], [[concepts/product/definition-of-done]], [[summaries/current-impl]].

## Short answer

You already have a **credible alpha control room** (TUI + engine + adapters + gate + receipts).  
A **high-quality 1.0.0** is blocked by **control plane + trust + proof + install**, not by more screens.

## Ordered next work

### 1. P0 Stabilize (this week)

- Fix red Rust CI on PR #11 / main  
- Merge surface+engine only when 3-OS green  
- Keep CURRENT.md honest after merge  

### 2. P1 Control plane (highest product leverage)

Without these, Kit is a **launcher**, not a control room:

| Feature | Spec |
|---------|------|
| Handle registry | [[concepts/engine/handle-registry]] |
| Kill | registry + `k` |
| Retry | fail context + new run |
| Timeout kill | [[concepts/Run/bounds]] |
| Max 8 concurrency | queue excess |

### 3. P2 Gate trust (wedge integrity)

| Feature | Spec |
|---------|------|
| Vacuous policy | [[concepts/Gate/vacuous-vs-real]] |
| Optional inference | [[concepts/Gate/kit-toml]] |
| Demo failure | deliberate FAIL path |

### 4. P3 Concurrency proof

- 8-run harness, no interleave, Windows included  
- Maps to PRD M2 kill criterion  

### 5. P4 Completeness (thin)

- `doctor --json`, `receipt show/list`, schemaVersion  
- Dispatch path picker  
- Attach only if time; else 1.0.1  

### 6. P5 Ship

- Platform binaries + npm launcher + curl installer  
- Third-party clean-machine test  
- 60s demo of gate catch  

## What NOT to do next

- Mascot / mouse hit-testing before kill  
- Marketplace / registry revival  
- True Board pull-queue as a gate for 1.0  
- Large contract rewrites without Claude  

## Success definition

All boxes in [[concepts/product/definition-of-done]] checked;  
workstreams P0–P3 green; P4 minimum JSON+receipts; P5 install verified by a third party.
