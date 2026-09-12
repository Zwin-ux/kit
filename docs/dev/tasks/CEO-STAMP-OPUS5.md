# CEO stamp — Opus 5 swarm cycle (2026-09-11 → 09-12)

```yaml
alpha_only: true
do_not_merge: true            # no agent merged PR #15; the human decides
version: 1.0.0-alpha.1        # unchanged: root Cargo.toml has no diff from 815e5d7
v1_0_0_tag: none              # origin tags: v0.1.0, v0.1.0-alpha, v0.1.2, v0.1.3, v0.1.4
pr: https://github.com/Zwin-ux/kit/pull/15   # head 815e5d7, open, mergeable, NOT merged
session_b: RECEIPT 01M2A7DVMF9N1RX7XYQGX2A5Q4  # live grok, five-fact bar met; lives on factory/opus5-session-b, not on PR #15
queued_proven_from_run_delta_state: true     # factory/opus5-queued; not on PR #15
ci_on_pr_head: green — 11/11 pass, Sourcery skipped, at 815e5d7 (rechecked at stamp time)
combined_tip: factory/opus5-queued @ d50e1d9 — 17 commits on 815e5d7, local tests green, not yet in CI
security: cmd /C prompt injection in codex/claude adapters (Windows) — real mechanism, latent today, already on main
branches:
  factory/opus5-session-b: 6365179   # 10 commits — Session B
  factory/opus5-queued:    d50e1d9   # session-b + 7 commits — Queued, receipts, clean --json stdout
  ceo/opus5-stamp:         d50e1d9 + this file
```

## Verdict

Alpha-only. No agent merges PR #15.

- **Session B has a real receipt for the first time.** Live grok run `01M2A7DVMF9N1RX7XYQGX2A5Q4` meets the five-fact bar below. Factory, Skeptic and the CEO each checked it on disk.
- **Queued is a real engine state**, proven from `RunDelta::State`.
- **Neither is on PR #15 yet.** Both slices are stacked, without conflicts, on `factory/opus5-queued` (`d50e1d9`), and pass the full suite locally. They reach PR #15 only when a human fast-forwards it. PR #15's own head, `815e5d7`, cannot produce the Session B receipt: it still has both the exit-poll stall and the argv truncation.
- **Two more product holes surfaced:**
  - a Windows `cmd /C` prompt-injection mechanism in the codex and claude adapters (already on main, latent today)
  - a gate hole where a declared check that cannot run counts as passed

Nothing here is 1.0.

## Teammate outcomes

| Teammate | Slice | Output | Outcome | Skeptic |
|---|---|---|---|---|
| factory | Session B (P0) | `factory/opus5-session-b`: 3150cd8, 8eb9784, 063336c, 87fe52c, 04a138f, aab7e4f, d2d01ce, e613bfb, 8c592ff, 6365179 | Session B **receipt**. Exit-poll regression fixed, grok argv fixed, run output streamed, ollama fail-fast, stdin deadlock fixed | approve, 10/10 |
| factory-queued | Queued (P1) + receipt honesty | `factory/opus5-queued` (rebased onto session-b): 1abc927, 04bc2ad, b21575b, ab21b1f, 858cc33, 34e338b, d50e1d9 | Queued proven. Every terminal path writes a receipt first, including kill-before-start and failed runs. Lost-kill race fixed; `--json` stdout clean; test-isolation leak closed | approve, all commits, reviewed before commit |
| luna | TUI remaining gaps (read-only) | report | rustfmt fix PASS. 10 gaps, 6 of them P1 (G1–G6) | G1, G2, G4, G5b, G6 real; G3 real but only at ≤13 rows |
| luna | security shard (read-only) | report + scratch probe | `cmd /C` injection mechanism real | real mechanism, unreachable in the shipped prompt shape |
| skeptic | fail-closed verification | 8 reports | Brief ground truth 12/12. Found the 13f9ca9 regression, the gate skip trap and the run-store leak | — |

After the rebase, the CEO and Skeptic each checked the combined tip independently.

