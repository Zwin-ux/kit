## Current private alpha — 2026-10-02

Native Mod 2.0.0-alpha.1 is prepared locally as a folder and checksum-verified archive. Read [release stages](../../dev/NATIVE-RELEASES.md) and [owner review guide](../../dev/NATIVE-ALPHA.md). Public beta/stable are promotion gates, not current releases. Version-only readiness is corrected and has a passing real-CLI regression; native interactive acceptance remains pending.

## Completed extension — 2026-10-02

The approved Frontend/Backend/Security/Product lineup now preloads four substantive skills each: fifteen upstream skills plus Kit-authored Animate UI integration guidance. Full resources, exact source pins, licensing and adaptations are in the native plugin provenance.json. The original eight-skill findings below are historical; current native discovery confirms sixteen. See [current plugin contents](../../../crates/kit-cli/claude-plugin/README.md) and [local/cloud environment evidence](../../dev/NATIVE-ENVIRONMENT.md).

Local setup, package integrity/runtime loading, strict plugin validation, installed-version TypeScript, native Rust unit/integration, formatting and Clippy pass. Real CLI prepares 229 files; all 214 skill-file hashes match. Cloud environment Kit native Claude is saved and its remote PR37 baseline verification passed. Changes are not pushed. The lifecycle repairs below now have twenty passing source regressions. Full workspace retains the known NO_COLOR TUI failure; hooks test runner remains rollout-off; interactive native painting and worktree agent execution remain outstanding.

# Kit native Claude continuation plan

> **For agentic workers:** Use `subagent-driven-development` or `executing-plans` task by task. Source lifecycle repairs and independent follow-up review are complete; native acceptance remains outstanding. Follow the existing repository ownership rules. Read this file before historical Kit Red plans.

**Goal:** Make PR #37 a reliable, restrained Kit addition to native Claude, with safe drafts, truthful status and verified terminal behavior.

**Architecture:** Continue the shared native plugin at `crates/kit-cli/claude-plugin`, packaged by both the Rust CLI and Node exporter. Claude owns the conversation, prompt, subagents, permissions and theme selection. Repair the existing seams before adding anything.

**Tech stack:** TypeScript Mods hooks, installed Claude Code 2.1.287 declarations, Node 24, Rust Kit CLI. No new runtime dependencies.

## Workspace and authority

