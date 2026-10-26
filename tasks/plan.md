# Implementation Plan: B2 agent adapters + skills in TUI

Skills active: `spec-driven-development`, `planning-and-task-breakdown`,
`incremental-implementation`, `test-driven-development`, `api-and-interface-design`.

## Overview

Enable spinning **real** coding agents (Codex / Claude / Grok / Ollama) from the
Control Room Dispatch form, with the **addyosmani agent-skills** pack injected
into every worktree and prompt — Codex-like headless workflow, multi-agent.

## Architecture decisions

1. **Implementations live in `kit-agents`** against the frozen `Agent` trait.
2. **Skills are filesystem + prompt**, not a new MCP protocol (v1).
3. **Default live when binary on PATH**; `--dry-run` for CI and offline.
4. **Codex sandbox default `workspace-write`**; full auto only via `KIT_FULL_AUTO=1`.
5. **One runner** (`kit-cli` engine) calls adapters; TUI already forwards Dispatch jobs.

## Tasks

### Phase 1 — Skills pack
- [x] Spec in `docs/dev/tasks/B2-agent-adapters.md`
- [x] `kit-agents::skills` resolve + install into worktree + build preamble

### Phase 2 — Adapters
- [x] Shared process handle + streaming stdout/stderr → `RunDelta::Output`
- [x] Codex adapter (`codex exec … --json`)
- [x] Claude adapter (`claude -p`)
- [x] Grok adapter (`grok -p --cwd --always-approve`)
- [x] Ollama adapter (`ollama run`)
- [x] `probe()` via `--version`

### Phase 3 — Wire engine + TUI defaults
- [x] `execute()` calls adapter when live
- [x] Auto live if installed; dry-run fallback
- [x] TUI Dispatch uses live path
- [x] `kit doctor` probes all four
- [x] `kit run` defaults live (`--dry-run` opt-in)

### Phase 4 — Docs / verify
- [x] CURRENT.md + README
- [ ] cargo test + clippy
- [ ] Manual: doctor + dry-run still green

## Risks

| Risk | Mitigation |
|------|------------|
| Agent blocks on approvals | `KIT_FULL_AUTO`; document sandbox |
| Windows .cmd path | `Command::new` uses PATH |
| Huge skill copy cost | copy only if missing; optional env to skip |
| JSONL noise in stream | pass through; pretty later |

## Open questions

- Skill multi-select in Dispatch UI → later (F4.1)
- Attach/PTY → B2-pty

## Proposed follow-on: Kit Red (2026-10-01)

Independent of the existing B2 plan above. Planning only; no implementation started.
- [Product specification](../docs/superpowers/specs/2026-10-01-kit-red.md)
- [Phased implementation plan](../docs/superpowers/plans/2026-10-01-kit-red.md)

Updated scope: [Kit Red versioned build pack](../docs/kit-red/README.md) is authoritative: V1 visual-only first, V2 Kit inside Claude second.

2026-10-01 correction: user rejected expanded Kit Red concepts/plans. Do not execute the proposed A–D/V1–V2 tasks above. Follow docs/kit-red/README.md: native Claude with only a subtle accent/mark; expanded integration parked.

### Current Kit mod slice — 2026-10-01

Concept direction provisionally approved. Current implementation and proof requirements: [MOD-BUILD](../docs/kit-red/MOD-BUILD.md) and [HANDOFF](../docs/kit-red/HANDOFF.md). The small mod is implemented under `mods/kit-red`; three offline scenario tests and plugin validation pass. Native execution remains blocked by the remotely disabled hooks rollout. Keep Claude conversation/prompt, four real coding roles, and no visible stage/progress labels. This supersedes earlier Kit Red summaries only; the existing B2/engine roadmap is unchanged.

### Kit mod continuation evidence — native runner available

Five local scenarios, three native Claude tests and plugin validation pass on 2.1.287. Real headless `/kit frontend` loads successfully with zero model turns/API cost. Fox and role assets refined; exact-code preview is docs/kit-red/production-sprites.html. The old remote rollout error stopped recurring; physical Terminal painting/focus inspection remains blocked by computer-control app restrictions. Read the current MOD-BUILD/HANDOFF rather than older blockers or rejected scopes.