CEO checks:
- `6365179` is an ancestor of `d50e1d9`.
- e613bfb's guard survives at `runner.rs:567`.
- Queued adds only `kit-cli/src/engine/{mod,paths,runner,supervisor,worktree}.rs` and `tests/run_json_stdout.rs` on top of session-b.
- The root `Cargo.toml`, `kit-core` and `kit-tui` are unchanged since 815e5d7.

Skeptic re-check (2g), verdict APPROVE `d50e1d9`:
- `supervisor.rs`, `worktree.rs` and `tests/run_json_stdout.rs` have the same blob ids (CRLF-normalized) as the files Skeptic reviewed.
- `runner.rs` is the reviewed file plus exactly e613bfb's hunks: the guard at `:540`, `:567` and `:579`, and its test at `:898` and `:968`.
- There are no frozen-contract or version changes anywhere in `815e5d7..d50e1d9`.

## Session B — RECEIPT

**Receipt `01M2A7DVMF9N1RX7XYQGX2A5Q4`.** 2026-09-12, 07:13:37Z → 07:14:01Z. Live grok, one attempt, no `KIT_FULL_AUTO`, exit 0 in 24.7 s. Task: append `session-b ok` to README.md.

**The five-fact bar.** All five were checked on disk. This bar replaces the brief's "receipt.json with gateVacuous: false".
1. **New id, not test junk.** Not `01KILLQ…` or `01P3PROOF…`; `spec.agent` is `grok`.
2. **`gate.checks` is non-empty:** `test: echo ok`.
3. **Every check passed, not skipped.** Status `pass`, exit_code 0, 22 ms. `echo` resolved to Git's `echo.exe` under Git Bash.
4. **`output.log` shows spawn and exit.** L2 `kit: spawning grok -p --cwd …\01M2A7DVMF9N1RX7XYQGX2A5Q4 --always-approve`; L207 `kit: grok exited with code 0`.
5. **Not a dry run.** `output.log` L1 is `kit: installed 39 skills for grok`, not `kit dry-run`.

**Corroboration.**
- The diff adds `session-b ok` to README.md.
- grok's own log: pid 29056, `handle_prompt.done` 07:13:59.461Z, `worker_join` 07:14:00.673Z. Kit's receipt ends 07:14:01.779Z.
- Envelope: `ok: true`, `state: pass`, `gatePassed: true`, `gateVacuous: false`. This needed git's leaked `HEAD is now at …` line stripped; d50e1d9 fixes that.
- A second, zero-credit proof: ollama/llama3.1 `01M2A7H6HX5PYN18RV4EP0A4EC`, which meets the same five facts.
- This is the first receipt in the store with a live agent **and** a non-vacuous gate. Session A (`01M06A2PXBBH43ZFF3GJ9VQW94`) was a dry run.

**Open caveats.**
- **Provenance.** The binary was built at 00:13:05 local from the then-uncommitted tree; `e613bfb` was committed at 00:15:29. Its behaviour matches the committed fix, but byte equality is unproven.
- **The closing proof was prepared but NOT run.** It uses a clean build of the committed combined tip `d50e1d9`, a `git --version` gate, and stdout that must parse as one envelope. Claude Code's auto-mode permission classifier denied factory-queued's live grok launch ("Create Unsafe Agents"). The swarm did not route around the denial. It is the human's decision; the command is under Human-only.
- **`echo ok` proves the plumbing, not quality.**

**Root causes.**
1. **The stall was Kit's, not grok's.** grok's own log shows both original Session B runs finishing ok and exiting minutes before Kit was killed.
   - The bug is in `runner.rs` `live_agent`, a `biased` select loop. Once the agent exits, its pipe readers drop every sender, so `recv()` returns `None` immediately, forever, and starves the `try_wait` tick. Kit then spins until the 30-minute bound.
   - This is a regression from `13f9ca9` (2026-08-01), which replaced `None => break wait_fut.await` with an empty arm. No test reached `live_agent`.
   - Fixed in `e613bfb`: an `if pipes_open` guard plus `live_agent_sees_exit_after_output_pipes_close`, which fails on the old loop with "exit poll starved".
   - This also mattered for Queued: every live Control Room run used to hold its permit for 30 minutes after the agent exited.
