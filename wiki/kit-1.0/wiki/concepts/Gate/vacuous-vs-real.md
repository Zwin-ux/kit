---
title: Vacuous vs Real Gate
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [prd-1.0, current]
tags: [policy]
status: partial
---

# Vacuous vs Real Gate

## Definitions

| Kind | Condition | `GateOutcome.passed` today |
|------|-----------|----------------------------|
| **Real** | ≥1 configured check ran (or skipped intentionally) | true iff all non-skipped pass |
| **Vacuous** | No checks configured (`gate.is_empty()`) | `true` via `GateOutcome::vacuous()` |

Today engine logs: *“vacuous — not a substitute for CI”*.

## Product tension

- **PRD:** no success without proof; also “useful before configured”.
- **Risk:** users treat vacuous PASS as green light.

## Options for high-quality 1.0 (decide one)

| Option | Behavior | Pros | Cons |
|--------|----------|------|------|
| A. Vacuous = PASS + WARN badge | Keep pass; UI shows `GATE ~` | Soft onboarding | Weak proof story |
| B. Vacuous = ERROR until configured | Block Pass | Strong proof | Friction first run |
| C. Infer defaults | Detect npm/cargo → real checks | Best of both | Heuristics bugs |
| D. Vacuous PASS only in dry-run | Live requires kit.toml | Clear | Surprising |

**Recommendation for 1.0 quality:** **C then A fallback** — infer common defaults; if still empty, show WARN state `UNCONFIGURED` (not PASS) in Control Room, exit code non-zero for `kit run` unless `--allow-vacuous`.

## Acceptance once decided

- [ ] Spec written into PRD/CURRENT
- [ ] UI column distinguishes PASS vs UNCONFIGURED
- [ ] `kit run --json` exposes `gatePassed` + `gateVacuous`
- [ ] Demo script uses a real failing gate
