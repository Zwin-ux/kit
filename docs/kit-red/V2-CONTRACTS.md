> REJECTED / SUPERSEDED: The user rejected this expanded design on 2026-10-01. Do not implement it. Read README.md for the minimal native-Claude direction.

# Kit Red V2 — additive CLI contracts and verification

Status: proposed implementation contracts, not existing capabilities. Source baseline: Zwin-ux/kit `c35f60855a62534f2ed71edac3d1dc52835b87a3`. This document was produced by source inspection, not running the proposed commands.

V1 Kit Red is visual-only: zero Kit CLI discovery, version, doctor, list, run, or other process calls. Everything below belongs to V2 Kit inside Claude. V1 must not grow partial integration to satisfy these contracts.

## Build baseline and compatibility

The local checkout inspected earlier is older than this baseline. Implement in an isolated current-upstream checkout, preserving user work. The Rust CLI crate directory is `crates/kit-cli`, but its Cargo package name at this baseline is **`kitctl`**, its executable is **`kit`**, and its version is inherited from workspace `Cargo.toml`. The mod version and compatible CLI version are separate. Do not publish a guessed minimum Kit version before these additions exist in a released artifact.

Retain `schemaVersion: 1` and every existing default CLI behavior. Add fields/flags without renaming old receipt fields. Unknown new flags on older Kit must produce an Update Kit state, never fallback to an active/mutating operation. Existing `run --json` keeps exactly one final envelope. JSONL is a separate opt-in mode.

Envelope remains:

```ts
type Envelope<T> = {
  schemaVersion: 1;
  command: string;
  ok: boolean;
  data: T | null;
  error: string | null;
  warnings: string[];
};
```

Add optional `errorCode: string | null` to affected commands; do not require this field when reading older envelopes. New refusals use `PRECONDITION_CHANGED`, `UNSUPPORTED_OPTION`, `CANCEL_CONTROL_LOST`, or `INSPECTION_FAILED`; preserve a human-readable error. `ok:false` may coexist with useful `data`, and existing FAIL/UNCONFIGURED final run envelopes can have `error:null`. Parse structured output before interpreting exit code. New input/refusal errors use exit 2; run PASS remains 0, FAIL/UNCONFIGURED remains 1 except existing explicit opt-ins.

Times in NEW convenience fields use nullable RFC3339 UTC strings (`2026-10-01T12:34:56.123Z`). Existing receipt detail still contains Rust snake_case/SystemTime fields; do not migrate stored receipts merely to simplify JavaScript. Add conversion at the CLI boundary, with a round-trip timestamp test. Reuse an already-present timestamp dependency if one exists; do not add a broad date framework.

## 1. Passive doctor — required before V2 Overview

Command: `kit -C <repo-root> doctor --passive --json`.

`--passive` defaults false. It conflicts with `--start-mcp`. The new mode performs file/config/binary-presence inspection, but executes no agent binary, version/auth probe, custom kit check, or MCP server. Executing the trusted Kit binary itself and bounded Git repository discovery is inherent in this command; it is not a claim that the operating system launches zero processes.

Retain existing doctor fields; add:

```ts
type PassiveDoctorAdditions = {
  inspectionMode: "passive";
  repoPath: string | null;
  capabilities: string[];
  agents: Array<{
    agent: "claude" | "codex" | "grok" | "ollama";
    installed: boolean;
    ready: false;            // no readiness was established by passive inspection
    version: null;
    remedy: string | null;
    readiness: "not-probed" | "missing";
  }>;
  checksSkipped: Array<{kit: string; kind: "command" | "mcp"; reason: string}>;
};
```

`ready:false` is retained for backward compatibility and must not become an “authentication failed” label. V2 uses `readiness`; explicit user-triggered diagnostics can establish actual auth/probe results. Existing kit check arrays contain only checks actually performed; skipped executable checks belong to checksSkipped, never fabricated PASS/FAIL rows. Inspection failure produces `ok:false`, code INSPECTION_FAILED and available partial data, rather than an empty successful list.

`capabilities` is the exact list of implemented features from: `doctor.passive.v1`, `receipt.scoped.v1`, `kits.review.v1`, `run.review.v1`, `run.events.v1`, `run.stdin-cancel.v1`, `land.review.v1`. This is a small fixed feature list, not a plugin protocol registry. Advertise only shipped/tested capabilities. No banner-triggered capability call in V1.

