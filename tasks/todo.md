# Kit surface / ship todo

## Done (main / PR #11)

- [x] F1–F3 Control Room + run detail + attach stub
- [x] Dispatch + Board prefill + theme craft
- [x] M1 engine: worktree → stream → gate → receipt
- [x] Live adapters + skills inject
- [x] P1 kill/retry/timeout/max-8
- [x] P2 vacuous → UNCONFIGURED
- [x] P3 concurrency proof
- [x] P4 JSON envelope (`run` + `doctor`)
- [x] Land PR #11 on main
- [x] Product polish: README 1.0-first, demo FAIL, agent strip
- [x] Optional Harness / `skills/` packs
- [x] `kit receipt list|show` (+ `--json`, `--output`)

## Horizon 0 — stabilize truth

- [ ] Dogfood day: multi-agent fan-out on Kit (checklist in `docs/dev/DOGFOOD.md`)
- [ ] Tag `v1.0.0-alpha.2` after dogfood notes
- [ ] Fix dogfood bugs as they appear

## Horizon 1 — 1.0.0 trust box

- [ ] Gate inference hardening + documented PASS meaning
- [ ] Dispatch real multi-repo paths (not fixture names only)
- [ ] Filter `f` ALL/FAIL/RUNNING/DONE
- [ ] npm platform binaries + release CI
- [ ] curl installer + checksums
- [ ] 60s demo GIF (FAIL → retry)
- [ ] Third-party clean-machine install
- [ ] Tag **v1.0.0**

## Horizon 2 — 1.0.1

- [ ] B2-pty attach (Esc detach without kill)
- [ ] Optional skill multi-select on Dispatch

## Horizon 3 — 1.1

- [ ] Board pull-queue / persistence
- [ ] Cross-restart recovery
- [ ] Gate modes strict/warn/off

## Explicit non-goals (still)

- Marketplace / registry / accounts
- Services / trading plugins
- Mascot-as-product before dogfood

## Proposed: Kit Red (2026-10-01)

See [spec](../docs/superpowers/specs/2026-10-01-kit-red.md) and [implementation plan](../docs/superpowers/plans/2026-10-01-kit-red.md). Preserve the existing roadmap above; these are new proposed slices.

- [ ] A: Verify mod surfaces; ship red visual layer and standalone Kit palette.
- [ ] B: Connect installed kits, readiness, receipts, and honest error/proof states.
- [ ] C: Prove reviewed management and agent lifecycle; add run/retry/stop/land where supported.
- [ ] D: Verify real workflows, publish MIT plugin, document compatibility and exact removal.

Updated scope: [Kit Red versioned build pack](../docs/kit-red/README.md) is authoritative: V1 visual-only first, V2 Kit inside Claude second.

2026-10-01 correction: user rejected expanded Kit Red concepts/plans. Do not execute the proposed A–D/V1–V2 tasks above. Follow docs/kit-red/README.md: native Claude with only a subtle accent/mark; expanded integration parked.

### Current Kit mod slice — 2026-10-01

Concept direction provisionally approved. Current implementation and proof requirements: [MOD-BUILD](../docs/kit-red/MOD-BUILD.md) and [HANDOFF](../docs/kit-red/HANDOFF.md). The small mod is implemented under `mods/kit-red`; three offline scenario tests and plugin validation pass. Native execution remains blocked by the remotely disabled hooks rollout. Keep Claude conversation/prompt, four real coding roles, and no visible stage/progress labels. This supersedes earlier Kit Red summaries only; the existing B2/engine roadmap is unchanged.

### Kit mod continuation evidence — native runner available

Five local scenarios, three native Claude tests and plugin validation pass on 2.1.287. Real headless `/kit frontend` loads successfully with zero model turns/API cost. Fox and role assets refined; exact-code preview is docs/kit-red/production-sprites.html. The old remote rollout error stopped recurring; physical Terminal painting/focus inspection remains blocked by computer-control app restrictions. Read the current MOD-BUILD/HANDOFF rather than older blockers or rejected scopes.
