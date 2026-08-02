# Kit 1.0 — Product Requirements

**Status:** Draft for build. Supersedes `ROADMAP.md` "Next (v1.x)" and the registry/marketplace track in `ARCHITECTURE.md`.
**Owner:** Zwin
**Target:** `kit` 1.0.0 — Rust binary, npm-first distribution, public dev tool.

---

## 1. The job

**Kit is the control room for parallel agent work.**

You run five agents across three repos. Kit is the one surface that shows what each is doing, stops the ones going wrong, and refuses to let any of them claim "done" until the code actually builds.

One line: **Dispatch many agents. Watch them in one place. Nothing ships unproven.**

### What Kit 0.1 got wrong

0.1 was a skill-pack installer with four lanes — Skills, Agents, Services, Ops. `DEVELOPER_CRITIQUE.md` diagnosed it correctly: *"a product org chart, not a setup flow… feels like a launcher skin."* Every lane was a thin wrapper over a one-line shell command. Installing a skill is a file copy. Running an agent is spawning `codex`. Nothing was load-bearing enough that its absence would hurt.

1.0 inverts this. The skill library stops being the product and becomes substrate — it is *what you dispatch*, not the point. The product is the run, the gate, and the receipt.

---

## 2. Why this wins

The multi-agent workflow is now normal and completely unserved:

| The pain today | What people do | What Kit does |
|---|---|---|
| Six terminals, no idea which is on which branch | tmux, memory, hope | One table. Repo, agent, task, state, gate. |
| Agents collide on the same working tree | manual `git worktree` juggling | Every run gets an isolated worktree, auto-created and auto-cleaned. |
| Agent says "done"; `tsc` says otherwise | you find out at review | The gate runs before the run can report done. FAIL blocks it. |
| An agent fat-fingers `rm -rf` with a bad path | pray | Blast-radius firewall on every subprocess. |
| "What did that agent actually change?" | `git diff` archaeology | Every run leaves a receipt: diff, gate log, timing, output. |

**The gate is the differentiator.** Session managers exist. Nothing enforces a definition of done. That is the wedge, and you already built the engine for it — Guardian's `done-gate.js` and `guard.js` are 700 lines that need porting, not designing.

---

## 3. Who it's for

A developer running two or more coding agents at once, daily, across real repositories. They already have Codex, Claude Code, or Grok Build installed and authenticated. They are not looking for a new model or a new editor. They are drowning in windows and they do not trust "done."

**First user is you.** You will be running grok, codex, and claude against Kit itself while building it. If Kit does not make that specific job better, it has failed and we will know within a week.

---

## 4. Product surface

### 4.1 The Run — the one primitive

Everything in Kit is a view over Runs. A Run is:

```
Run {
  id          ULID, sortable, stable
  repo        target repository
  worktree    isolated git worktree (auto-created, auto-removed if unchanged)
  agent       codex | claude | grok | ollama
  task        the prompt, or a skill reference
  bounds      timeout, output cap, write-scope, network policy
  gate        the definition of done
  receipt     immutable record written to ~/.kit/runs/<id>/
}
```

States: `QUEUED → RUNNING → GATING → PASS | FAIL | KILLED | ERROR`.

`GATING` is the state that does not exist in any competing tool.

### 4.2 Screens

> **Full surface contract:** `docs/dev/SPEC-surface-1.0.md`  
> **Visual system + concept art:** `docs/dev/DESIGN-tui.md` + `docs/dev/assets/concept-*.jpg`

**Control Room** (default surface, `kit`)

```
KIT / CONTROL ROOM                         2 RUNNING  1 FAIL  0 GATED
+------------------------------------------------------------------+
| REPO            AGENT    TASK                   STATE    GATE     |
| kit             codex    port guard.js          RUN 2m   --       |
| kit             grok     frame clock            RUN 2m   --       |
| trenchwire      codex    fix red CI             DONE     FAIL     |
|                                               ^ tsc: 3 errors     |
| guardian        claude   855-case suite         DONE     PASS     |
+------------------------------------------------------------------+
 [d]ispatch  [b]oard  [enter] open  [g]ate  [k]ill  [r]etry  [?]help
```

Live. Sorted by state, then age. Never scrolls under you while you read it.  
Semantic colors for STATE/GATE (always paired with text). FAIL rows get a danger wash + first-error annotation. Empty room says `press d to dispatch`. Vacuous gates render **UNCONFIGURED**, never PASS.

**1.0 surface acceptance (Control Room):**
- Demo fixture shows FAIL unmissable without reading logs
- `NO_COLOR` remains usable (modifiers only)
- Snapshots green at 80×14 and 60×12
- One footer grammar shared with every other screen

**Run detail** — streamed output, live diff, gate log, receipt. Tabs `stream | gate | diff`. Stream highlights error lines; diff paints `+`/`-`. `[a]ttach` is an honest stub until 1.0.1 PTY; `Esc` detaches without killing.

**Dispatch** — choose repos × agents × one task, fan out. Focus ring on the active field. The fan-out is the feature: one task, N repos, N isolated worktrees, N gates.