Files: `crates/kit-cli/src/cli.rs` DoctorArgs; `main.rs` print_doctor; `kits/doctor.rs` check_installed/check; `docs/json-contract.md`; existing `tests/kits_cli.rs`.

Checks: custom check that writes a sentinel is skipped; fake claude/codex/grok/ollama binaries that write invocation sentinels are never invoked; --passive plus --start-mcp refuses; corrupted local ownership record is visible failure; skipped checks do not become PASS. Preserve current explicit doctor behavior in a sibling fixture.

## 2. Receipt list and detail — required before V2 Runs

Command: `kit -C <repo-root> receipt list --current-repo --limit 50 --json`.

`--current-repo` defaults false (existing CLI remains all projects). `--limit` keeps default 50, with existing 0 meaning unlimited; the UI always requests 50. Do not overload global `--repo`/`-C`, which currently selects cwd. Filter on the full recorded repository identity before limit.

Add to list data:

```ts
type ReceiptListAdditions = {
  filter: {repoPath: string | null}; // null means all projects
  hasMore: boolean;
  receipts: Array<{
    // Existing id/state/agent/repo/task/gatePassed/dir remain unchanged.
    repoPath: string;             // absolute stored root, canonicalized if it exists
    startedAt: string | null;
    endedAt: string | null;
    gateVacuous: boolean | null;  // null when proof is absent
    hasChanges: boolean;
  }>;
};
```

The existing `repo` remains the basename display label. `count` remains returned count, not total. Keep current directory-mtime/id sort to avoid an unrelated behavioral migration; document this as stored-history order, not guaranteed creation order. `hasMore` uses one extra matching row. Invalid receipts skipped during enumeration produce a bounded warning/count, never raw corrupt contents.

Repository identity: resolve current project through existing `worktree::resolve_repo`. Canonicalize recorded paths when possible; for deleted paths retain their absolute recorded value. Do not match moved repositories by basename or guess identity across worktrees. Treat linked worktrees as distinct roots for this release. UI says no matching history after an unrecorded move; all-project view remains available.

`receipt show <full-id> --json` retains the existing raw receipt. Add `repoPath`, `startedAt`, `endedAt`, `baseCommit` (from validated base.txt or null), `gateVacuous`, `hasChanges`. With --output only, add `outputTailTruncated:boolean`; existing `output_truncated` describes total run capture, not this tail. Current output tail remains 64 KiB. Validate full action IDs as ULIDs at the UI boundary; prefix support remains existing CLI behavior.

Files: cli.rs ListArgs; main.rs cmd_receipt_list/cmd_receipt_show; engine/store.rs ReceiptSummary/list_receipts/read_output_tail; existing store unit tests; add `tests/receipt_cli.rs` only if no existing integration file naturally owns this behavior.

Checks: two repos with identical basenames; 60 newer foreign records and older matching history; limits 0/1/50; no/missing/corrupt receipts; symlinked current cwd; deleted recorded root; legacy vacuous PASS; absent gate stays unknown; absent timestamp; exact vs truncated tail. Full detail task is authoritative; list task is currently shortened to 48 chars.

## 3. Reviewed add/remove — required before V2 management buttons

Reuse current `add --print --json` and introduce `remove --print --json`. Keep `add --json` without --yes as its existing preview. Keep existing remove default behavior for terminal users except that the new --print path must return before ALL writes, including metadata-only requested-flag changes.

Apply syntax:

```text
kit -C <root> add <source> --agent claude --no-code --yes --expect-plan sha256:<64-lowercase-hex> --json
kit -C <root> remove <installed-name> --yes --expect-plan sha256:<64-lowercase-hex> --json
```

`--expect-plan` defaults absent; it requires --yes and conflicts with --print and --force. The UI always supplies it and never --force. Global scope remains explicit --global. Remove affects the installed kit in its selected scope with existing dependency ownership semantics; do not imply per-agent removal support that does not exist.

Add `review` to add/remove preview and matching apply result:

