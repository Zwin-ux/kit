# Kit V2 — design

- **Date:** 2026-09-24
- **Status:** Revised 2026-09-26. The release named 2.0.0 is the kits release (see below). The pillars in this document are the roadmap after it.
- **Owner:** Mazen Zwin (Zwin-ux)
- **Pitch:** *Dispatch any agent. Trust none. Land only what's proven.*

> **Revision, 2026-09-26 (owner decision).** Two days after this draft, the owner made kits the lead: a kit is a bundle that sets coding agents up for one job (skills, rules, MCP servers, hooks), installed from a starter set, from GitHub, or later from a hosted index. Kit **2.0.0 ships kits plus the proof engine that exists today** (`kit run`, `kit.toml` checks, receipts, `kit land`, the Control Room) and the launch bar of §11 (one-line install on three OSes, crates.io as `kitctl`, checksums and build attestations). See `docs/dev/DESIGN-KITS.md` and `CHANGELOG.md`.
>
> What changes in this document:
> - The non-goal "a skill marketplace or registry service" is withdrawn. Kits and a kit index are the product's front door. A hosted server comes later; the first index is a public GitHub repo.
> - Pillars M1–M4 (drivers and `kit verify`, policy engine and sandbox, merge queue and best-of-N, `kitd` and `kit mcp`) are **not** in 2.0.0. They are the roadmap for 2.x and later, each shipped behind its own minor version when ready.
> - §11's tooling choices (`dist`, `release-plz`) are not adopted for 2.0.0; the existing `release-npm.yml` pipeline covers the same channels. Homebrew and `cargo binstall` follow after 2.0.0.
> - The rest (architecture, contracts, quality bar) stands as the direction for that roadmap.

This is the umbrella design for Kit 2.0. It is split into six sub-projects (M0–M5). Each sub-project gets its own implementation plan; this document fixes the architecture, contracts, and decisions they share.

---

## 1. Goals and non-goals

### Goals

1. **Public launch-grade Rust package.** Installable in one line on macOS, Linux, and Windows; published libraries on crates.io; docs, CI on three operating systems, semver discipline.
2. **Proof, not vibes.** Every run ends with checks Kit ran itself and a tamper-evident receipt that anyone can re-verify with `kit verify`, locally or in CI.
3. **Provider-neutral fleet.** Claude, Codex, Grok, Gemini, Copilot, OpenCode and any ACP agent, through one typed event model. Ollama and other local models plug in as model providers.
4. **Safe unattended runs.** Kit answers agents' permission requests through a policy engine, enforces budgets, and can wrap runs in an OS sandbox.
5. **Land, don't just generate.** A local merge queue that re-checks each change on the current trunk before it lands; best-of-N across agents ranked by checks.
6. **Always on.** Runs survive the TUI (`kitd`), Kit is callable from other agents (`kit mcp`), and people get notified when something needs them.

### Non-goals for 2.0