2. **Grok argv truncation.** `cmd /C` ends the command line at the first newline, so grok got only `-p "# Kit Control Room — agent run"`. Fixed in `3150cd8` (grok is spawned directly). `--output-format streaming-json` was never the problem.
3. **Invisible runs.** `kit run` called `execute(opts, None, None)` in both modes. Fixed in `8eb9784`: deltas stream to stderr, with a 30 s silence notice.
4. **Ollama.** The default `llama3.2` isn't pulled here, so a run silently pulls it. Fixed in `063336c` (fail fast with a remedy). `aab7e4f` separately fixes a stdin-before-readers deadlock, reproduced with an 8 MiB prompt.

## Queued is a real engine state — proven

- **Queued is always first.** The job task sends `State(Queued)` on the same delta channel before the permit `select!`. `Running` comes later, from the same task, so Queued can never follow Running or a terminal state.
- **Test timing is out of production.** `ConcurrencyProbe`, the 250 ms hold, the in-body assert and `proof_dispatch_n` are all deleted.
- **Every run ends with exactly one terminal delta, and its receipt is written first on every path.** That covers the execute tail, an in-run kill, a kill before start (queued, or between permit and execute), and a failed run (`finalize_failed`, which sends Error even if its receipt write fails). The permit is held until the terminal delta is out, so the cap holds in channel order.
- **The proof is the delta stream alone,** through the real `run_supervisor`, forced dry.
  - A fixture gate latch pins 8 runs in Gating while 4 sit Queued. That is fixture timing, not engine sleeps.
  - A ledger asserts on every delta: Queued comes first, nothing follows a terminal state, and no more than 8 runs are Running.
  - After release there are exactly 12 id-keyed receipts, each `[Queued, Running, Gating, Pass]`, and 12 unique worktrees. No log mentions another run's id or task.
  - `kill_while_queued_writes_receipt_without_worktree`, `kill_right_after_start_is_not_lost`, `failed_run_ends_with_error_receipt`, and `panicking_teardown_keeps_kit_home_on_scratch` each failed on the old code first.
- **`--json` stdout is one JSON value.** Git's worktree output now goes to stderr, pinned by `tests/run_json_stdout.rs` against the real binary.
- **Open gaps (slice 3):**
  - Receipt-first on started runs has no test; reverting it stays green.
  - Headless `kit run` errors leave no receipt and no envelope.
  - The test teardown keeps a small window: the lock releases before stray jobs are cancelled.
  - Kill between permit and execute has no dedicated test.

## Security — `cmd /C` prompt injection (real mechanism, latent today)

- **Mechanism.** `codex.rs:68` and `claude.rs:49` pass the prompt through `command_for` → `cmd /C` (`process.rs:16-21`). Rust quotes `"` as `\"`, but cmd.exe honours no escapes.
- **Proof.** A harmless probe showed `a" & echo KIT_INJECTED & "b` running `echo KIT_INJECTED` as a host command.
- **Untrusted input.** The retry task embeds the first line of gate output verbatim (`kit-tui/src/app.rs:1242`, `:1259-1261`; `kit-gate/src/lib.rs:879`, `:946-966`), and `r` dispatches with no confirmation.
- **Why it's latent.** `build_prompt` always starts with a fixed heading and a newline, and cmd discards everything after the first newline. So today's truncation bug keeps the payload away from cmd.
- **When it goes live.** A Kit-shaped probe confirmed that a "fix" which flattens the prompt to one line under `cmd /C` executes the payload.
- **Scope.** Already on `origin/main` (`codex.rs:49`, `claude.rs:48`, `process.rs:20`). Not affected: grok on these branches (direct spawn), ollama (stdin), probes, Unix.

## Other verified findings

