---
title: Roadmap to High-Quality v1.0
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, current]
tags: [roadmap]
status: planned
---

# Roadmap to High-Quality v1.0

This is the **execution plan** for a shippable 1.0.0 — not an idea list.

## Sub-pages

- [[concepts/roadmap/status-matrix|Status matrix]] — every feature done/partial/planned
- [[concepts/roadmap/workstreams|Workstreams]] — sequenced work packages with owners
- [[concepts/roadmap/release|Release (M5)]] — distribution + launch checklist

## North star kill criteria (must all pass)

1. Clean-machine install 3 OS from README  
2. Cold start &lt;100ms; idle &lt;1% CPU  
3. 8 concurrent runs stable frames  
4. Live `kit run` + TUI dispatch with real agent  
5. Gate catches deliberate failure in demo  
6. Kill works  
7. Receipts documented  
8. `--json` on data commands  

## Phases (summary)

| Phase | Name | Outcome |
|-------|------|---------|
| P0 | Stabilize | Land PR #11; fix Rust CI red |
| P1 | Control plane | Handle registry, kill, retry, timeout |
| P2 | Gate trust | Vacuous policy, inference, demo failure |
| P3 | Concurrency proof | 8-run harness + interleave tests |
| P4 | Polish enough | Theme usable, reduced motion, doctor --json |
| P5 | Ship | npm platform bins, installer, docs rewrite, 1.0.0 tag |

## Explicit deprioritize until after P1–P3

- Mascot animation parity  
- Mouse hit-testing  
- True Board pull-queue  
- Library screen  
- Full 0.1 CLI port  