- A web or mobile dashboard (2.0 ships the local API it would use, not the UI).
- Cloud-hosted runners or remote execution.
- An OS sandbox tier on Windows (Windows gets approval-time policy and the agents' native sandboxes).
- ~~A skill marketplace or registry service (the legacy registry API is archived).~~ Withdrawn 2026-09-26: kits and a kit index are in scope (see the revision note above).
- Storing model API keys. Kit keeps using each agent CLI's own login ("no credential custody").

---

## 2. Decisions log

| # | Decision | Chosen | Alternatives rejected |
|---|----------|--------|-----------------------|
| D1 | Primary audience for 2.0 | Public launch-grade | Owner daily driver first; portfolio showpiece |
| D2 | 2.0 scope | All four pillars: Proof engine, Safe full-auto, Land it, Always-on platform | Ship pillars across 2.x |
| D3 | Legacy TypeScript 0.1 | Archive (separate repo, npm deprecate, drop Node CI); skill catalog becomes a versioned data pack | Port to Rust; keep alongside |
| D4 | Platforms | macOS + Linux first-class; Windows supported without an OS sandbox tier | Three-way parity; drop Windows |
| D5 | Path from 1.0-alpha | **Baseline then replace:** merge the current stack as the trunk, add the V2 crate skeleton, swap the engine pillar by pillar, ship `2.0.0-alpha.N` throughout | Clean-room rewrite; evolve in place |
| D6 | Agent runtime | Kit-owned event model + native drivers for Claude and Codex + generic ACP driver + exec fallback | ACP-only; SDK-embedded; per-CLI adapters only |
| D7 | Default policy profile | `build` | `full` |
| D8 | Cross-vendor reviewer | Advisory by default, blocking is opt-in | Blocking by default |
| D9 | `kit land` default mode | Local fast-forward; PR mode opt-in | PR by default |
| D10 | Daemon | `kitd` auto-started by default (`daemon = "auto"`); supersedes ADR-0001 via a new ADR | Opt-in daemon |
| D11 | Published crate names | Libraries `kit-proto`, `kit-engine`, `kit-drivers`, `kit-gate`, `kit-policy`, `kit-store`, `kit-sandbox`, `kit-mcp`, `kit-tui`; binary crate **`kitctl`** installing the `kit` binary; daemon crate `kitd` | `kit` and `kit-cli` are taken on crates.io (checked 2026-09-24) |
| D12 | Library versioning | Libraries publish at `0.x` until their APIs settle; the product (`kitctl`, the `kit` binary) versions as `2.0.0` | Lockstep versions for every crate |
| D13 | npm package | `@mzwin/kit` 2.x becomes the thin wrapper that installs the Rust binary; 0.1.x is deprecated with a migration note | A new npm name |

D1–D10 were decided by the owner during brainstorming (D6–D10 by approving design sections). D11–D13 are proposed defaults to confirm during spec review.

---

## 3. Where Kit is today (2026-09-24)

- **Trunk:** `main` = `fb0ddf1` (2026-08-14). The real development tip is PR #16 `feat/npm-platform-dist` (`0403696`), 55 commits ahead through seven stacked branches, with Rust CI red on all three OSes (`land_cli` 5/5 failing: the receipt diff is taken before the gate, but the land fixtures create changes through the gate). npm `alpha` (`1.0.0-alpha.1`) was published from that unmerged branch.
- **Workspace:** `kit-core` (frozen shared types), `kit-agents` (four adapters, process-tree kill, skill injection), `kit-gate` (checks + Guardian firewall port), `kit-tui` (ratatui Control Room), `kit-cli` (hand-rolled argv + the whole engine). ~9.6k Rust LOC on main, ~15.5k on the tip. Plus ~22k LOC of legacy TypeScript 0.1 (`packages/`, `apps/registry-api`) still in CI.
- **Health on main:** fmt clean, clippy clean, 89 tests pass; no live-adapter integration tests.

### Defects V2 must remove (evidence on `main`)

1. `kit --demo`, the README's headline command, prints "unknown command" (`crates/kit-cli/src/main.rs:32-50`).
2. `--demo` wires the real engine, so `r` on a fixture row starts a live, billable run in the current directory (`main.rs:62-78`).
3. Kit contaminates the tree it judges: skill packs and `AGENTS.md` are copied into every worktree (`kit-agents/src/skills.rs:82-92,134-160`), so worktrees never count as clean and are never removed.
4. A run that changes nothing can PASS: no empty-diff check (`kit-cli/src/engine/runner.rs:280-286`); a vacuous gate carries `passed: true` (`kit-core/src/gate.rs:61-69`); scope checking fails open (`kit-gate/src/lib.rs:127`).
5. Adapters are weaker than the legacy TypeScript workbench: Claude runs with no `--permission-mode` (effectively read-only unless a full bypass is set, `claude.rs:48-58`); Grok is always `--always-approve` (`grok.rs:48-56`); Ollama is chat-only and can never edit (`ollama.rs:54-58`), contradicting ADR-0003.
6. Receipts drop new files (`git diff HEAD`, `worktree.rs:53-59`) and are written non-atomically (`store.rs:12-26`).
7. The firewall never sees agent commands (`Gate::screen` unused; `firewall_blocks` always empty, `kit-gate/src/lib.rs:134`); `Bounds.write_allow/deny` are never read.
8. Windows spawns go through `cmd /C`, which truncates multi-line prompts and has an injection path (`kit-agents/src/process.rs:16-28`).
9. Ctrl-C during `kit run` and `q` in the TUI orphan agents and worktrees (no signal handling, no drain); kill is SIGKILL with no grace.
10. Engine errors print over the TUI's alternate screen (`supervisor.rs:82,150-155`).
11. Retry starts a fresh session with a one-line summary and drops lineage (`kit-tui/src/app.rs:1082-1134`, `supervisor.rs:85`).
12. Layering: the engine lives in the binary crate, the engine protocol is defined in the UI crate (`app.rs:27-42`), there are three run models, and `app.rs` is 2,172 lines.

Reproduced with fake agent binaries (P0 — fixed first, in M0):

13. **Every live run spins until the 30-minute timeout.** The biased `select!` polls `recv()` before the exit tick; once the agent's streams close, `recv()` returns `None` forever (`runner.rs:478-509`). The TUI loop already guards this pattern (`kit-tui/src/loop.rs:88-109`).
14. **`--json` stdout is not valid JSON:** `git worktree add` shares Kit's stdout (`worktree.rs:20-29`).
15. **A missing check tool counts as a pass:** unrunnable checks become `Skipped` (`kit-gate/src/lib.rs:856-859`) and `Skipped` counts as passed (`kit-core/src/gate.rs:41-45`); a `kit.toml` typo is swallowed (`store.rs:192`).
16. **`kit run --task help` prints help** (every argv entry is checked for "help", `main.rs:24-30`); starting `kit` without a TTY fails with `Device not configured (os error 6)`.
17. **Output streams stop on invalid UTF-8** (`process.rs:133-147`) and the output cap can panic on a multi-byte boundary (`runner.rs:552`).

### Quality scorecard (2026-09-24)

| Area | Grade | Headline |
|---|---|---|
| Workspace, crate design, performance | C+ (perf A-) | `kit --version` 6.4 ms, 1.87 MB binary, 102 crates; engine locked in the binary; env reads inside libraries; exhaustive pub types |
| Async runtime, process management | D | spin bug, orphaned agents, `cmd /C`, SIGKILL only, blocking I/O in async |
| TUI | B- | pure reducer with 58 tests and 12 snapshots; blank screen until probes finish; redraw per event; `q` orphans runs; mouse capture breaks selection |
| Errors, observability | D | anyhow everywhere, no tracing or log file, `eprintln!` under the TUI, `--json` errors not enveloped |
| Config, state | D+ | parse errors swallowed; receipt v1 lacks base SHA, versions, exit codes, RFC 3339 times |
| Testing, CI | C | 89 unit tests and a 3-OS matrix; no integration, CLI-contract, property or fake-agent end-to-end tests; no deny/audit/MSRV/semver/doc jobs |
| Distribution | F | no Rust release pipeline; `cargo package` fails (path deps without versions) |
| Docs | C- | strong README pitch; changelog and contributing guide describe the Node era; 162 missing docs |
| Correctness, safety | D- | items 1–17 above |

---

## 4. Architecture

### 4.1 Crates

| Crate | Responsibility | Depends on |
|---|---|---|
| `kit-proto` | **Public contract.** Versioned serde types: `RunSpec` (v2), `KitEvent`, `Receipt` (v2), `Plan`/`Step`, `Policy`, `Budget`, engine API request/response types. JSON Schema export (`schemars`). Semver-checked. | serde, schemars, ulid |
| `kit-store` | SQLite (WAL) event log with a per-run hash chain; projections (`runs`, `receipts`, `approvals`, `land_queue`); content-addressed blobs under `~/.kit/blobs/sha256/`; reads v1 receipts. | kit-proto, rusqlite |
| `kit-drivers` | `Driver` / `Session` traits and implementations: `ClaudeStreamJson`, `CodexAppServer`, `Acp` (manifest-driven), `Exec`. Process management (spawn without shells, stdin prompts, process-group / job-object kill, graceful escalation). | kit-proto, tokio, agent-client-protocol |
| `kit-gate` | Definition of done: check runner, inferred checks, hidden checks, tamper flags, scope, diff rules, optional reviewer. | kit-proto |
| `kit-policy` | Approval decisions (firewall, write scope, network, env), budget accounting, escalation queue. | kit-proto |
| `kit-sandbox` | Optional OS sandbox: Seatbelt profiles (macOS), bubblewrap (Linux), egress proxy with a domain allowlist. | kit-proto |
| `kit-engine` | Run lifecycle, worktree manager, scheduler, orchestration (fan-out, best-of-N, retry-in-session), land queue, engine API implementation, event bus. | all of the above |
| `kitd` | Daemon hosting `kit-engine`; JSON-RPC 2.0 over a Unix socket / Windows named pipe; loopback HTTP/WebSocket API. | kit-engine |
| `kit-mcp` | `kit mcp` server (rmcp) and the per-session MCP server injected into agent sessions. | kit-engine client |
| `kit-tui` | ratatui client of the engine API. No engine types inside the UI. | kit-proto, engine client |
| `kitctl` | The `kit` binary: `clap` CLI, in-process engine or `kitd` client, TUI launcher. | everything |

`kit-core` and `kit-agents` are replaced during M0–M1 (their types move into `kit-proto`; their proven code — process-tree kill, Windows job objects, the gate runner, the firewall tokenizer — moves into `kit-drivers`, `kit-gate`, `kit-policy`).

### 4.2 Engine API

One API, three transports: in-process (library call), `kitd` JSON-RPC (Unix socket `~/.kit/kitd.sock`, mode 0600, or named pipe), and MCP. Methods:

`dispatch(RunSpec | Plan) -> RunId[]`, `list(filter)`, `get(RunId)`, `subscribe(RunId | all, from_seq) -> stream<KitEvent>`, `approve(ApprovalId, Decision)`, `kill(RunId)`, `retry(RunId, extra_context?)`, `land(RunId[] | all_pass, mode)`, `verify(ReceiptRef)`, `doctor()`.

### 4.3 Event model (`KitEvent`)

Every event carries `run_id`, `seq`, `ts`, and `prev_hash`/`hash` once stored.

- Session: `SessionStarted{driver, cli_version, model, resume_token}`, `TurnEnded{stop_reason}`
- Content: `Message{role, text | delta}`, `Thought{text | delta}`, `Plan{steps}`
- Tools: `ToolCall{id, kind, title, input}`, `ToolUpdate{id, status, output_tail}`, `FileChange{path, patch?}`
- Usage: `Usage{input, output, cached, reasoning, cost_usd?, cost_estimated: bool, context_used?, context_size?}`
- Policy: `PermissionAsked{id, request}`, `PermissionDecided{id, decision, by: policy | human, rule}`, `BudgetExceeded{kind, limit, used}`
- Checks: `GateStarted`, `GateCheck{label, status, exit_code, duration, output_blob}`, `GateVerdict{passed, reasons}`
- Lifecycle: `StateChanged{from, to}`, `Error{code, message}`, `Landed{target, commit}`, `LandFailed{reason}`

Run states: `queued → preparing → running → gating → pass | fail | killed | error | budget | interrupted`, with `landing → landed | land_fail` after `pass`.

### 4.4 Data flow

1. A client calls `dispatch`. The engine expands a `Plan` into steps and the scheduler takes a slot (per-provider semaphores, global cap, rate-limit backoff).
2. The worktree manager creates the worktree (`git worktree add --detach <base>`), runs `[worktree]` bootstrap, and records the base SHA.
3. The isolation layer applies the policy profile and, if enabled, the OS sandbox.
4. The driver starts a session: instructions go in as system/developer instructions, the prompt goes in on stdin or over the protocol, and the skill pack is delivered outside the tree.
5. Driver output becomes `KitEvent`s → secret redaction → hash-chained append in `kit-store` → broadcast to subscribers (TUI, CLI, MCP, API).
6. Permission requests go to `kit-policy`: allow, deny (with reason back to the agent), or escalate to a human with a timeout that defaults to deny.
7. When the turn ends, `kit-gate` runs the definition of done.
8. On FAIL, the retry policy resumes the same session with the full failure output, up to the attempt cap. On PASS, the run can enter the land queue.
9. The receipt is sealed; the worktree is removed if landed or discarded, kept otherwise.

---

## 5. Pillar 1 — Proof engine (M1)

### 5.1 Drivers

`Driver::probe() -> Capabilities` reports: installed version, authenticated (real checks: `claude auth status`, `codex login status`, ACP `initialize`), approval support (`host | self | none`), resume, cost reporting, native budget, MCP injection method, native sandbox, models.

| Driver | Launch | Approvals | Resume | Usage |
|---|---|---|---|---|
| **Claude** | `claude -p --output-format stream-json --input-format stream-json --session-id <uuid> --permission-mode default --permission-prompt-tool mcp__kit__approve --mcp-config <session server> --strict-mcp-config --setting-sources project,local` (plus `--max-budget-usd` when a budget is set) | Kit's per-session MCP tool answers | `--resume <session>` | `total_cost_usd`, per-message usage |
| **Codex** | `codex app-server` over stdio JSON-RPC; `thread/start{cwd, model, sandbox: workspaceWrite{writableRoots: [worktree, worktree gitdir]}, approvalPolicy: onRequest, developerInstructions}` | Kit answers `item/commandExecution/requestApproval`, `item/fileChange/requestApproval` | `thread/resume` | `thread/tokenUsage/updated` (tokens; cost estimated) |
| **ACP** | Per-agent manifest (TOML): `grok agent stdio`, `gemini --acp`, `copilot --acp`, `opencode acp --cwd <wt>`, others from the ACP registry | `session/request_permission` | `session/load` when supported | `usage_update` when supported |
| **Exec** | Fallback for CLIs without a protocol (e.g. aider): spawn directly (never through a shell), prompt on stdin, per-CLI line parser | none (policy is sandbox-only) | none | none |

- Codex types are generated from `codex app-server generate-json-schema` and pinned per supported CLI version; contract tests replay recorded transcripts.
- Local models (Ollama, LM Studio) are **model providers**, used through Codex `--oss --local-provider` or an ACP harness such as OpenCode. `kit doctor` reports Ollama client/server version mismatches.
- Shutdown escalates: protocol interrupt (`turn/interrupt`, `session/cancel`, SIGINT) → SIGTERM after 5 s → process-group SIGKILL / job-object termination after 10 s.
- Windows: no `cmd /C`. Binaries are resolved to real executables or their script entry points and spawned directly; prompts always go over stdin or the protocol.

### 5.2 Definition of done (`kit-gate`)

A run passes only if **all** of the following hold:

1. **Non-empty change.** The diff (tracked + untracked, excluding Kit-owned paths) is non-empty, unless the task is declared `expect = "no-change"`.
2. **Checks ran and passed.** Checks come from `kit.toml` `[gate]`; if none are configured they are inferred from the repo and labelled `inferred` in the receipt. A gate with no runnable checks is `UNCONFIGURED`, which is never PASS (exit code 1 in CLI mode, unless `--allow-unconfigured`).
3. **Hidden checks.** `[gate.hidden]` checks are executed from a copy of the check definitions held outside the worktree; the agent never sees their commands.
4. **No tamper.** If the diff touches test files, CI config, `kit.toml`, or check scripts, the receipt carries `tamper` flags; with `strict_tamper = true` this fails the run.
5. **In scope.** Writes outside `[scope].allow` or inside `[scope].deny` fail the run (fail closed; scope errors are failures, not passes).
6. **Evidence captured.** Each check's full output is stored as a blob; the last 200 lines of each failing check are carried into retry context.

**Reviewer (optional).** `[gate.reviewer] agent = "codex"` runs a second agent (different vendor by default) on the diff with a JSON-schema verdict `{verdict: approve | request_changes, findings[]}`. Advisory by default; `blocking = true` makes `request_changes` a FAIL.

### 5.3 Worktree hygiene and bootstrap

- **No contamination.** Skill packs are delivered outside the tree: through native mechanisms where available (Codex `skills/extraRoots/set`, Claude `--add-dir`/plugin dirs), otherwise copied into a Kit-owned path listed in the worktree's `info/exclude`. Kit never writes `AGENTS.md` into the target repo; it passes instructions as system/developer instructions.
- **Skill packs are explicit inputs.** Packs are versioned data (`packs/<name>@<version>`), selected in `kit.toml` or with `--pack`, never discovered by walking up from the current directory.
- **Bootstrap** via `kit.toml`:

```toml
[worktree]
setup = ["pnpm install --frozen-lockfile"]   # run once per worktree, before the agent
copy = [".env.local"]                          # copied from the main checkout
include = ".worktreeinclude"                   # compatible with Claude Code's file
ports = { base = 4000, stride = 10 }           # KIT_PORT = base + index * stride
shared_cache = ["target", "node_modules/.cache"]  # opt-in shared caches
```

### 5.4 Receipts v2 and `kit verify`

A receipt is a sealed manifest stored in `kit-store` and exported as `receipt.json`:

- run id, parent run id (retry lineage), spec, base SHA, final state
- driver, agent CLI version, model, policy profile, sandbox tier
- every executed command with exit code (from tool events and gate checks)
- check results with output blob hashes
- diff blob hash (sha256) and file list
- usage and cost (flagged if estimated)
- approvals summary (asked / allowed / denied / escalated)
- head hash of the run's event chain
- optional ed25519 signature (`kit keys init` creates a local key; never uploaded)

`kit verify <receipt | run-id | path>`:

1. Recompute the event hash chain and the manifest hash; check the signature if present.
2. Create a clean worktree at the base SHA, apply the diff blob, and re-run the checks (hidden checks included when available).
3. Exit 0 only if the recomputed verdict matches the receipt. Works identically in CI.

Exports: `kit receipt export --format agent-trace | in-toto | trailer | git-note`. Land commits carry `Kit-Receipt: sha256:<manifest-hash>`.

### 5.5 Retry

`r` in the TUI, `kit retry <id>`, or the auto-retry policy (`[retry] max_attempts = 2`) resumes the **same** session with the full failure context and any reviewer findings or diff comments. Each attempt is its own run with `parent` lineage.

---

## 6. Pillar 2 — Safe full-auto (M2)

### 6.1 Policy profiles

```toml
[policy]
default = "build"

[policy.build]
write = { allow = ["**"], deny = [".git/**", "**/.env*"] }   # relative to the worktree
commands = { firewall = "strict" }                            # Guardian firewall rules
network = { mode = "allowlist", allow = ["registry.npmjs.org", "crates.io", "static.crates.io", "pypi.org", "files.pythonhosted.org", "api.anthropic.com", "api.openai.com", "api.x.ai", "generativelanguage.googleapis.com"] }
env = { pass = ["PATH", "HOME", "LANG", "TERM"], secret = ["GITHUB_TOKEN"] }  # secret = passed but redacted
escalate_timeout = "5m"                                       # unanswered escalation → deny
```

Built-in profiles: `inspect` (read-only, no network except model APIs), `build` (default), `full` (wider command set and network), `unsafe` (passes the agents' own bypass flags; requires `--policy unsafe --yes-i-understand` and is recorded prominently in the receipt). `KIT_FULL_AUTO` is removed.

### 6.2 Decisions

Every approval request from every driver becomes `PermissionAsked`. The policy engine answers **allow**, **deny** (the reason is returned to the agent so it can adapt), or **escalate** (queued in the TUI's "Needs input" view, notification sent; timeout → deny). Every decision is `PermissionDecided` and summarized in the receipt.

### 6.3 Enforcement layers

1. **Approval-time** policy through the driver protocols — all platforms.
2. **Native agent sandboxes** configured from the policy: Codex `sandbox`/`writableRoots`, Grok `--sandbox`, Gemini `-s`, Copilot URL allow/deny lists.
3. **Kit OS sandbox** (`--sandbox os` or `[sandbox] tier = "os"`, macOS and Linux): Seatbelt profile or bubblewrap namespace restricting writes to the worktree and Kit-owned paths, network forced through Kit's egress proxy with the policy allowlist. When tier 3 is active, the agent's own sandbox is set to external/full access to avoid nested sandboxes.

Gates run under the same policy as the agent (agent-written code runs during checks).

### 6.4 Budgets

`[budget]` per run and per fleet: `max_usd`, `max_tokens`, `max_wall`, `max_attempts`, and `[concurrency]` per provider. Native caps are passed through (`--max-budget-usd`, `--max-turns`); otherwise Kit meters usage events against a versioned price table (`cost_estimated = true`). Exceeding a budget interrupts gracefully, sets state `budget`, and records the overage.

### 6.5 Secret hygiene

Agents get an env allowlist, not the full environment. Before any event is stored, values of named secrets, common token patterns (`sk-`, `ghp_`, AWS keys, PEM blocks, JWTs), and high-entropy strings are redacted. Hashes cover redacted content only.

---

## 7. Pillar 3 — Land it (M3)

### 7.1 `kit land`

`kit land [<run-id>…] [--all-pass] [--into <branch>] [--mode local|pr]`

For each run, serially, in queue order:

1. Create a land worktree at the **current** head of the target branch.
2. Apply the run's diff (three-way); a conflict → `land_fail{conflict}`.
3. Re-run the full definition of done, hidden checks included.
4. PASS → `local`: fast-forward the target branch with a commit carrying the `Kit-Receipt` trailer; `pr`: push a branch and open a PR through `gh` with the receipt summary. Record a land receipt linked to the run receipt.
5. FAIL or conflict → `land_fail`; with `[land] auto_retry = true`, resume the original session with the rebase conflict or check output.

PR #16's `kit land` implementation (after the M0 fix) is the starting point.

### 7.2 Best-of-N

`kit run --n 3 --agents claude,codex,grok -t "…"` (or `[[plan.fanout]]`) dispatches the same task to several drivers/models. Ranking: definition of done passed → hidden checks passed → no tamper flags → smaller diff → lower cost → shorter wall time. The optional reviewer breaks ties among passing candidates only. The winner is queued to land; the rest are kept with receipts.

### 7.3 Conflict prediction

The engine tracks each live run's touched files from `FileChange` events and periodic `git status`. The TUI shows an **overlaps** column; `dispatch` warns when a new task's likely files (from `[ownership]` globs in `kit.toml` and recent diffs) intersect a live run.

### 7.4 Review that steers

The TUI diff view supports line comments; submitting them becomes a retry of the same session with the comments as context.

---

## 8. Pillar 4 — Always-on platform (M4)

- **`kitd`:** auto-started by `kit` when `daemon = "auto"` (default); `daemon = "off"` keeps everything in-process. On restart, in-flight runs become `interrupted` and can be resumed through their driver resume tokens. Without the daemon, quitting the TUI with live runs asks: detach to `kitd`, kill, or wait. A new ADR supersedes ADR-0001.
- **`kit mcp`:** MCP server over stdio (rmcp). Tools: `dispatch`, `status`, `wait`, `events_tail`, `receipt`, `diff`, `retry`, `kill`, `verify`, and `land` (subject to policy). Resources: `kit://runs/<id>`. Long runs use the MCP Tasks extension. `kit mcp install claude|codex|gemini` writes the client config.
- **Per-session MCP server** (injected into each agent session): `kit_approve` (Claude permission prompt tool), `kit_report_progress`, `kit_ask_human`.
- **Local API:** HTTP + WebSocket on `127.0.0.1` with a bearer token from `~/.kit/api.token` (mode 0600): event streams, runs, receipts; writes go through the same policy checks.
- **Notifications:** needs-input, run pass/fail, land done, budget hit → macOS notification center, `notify-send` on Linux, optional ntfy/Slack webhooks in `[notify]`.
- **Observability (optional):** OpenTelemetry export, one span per run with usage attributes.

---

## 9. Foundation (M0)

1. **Baseline trunk.** Fix `land_cli` (take the receipt diff after the gate, or change the fixtures so the agent step makes the change), get PR #16 green on all three OSes, merge the stacked branches into `main` in order, tag `v1.0.0-alpha.2`. Retire the `kit-npm-recovery` worktree after salvaging anything not already in PR #16.
2. **P0 fixes on the 1.x engine, before any restructuring** (each with a failing test first, using a fake agent binary):
   - the runner spin (item 13): stop polling the closed channel and wait on the child directly;
   - signal handling for `kit run` (INT/TERM/HUP → cancellation → graceful agent shutdown → `killed` receipt → worktree cleanup);
   - clean `--json` stdout (capture git output; human text to stderr; errors inside the envelope);
   - a nonexistent `--repo` is an error; TUI dispatch stores absolute paths; hardcoded sibling repos removed;
   - a missing check tool or unparseable check is `error`, never `skipped`; `kit.toml` parse errors are fatal;
   - `kit --demo` works; demo mode uses a fixture engine that cannot start live runs;
   - a friendly error when stdout is not a terminal; `kit run --task help` runs the task;
   - lossy UTF-8 line reading with a length limit; output cap enforced at read time without byte-slicing;
   - `cargo update -p lru` (RUSTSEC-2026-0253);
   - all diagnostics go through `tracing` to the log file, never to the terminal while the TUI owns it.
3. **Archive TypeScript.** Move `packages/`, `apps/`, and the Node CI to `Zwin-ux/kit-legacy` (history preserved with `git filter-repo` or a tagged snapshot), `npm deprecate @mzwin/kit@"<1.0.0"` with a pointer to 2.0, remove the catalog keep-alive bot, and convert `skills/` + `packs/` into versioned pack data.
4. **Skeleton.** Create the V2 crates (§4.1) with `kit-proto` first; move the engine out of the binary crate into `kit-engine`; move the engine protocol out of `kit-tui`; replace the three run models with `kit-proto` types.
5. **Contracts.** Replace the "frozen files are Claude-only" rule in `AGENTS.md` with: `kit-proto` changes require a semver-appropriate version bump, enforced by `cargo-semver-checks` in CI.
6. **CLI.** `clap` (derive) with shell completions and a man page (`clap_complete`, `clap_mangen`); the `--json` envelope (`schemaVersion`, `command`, `ok`, `data`, `error`, `warnings`) stays compatible, bumped to `schemaVersion: 2` only where data shapes change.

---

## 10. Quality bar

- **Errors:** `thiserror` in libraries, `miette` diagnostics in the CLI, stable error codes (`KIT-E####`) documented in the docs site.
- **Crash safety:** panic hook restores the terminal, writes `~/.kit/logs/crash-<ts>.log`, prints its path.
- **Async correctness:** no blocking I/O on async threads — `tokio::process` for subprocesses, `spawn_blocking` for git and filesystem work; probes run after the TUI's first paint.
- **Process safety:** signal handling for `kit run` (Ctrl-C → graceful shutdown of the session and a `killed` receipt); TUI quit drains or detaches.
- **Testing:**
  - `cargo nextest` as the runner.
  - `insta` snapshots of TUI screens (ratatui `TestBackend`) and CLI output.
  - Fake agent binaries (Claude stream-json, Codex app-server, an ACP agent) that replay recorded transcripts, used for driver contract tests on all three OSes.
  - `proptest` for policy decisions, glob scope matching, and redaction.
  - An end-to-end fixture repo exercising dispatch → gate → receipt → verify → land in CI.
- **CI (GitHub Actions, macOS / Linux / Windows):** `cargo fmt --check`, `clippy -D warnings` (plus a curated pedantic subset), tests, `cargo-deny` (licenses, advisories, bans), `cargo-machete`, `cargo-semver-checks` on published libraries, MSRV job, startup budget (`kit --version` cold < 100 ms; TUI first paint < 150 ms on the fixture).
- **Docs as tests:** README quickstart commands are executed in CI against the fixture repo.
- **API hygiene for published crates:** `#[non_exhaustive]` on public enums and structs, private fields with constructors, native `async fn` in traits instead of `async-trait` (dropping `syn` 3 from the build), no environment-variable reads inside libraries (configuration is passed in), `[workspace.lints]` with pedantic warnings and `unwrap_used` / `expect_used` / `print_stdout` denied in library code.
- **Directories:** platform directories via `etcetera` (XDG on Linux, `~/Library/Application Support` on macOS), `KIT_HOME` override, one-time migration from `~/.kit`.
- **Receipts:** RFC 3339 timestamps, one casing (camelCase, matching the CLI envelope), atomic writes (temp file + rename), JSON Schemas published from `kit-proto`.
- **MSRV:** latest three stable releases (N-2), never below what `ratatui` 0.30 requires, tested in CI.
- **Pinned toolchain (verified 2026-09-24):** clap 4.6.7, clap_complete 4.6.11, clap_mangen 0.3.3, process-wrap 10.0.1 (`tokio1`), tokio-util 0.7.19 (`CancellationToken`), which 8.0.6, tracing 0.1.44 / tracing-subscriber 0.3.23 / tracing-appender 0.2.5, miette 7.6.0, thiserror 2.0.21, human-panic 2.0.8, etcetera 0.11.0, toml 1.1.6, schemars 1.2.2, jiff 0.2.37, tempfile 3.27.0, rusqlite 0.40.2 (bundled), insta 1.48.0, assert_cmd 2.2.2, trycmd 1.2.1, proptest 1.11.0, cargo-nextest 0.9.146, cargo-deny 0.20.2, cargo-audit 0.22.2, cargo-semver-checks 0.50.0, cargo-llvm-cov 0.9.1. Latest stable Rust: 1.98.1.

**Why SQLite for the event log** when one folder per run is fast enough today (400 receipts list in 39 ms): the daemon needs `subscribe(from_seq)` across runs, approvals need a durable queue, and the hash chain needs ordered append with integrity checks. Receipt folders remain the human-readable export.

## 11. Launch (M5 → 2.0.0)

- **Binaries:** `dist` 0.32.0 builds GitHub Release artifacts for macOS (arm64, x64), Linux (x64/arm64, gnu and musl), and Windows (x64, arm64 MSVC). `dist-workspace.toml`: `installers = ["shell", "powershell", "homebrew", "npm"]`, `tap = "Zwin-ux/homebrew-tap"`, `publish-jobs = ["homebrew", "npm"]`, `github-attestations = true`, `cargo-auditable = true`, `cargo-cyclonedx = true` (SBOM), `install-updater = true`. `[package.metadata.binstall]` for `cargo binstall`. The npm package becomes the thin wrapper that installs the release binary (D13).
- **Releases:** `release-plz` 0.3.169 with `git-cliff` 2.14.2 for version bumps and a Keep a Changelog file; GitHub artifact attestations for build provenance; SHA-256 checksums with every release. Windows code signing and macOS notarization follow once dist's signing support leaves pre-release.
- **Packaging prerequisites:** every path dependency declares a `version`; every published crate has `description`, `readme`, `keywords`, `categories`, `documentation`, `homepage`, `license = "MIT"`.
- **crates.io:** publish the library crates and `kitctl` (installs the `kit` binary).
- **Docs:** new README (pitch, 30-second quickstart, demo clip), mdBook docs site (concepts: run, definition of done, receipt, policy, land; guides; error codes; `kit.toml` reference), docs.rs for public crates, `SECURITY.md`, `CONTRIBUTING.md`.
- **Launch checklist:** clean install verified on the three OSes from each channel; `kit doctor` green on a fresh machine; demo clip recorded from the release binary.

## 12. Milestones and exit criteria

Each milestone ships as `2.0.0-alpha.N` and must be dogfooded on the Kit repo itself (Kit lands its own changes through Kit from M3 on).

| Milestone | Exit criteria |
|---|---|
| **M0 Foundation** | Trunk green on 3 OSes; every P0 in §3 (items 1–2, 10, 13–17) fixed with a fake-agent test that failed before the fix; `v1.0.0-alpha.2` tagged; TypeScript archived; V2 crates exist with `kit-proto` published as `0.x`; no terminal writes under the TUI. |
| **M1 Proof engine** | Claude, Codex, and at least two ACP agents (Grok, Gemini) run through drivers against fake-binary contract tests and one live smoke each; definition of done rejects no-op, unconfigured, out-of-scope, and tampered runs in tests; receipts v2 sealed; `kit verify` re-derives the verdict in CI on the fixture repo. |
| **M2 Safe full-auto** | Every driver's approvals flow through `kit-policy`; escalation queue and notification work; budgets stop a run in tests; OS sandbox blocks out-of-worktree writes and non-allowlisted network on macOS and Linux; redaction property tests pass. |
| **M3 Land it** | `kit land` lands a queue of 5 runs serially with re-gate, handles a conflict and a post-rebase failure; best-of-N ranks three agents in a fixture scenario; overlaps column shows live collisions. |
| **M4 Always-on** | Runs survive TUI exit under `kitd` and resume after a daemon restart; `kit mcp` lets Claude Code dispatch a Codex run and read its receipt; notifications fire on needs-input. |
| **M5 Launch** | All channels install cleanly on 3 OSes; docs site live; crates published; `2.0.0` tagged. |

## 13. Risks

| Risk | Mitigation |
|---|---|
| Protocol churn (ACP crate 1.x → 2.2 in two months; Codex app-server labelled experimental) | Version-pinned generated types, recorded-transcript contract tests, exec driver fallback, capability probing at runtime |
| Claude auth policy for third-party products | Kit only spawns the user's own `claude` CLI with its own login; no Agent SDK embedding, no claude.ai login inside Kit |
| Cost reporting differs by vendor | Receipts flag estimated costs; budgets can use tokens instead of dollars |
| Nested sandboxes (Seatbelt cannot nest) | When Kit's OS sandbox is on, agents' own sandboxes are set to external/full |
| First-party control rooms (Claude agent view, Codex app) | Kit's position is cross-vendor proof, policy and landing, not session management |
| Scope size (four pillars in 2.0) | Milestone gating with dogfood alphas; each pillar has its own plan and exit criteria |
| Subscription rate limits under fan-out | Per-provider concurrency caps and backoff by default |
| The `kit` binary name collides with the KitOps `kit` CLI (CNCF) on PATH | Installers detect an existing non-Kit `kit` and warn; `kitctl` is always installed as an alias so users can resolve the collision; the docs cover it |
