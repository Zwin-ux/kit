> REJECTED / SUPERSEDED: The user rejected this expanded design on 2026-10-01. Do not implement it. Read README.md for the minimal native-Claude direction.

# V2 implementation sequence

Start only after the V1 release gate. This plan implements [V2-SPEC](V2-SPEC.md) using [V2-CONTRACTS](V2-CONTRACTS.md); the contract file defines proposed flags and exact payloads, not capabilities currently available.

## Execution rules

Use the existing Rust tests and mods test runner. Keep changes in isolated current-upstream work. Every slice leaves a useful tested state; do not create all placeholder screens first. Commit/review by coherent slice under the repository's normal workflow.

The mod keeps all API calls and helpers receiving `$` in `hooks/register.js`. Extract only pure parsing/formatting to `hooks/kit.js` and pure action transitions to `hooks/actions.js`. Do not import a helper that receives `$`; validation will reject it.

Source of truth is Rust Kit. The mod never edits Kit locks, receipts, worktrees, or Claude settings directly.

## Dependency order

    V1 complete
       ├─ passive doctor contract ─┐
       └─ scoped receipts contract ├─ passive workspace + parser tests
                                   ├─ reviewed kit management
                                   ├─ reviewed branch creation
                                   └─ run review + events + cancellation proof
                                         └─ run/retry/stop UI
                                                └─ integrated V2 release

The independent CLI slices can use separate agents after their contracts are fixed. One coordinator owns integration. Two agents must not simultaneously edit main.rs/cli.rs; assign ownership or merge narrow patches sequentially.

## Slice 1: Passive diagnostics

Files: `crates/kit-cli/src/cli.rs`, `main.rs`, `kits/doctor.rs`, `tests/kits_cli.rs`, `docs/json-contract.md`.

- [ ] Add a failing test with executable-check/MCP/agent sentinel programs; passive inspection must not invoke any.
- [ ] Implement `doctor --passive`, conflict with `--start-mcp`, exact readiness and skipped-check fields.
- [ ] Preserve inspection errors and partial data; never fake successful empty lists.
- [ ] Advertise only implemented capability strings from the fixed contract list.
- [ ] Run `cargo test -p kitctl --test kits_cli` and the affected doctor unit tests.

Done: the documented passive envelope is available with zero configured-check/MCP/agent execution. Existing explicit diagnostics retain their tested behavior.

## Slice 2: Receipt identity and evidence

Files: CLI `cli.rs`, `main.rs`, `engine/store.rs`, existing/new focused receipt tests, JSON contract docs.

- [ ] Failing fixtures: same-name repositories, more than 50 newer foreign receipts, corrupt receipts, missing evidence, legacy vacuous pass, output-tail truncation.
- [ ] Implement current-repo filtering before limit, canonical recorded paths, nullable evidence convenience fields, and hasMore.
- [ ] Preserve existing fields/default CLI ordering; document stored-history order honestly.
- [ ] Add detail convenience fields and separate capture/tail truncation. Avoid rewriting stored receipts.
- [ ] Run store tests and receipt CLI fixtures; compare a real receipt list/detail in a disposable repo.

Done: current-repo history is accurate even when the global newest rows belong elsewhere. Unknown proof stays unknown.

## Slice 3: Passive workspace

Files: mod `register.js`, pure `kit.js`, pure view module if V1 now needs extraction, `tests/kit.test.ts`, README.

- [ ] Add parser fixtures from real baseline and extended CLI envelopes. Assert command identity (`doctor`, `list`, `receipt.list`, `receipt.show`), schema and nested mixed-case fields.
- [ ] Add bounded argv process reads with explicit cwd and trusted binary. No fallback from unsupported passive mode to active doctor.
- [ ] Build Overview/Kits/Runs and retain Appearance. One `list --json` already contains both scopes.
- [ ] Implement generation-aware refresh, stale last-good data, no-repo/missing CLI/incompatible/denied/empty states.
- [ ] Add lazy run detail/output reads and bounded sanitized display.
- [ ] Verify opening or refreshing invokes only passive reads; painting invokes none. Plugin load alone performs no integration reads.

Done: actual CLI output matches the visible selected scope. User can navigate entirely by keyboard and distinguish this session from Kit receipts. This is a releasable read-only V2 beta, not full V2.

## Slice 4: Complete reviewable install/remove contracts

Split this into two narrow Rust changes if shared main.rs/cli.rs work would otherwise overlap.

Files: CLI `cli.rs`, `kits/install.rs`, `plan.rs`, `lock.rs`, affected sync writer, `tests/kits_cli.rs`/`marketplace_cli.rs`.

- [ ] Extend add review to expose all pins/licenses/stale removals/code-running commands.
- [ ] Extract true remove preview before all target/metadata writes.
- [ ] Implement deterministic review digest and apply expected-plan precondition with no second resolution after validation.
- [ ] Reuse existing rollback/ownership code and scope writer coordination. Reject concurrent or edited preconditions without force.
- [ ] Test source/target/options races, metadata-only remove, retained dependency/manual edits, failed apply rollback, and duplicate apply.