**Board** — curated Dispatch prefill list for 1.0 (**not** a pull-queue). Pull/lease orchestration is 1.1+.

**Doctor** — CLI `kit doctor` for 1.0 (what's installed / broken / fix). TUI doctor optional later.

**Library** — installed skills, minimal. Supporting screen only; not the product.

### 4.3 The gate

Declared per repo in `kit.toml`, checked into the project:

```toml
[gate]
format    = "pnpm format:check"
typecheck = "pnpm typecheck"
test      = "pnpm test"
timeout   = "5m"

[gate.scope]
allow = ["src/**", "tests/**", "docs/**"]
deny  = [".github/**", "*.lock"]

[firewall]
mode = "block"   # block | warn | off
```

The gate runs **in the run's worktree, after the agent stops, before the run reports done.** FAIL surfaces the failing command and its first error line directly in the Control Room. `[r]etry` re-dispatches the same task with the gate failure appended as context — the loop that makes agents self-correct without you reading logs.

Sensible defaults are inferred when `kit.toml` is absent, so the tool is useful before it is configured.

### 4.4 Principles

1. **Bounded by default.** Every run has a timeout, an output cap, and a write scope. No unbounded process, ever.
2. **Proof or it didn't happen.** No run reports success without a gate result. Receipts are immutable.
3. **Local-first.** No Kit server. No account required. No telemetry.
4. **No credential custody.** Kit uses each provider's existing login. It never reads, stores, or copies keys.
5. **Fails open on Kit's own bugs.** A Kit defect must never block real work — inherited from Guardian's threat model.
6. **The terminal is the product.** Not a web dashboard with a CLI bolted on.

---

## 5. Architecture

### 5.1 Rust workspace

Following the `fennec` pattern, which is proven in-house and already ships this way:

```
crates/
  kit-core       runs, worktrees, receipts, config, skill library
  kit-agents     adapters: codex, claude, grok, ollama
  kit-gate       definition-of-done engine + blast-radius firewall
  kit-tui        ratatui control room
  kit-cli        headless surface, --json for automation
```

**Stack:** `ratatui` 0.30 + `crossterm` 0.29 (event-stream) + `tokio` + `futures`. Snapshot tests via `insta`. This is exactly `fennec-tui`'s dependency set — a working reference implementation you already wrote.

### 5.2 The single frame clock — fixing the motion problem

Kit 0.1's animation reads as jitter because eight components each own a private `setInterval` and call `setState` independently: `Motion.tsx`, `MascotPlayer`, `CountUp`, `StaggerLines`, `TypeLine`, `useIntervalFrame`, `ActionFlash`, `FadeSteps`. Nothing shares a beat, and every tick re-renders a 3,192-line React tree through Ink.

`fennec-tui` already solved this. One event enum, one clock:

```rust
enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    AnimationTick,        // the only motion source
    RunUpdate(RunId, RunDelta),
    GateResult(RunId, GateOutcome),
}
```

A single `tokio::select!` loop merges the crossterm event stream, one animation interval, and run updates. Motion is a pure function of tick count. Frame budget is enforced in one place. Redraw only on state change.

**Non-negotiable:** no crate outside the event loop may own a timer.

### 5.3 Render and hit-testing from one source

0.1 keeps mouse geometry hand-mirrored across `workbenchGeometry.ts`, `workbenchHits.ts`, and `HitMap.ts` — which is why clicks drift from what's drawn. In 1.0, ratatui's layout solver produces the rects, and hit-testing reads *those same rects*. One source. Drift becomes structurally impossible.

### 5.4 Concurrency

Each run is a `tokio` task owning a PTY. Output streams through a bounded channel into the UI. N concurrent runs never block the frame loop. This is the reason for Rust — Node could not stream ten PTYs and hold a stable frame budget.

---

## 6. Distribution

npm stays the primary channel so the existing `@mzwin/kit` install path and its users carry forward unbroken.

- **`npm i -g @mzwin/kit`** — launcher resolves a platform binary package. This is the pattern already built and proven in `dist/npm/trenchwire-*` — reuse it directly.
- **`cargo install kit-cli`** — for Rust users.
- **`curl -fsSL … | sh`** — checksum-verifying installer, copied from `fennec/scripts/install.sh`.

Targets: macOS arm64/x64, Linux glibc arm64/x64, Windows x64.

`kit` 1.0.0 is a hard version boundary: 0.1.x was Node, 1.0 is a binary. Same command, same install line, no runtime dependency.

---

## 7. What we are cutting

Decisiveness matters more than coverage here. Explicitly out of 1.0:

- **Skill marketplace, registry API, Railway backend, Postgres catalog, accounts, profiles, following, publishing.** The entire social/registry track in `ARCHITECTURE.md` is dead. Skills are local substrate.
- **Services lane / trading plugins.** Trenchwire is its own product. It does not belong in a control room.
- **Workshop skill editor.**
- **The Ink TUI.** See below.

### The cost of the Rust decision — stated plainly

The 2,938 uncommitted lines in the working tree (the TUI brand pass, Setup screen, mouse hit-map fixes) **do not ship as written.** They become the visual and interaction specification for the ratatui port.

**Do this before anything else:** commit that work to `archive/ink-tui-final` and push it. It is 2,938 lines of design decisions, and right now a stray `git checkout` destroys it.

`kit-core`'s domain logic — skill parsing, path normalization, unify, doctor, recommend — ports to Rust largely intact. It is the smallest part of the work.

**PR #7 lands first, before the rewrite starts.** Its E2E harness defines the behavioral contract the Rust implementation must satisfy, and its JSON contract v1 is the automation surface we keep. It is green on all six CI legs today. Merging it costs nothing and gives the port a spec to pass.

---

## 8. Milestones

Each milestone has a kill criterion — an objective test that says whether to continue.

**M0 — Foundation**
Rust workspace, five crates, unified event loop, single frame clock, theme, `insta` snapshot harness, 3-OS CI.
*Kill criterion:* cold start under 100ms; idle CPU under 1%; snapshot tests green on all three OSes.

**M1 — One run, end to end**
Dispatch one task to one agent in an isolated worktree. Stream output. Write a receipt.
*Kill criterion:* `kit run --repo . --agent codex --task "…"` produces a correct worktree, a live stream, and a readable receipt. Worktree is cleaned when unchanged.

**M2 — Control Room**
N concurrent runs, live table, attach/detach, kill, run detail with diff.
*Kill criterion:* eight concurrent runs across three repos with a stable frame rate and no output interleaving.

**M3 — The gate**
Port Guardian's `done-gate.js` and `guard.js` to `kit-gate`. `GATING` state, PASS/FAIL surfacing, retry-with-failure-context.
*Kill criterion:* Guardian's existing fixture suite passes against the Rust port; a deliberately broken agent run is caught and blocked.

**M4 — Board and fan-out**
Shared task queue, agents pull work. Dispatch one task across N repos.
*Kill criterion:* one task fanned to four repos completes with four independent gate results.

**M5 — Ship**
npm platform packages, cargo, curl installer, docs rewrite, demo recording, 1.0.0.
*Kill criterion:* clean-machine install on all three OSes, verified from the README, by someone who is not you.

---

## 9. Build plan — agent orchestration

Claude orchestrates. Work is allocated by what each model is actually good at.

| Agent | Owns | Rationale |
|---|---|---|
| **Claude (orchestrator)** | Architecture, gate semantics, event-loop design, integration, review, ship | Judgment work and anything crossing crate boundaries |
| **Codex** | Mechanical ports: Guardian JS→Rust, kit-core TS→Rust, test scaffolding, fixtures | High-volume translation with a clear spec on both ends |
| **Grok Build** | ratatui screens, crossterm event handling, PTY supervision | Native Rust harness; strongest on Rust-idiomatic work |
| **Ollama** | Docs drafts, changelog, scratch | Free; nothing on the critical path |

**Rules.** Every agent run goes through a worktree and a gate — we dogfood M1–M3 as soon as they exist, and Kit builds itself from M3 onward. No agent merges its own work. Claude reviews every crate-boundary change. Parallelize inside a milestone, never across one.

---

## 10. Definition of done for 1.0

- Clean-machine install works on macOS, Linux, and Windows, verified from the README by a third party
- Cold start under 100ms; eight concurrent runs hold a stable frame
- Every README command verified from a clean environment
- Guardian's fixture suite green against `kit-gate`
- Reduced-motion honored (`NO_COLOR`, `KIT_MOTION=off`); usable without color
- Typed error codes; every error names the fix
- `--json` on every command that produces data
- Receipts are stable and documented — a third-party tool can read `~/.kit/runs`
- No credential ever read, stored, or copied
- Architecture doc and a 60-second demo that shows the gate catching a real failure

---

## 11. Risks

| Risk | Mitigation |
|---|---|
| **Rewrite never lands** — the classic failure | Milestone kill criteria. If M0 misses its numbers, stop and reconsider the stack rather than pushing through. |
| **Agent CLIs change under us** | Adapters isolated in `kit-agents` behind one trait. A broken adapter degrades one agent, never the control room. |
| **PTY behavior differs on Windows** | Windows in CI from M0, not M5. This is where cross-platform TUIs die. |
| **Scope creep back toward the marketplace** | Section 7 is a contract. Registry work is a 2.0 conversation. |
| **Gate too strict → people disable it** | Ships with inferred defaults, `warn` mode available, always overridable. A gate that cries wolf gets turned off. |
| **Doc bloat** — 405 markdown files vs 183 source files happened on Atlas | This PRD is the single source. New docs need a reason to exist. |

---

## 12. Success metrics

**Ninety days after 1.0:**

- You personally run Kit daily and would be annoyed to lose it — the only metric that cannot be gamed
- 500+ npm downloads/week (0.1.x baseline: 18)
- 100+ GitHub stars
- Ten or more issues filed by people who are not you — proof of real usage
- At least one written account of the gate catching something real

**Leading indicator:** the first week Kit orchestrates its own development end to end.