- **Gate hole.** kit-gate records a declared check that cannot be parsed, found or started as `Skipped` (`kit-gate/src/lib.rs:852`, `:856-857`, `:859`). `GateCheck::passed()` counts `Skipped` as passed (`kit-core/src/gate.rs`), so a check that never ran reports PASS. On this machine's PowerShell PATH there is no `echo.exe`.
- **Windows probe reports ready for missing agents** (Luna G5b, reproduced at zero credit). With grok and ollama removed from PATH, `kit doctor --json` still says `ready: true`, because cmd's "not recognized" stderr counts as success.
- **`--demo` reaches the real engine** (Luna G1, P0). `r` on the demo FAIL row starts a billable live run in the launch repo with the fixture brief.
- **Run-store pollution.** Test junk sits in the real `~/.kit/runs`:
  - `01KILLQ0006`, from this cycle's dev loop (the path is closed by `858cc33`)
  - `01P3PROOF00000000000000000006`…`0010`, from 2026-08-02
- **Live worktrees are never removed.** Kit's own untracked `.agents/` and `AGENTS.md` make every live worktree look dirty.
- **A pre-existing `live_agent` race.** If the exit is seen while the pipes are still open, trailing lines can be dropped. Rare; present since 13f9ca9.
- **kit-gate local test failure.** `gate_child_does_not_write_parent_cargo_target_dir` fails only on this machine (`NoDefaultCurrentDirectoryInExePath=1`); CI passes it.

## CI matrix

**PR #15 head `815e5d7` (GitHub).**

| Workflow | Job | Result | What it actually proves |
|---|---|---|---|
| Rust | ubuntu-latest | pass | `cargo fmt --check`, `clippy --workspace --all-targets -D warnings`, `cargo test --workspace` (124 tests) |
| Rust | macos-latest | pass | same, 124 tests |
| Rust | windows-latest | pass | same, 124 tests |
| Rust | startup budget (ubuntu) | pass | `kit --version` mean 1.1 ms (budget 100 ms). Not first paint (P2 open). |
| Rust | startup budget (windows) | pass | `kit --version` mean 6.8 ms |
| CI (Node) | Quality × 6 (3 OS × Node 20/22) | pass | legacy 0.1 `packages/`. It runs on this PR because the diff touches `.github/workflows/ci.yml` and adds `scripts/use-rust-kit.ps1`, which matches `scripts/**`. |
| Sourcery | review | skipping | — |

The previous head, 225a6e6, failed on Format only, on all 3 OSes. 815e5d7 (+8/−1 in `kit-tui/src/theme.rs`) fixed it.

**Combined tip `d50e1d9` (local Windows only; not yet in CI).**
- `cargo fmt --all --check` passes.
- `cargo clippy --workspace --all-targets -D warnings` passes.
- `cargo test --workspace` passes: kit-agents 14, kit-cli 27 plus `run_json_stdout` 1, kit-core 4, kit-tui 82. kit-gate is 7/8, from the local environment failure above.
- Supervisor tests passed 5/5 runs; the JSON guard passed 3/3.
- The Unix branch of the latch fixture compiles for the first time in CI.

## Kill criteria (8)

1. **CI red.** Any Rust job red on the PR head (fmt, clippy `-D warnings`, `cargo test --workspace` on 3 OSes, startup budget) → no merge, no stamp upgrade.
2. **Terminal without receipt.** Any terminal `RunState` without `receipt.json` → the "nothing ships unproven" claim is blocked. Supervisor paths are closed by the combined tip. Still open: headless `kit run` errors and Ctrl-C.
3. **Session B without a live receipt.** Any of the five facts missing → Session B BLOCKED, alpha.2 (S1) blocked. Dry-run, `--demo`, a dry-run fallback, and hung spawns never count.
4. **Silent or spinning run.** A live `kit run` that writes nothing to stderr for more than 120 s, or keeps running after its agent exited → design failure; Session C blocked.
5. **Proof hygiene.** Any of these voids the proof:
   - a sleep, hold or assert inside a production path, used as evidence
   - a test that does not fail without its fix
   - a declared gate check that never ran, counted as passed
