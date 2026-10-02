# Native Claude Code checkpoint

The current request is a native Claude Code Mod with four categories: Frontend, Backend, Security and Product. It is independent of Kit's existing headless `claude -p` execution adapter and ratatui Control Room. The historical 1.0 plans do not define this new surface.

## Implemented

- `kit claude check` checks only the installed version against 2.1.287. It does not check authentication.
- `kit claude install` copies the embedded native payload into a fresh directory, preserving existing destinations. `--print` writes nothing.
- `kit claude launch` opens the native interactive CLI with only the explicit plugin path; `--print` previews the argument vector. A pipe or JSON-mode launch fails before opening Claude.
- `crates/kit-cli/claude-plugin` is the single package source, included in the Rust crate and shared by `scripts/sync-claude-plugin.mjs`.
- Four explicitly declared, flat agent files preload sixteen skills, four per approved role. Category hierarchy lives in `hooks/catalog.ts` rather than depending on recursive agent discovery.
- `/kit` lists the catalog, prepares visible invocation drafts, shows session-local native jobs, and prepares review/handoff drafts tied to immutable result IDs. It never submits a prompt or spawns an agent. Missing status remains unknown or Not attached. Completion requires review.

Fifteen selected upstream skills and one Kit-authored Animate UI integration guide are now bundled with complete supporting resources, source pins and licenses. No external scanners, service integrations or Animate UI components are installed. There are no approval-intercept hooks, MCP declarations, hidden prompt additions or authentication changes. The three writing roles request native worktree isolation. The approved swarm verified all four native roles on alpha.1; the three writers used native worktrees. The subsequent alpha.3 exact-result loop is documented below; visible composer/UI acceptance remains separate.

## Current integration — 2026-10-02

Current release track is a local **2.0.0-alpha.4** revision in preparation for owner review, then public beta and stable only after their native acceptance gates. See [release stages](NATIVE-RELEASES.md) and [private alpha guide](NATIVE-ALPHA.md). The native plugin version is independent of the unchanged Kit CLI version. The immutable alpha.1 archive remains retained and audited. Alpha.2 incorporates reviewed UI repairs and repeatable packaging; its artifact checks are recorded separately below. The current implementation also retains verified native start identity so a completed foreground result survives pruning. Native owner/visible-draft acceptance remains pending and nothing is publicly released.

The isolated checkout is `kit-native-claude`, branch `codex/kit-native-prep-20261002`, based on PR #37 at `479d53f0dcdc2cf50945868dfd42fb0553296f33`. The old dirty checkout is preserved. The user approved yesterday's Frontend/Backend/Security/Product roles. See [the plugin README](../../crates/kit-cli/claude-plugin/README.md) for the four skills per role and [the environment setup](NATIVE-ENVIRONMENT.md) for repeatable commands.

Node packaging checks verify sixteen unique preloads, role permissions, every resource's SHA-256 and exact source-to-package copies. Claude 2.1.287 actually loads four declared agents and sixteen skills through the local `/kit catalog` command in disposable configuration: zero model turns, zero API milliseconds and zero cost. This is loader evidence, not live specialist execution. Strict validation and installed-version TypeScript checks are separate from the hooks runner. The swarm repaired native fixture startup ordering and passed 38 fixtures, then independently reproduced an engine-under-height composition restriction. On the continuation checkpoint, the naturally enabled native runner passed all 39 fixtures, including the repaired engine composition. Native painting remains outstanding. Four actual native tasks completed on alpha.1, three in writer worktrees, with a separate read-only Security review of the exact Backend artifact. The shared local baseline check passes; the latest full workspace run retains one pre-existing NO_COLOR TUI failure after 411 passes. Real Rust installation of 229 files and strict validation passed. The configured cloud environment verified the remote PR37 baseline separately; see the environment document for scope and results.

Kit code/guidance is MIT; Trail of Bits knowledge is CC-BY-SA-4.0. The plugin license expression and provenance distinguish the mixed bundle. The historical April 28 Trail of Bits pin preserves compatible source-inspection workflows; it is deliberately not described as the newest revision.

## Lifecycle repairs — 2026-10-02

Preparation now appends only the selected agent mention or fixed-result request, never replays/replaces user text, and refuses an observed existing agent mention. Later typing survives. Captured task/answer mentions use JSON unicode escapes while decoding unchanged. There is no native atomic draft/session guard: an already-submitted fill may append after a reset, so Kit warns and never attempts a replacement rollback. Review the visible prompt before sending.