```ts
type KitReview = {
  version: 1;
  digest: string; // sha256:<hex>
  operation: "add" | "remove";
  scope: {kind:"project"|"global"; root:string};
  agents: string[];
  noCode: boolean;
  sources: Array<{
    kit:string; source:string|null; resolvedRevision:string|null;
    contentDigest:string; license:string|null;
  }>;
  changes: Array<{
    kit:string; key:string; path:string|null;
    operation:"add"|"replace"|"remove"|"keep"|"skip";
    description:string;
    runsCode:boolean;
    commands:string[];
    conflict:string|null;
  }>;
};
```

All new review fields are always present. For removal, noCode is false because the flag does not apply, agents lists the agents affected by the existing installation record, and source:null means the historical source was not recorded. `license:null` means not declared, not an inferred permissive license. Changes include stale removals, metadata-only changes and shared/dependency-retained items. Commands include doctor-check code as well as hook/MCP/gate code. For removal, source metadata comes from the local authoritative installation record if available; no remote fetch is necessary merely to uninstall, and unknown metadata remains null.

Digest is computed by Rust over a fixed serialized review input using existing sha2, not over terminal prose or JS-rendered text. Internal hashed input additionally includes full source file paths/bytes, resolved dependency content, options, authoritative lock content and target preconditions. Canonical serialization uses fixed structs and deterministic sorted maps/arrays; strip no meaningful source bytes. Never hash only version/name/skill files: rules/hooks/MCP also matter. Digest is a review-consistency check, not authentication or a permission credential.

Apply rebuilds the plan, checks the digest before the first change, then applies THAT resolved plan through current plan.rs apply/rollback. It must not resolve again after validation. On changed source/target/lock/option return PRECONDITION_CHANGED and `data:{review:<new-review>,applied:false}`. UI returns to review; it never auto-confirms the new plan. Reuse `install::Expect` where applicable but do not mistake its version/skill checks for complete binding.

Handle concurrent Kit writers through one scope mutation guard used by all affected Kit install/remove/sync write paths, plus immediate target precondition checking. Reuse existing guard if present; otherwise add the smallest shared guard scoped to the authoritative ownership record. Define crash/stale recovery before shipping rather than deleting an arbitrary lock file automatically. This does not promise isolation from an external editor rewriting files during apply; retain existing manual-edit preservation and rollback guarantees, and report conflicts rather than blanket overwrite. Global and project records must not be protected by unrelated per-pane flags alone.

Files: cli.rs AddArgs/RemoveArgs; kits/install.rs prepare/add_to/cmd_remove/print_json; kits/plan.rs existing Action/Applied/apply_all; kits/lock.rs ownership; kits/sync.rs shared writer entry where needed; fetch.rs existing content hash; tests/kits_cli.rs and marketplace_cli.rs. Do not add a second JS installer or serialize executable Rust Action objects into UI-owned files.

Checks: no target/ownership writes during either preview (remote cache fetch is allowed and documented for add); changed tag, local skill, rule, hook, MCP, dependency, target file or lock invalidates apply; different no-code/scope/agent options invalidate; metadata-only remove preview makes zero writes; concurrent duplicate applies converge/refuse safely; shared assets remain; edited files remain; apply failure uses current rollback path. Preview must clearly label that source retrieval can use network/cache, while installing remains unperformed.

## 4. Run review, base and checks — required before V2 launch

New non-launching command shape: `kit -C <root> run --task <text> --agent <agent> --print --json`.

--print conflicts with --dry-run and --allow-vacuous. Unlike dry-run, it runs neither agent nor checks, creates no worktree/receipt and consumes no model call. It uses the real engine configuration/inference path to load exactly the proposed checks.

Response command remains `run`; data:

```ts
type RunReview = {
  preview:true;
  reviewVersion:1;
  planDigest:string;
  repoPath:string;
  baseCommit:string; // validated full Git object id, not hardcoded SHA-1 length
  agent:string;
  task:string;
  checks:Array<{label:string;command:string}>;
  checksDigest:string;
  gateSource:"configured"|"inferred"|"empty";
  uncommittedChanges:number;
  launchReadiness:"not-probed";
};
```

Task/agent/base/check digest/bounds/config interpretation are in planDigest. New launch flags `--expect-plan <digest> --expect-base <oid> --expect-checks <digest>` must be supplied together; they are optional for legacy terminal invocation, mandatory for the mod. checksDigest covers normalized ordered gate configuration, timeout and execution-relevant settings, including inference inputs/results. Changing only irrelevant display whitespace need not invalidate a normalized configuration; changed effective commands must.

