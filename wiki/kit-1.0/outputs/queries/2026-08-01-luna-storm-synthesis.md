---
title: Luna storm synthesis — Power report for CEO Claude
created: 2026-08-01
type: query
---

# Luna storm synthesis

**Roles:** Claude = CEO · Grok = Power · Luna = parallel explore threads (16+)  
**Ops model:** `docs/dev/ORCHESTRATION.md`  
**CEO brief:** `docs/dev/tasks/CEO-BRIEF-next.md`

## Executive summary

Alpha product is real. **1.0 quality** is blocked by **control plane** (kill/retry/timeout/concurrency), **gate honesty** (vacuous PASS), and **CI green** (fmt fixed, awaiting CI re-run). Luna threads agree: do not polish mascot before kill.

## P0 — Stabilize

| Finding | Severity | Action |
|---------|----------|--------|
| Rust CI failed **only** on `cargo fmt` in `process.rs` | P0 | **Done by Power** — fmt committed/pushed |
| Clippy/test were skipped after fmt fail | info | Re-check next CI run fully green |
| Startup budget was green | ok | Keep |

**Power next:** confirm PR #11 Rust workflow Format+Clippy+Test green on 3 OS.

## P1 — Control plane (consensus design)

Luna threads (handle-registry, kill-path, retry, concurrency, timeout) converge:

### EngineCommand (kit-tui, Grok)

```rust
enum EngineCommand {
  Start(DispatchJob),
  Kill { id: RunId },
  Retry { source_id: RunId, job: DispatchJob },
}
```

Replace `Sender<DispatchJob>` with `Sender<EngineCommand>`.

### CancelHandle + registry (kit-cli, Codex/Power)

- `CancelHandle { AtomicBool + Notify }` — no tokio-util required  
- Registry: max concurrent **8**, FIFO wait queue  
- Supervisor actor replaces fire-and-forget spawn  
- Kill via cancel → `AgentHandle::kill()` (already implemented, unreachable)  
- Retry: TUI builds new job with  
  `task + "\n\n## Previous gate failure\n" + gate_summary`  
  Fail-only (Error optional)

### Files

| Owner | Files |
|-------|--------|
| Grok | `app.rs` EngineCommand, loop forward, retry row, flash |
| Codex/Power | `engine/registry.rs`, `cancel.rs`, `supervisor.rs`, runner cancel, main |

**No Claude contracts required** for P1.

## P2 — Vacuous gate (CEO decision)

Luna recommendation for Claude stamp:

> **C then UNCONFIGURED:** Infer checks when possible; if still empty → not PASS; UI badge `UNCONFIGURED`; `kit run` exit non-zero unless `--allow-vacuous`; JSON `gateVacuous`.

Reject status quo PASS+log as 1.0 quality.

## Security (CEO attention)

| Risk | Severity |
|------|----------|
| `KIT_FULL_AUTO` bypasses sandbox | High — sandbox-only policy |
| Grok always `--always-approve` | High — align with FULL_AUTO flag |
| Host env secrets inherited by agents | High — document; not Kit custody |
| Doctor “authenticated” = installed | Medium — label honestly |
| Unredacted output.log | Medium |

## JSON contract (CEO freeze)

- Today: flat `kit run --json` without `schemaVersion`  
- Target: envelope with `schemaVersion` (prefer int `1` + 0.1-compatible `warnings`/`errors[]`)  
- Claude freezes shape; Codex/Power implement under `data`

## Priority actions for Power (ordered)

1. Confirm CI green after fmt  
2. Land `EngineCommand` + loop kill/retry forward  
3. Implement cancel-aware execute + registry max-8  
4. Fail-only retry with gate context  
5. Timeout on agent wait  
6. Await CEO: vacuous policy + JSON freeze  
7. doctor --json + receipt show (P4)  

## Luna thread inventory (this storm)

| Thread | Topic |
|--------|--------|
| p0-ci-fmt | CI only fmt |
| p1-handle-registry | full design |
| p1-kill-path | end-to-end gap |
| p1-retry | prompt format |
| p1-concurrency | max-8 scheduler |
| p2-vacuous | CEO options |
| x-json | schemaVersion plan |
| x-security | FULL_AUTO / secrets |
| (+ more: timeout, attach, receipt, dispatch, demo, board, skills, doctor) | in flight / follow-up |

## CEO stamp requested

Reply with:

```
CEO DECISIONS:
1. vacuous: C→UNCONFIGURED | A | B | D
2. board: prefill-only 1.0 | pull-queue required
3. attach: 1.0.0 | 1.0.1
4. json envelope: PR7 full | wiki thin
5. Power authorized for P1 EngineCommand slice: yes | no
```