6. **Untrusted text on a shell line.** Any adapter passing prompt, task or gate text through `cmd /C` (or any shell) → no alpha.2, no promotion, no stranger install instructions.
7. **Contracts, version, tag, merge.** Any of these stops the cycle and gets reverted:
   - a frozen-contract edit without a CEO stamp (kit-core run/config/gate, kit-agents trait, `event.rs`)
   - `Cargo.toml` ≠ `1.0.0-alpha.1`
   - any `v1.0.0` tag
   - any agent merging PR #15
8. **Control Room dishonesty.** Any of these blocks the Power slice and Session C:
   - a QUEUED row that animates or reads as RUNNING
   - rows hidden with no count (fan-out 16 vs engine cap 8)
   - `--demo` fixture rows reaching the engine

## Next three slices

1. **Factory — Windows adapter hardening (P1, security). Runs in parallel with slice 2.**
   - codex takes the prompt on stdin: `codex exec -`. Its npm `.cmd` shim can't take a multi-line argument, and Rust refuses to pass one.
   - claude takes the prompt on stdin, or is spawned directly as the native `claude.exe` when it resolves.
   - Never flatten the prompt under `cmd /C`.
   - On Windows, the probe treats cmd "not recognized" or a non-zero exit with no version as **missing** (G5b).
   - Acceptance:
     - exact-argv tests show no prompt text in any `cmd /C` argv
     - the Kit-shaped injection probe runs nothing through the real adapter path
     - `kit doctor --json` with grok hidden from PATH reports it missing
     - one live smoke per adapter with a one-line task, on human-authorised credits
   - Files: `crates/kit-agents`. No contract change.
2. **Power — Control Room: demo safety and 16-way honesty.**
   - `--demo` fixture rows are inert: `r`/`k` flash "demo row — press d" and emit no `EngineCommand` (G1, P0).
   - A viewport with "+N more" (G3).
   - The header keeps counts and flashes when the probe strip is present (G2).
   - QUEUED is counted as `8 running · N queued` (G7).
   - Repo basenames in rows (G4).
   - The reduced-motion clock (ruling below).
   - Acceptance: snapshots at 80×14 and 60×12 with 16 rows and a mixed probe strip; a test proving demo rows emit no engine command.
   - Owner: kit-tui. No `event.rs` change.
3. **Factory — Engine and gate honesty. Follows slice 1.**
   - kit-gate: a declared check that cannot run → **Fail** with a remedy, never Skipped.
   - Headless `kit run` errors → an Error receipt plus a `--json` error envelope (`main.rs` calls `finalize_failed`).
   - Ctrl-C of `kit run` → a Killed receipt.
   - Tests that pin receipt-first for started runs, and the stderr echo wiring.
   - Harness aborts job tasks before the lock drops.
   - git worktree output goes to `RunDelta::Output`, so it stops scribbling over the Control Room.
   - Worktree cleanup ignores Kit's own injected files.
   - A P7 mock-binary live adapter test.
   - The kit-gate test calls its probe by absolute path.
   - Acceptance:
     - each item has a test that fails today
     - `kit run --json | jq` parses on success and on error
     - a missing gate command fails the gate

## CEO rulings this cycle

- A never-started run records `started_at: None` and `branch: None`.
- `--demo` fixture rows never reach the engine. Dispatches made from inside `--demo` are real, because Session C needs them.
- Reduced motion stops the spinner, not the clock. Elapsed text still advances, at 1 Hz or slower, as a text-only repaint. Power applies this in slice 2; the CEO amends `DESIGN-tui.md:87` then.
- The Session B bar is the five facts above.
- A declared gate check that cannot run is **Fail**, decided in kit-gate. `Skipped` keeps meaning "not configured", and the `kit-core` contract is unchanged.
- The codex/claude truncation fix uses stdin, or direct spawn of a native binary. It never flattens the prompt under `cmd /C`.
- G6 and receipt-before-terminal were accepted into this cycle, because they are the brief's "terminal without receipt is a bug".

## Corrections to the brief (verified)

