---
title: Definition of Done (1.0.0)
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0]
tags: [quality, release]
status: planned
---

# Definition of Done (1.0.0)

Ship bar for **version 1.0.0** (not alpha). Derived from PRD §10.

## Product

- [ ] Clean-machine install works on **macOS, Linux, Windows** from README, by someone who is not the author
- [ ] Default command opens Control Room; documented keys work
- [ ] `kit run` one live agent produces worktree (or clean remove), stream, receipt
- [ ] Gate FAIL surfaces first error in Control Room
- [ ] Retry appends gate failure context (when implemented)
- [ ] Kill stops a live run (when implemented)
- [ ] 60-second demo: gate catches a real failure

## Performance

- [ ] Cold start &lt; **100ms** (binary to first paint / ready)
- [ ] Idle CPU &lt; **1%** (empty or quiet room)
- [ ] **Eight concurrent runs** hold stable frame rate, no output interleaving

## Quality / contracts

- [ ] Guardian fixture suite green against kit-gate
- [ ] Reduced motion: `NO_COLOR`, `KIT_MOTION=off`
- [ ] Usable without color
- [ ] Typed errors; every error names the fix
- [ ] `--json` on every data-producing command
- [ ] Receipts stable + documented for third-party readers
- [ ] No credential custody (manual + automated checks)

## Docs

- [ ] README is 1.0-first (0.1 clearly legacy if kept)
- [ ] Architecture / CURRENT accurate on ship day
- [ ] Receipt schema documented

## Explicitly not required for 1.0.0

- Pixel-perfect mascot
- Mouse hit-testing (keyboard-complete is enough)
- Full Board orchestrator (fan-out Dispatch is enough if Board is “queue UI”)
- Marketplace