Command/UI hooks yield until registration succeeds; refusal preserves downstream startup and starts no clock. Successful registration survives clear/resume. Denied motion settings default off and continue native startup. Latest-request/session ownership prevents stale status writes, while completion attribution uses its own same-session metadata so an overlapping status read cannot lose the immutable result. Both pane-opening paths cancel if a reset occurs while awaiting refresh; a submitted host UI call is not cancellable by Kit.

`node --test scripts/native-lifecycle.test.mjs` runs production-hook regressions with controlled host doubles; the pre-UI checkpoint had twenty, the initial UI revision twenty-seven. Independent follow-up review found no remaining defects in the three extra race fixes. Official native fixtures now register all host stubs before dispatching startup and model append behavior. Thirty-eight passed at the swarm checkpoint; the engine ref0 review cases exposed the composition defect. Installed-version TypeScript passed. An enabled-host continuation rerun now passes 39 fixtures, including the repaired engine path. Direct close-event native fixtures hit a host call-list restriction and were removed; close/reset and structured denial use source-level regression evidence. Real loader tests, strict validation and shared baseline checks pass. Fresh Rust payload `.kit/claude-plugin-20261002-ready` has 229 exact source-matching files and 214 verified resource hashes. These prove source/packaging behavior, not native painting or mention parsing on submission. The separate alpha.1 native role execution evidence does not prove the visible Mod task/review loop. All changes remain local; cloud evidence is limited to the remote PR37 baseline.

## Retained alpha.2 artifact