1. **The Session B bar was mis-specified.** `gateVacuous` is envelope-only, and it also reads `false` for a dry-run fallback.
2. **The grok flag "likely cause" was wrong.** The flag is valid. The hang was the engine exit poll (`13f9ca9`); the argv truncation was a separate bug.
3. **Ollama's "hang" was a silent pull of the missing default model.** Its stdin deadlock was a separate, real risk; it is now fixed.
4. **Session A was a dry run.** Its `output.log` reads `kit dry-run · agent=codex`, and the diff is empty.
5. **`kit run` was blind in both modes,** not only under `--json`.
6. **N1 has a second trigger.** `scripts/**` also triggers Node CI, so "Rust-only PRs after merge" proves N1 only for PRs that stay out of `scripts/**`.
7. **Queued was already painted by the TUI** as an assumption. The real gap was engine confirmation, plus receipts for kills that never started.

## Process notes (swarm harness)

- **`EnterWorktree` is session-global under agent teams.** One teammate entering its worktree re-pins every teammate and the lead, and git outside the pinned tree is refused. The pin moved about seven times this cycle. It was then run lead-controlled: teammates edited and built by absolute path and committed only while holding the pin.
  - Next swarm: the CEO pre-creates worktrees and hands the pin over explicitly.
- **Messages to busy teammates crossed twice.** Scope instructions for G6 flipped once before settling. Next time: one instruction per message, and wait for an ack before issuing a reversal.
- **The final live proof was blocked by Claude Code's permission classifier.** It was surfaced to the human, not routed through another agent.
- **Credits.** Two live grok runs were spent: one stalled run, `01M2A5KZ7G5ARQ0A06S4V3WB8N`, and the receipt run. Direct A/B probes were minimal.
- **Trailer nit.** 14 of 17 Factory commits say "Claude Opus 5 (1M context)".

## How to integrate (human decision; the CEO did not merge)

The history is linear: `815e5d7` → session-b (10) → queued (7) → this stamp (1).

```text
git fetch origin
git switch feat/1.0-dogfood-kit-toml
git merge --ff-only origin/ceo/opus5-stamp        # or origin/factory/opus5-queued for code only
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git push                                          # PR #15 re-runs the full matrix on the combined head
```

## Human-only

- **Optional closing proof for Session B.** Your call: it spends grok credits, and the auto-mode classifier blocked an agent from launching it. It needs the factory-queued worktree's `target/` (built binary and fixture). Afterwards, check:
  - a new run id
  - `git --version` status `pass`
  - spawn and `exited with code 0` lines in `output.log`
  - not a dry run
  - `target/sessionb-final-out.json` parses as one JSON value

  ```text
  cd C:/Users/mzwin/kit/.claude/worktrees/factory-queued
  timeout 120 target/debug/kit.exe run --agent grok --live --json --repo C:/Users/mzwin/kit/.claude/worktrees/factory-queued/target/sessionb-final-fixture --task "Append exactly one line 'session-b final' to README.md. One-line docs edit: do not run tests or explore." > target/sessionb-final-out.json 2> target/sessionb-final-err.log
  ```
- **The PR #15 merge decision,** after the combined head is green in CI. Never an agent.
- **I1:** GitHub About / topics / the v0.1.4 "not 1.0" note.
- **N3:** the npm `@mzwin/kit` bin decision.
- **Credits** for the codex and claude smoke runs in slice 1.
- **Cleanup, once no longer needed as evidence:**
  - run-store junk: `~/.kit/runs/01KILLQ0006`, `01P3PROOF00000000000000000006`…`0010`
  - Session B worktrees: `~/.kit/worktrees/01M2A1QHG4…`, `01M2A25VGD…`, `01M2A2GSTN…`, `01M2A5KZ7G5ARQ0A06S4V3WB8N`, `01M2A7DVMF9N1RX7XYQGX2A5Q4`, `01M2A7H6HX5PYN18RV4EP0A4EC`
  - this cycle's `.claude/worktrees/{factory-session-b,factory-queued,ceo-opus5,ro-815e5d7}`, via `git worktree remove` after integration
  - older: `C:\Users\mzwin\kit-factory-n1`