Done: a user-reviewed plan binds the content actually applied; either preview makes no target/ownership mutation. Add's disclosed source cache/network behavior is separate.

## Slice 5: Management UI

Files: mod entry, pure actions/view, action tests.

- [ ] Build source/scope/agent selection and skills/rules-only choice.
- [ ] Render every structured review item, including code-running entries and retained/skip/conflict reasons.
- [ ] Confirm exact digest; disable duplicate submissions; changed digest returns to review.
- [ ] On uncertain interruption inspect authoritative records before retry.
- [ ] Reuse existing removal semantics; do not invent per-agent removal if the engine removes the kit at scope level.

Done: real add/remove flows match reviewed inputs and preserve edited files. No hidden global change or force action.

## Slice 6: Review-bound branch creation

Files: CLI `cli.rs`, `land.rs`, `land/git.rs`, `tests/land_cli.rs`; then mod entry/actions/view/tests in a separate change.

- [ ] Add land preview and expected-plan validation using existing proof/base/patch/idempotence paths.
- [ ] Test changed receipt/patch/HEAD/branch and dirty working tree protection.
- [ ] UI action is Create result branch; compare preview repo identity to selected scope.
- [ ] Execute only normal branch mode, display returned branch/commit and next action.

Done: real branch result matches review, unchanged duplicate request is idempotent, no merge/push/force occurs.

## Slice 7: Review-bound run launch

Files: CLI `cli.rs`, `main.rs`, `engine/runner.rs`, `worktree.rs`, affected inference/tests.

- [ ] Add no-launch run preview through the same config/inference path as execution.
- [ ] Validate reviewed task/agent/base/check-plan before worktree or agent creation.
- [ ] Pass the validated base and frozen checks forward; no later HEAD lookup can substitute an unreviewed commit.
- [ ] Test changed HEAD/config/inference inputs, missing commit, leading-dash/metacharacter task, duplicate start, and zero side effects during preview.

Done: execution uses exactly the reviewed input set or refuses before starting work.

## Slice 8: Live events and owned cancellation

Files: CLI event writer/runner/cancel integration and existing run/process tests. API proof in a disposable mod session.

- [ ] Implement opt-in JSONL from existing RunDelta, one sequence owner, bounded encoded lines, one final result, existing final-only JSON unchanged.
- [ ] Prove supported signal delivery from mods API to the owned Kit child. Do not assume closing a stream terminates descendants.
- [ ] Only if signal delivery is unavailable, consider the specified narrow stdin cancel transport; do not implement both without need.
- [ ] Test cancel during agent/checks, pipe loss, duplicate cancel, receipt-write failure, no orphan descendants, unrelated sessions survive.
- [ ] Prove real nested Claude execution and approval experience without bypass variables. Evaluate other adapters independently.
- [ ] Define host reload/disable/exit behavior from actual evidence. On lost authority, unknown/detached is honest; Stop is unavailable.

Done: live state and stop behavior are authoritative. Unsupported platform/adapter combinations remain handoffs. No daemon added to avoid proving lifecycle.

## Slice 9: Run/retry/stop UI

Files: mod entry/pure actions/view/action tests, README.

- [ ] Build task review with committed base/checks/uncommitted-change exclusion.
- [ ] Keep one owned active run per session and separate pending/starting/running/gating/stopping/final/unknown states.
- [ ] Read typed events and final envelope; output is display data only.
- [ ] Retry is a reviewed new task with explicit parent relationship if supported, never an automatic loop.
- [ ] Stop is only for a current owned run, shows identity, and waits for acknowledged final state.
- [ ] Closing UI does not stop work; unloading follows the proven lifecycle. Reopening never duplicates the run.

Done: a real failed run is understood, reviewed retry passes real checks, and the result can be reviewed into a branch; stop leaves no orphan.

## Slice 10: Integrated release

- [ ] Preserve V1 appearance key and direct command through upgrade; no duplicated `/kit` or band.
- [ ] All required native mod/Rust checks pass on the release commit.
- [ ] Complete real terminal and separately supported Desktop journeys, including permission refusal and missing/incompatible CLI.
- [ ] Test from release artifacts in a clean environment; document exact minimum Kit/Claude versions and supported capability matrix.
- [ ] Capture real evidence and keep conceptual mockups labeled concept art.
- [ ] Publish truthful feature status: read-only beta vs full enabled V2 actions. A compile-time feature is not runtime support.

## Full verification commands

```bash
claude plugin validate ./mods/kit-red
claude plugin test ./mods/kit-red
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run focused tests after each slice; run the full gate once the integrated change is ready. Repeat broader tests only after relevant changes or new evidence.
