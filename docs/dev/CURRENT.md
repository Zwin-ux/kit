# Kit 1.0 — Current architecture state

**Date:** 2026-08-16  
**Ground truth:** [`docs/dev/design-package/00-HONEST-STATE.md`](design-package/00-HONEST-STATE.md)  
**Goal:** a widely used, high-quality developer tool — one `kit` verb, one Control Room, a loop people trust. Not a persona studio.

Version: **`1.0.0-alpha.1`**. Not 1.0.0. Not tagged alpha.2 until dogfood notes exist.

---

## Product one-liner

Dispatch many agents. Watch them in one place. Nothing ships unproven — and the product must prove that on itself.

## What is real today

| Layer | Status | Prove it |
|-------|--------|----------|
| Control Room TUI | Real (1.0 craft) | `cargo run -p kit-cli -- --demo` — FAIL selected + wash; `f` cycles ALL/FAIL/RUN/DONE |
| Dispatch / Board / Detail | Real | Board is **prefill only**. Dispatch is repos × agents × **personas** (product/design/eng/qa). Persona is TUI-local — prepended to the engine task, not a kit-core field |
| Gate (Guardian) | Real, thin | ~50 firewall fixtures in one test — not the PRD's 855-case suite |
| **kit.toml (this repo)** | **Real** | root `kit.toml` — fmt + clippy -D warnings + `cargo test --workspace`, 15m, firewall block |
| Worktree + receipt | Real | `kit run --dry-run --json` |
| Agent adapters | Live | codex / claude / grok / ollama — `probe()` reports authenticated if the binary exists |
| Skills injection | Live | `.agents/skills` → worktree + prompt |
| PTY attach | Stub | 1.0.1 (CEO stamp) |
| Kill mid-run | Wired | proven on dry-run handles, **not** yet on a live Codex dogfood |
| Retry fail | Wired | fail-only; gate failure context in new task |
| Max concurrency | Wired + proven | semaphore 8; P3 harness uses a **bare git fixture** (not this repo's gate) |
| Vacuous gate | Wired | empty `kit.toml` → infer cargo/npm **on live only**; dry-run stays vacuous without a file; UI `UNCONFIGURED` |
| JSON envelope | Wired | `schemaVersion: 1` on `run --json` + `doctor --json` |
| Help overlay | Wired | `?` / Esc; arrows move, `k` kills (not j/k) |
| Receipt browser | Wired | `kit receipt list` / `show` |
| Dogfood | **Session A proven** | receipt `01M06A2PXBBH43ZFF3GJ9VQW94` — `gateVacuous: false`, fmt+clippy+test PASS. QA note: `docs/dev/dogfood-notes/2026-08-16-qa.md`. B still open |
| Install / PATH | **In progress (2026-08-16)** | Repo `kit.cmd`/`kit.ps1` + `scripts/use-rust-kit.ps1` on disk. GitHub About/topics/release still sell 0.1 (human-only). No installer. |
| 0.1 Node tree | Legacy, still in repo | `packages/` + Node CI (6 jobs) + keep-alive catalog bot |

## Spine

```
kit (TUI Dispatch or `kit run`)
  → engine::execute
       → git worktree
       → kit-agents::adapter(kind).spawn  (or dry-run)
            → install .agents/skills
            → skills preamble + user task
            → codex exec | claude -p | grok -p | ollama run
            → stream → RunDelta → Control Room
       → kit-gate (kit.toml or live inference)
       → ~/.kit/runs/<id>/receipt.json
```

## How to run

Windows: prefer the repo shims. Bare `kit` on this Windows PATH is still npm `@mzwin/kit@0.1.3` until you dot-source `scripts/use-rust-kit.ps1` or put `target\release` on PATH.

```bash
.\kit.cmd doctor
.\kit.cmd --demo
. .\scripts\use-rust-kit.ps1    # then `kit` is Rust for this session

# cargo remains valid
cargo run -p kit-cli -- doctor
cargo run -p kit-cli -- --demo
cargo run -p kit-cli -- run --dry-run --task "smoke" --json
```

## Specs

- What to do next (full inventory): [`SPEC-next.md`](SPEC-next.md)
- Design package (start here): `docs/dev/design-package/`
- PRD: `docs/dev/PRD-1.0.md`
- Surface: `docs/dev/SPEC-surface-1.0.md`

## Next (do not reorder)

1. Horizon 0 in `SPEC-next.md` / `tasks/todo.md` — I1 GitHub (human), I2 commit when asked, I3–I5 Sessions B–D
2. P1 isolate `CARGO_TARGET_DIR` in worktrees
3. N1 path-filter Node CI
4. U1 FAIL annotation at 60/80 — one TUI slice
5. PTY, installer, alpha.2 — **not before Horizon 0 checkpoint**

**1.0.1:** PTY attach. **1.1:** board pull-queue.