Engine loads/checks once, validates all preconditions before creating worktree or probing/starting the agent, then passes that validated base explicitly into existing worktree creation. Existing create_worktree captures HEAD itself; add an optional expected base at that boundary so a second HEAD lookup cannot silently choose a new commit. An already-validated commit may be used if HEAD changes after validation; the result must still derive from exactly the approved commit. Freeze the reviewed checks in the in-memory run plan, as current engine already freezes configuration before agent edits. If source/inference inputs change before validation, refuse PRECONDITION_CHANGED with fresh review data and no launched process.

Use existing 30-minute agent bound and 8 MiB captured-output bound unless the current source changes; present actual values in review if made configurable. The 10-second metadata-read deadline does not apply to agent execution. Permissions inherited from environment remain visible: the mod never sets or silently exploits KIT_FULL_AUTO; refuse plugin launch with unsafe effective bypass settings rather than pretending to preserve controls. Nested Claude, agent auth and process ownership remain runtime gates even after these flags compile.

Files: cli.rs RunArgs; main.rs cmd_run; engine/runner.rs config preparation and RunOptions; engine/worktree.rs create_worktree; engine/infer.rs existing inference; tests/run_endings.rs and run_diff_scope.rs; a focused run-review fixture test if useful. Avoid editing frozen kit-core Receipt/RunSpec solely for review metadata; use internal runner structs.

Checks: empty task; task beginning `--` passed via explicit --task; malicious shell text remains data; changed HEAD between preview/start; changed kit.toml and inferred package script; missing commit; two rapid launches; review writes no worktree/receipt; validated expected base is the actual base.txt; check commands run only after explicit launch.

## 5. Events and cancel — runtime-gated V2 capability

Command: `kit ... run ... --events=jsonl` with reviewed-launch flags. It conflicts with --json; normal --json remains untouched. Stdout consists solely of newline-delimited event objects, stderr remains human diagnostics. No ANSI/prose on stdout.

```ts
type RunEvent = {
  schemaVersion:1;
  command:"run.events";
  runId:string|null;
  sequence:number; // starts at 0, strictly increases for this invocation
  type:"started"|"state"|"output"|"result";
  data:unknown;
};
```

Exact payloads: started `{repoPath,baseCommit,agent}`; state `{state:"queued"|"running"|"gating"}`; output `{text:string,stream:"combined"}`; result is the unchanged final `Envelope<RunResultData>`. Use combined because existing RunDelta::Output does not preserve trustworthy stdout/stderr provenance. The final envelope, not an earlier optimistic state, determines PASS/FAIL/KILLED. A preflight failure emits a single result event with runId:null and exit 2; a started event emits a full allocated RunId before agent execution. Do not emit separate terminal-state and result events that can disagree.

Reuse RunDelta and the current echo channel; one event writer owns sequence/order. Cap line/chunk sizes at 64 KiB while preserving Unicode boundaries and honor the existing total output bound. Escape newlines/control characters through JSON encoding; UI still sanitizes display. Closing stdout or losing the controller invokes cooperative cancellation rather than silently running unattended. Explain any platform where this cannot be proven and keep launch disabled there.

Preferred cancel path: send one supported cooperative interrupt to the exact plugin-owned Kit child using the proven mods process API. Existing headless first Ctrl-C sets CancelHandle and writes KILLED; second Ctrl-C abandons receipt. Do not send a second interrupt automatically while waiting for finalization.

Conditional fallback only if owned-process signal delivery cannot be proven: new `--control-stdin` flag, allowed only with --events=jsonl. Accept exactly a newline-terminated `{"cancel":true}` control message (maximum 128 bytes). A repeated valid cancel is idempotent; malformed/oversized control or unexpected EOF invokes cancellation and records diagnostic reason. No command execution, generic RPC or cancel-by-PID feature. Child agent prompt remains its own stdin pipe, so it does not consume controller messages. Advertise run.stdin-cancel.v1 only if this option is implemented and tested. If the mods API supports neither signals nor writable stdin with ownership, do not embed launch; give terminal handoff. No daemon is required.

