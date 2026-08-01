# M3 · Guardian port — the gate engine

**Owner:** Codex · **Crate:** `kit-gate` · **Branch:** `m3-gate-port`

Port Guardian's blast-radius firewall and definition-of-done gate from
JavaScript to Rust. This is Kit's differentiator: session managers exist, but
nothing else refuses to let an agent claim "done" until the code actually builds.

## Source — the thing being ported

`C:/Users/mzwin/grok-build-guardian/`

| File | Lines | Becomes |
|---|---|---|
| `hooks/guard.js` | 546 | `Gate::screen` — the blast-radius firewall |
| `hooks/done-gate.js` | 154 | `Gate::evaluate` — the definition-of-done gate |
| `tests/firewall.test.js` | 97 | **the acceptance oracle** |

`tests/firewall.test.js` holds 50 fixtures — 30 must-allow, 20 must-block.
**Every one must pass against the Rust implementation.** The port is done when
the fixtures pass, not when the code reads well. Port the fixture table verbatim
into Rust tests; do not paraphrase the cases or drop any.

Read `README.md` in that repo first. Its threat model and its allow/block table
are the specification.

## What to build

Replace the inert scaffold in `crates/kit-gate/src/lib.rs` with a real
implementation of the frozen `kit_core::Gate` trait.

1. **`screen(&self, command: &str) -> FirewallVerdict`**

   Port `guard.js`'s command analysis. It must block `rm -rf /`, `rm -rf ..`,
   `rm -rf .git`, `curl … | sudo bash`, secrets piped to the network, `dd` to a
   raw device, `mkfs`, `chmod -R 000 /` — while allowing `rm -rf node_modules`,
   `rm -rf dist build`, `git clean -fd`, `git reset --hard`, `curl … | jq`, and
   `dd if=/dev/zero of=./disk.img`.

   Honour `FirewallMode`: `Block` refuses, `Warn` logs and allows, `Off` is a
   no-op. Default is `Block`.

2. **`evaluate(&self, worktree, config) -> GateOutcome`**

   Run `GateConfig::checks()` in order, inside `worktree`, each as a child
   process with no shell where possible. Enforce `GateConfig::timeout` across
   the whole gate, not per command. Populate one `GateCheck` per command.

   `GateCheck::summary` is the single most important field in this port: it is
   the line the user reads in the Control Room *instead of* opening a log. For a
   failing `tsc` it should read like `tsc: 3 errors`, not the first 200 bytes of
   stdout. Extract a real summary per failure; fall back to the first non-empty
   error line.

   A `None` command is `CheckStatus::Skipped`, never a failure — a repo without a
   type-checker is not a broken repo.

3. **Scope enforcement** — after the agent stops, diff the worktree and record
   any file written outside `ScopeConfig::allow` or inside `deny` into
   `GateOutcome::scope_violations`.

4. **`is_implemented()`** returns `true` once this lands.

## Non-negotiable behaviour

- **Never panic.** A Kit defect must not block real work. Every failure path
  returns a `GateOutcome` or a `FirewallVerdict`, never an unwind.
- **Fail open on our own bugs.** If the firewall cannot parse a command, allow
  it and record why. The threat model is an agent making a good-faith mistake,
  not a human evading the check.
- **Low false positives.** A firewall that cries wolf gets turned off, which
  makes it worth less than nothing. The 30 must-allow fixtures matter more than
  the 20 must-block ones.

## Hard boundaries

- **Never edit** `crates/kit-tui` — Grok Build is working there in parallel
- **Never edit** the frozen contract files: `kit-core/src/run.rs`,
  `kit-core/src/config.rs`, `kit-core/src/gate.rs`, `kit-agents/src/lib.rs`,
  `kit-tui/src/event.rs`. If you need one changed, stop and say so in your final
  message rather than editing it.
- Add dependencies only to `crates/kit-gate/Cargo.toml` and the workspace
  `[workspace.dependencies]` table

## Acceptance

- All 50 Guardian fixtures ported and passing
- `cargo test --workspace` passes
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- A test proves a deliberately failing check produces `passed: false` with a
  readable `summary`
- A test proves an unparseable command is allowed rather than blocked
- Green on ubuntu, macos, and windows in CI

## When you finish

Open a PR against `main`. Do not merge it — Claude reviews every crate-boundary
change. In your final message, list any fixture you could not make pass and why.