The repeatable packager now produces `.kit/releases/kit-claude-2.0.0-alpha.2.tar.gz` and the extracted preview `.kit/releases/2.0.0-alpha.2/kit`. Exact checksum, clean extracted-loader proof and 229-file Rust/Node/source parity are recorded in [the release evidence](NATIVE-RELEASES.md#current-alpha2-evidence--2026-10-02). The structured close-denial correction keeps the fox clock running when closing is refused; its source regression uses the real host response envelope. All 39 native fixtures and the shared host check pass on current source. Owner native appearance/draft/result-review acceptance and candidate CI remain pending.

## Current actual-use evidence

Alpha.4 is the current owner launch target: `.kit/releases/2.0.0-alpha.4/kit`. Both alpha.3 and alpha.4 exact extracts completed the native Backend → immutable capture → generated exact-result request → read-only Security → discoverable result loop on Claude 2.1.288. The alpha.4 artifact passed 22 independent assertions, and both completed answers remain Needs review. See [current release evidence](NATIVE-RELEASES.md#current-alpha4-evidence--2026-10-02).

Headless `prompt.fill` correctly refused with `no_composer`; a private pass-through observer captured the actual draft, and the owner explicitly submitted its unchanged text. This demonstrates real mention routing, isolated artifact work and result/review binding, while leaving visible composer/focus/paint unverified. Security's initial alpha.4 report omitted explicit TypeError discussion. A supplemental actual native Security review read the same unchanged artifact, assessed all three throwing paths, and Kit captured its result. Independent assertions cover the error behavior; no automatic security approval is claimed. Alpha.4 changes only version/provenance/README inside the payload; hooks/agents/skills/theme match alpha.3 exactly. The preceding failed source run is retained and never counted as acceptance.

## Mason foundation evidence (8244c43)

Historical eight-skill foundation checkpoint follows; the current alpha has sixteen skills.

Claude Code **2.1.287** was already installed; Node **24.18.0** and Rust/Cargo **1.97.1** were available. Setup did not install or update them. The locked Cargo build dependencies were downloaded for this checkout.

`plugin details` reports `Agents (0)` even with a minimal flat agent fixture. The real runtime loads all four explicitly declared Kit files and reports `Total plugin agents loaded: 4`, plus eight skills, during the local no-model `/kit catalog` command. `scripts/claude-plugin.test.mjs` packages a fresh bundle and checks these exact loader entries under a disposable Claude configuration. It also checks native IDs, manifest paths, skill preloads and preservation of an existing output directory. This is loading evidence, not successful model execution.

Installed-version declarations are generated locally under `.claude-plugin/types` and excluded from Git and the shipped payload. The public declaration file was older than the installed runtime. Offline hook tests use the generated 2.1.287 definitions, including complete `command.run` and `turn.complete` fixtures.

Passing at the checkpoint: TypeScript no-emit check, 9 offline Mods tests, 2 packaging/runtime-discovery tests, strict plugin validation, 4 native Rust unit tests, 2 real-binary Rust integration tests and workspace formatting. Cargo's package file list includes the payload and excludes generated declarations. Mason subsequently reported formatting and full-workspace Clippy passing at this frozen SHA. Workspace tests: 383 pass and one baseline failure, `ui::tests::past_fail_row_has_no_wash` at `crates/kit-tui/src/ui/mod.rs:645`, reproduced on c35f608 with `NO_COLOR=1`; unsetting only that test process's variable makes it pass. This is not a fully green workspace test result.

The completeness inventory found 13 public functions and zero stubs. It heuristically flagged four helpers as untested: `catalogText`, `findAgent` and `resultDraft` are exercised through the native command tests; `command_name` has a direct Rust unit test. The inventory is a prompt for review, not execution coverage evidence.

## Cloud UI checkpoint (2026-10-02)

Cloud fetched and verified exact foundation `8244c43ce1125e415fcc56c64322d3dbb11f0b04` in an independent Kit clone. The unrelated mounted VR repositories were not modified or run. Node 24.19.0 and the vendor's checksum-verified Claude Code 2.1.287 were used. Installed-version declarations were generated by a local `/kit catalog` command in disposable configuration; no credentials, agreements or model calls were needed.

Implemented:

- A native AbovePrompt strip for Frontend, Backend, Security and Product, with a crimson active segment, two-row narrow layout and no prompt hotkeys. It yields to native surveys.
- A user-opened native Pane (`/kit open` or bucket button), showing the selected real specialist and skills, explicit Prepare draft, all observed session attempts, Refresh status and fixed-result Prepare review. Catalog selection neither creates tasks nor hides other categories' observed work.
- Independent local task IDs, specialist IDs, native attempt IDs and parent-owner IDs. Each newly observed attempt receives its own local task; repeated text does not establish a retry relationship. Completion remains Needs review; unknown or unavailable state is explicit. Clear/resume/branch reset the old session's records.
- The original approved fox grid, rendered as one anchored 22×8 native terminal Raster. It appears only in an expanded empty pane with room, stays out of the compact strip, and supports idle/ears/blink without translation. A native 100 ms timer quantizes suggested gesture durations; only frame changes request redraws. `NO_COLOR`, `KIT_MOTION=off`, `REDUCE_MOTION=1` and Pause motion hold idle.
- A Kit Red native theme, included by both the Node exporter and Rust payload. Users choose it through `/theme`; no global settings are rewritten. Native layout governs docking and prompt width.

[Rendered fox cells](../assets/kit-native-fox-proof.png) are an offline visual proof from implementation data, **not** a live terminal screenshot. The interactive cloud probe stopped at first-run onboarding; no onboarding choice, agreement or authentication was completed.

Cloud verification: **37 offline Mods tests pass**, **2 packaging/runtime-discovery tests pass**, TypeScript no-emit against generated 2.1.287 types passes, strict plugin validation reports no warnings/errors, and completeness inventory reports 2 public symbols in the current follow-up diff, zero stubs and zero heuristically untested symbols. Validation lists only agent-list, clock, command registration, environment reads, prompt read/fill and native UI calls. No model/spawn/submit/permission/auth/network calls are added.

Rust/Cargo is unavailable in cloud and was not installed. The small new Rust theme inclusion and assertion therefore need Mason's formatting/build/unit/integration rerun; foundation Rust results do not verify this new diff. Keep the existing `NO_COLOR` baseline failure separate. Parent also reported foundation CI run [36970881268](https://github.com/Zwin-ux/kit/actions/runs/36970881268) failing Clippy on all three operating systems under Rust 1.99 (`double_must_use` from the `async_trait` Gate at `kit-core/src/gate.rs:105`), despite Mason's Rust 1.97.1 local pass. Mason owns that isolated diagnosis; this UI change does not touch Gate or claim CI green. Real terminal paint, keyboard/focus, narrow/short-window scrolling, native theme selection and live worktree execution remain unverified.

## Independent-review fixes after 1046498

The AbovePrompt handler now awaits `next(e)` with the original read-only props and retains the returned tree. It reserves a conservative row bound for plain text/vertical boxes, then fits four Kit buttons into the remaining budget. Narrow 3–4-row bands drop optional rules/padding; current narrow one-row budgets use a single Kit opener. A full or opaque downstream drawing is returned unchanged rather than clipped or suppressed; `/kit open` remains the fallback. Regression tests preserve a downstream sentinel alongside all four buttons and cover a full downstream band.

All UI and command draft paths now share a read→fill transaction whose lock is acquired **before** `prompt.read`. A session-generation check stops a delayed old read before `prompt.fill`; a unique transaction token prevents an old `finally` block from unlocking a newer action. Tests reproduce the former failures for clear/resume/branch, use/review/handoff, double-click and overlapping sessions. These are offline control-flow proofs, not claims about live host cancellation of a fill already submitted to the host.

[Installed-type excerpts and reviewer notes](CLAUDE-MODS-REVIEW-NOTES.md) record the exact 2.1.287 contracts used. No Rust or Gate files are changed in this follow-up.

## Remaining terminal QA

1. Load the branch in an already authorized Claude 2.1.287 terminal. Choose Kit Red through `/theme`, open `/kit open`, and inspect docked/inline layouts at 80/120/160 columns and short heights.
2. Type ordinary letters/digits in Claude's prompt, select each bucket, and confirm nothing starts. Prepare a draft and inspect exact preservation of its task text. Do not send a model request merely for UI QA.
3. Check fox idle/ear/blink anchoring, Hide fox, Pause motion and a process launched with `NO_COLOR=1`. Inspect native focus cycling, Esc, surveys, and existing prompt preservation during review.
4. Only on an explicitly requested live task, check native agent execution/worktree isolation and exact completion/result evidence. Refresh the session list and confirm task/attempt/owner identities, Needs review and missing/unknown status handling.
5. Rerun Rust formatting, native tests/integration, workspace Clippy and workspace tests on the final UI commit. No merge/release is authorized.

## Historical source queue (not installed at the foundation checkpoint)

The following original research candidates are retained as history. The current sixteen-skill selection supersedes this queue; its actual installed pins and resources are recorded in the plugin provenance.json.

| Category | Source and full revision | Candidate / constraints |
| --- | --- | --- |
| Frontend / Backend | `addyosmani/agent-skills` at `2686b620fc1fed2e8f60c704839c766b8594c6b6` | `frontend-ui-engineering`, `api-and-interface-design`; MIT |
| Frontend | `addyosmani/web-quality-skills` at `afa8da942115f2961fdbfa80807ea0b232ff6c00` | accessibility / core-web-vitals; MIT; preserve `../performance/references/MEASUREMENT.md` dependency |
| Backend | `supabase/agent-skills` at `551274ed2fe97c8fea1325f7ceb05803a542f8df` | Postgres v1.1.1; MIT |
| Backend | `aws/agent-toolkit-for-aws` at `acc890da1028c9c9e7c4d496238789d4d77f201d` | `skills/core-skills/aws-well-architected-review`; Apache-2.0; not deprecated samples |
| Backend | `microsoft/GitHub-Copilot-for-Azure` at `fda21b063fb8f2f93f423c8e7c61e75f2e58b7ad` | `plugins/azure-skills/skills/azure-enterprise-infra-planner`; MIT; do not inherit provisioning or `npx @latest` MCP startup |
| Security | `trailofbits/skills` at `82fe8226252622fa807643bdca1710901198553a` | differential-review / sharp-edges / variant-analysis; CC-BY-SA-4.0; retain attribution and license |
| Security / Product | gstack at `7fca42ad8b6c707b8a38f579f72bf3c4f7de6d85` | cso / office-hours / plan-ceo-review / spec; MIT; full runtime dependencies, optional providers/scanners/local state; never imply ready from prose alone |

Animate UI has no `SKILL.md` and its license includes Commons Clause. Use original Kit guidance and links, not a vendored skill. Readiness should distinguish **catalogued**, **files installed**, **runtime ready**, and **execution verified**. Do not preload entire large upstream suites.

Kit's existing `KIT.toml` schema remains unchanged. Its skill provenance uses `source`, `path`, full `rev`, and `licence`; native first-party metadata is separately recorded in `provenance.json`.

## Primary references

- [Mods overview](https://code.claude.com/docs/en/plugins/mods/overview), [create](https://code.claude.com/docs/en/plugins/mods/create), [test](https://code.claude.com/docs/en/plugins/mods/test), [interface](https://code.claude.com/docs/en/plugins/mods/interface), [reference](https://code.claude.com/docs/en/plugins/mods/reference)
- [Plugin components](https://code.claude.com/docs/en/plugins/components), [manifest reference](https://code.claude.com/docs/en/plugins/manifest-reference), [skills](https://code.claude.com/docs/en/skills), [subagents](https://code.claude.com/docs/en/sub-agents)

No relevant canonical Kit Notion/Brain page was found in the initial context search. Current explicit product instructions and installed-runtime evidence take precedence over the historical local plans.