- Checkout: `/Users/entreprenurecosystem/kit-native-claude`.
- Branch: `codex/kit-native-prep-20261002`.
- Starting commit: `479d53f0dcdc2cf50945868dfd42fb0553296f33`.
- PR: [#37](https://github.com/Zwin-ux/kit/pull/37), still draft. This preparation is local; nothing has been pushed, merged or installed globally.
- `/Users/entreprenurecosystem/kit` contains separate, uncommitted `mods/kit-red` work. Its older HEAD is `fb0ddf1`. Do not reset it, copy its whole plugin into this branch, or treat its handoff as the native PR's implementation map.
- Existing native handoffs: `docs/dev/CLAUDE-NATIVE.md`, `docs/dev/CLAUDE-MODS-REVIEW-NOTES.md`, plugin `README.md`. They describe historical checkpoints; the table below records today's independent checks.

## Global constraints

- Preserve native conversation, ordinary typing, downstream UI and surveys.
- Prepare visible drafts only. Selecting a role never sends a prompt or starts work.
- Never trade user text for a cleaner invocation. A second read is not an atomic write guard.
- Keep the native fox candidate; fix measured geometry before commissioning more concept art.
- Role labels remain meaningful without emoji, color or artwork.
- No dashboard, persistent fleet monitor, onboarding checklist, new persistence service or automatic task dispatch.
- No authentication/settings changes, rollout bypass, MCP installation, global installation, merge or release in preparation work.
- Frozen Rust contracts and `kit-tui` are outside this change. The existing TUI test failure is recorded rather than silently patched.

## Approved role lineup — 2026-10-02

The user chose yesterday's cloud version: **Frontend / Backend / Security / Product**. Keep PR #37's existing native agents and skill mapping. This supersedes the earlier Frontend Design / Full-stack Design / Backend Engineer / LLM Engineer proposal. Role approval does not establish native visual or execution acceptance.

The user repeatedly requested Claude's native look with a small Kit addition. The PR handoff claims its four-bucket layout was reviewed in another session, while this thread rejects persistent four-card workspaces. These are different surfaces, but approval of the exact persistent strip is not established here. Preserve it as the baseline, then assess native screenshots before deciding whether it earns its space.

## Independent baseline, 2026-10-02

Machine: Node `v24.14.1`, Claude `2.1.287`, Rust/Cargo `1.96.0`. GitHub checks at the starting SHA: all 15 pass; that is separate from these local results.

| Check | Observed result | What it establishes |
| --- | --- | --- |
| `claude plugin validate ./crates/kit-cli/claude-plugin --strict --json` | Exit 0; no errors/warnings | Manifest and statically analyzed hooks/API calls |
| `cargo fmt --all --check` | Exit 0 | Rust formatting |
| `cargo test -p kitctl native::tests --locked` | Four tests pass | Version parsing, argument shape, embedded payload, preservation of existing install |
| `cargo test -p kitctl --test native_cli --locked` | Two tests pass | Real Rust binary install/preview and pipe-launch refusal in fixtures |
| `node --test scripts/claude-plugin.test.mjs` | Two tests fail: `Refusing linked output path: /var` | macOS fixture defect; fails before discovery |
| `TMPDIR=/private/tmp node --test scripts/claude-plugin.test.mjs` | Two tests pass | Bundle contents and real four-agent/eight-skill loader discovery under canonical temporary parent |
| Source `/kit catalog` in disposable config, no tools/MCP | Exit 0; zero model turns, API milliseconds and cost | This exact source loads and handles a local command |
| Existing sibling compiler: `.../kit/node_modules/.bin/tsc --noEmit -p crates/kit-cli/claude-plugin/tsconfig.json` | Exit 0 | Types checked against locally generated exact 2.1.287 declarations |
| `claude plugin test ./crates/kit-cli/claude-plugin` | Exit 1: rollout switch served off | Today's native runner is unavailable; the earlier 37 passing cases were not reproduced today |
| `cargo package -p kitctl --list --locked` | Exit 0; plugin present, generated types absent | Package file inventory, not registry publication |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Exit 0 | Local Rust 1.96 lint result; GitHub uses its own toolchain |
| `cargo test --workspace --locked` | Exit 101; `ui::tests::past_fail_row_has_no_wash` fails with `NO_COLOR` present | Workspace is not fully green in the current environment |
| `env -u NO_COLOR cargo test -p kitctl-tui ui::tests::past_fail_row_has_no_wash --locked` | One test passes | Confirms the isolated failure depends on NO_COLOR; does not establish a second full-workspace pass |

Generated declarations at `crates/kit-cli/claude-plugin/.claude-plugin/types/claude-code/index.d.ts` begin `// Written by Claude Code 2.1.287.` They stay ignored and must not ship. `PromptFillArgs` at line 8049 has text/mode/decorations only: no expected text, revision or compare-and-swap. Modes are replace/append/insert. **Append preserves the current draft; it does not prove correct native agent selection when several mentions exist.**

The native-runner rollout failure does not erase loader success. Neither result proves interactive painting or actual specialist execution. Computer control previously refused both Terminal and the Codex desktop app for safety reasons; do not route around those restrictions. Native QA requires an allowed, already authorized user terminal.

## Historical source map and review findings

| Area | Files | Finding / smallest next change |
| --- | --- | --- |
| User draft | `hooks/register.ts:26–44`, `tests/draft-races.test.ts` | Read then replace can discard newer typing. Existing lock protects Kit actions, not the user. Resolve without inventing a host API. |
| Status | `hooks/register.ts:12–20`, `hooks/state.ts` | Session generation does not stop older same-session list responses replacing newer completed status. Add latest-request ownership. |
| Command ownership | `hooks/register.ts:284–310`, `tests/native.test.ts` | Rejected registration still leaves `/kit` interception active. Register successfully before enabling Kit/timer; failure must call downstream. |
| macOS fixture | `scripts/claude-plugin.test.mjs:13–14` | Canonicalize the freshly created temp directory; retain exporter symlink rejection. |
| Readiness | `crates/kit-cli/src/native.rs:157–167` | Version-only check says Mods supported / `mods:true`; report version compatibility and execution/login unverified. |
| CI | `.github/workflows/ci.yml` | Existing native packaging test is not invoked by normal Node jobs; plugin changes alone do not match that workflow's paths. |
| Pane priority | `hooks/register.ts:143–196` | Eight-row fox precedes the useful action in a requested 30×22 pane. Actual body allocation/scrolling must be measured. |
| Theme / color | `hooks/register.ts:58,145,286`, `hooks/fox.ts:58` | Hardcoded dark text palette; NO_COLOR stops motion but source still emits colored Text/Raster. Host suppression is unverified. |
| Narrow controls | `hooks/band.ts:39–44`, `hooks/register.ts:94–112` | Strip can occupy 3 wide / 5 narrow rows; truncated one-row labels become ambiguous. |
| Desktop / animation | `hooks/register.ts:278–298` | Fox controls remain without Raster; timer wakes even when no fox is visible. Make controls and timer conditional on applicable art. |
| Human copy | `hooks/register.ts:154–272` | Role/task/status/next action should precede diagnostic IDs and implementation vocabulary. IDs remain available through existing commands. |

All hook paths in this table are relative to `crates/kit-cli/claude-plugin` and reflect the starting SHA. Draft, status, registration, macOS fixture, readiness and CI repairs are implemented locally. Geometry and host suppression remain native unknowns.

### Current runnable regression

```sh
cd /Users/entreprenurecosystem/kit-native-claude
node --test scripts/native-lifecycle.test.mjs
```

Twenty source-level cases now pass, with no skips. The original defect-presence diagnostic was retired after converting it to fixed-behavior checks. The suite covers append-only preservation, observed mention refusal, registration refusal/pending startup, session invalidation, out-of-order status, fixed completion capture, stale pane opens and denied motion settings. It joins the host-free CI lane. Official native fixtures also use successful startup and append semantics, but their runner is rollout-disabled; TypeScript checking is not execution proof.

## Ordered execution

### 1. Restore ordinary macOS packaging verification

Files: `scripts/claude-plugin.test.mjs`. Independent of role choice.

- [x] Keep the original failing invocation as the regression trigger.
- [x] Add `realpath` to the existing fs/promises import and change the fixture to:

```js
const temp = await realpath(await mkdtemp(path.join(os.tmpdir(), 'kit-plugin-test-')));
```

- [x] Run `node --test scripts/claude-plugin.test.mjs` with the ordinary environment. Both tests reach their intended assertions.
- [x] Verify existing destinations still retain `user-note.txt`; preserve `packagePlugin`'s linked-path refusal.

### 2. Keep drafts safe during real typing

Files: `hooks/register.ts`, `hooks/catalog.ts`, `tests/draft-races.test.ts`. Depends on an honest invocation contract, not on artwork.

- [x] Reproduce typing between read and pending fill for both role selection and result preparation.
- [x] Assert the fixed behavior retains the newer text. Keep double-click, clear/resume/branch and old-token fixtures; twenty source cases execute independently of the native runner.
- [x] Use only actual host methods. Do not add a second read and call replacement safe: it still races.
- [ ] Determine whether an appended scoped agent mention invokes the intended specialist in a controlled native fixture. Check zero, one existing Kit mention, and an unrelated mention. Record exact result; do not infer from string shape.
- [x] Use documented manual `@agent-<scoped-name>` mentions with non-destructive append and visible review copy. Refuse observed existing selectors. Actual submission parsing remains the native check above, including text/mentions added after the read.
- [x] Require an observed empty draft for fixed-result requests, then append. Preserve later typing and document that the host has no atomic empty/session guard; never promise empty-only replacement. Captured task/answer mentions are JSON-escaped without changing decoded content.
- [x] Keep Claude responsible for submission. Strict validation shows no prompt.submit/model/spawn calls.

### 3. Preserve command and status ownership

Files: `hooks/register.ts`, `tests/native.test.ts`, `tests/draft-races.test.ts`.

- [x] Convert the rejected registration case: startup continues, downstream `/kit` responds, and no Kit timer starts.
- [x] Enable `commandReady` only after successful registration. Yield command/drawings while unavailable and cancel the previous clock before registration.
- [x] Preserve successful registration through clear/resume; reset task/results/draft state. Clear persistence and resume preparation cancellation execute in the source suite; existing branch fixtures remain for the native runner.
- [x] Dispatch successful `session.start` in the official command/UI test setup before asserting Kit availability.
- [x] Later Completed remains Needs review after earlier Running/error reads resolve.
- [x] Gate shared status writes on monotonically increasing request ownership and session generation.
- [x] Return each successful refresh's own same-session snapshot for completion attribution. Overlapping `/kit jobs` cannot lose an immutable result or restore stale live status.
- [x] Catch denied motion reads, default motion off, and preserve downstream startup. Both pane-opening paths share a generation guard after refresh. Already-submitted host opens remain outside Kit's cancellation control.
- [x] Use documented `{ deny: 'unavailable' }` in the official status-error fixture; its source typechecks, but native execution awaits the runner.

### Checkpoint after reliability repairs

- [ ] Source typecheck, strict validation and existing native tests pass when the runner is enabled.
- [x] All three defect cases have passing fixed-behavior source regressions; retire the diagnostic.
- [x] Preserve user text, downstream ownership and newest observed status. Native semantics/painting remain explicit acceptance requirements.

### 4. Fit Kit into the native interaction

Files: `hooks/register.ts`, `hooks/band.ts`, `tests/ui.test.ts`; fox source only if native geometry demands it. Use the approved Frontend / Backend / Security / Product lineup for label/art changes.

- [ ] Place the draft action and its consequence within the first short-pane viewport. Suppress decoration before useful controls. Measure actual body dimensions rather than requested rows.
- [ ] Prefer inherited native text colors. Keep explicit red only where it communicates Kit identity or a real selection. Supply text fallback for NO_COLOR if the host retains colored Raster.
- [ ] At widths that cannot show four complete role labels, use one legible Kit opener or yield. Preserve native surveys and opaque/full downstream drawings.
- [ ] Remove ineffective Desktop/hidden-art controls. Stop animation work while art is not visible; use the existing native clock, not another timer/service.
- [ ] Lead with specialist, task, observed status and next action; leave exact task/attempt/owner/result keys in existing command/detail output.
- [ ] Name the review destination explicitly if Security remains the reviewer. Do not add new workflow terminology.
- [ ] Compare unmodified Claude and Kit at the same terminal size before deciding the persistent strip is acceptable. This is product acceptance, not a new dashboard design task.

### 5. Make readiness and packaging evidence repeatable

Files: `crates/kit-cli/src/native.rs`, native Rust tests, `.github/workflows/ci.yml`, `scripts/claude-plugin.test.mjs`.

- [x] Correct version-only readiness text/JSON; a real CLI Unix regression verifies no Mods activation/login/execution claim. Minimum version compatibility is separate from tested support.
- [x] Add plugin paths and host-free package/lifecycle checks to existing Node CI. Actual installed-host loader tests pass locally; remote CI on unpushed changes remains unverified. No credentials provisioned.
- [x] Compare fresh Node and real Rust payloads: 229 identical files. The extracted private-alpha archive validates and loads four agents/sixteen skills without a model call.
- [x] Keep generated declarations/development tests out of installed payloads and reuse existing packaging paths.

### 6. Native acceptance and handoff

Files: existing checkpoint and review notes, with screenshots/evidence only after an allowed native run.

| Scenario | Required visible result |
| --- | --- |
| 80/120/160 columns; then 40/32 and short heights | Complete actionable labels, useful action visible, preserved conversation space |
| Native dark/light, Kit Red, NO_COLOR | Readable text/focus; documented actual Raster behavior |
| Ordinary task typing, role choice, draft preparation, Escape | Text retained, no automatic submission, focus returns appropriately |
| Native survey and another mod's AbovePrompt content | Downstream controls remain usable; Kit yields when necessary |
| Desktop | Usable text controls; no unavailable fox controls |
| Failed / out-of-order refresh, missing agent, long descriptions | Honest observed status; no accidental retry or acceptance |
| Fixed-result review while prompt contains another task | Existing task retained; result ID binds the intended captured answer |
| Authorized live specialist task in disposable repo | Intended agent actually runs; writer isolation, exact changed artifact and checks recorded |

Use an already authorized terminal:

```sh
claude --plugin-dir /Users/entreprenurecosystem/kit-native-claude/crates/kit-cli/claude-plugin
```

UI QA should not send a model task. Live execution needs its own concrete scoped task and evidence; this preparation does not establish it. Keep PR #37 draft until reliability repairs and native acceptance are complete. No merge/release is part of the plan.

## Continuation prompt

> Work in `/Users/entreprenurecosystem/kit-native-claude` on `codex/kit-native-prep-20261002`, based on PR #37 commit `479d53f0dcdc2cf50945868dfd42fb0553296f33`. Read `AGENTS.md`, this plan, `docs/dev/CLAUDE-NATIVE.md`, and the shared native plugin. Preserve the dirty sibling `/Users/entreprenurecosystem/kit`. Frontend / Backend / Security / Product each preload four bundled skills. macOS packaging, readiness/CI and source lifecycle repairs are complete locally; twenty host-double regressions pass. Continue with allowed short-pane/color/Desktop/native QA and actual mention/worktree execution evidence. Preserve Claude's prompt, native UI, permissions and submission. Do not invent prompt-fill CAS, cancel an already-submitted host call or infer live execution from loader tests. Native runner remains rollout-gated; full workspace retains the NO_COLOR TUI failure. Cloud verified only the remote PR37 baseline, not these unpushed changes. No push, merge, release, global installation or control-denial bypass.

## Primary contracts checked

- [Mods API](https://code.claude.com/docs/en/plugins/mods/api)
- [Mods reference](https://code.claude.com/docs/en/plugins/mods/reference): startup does not repeat after clear/resume/branch; branch reports resume at session end.
- [Native interface](https://code.claude.com/docs/en/plugins/mods/interface): preserve downstream drawings, native geometry and reload/lifecycle rules.
- [Native tests](https://code.claude.com/docs/en/plugins/mods/test)

Official `.md` endpoints were retrieved during preparation because the web reader could not access the rendered pages. Installed 2.1.287 declarations govern exact parameter shapes; host behavior still needs runtime evidence.