Files: main.rs cmd_run/echo_deltas or adjacent focused event writer; engine/runner.rs execute_headless; engine/cancel.rs reused; kit-agents/src/process.rs existing process-tree cleanup; tests/run_json_stdout.rs/run_endings.rs. No active-state inference from receipt directories; they are published after completion.

Checks: sequence/result determinism; final FAIL data survives exit 1; huge output and embedded newline; cancel during agent/gate; duplicate cancel; malformed/EOF control; pipe closure; no orphan child/grandchild; unrelated session survives; receipt-write error never displayed as saved proof. Separately prove real host pane close, reload, disable and terminal exit. Closing pane preserves an owned run; unload/lost controller requests cancellation where support is established. No cross-session recovery claim or Stop button without recovered authority.

## 6. Land review and expected inputs — required before V2 Create result branch

New preview: `kit land <full-id> --print --json`. New apply: `kit land <full-id> --expect-plan <digest> --json`. Both use branch mode; the mod never passes --apply or --force. --expect-plan conflicts with --print/--apply/--force. Preserve all existing terminal defaults when these flags are absent.

Preview data:

```ts
type LandReview = {
  preview:true;
  reviewVersion:1;
  planDigest:string;
  id:string;
  repoPath:string;
  branch:string;
  baseCommit:string;
  baseSource:"recorded"|"worktree"|"head";
  patchDigest:string;
  receiptDigest:string;
  alreadyLanded:boolean;
  existingCommit:string|null;
  files:string[];
  gatePassed:boolean;
  gateVacuous:boolean;
};
```

Reuse existing proof check, patch selection, base resolution and default branch. Hash receipt bytes, selected patch bytes, base choice, destination repo identity, target branch/ref precondition, and relevant options. Display fallback base warnings rather than claiming base.txt exists. If falling back to HEAD, bind that exact HEAD. Apply revalidates before any branch/commit/worktree mutation and passes the reviewed base explicitly into existing land path; changed inputs refuse PRECONDITION_CHANGED with fresh preview. Existing Kit-Receipt trailer idempotence remains: if another invocation already landed the SAME reviewed receipt/patch on the expected branch, return the normal alreadyLanded result; never overwrite a different branch. Git ref creation/update must retain its existing atomic no-overwrite behavior.

The CLI, not cwd, chooses the receipt's repository today. Therefore preview must expose repoPath and UI must compare it to selected repository. Command execution uses full ID; -C alone is not a land scope guard. A renamed/moved source repository needs explicit repair/handoff, not basename guessing.

Files: cli.rs LandArgs; land.rs cmd_land/find_base/check_proof; land/git.rs existing branch creation; tests/land_cli.rs. Tests: pass/fail/vacuous/empty diff; changed receipt/patch/base/branch between review/apply; fallback HEAD moves; same-result idempotence; different branch collision; dirty user's working copy untouched in branch mode; result branch/commit exactly equals returned JSON; no merge/push.

## Minimum runnable checks and release proof

Use actual package names from the implementation checkout. At this baseline:

```sh
cargo test -p kitctl --test kits_cli
cargo test -p kitctl --test marketplace_cli
cargo test -p kitctl --test run_json_stdout
cargo test -p kitctl --test run_endings
cargo test -p kitctl --test run_diff_scope
cargo test -p kitctl --test land_cli
cargo test -p kit-agents kill_terminates_process_tree_no_orphans
```

`tests/receipt_cli.rs` is proposed; run `cargo test -p kitctl --test receipt_cli` only after creating it. Extend the repository's existing Rust tests instead of introducing a new harness. Run affected package tests while implementing, then existing workspace fmt/clippy/test gates once integrated. A mocked agent proves process plumbing; it does not prove nested Claude authorization or real check behavior. A real CLI run does not prove mods lifecycle and cleanup.

V2 release gates: passive doctor + scoped receipts + parser contract fixtures are mandatory for read-only companion; reviewed add/remove mandatory for management; reviewed run + structured events + supported owned cancellation + real nested-agent proof mandatory per launchable adapter; reviewed land mandatory for Create result branch. Unsupported adapters retain actionable terminal commands and precise reason. Record exact Claude version, Kit version/commit, operating system, capability list and test evidence in release documentation. Do not release unsupported buttons with optimistic Ready labels.
