# Native Kit development environment

Use the same checked-in commands locally and in a Linux cloud checkout. This setup prepares project dependencies; it does not install global tools, configure Claude authentication/settings, run vendored skill scripts, publish packages or modify the standalone TUI.

## Prerequisites and setup

Provide Node >=20, pnpm **10.12.4** (or existing Corepack), Rust **>=1.90**, Cargo, rustfmt, Clippy and Python **>=3.9** for archive checks. A working native linker/toolchain is also required for Cargo builds. Select runtime versions in the environment image/settings first; missing tools produce an actionable error. Rust's minimum is a compatibility floor, not an exact compiler pin: record the actual version because Clippy diagnostics change between releases.

```sh
bash scripts/setup-native.sh
```

Setup resolves the repository root from its own path. It uses existing pnpm 10.12.4, or `corepack pnpm` with the repository's `packageManager` pin. Corepack may download its pinned package-manager cache, but the script never enables global shims. Project install uses `pnpm install --frozen-lockfile --ignore-scripts`; no dependency lifecycle scripts are needed for this native Mod lane. `cargo fetch --locked` fills Cargo's dependency cache without building crates. Dependency/cache files may be created; lockfiles must remain unchanged. Repeating setup is suitable after a branch's locked dependencies change.

## Checks

After all agent-owned files are integrated:

```sh
bash scripts/check-native.sh
```

This runs Node package assertions excluding Claude discovery, a deterministic archive roundtrip/preservation regression, production-hook lifecycle regressions with controlled host doubles, Rust native unit/integration tests, workspace formatting and Clippy. It stops on any failure. It does not run the full workspace test suite or claim Claude-host readiness. Run `cargo test --locked --workspace` separately at the final integration checkpoint; retain any reproduced baseline failure explicitly rather than hiding it with global environment changes.

Optional installed-host checks:

```sh
bash scripts/check-native.sh --host
```

This additionally requires Claude on PATH, locally generated declarations at `crates/kit-cli/claude-plugin/.claude-plugin/types/tsconfig.json`, and project TypeScript. Generate declarations by loading the source Mod with the installed Claude version in an authorized environment, for example `claude --plugin-dir /absolute/path/to/kit/crates/kit-cli/claude-plugin`. Use a normal standalone terminal; preserve nested-session protections. Claude owns any sign-in or agreements. Do not copy stale declaration files from another version. Declarations stay untracked and outside the shipped payload.

The host lane performs strict validation, actual `claude plugin test`, generated-type checking and disposable-configuration runtime discovery. A hooks-rollout refusal is a failure, never a passing skip. Successful host automated tests still do not establish native keyboard/focus/paint, theme selection, Desktop behavior or live worktree execution. See [the native checkpoint](CLAUDE-NATIVE.md) for manual QA.

## Cloud configuration

Set the environment's setup script to:

```sh
bash scripts/setup-native.sh
```

Use the same command for its optional maintenance script so a resumed cache refreshes dependencies for the selected branch. Ensure the script exists on the branch used to prepare the environment. No credentials or Claude installation are necessary for setup or the baseline lane.

OpenAI documents internet availability during setup, optional maintenance when cached containers resume, and separate agent-phase network settings. Setup runs in a separate shell, so rely on environment settings for persistent runtime/PATH choices. Configure only the registry access required for frozen dependency retrieval. See [the official cloud environment documentation](https://learn.chatgpt.com/docs/environments/cloud-environment), reviewed 2026-10-02 (currently titled “Codex Cloud (Legacy)”).

The environment **Kit native Claude** was created and its saved settings verified on 2026-10-02: [environment 6abfd443bf948191b2d3ec2191584068](https://chatgpt.com/codex/cloud/settings/environment/6abfd443bf948191b2d3ec2191584068), repository Zwin-ux/kit, universal image, Node 22, Rust 1.95.0, caching on, agent internet off, no variables/secrets added. The UI runtime menu supports Node 22 and Rust 1.95.0; local preparation uses Node 24.14.1 and Homebrew Rust 1.96.0. Both clear the repository minimums.

Because these new scripts are not pushed yet, the saved initial setup is self-contained: `set -euo pipefail`, `corepack pnpm@10.12.4 install --frozen-lockfile --ignore-scripts`, `rustup component add rustfmt clippy`, `cargo fetch --locked`, and `cargo build -p kitctl --locked`, each on its own line. Saved maintenance runs `bash scripts/setup-native.sh` when present, otherwise the pinned frozen install and locked fetch. Replace initial setup with the checked-in script only after it is available on the cache branch.

[Read-only verification task](https://chatgpt.com/codex/tasks/task_e_6abfd4b9fa28832eb194e22a9e8eb6be) was submitted against existing remote PR37 branch `feat/native-claude-mods-setup-20261002`. Its results cannot verify the unpushed sixteen-skill changes. Cloud verification has completed; see the exact results below. Local execution does not establish that proof. Record Git SHA and Node/pnpm/Rust/Claude versions with results; do not copy credentials or private logs into evidence.

## Local preparation evidence

On 2026-10-02, `bash -n scripts/setup-native.sh scripts/check-native.sh` and actual `bash scripts/setup-native.sh` completed with exit 0 using Node 24.14.1, pnpm 10.12.4 and Homebrew Rust 1.96.0. pnpm reported the archived Node TUI's missing `packages/tui/dist/bin.js` bin target; setup does not build that separate legacy package. Cargo fetched the locked dependencies. This records setup only: the integrated check results and cloud execution must be recorded separately below.

## Final verification — 2026-10-02

Local integrated source:

- Setup and shell syntax passed; Cargo.lock and pnpm-lock.yaml unchanged.
- Shared baseline check passed after lifecycle repairs: one host-free package integrity test, twenty lifecycle regressions, four native Rust unit tests, two real CLI integration tests, formatting and full-workspace Clippy.
- Installed-version TypeScript passed. Both Node package/runtime tests passed; real Claude loader discovered four agents and sixteen skills, with zero turns/API milliseconds/cost.
- Cargo package inventory contains sixteen SKILL.md files and full supporting resources; generated host declarations are excluded.
- Real Rust CLI rebuilt and installed **229 files** into fresh `.kit/claude-plugin-20261002-ready`; every installed byte matched current source and all **214** skill resource/notice hashes matched. Development tests/generated types are excluded. Strict validation returned no warnings or errors; explicit launch argument preview passed. Older dated payload directories are preserved snapshots, not the current launch target.
- Full workspace execution observed **409 passing tests and one existing TUI failure**, `ui::tests::past_fail_row_has_no_wash`, under inherited NO_COLOR. This is not a fully green workspace result; TUI source was not changed.
- `claude plugin test` still exits 1 because this process's hooks rollout is off. No bypass was attempted. Interactive visual/worktree execution remains unverified.

Lifecycle checks reproduced and repaired typing loss, command-registration takeover and stale status writes. Follow-up review reproduced completion loss during an overlapping refresh, old pane actions crossing reset and environment denial interrupting startup; their regressions now pass. Captured task/answer agent mentions remain escaped JSON data. Official native fixtures were updated and typechecked but could not execute behind the rollout gate. Source doubles do not establish atomic native fill cancellation or actual agent-mention parsing on submission.

Cloud verification completed on exact upstream PR37 SHA `479d53f0dcdc2cf50945868dfd42fb0553296f33` (cloud branch label `work`): Node22.22.2, pnpm10.12.4, Rust/Cargo1.95.0, rustfmt1.9.0, Clippy0.1.95. Formatting and locked offline native Rust unit4/integration2 passed; package declarations passed. Claude was absent and discovery skipped. Tracked/untracked source remained clean. A fresh offline pnpm installation was not tested, so this evidence does not prove every pnpm tarball is cached. New sixteen-skill and lifecycle changes remain local and were not claimed as cloud-tested.

Cloud report: https://chatgpt.com/remote/task_e_6abfd4b9fa28832eb194e22a9e8eb6be

To launch the prepared local payload from an interactive terminal in this checkout:

```sh
./target/debug/kit claude launch --plugin-dir .kit/claude-plugin-20261002-ready
```

The command was previewed, not automatically opened. Claude remains responsible for host prompts, sign-in and execution permissions. Select Kit Red using its native theme picker.

## Private alpha follow-up — 2.0.0-alpha.1

At the retained alpha.1 checkpoint, the preview superseded the dated payload target above: `.kit/claude-plugin-2.0.0-alpha.1`. Its native manifest/provenance use 2.0.0-alpha.1; Cargo stays 2.0.0. The readiness text/JSON now claim only version compatibility; Mods activation, login and execution are unverified. A new real CLI regression passes after reproducing the old overclaim, bringing macOS native CLI integration tests to three (the new fake-host case is Unix-specific). Shared baseline, exact installed types, strict validation and both loader/package tests pass on the alpha source.

Node and Rust exports have 229 identical files; all skill-resource hashes remain verified. Local archive `.kit/releases/kit-claude-2.0.0-alpha.1.tar.gz` includes the preview and owner guide/checksums. Actual clean extraction, all member checksums, strict validation and real extracted-loader discovery pass without a model call. See [release stages and exact evidence](NATIVE-RELEASES.md) and [owner walkthrough](NATIVE-ALPHA.md). The actual native test runner is still rollout-off; native paint/execution acceptance remains pending. No public release occurred. Cloud evidence still covers only remote PR37.

```sh
claude --plugin-dir /Users/entreprenurecosystem/kit-native-claude/.kit/claude-plugin-2.0.0-alpha.1
```

## Retained alpha.2 continuation

Fresh `bash scripts/check-native.sh --host` exits 0 on Claude 2.1.287, Node 24.14.1, Python 3.14.7 and Rust/Cargo 1.96.0: package integrity, one deterministic archive regression, 34 source lifecycle checks, four native Rust unit tests, four native CLI integration tests, formatting, workspace Clippy, strict validation, 39 actual native fixtures, generated-type checking and both real loader/package tests pass. Loader evidence is four roles/sixteen skills with zero model turns/API milliseconds/cost. This naturally enabled host rerun supersedes earlier rollout-off observations; no host switch or restriction was bypassed.

The continuation fixes structured close denial stopping fox motion, and parallel Rust test temporary-directory collisions. Both regressions failed before their small corrections and pass afterward. Native direct close tests hit the host's static call-list restriction and were removed; close/reset behavior uses production-hook source doubles with actual `{ value }`/`{ deny }` envelopes. Automated native tests do not prove terminal painting or visible specialist submission/result review.

Latest full `cargo test --locked --workspace` observes 411 passing tests and the same one `ui::tests::past_fail_row_has_no_wash` failure under inherited NO_COLOR. No TUI source or environment flags were changed. Scoped completeness inventory finds zero stubs; catalog helper warnings are indirect-test heuristics and bundled upstream examples are reported separately.

The alpha packager reuses the Node exporter and refuses existing candidates/archives. Its regression also passed on Python 3.9.6 and 3.12.13. CI now includes that regression and preserves pinned resource bytes through scoped Git attributes; these uncommitted CI changes have not run remotely. The archive guide is a CI input. Cloud evidence remains the earlier remote PR37 baseline.

Current owner preview after packaging:

```sh
claude --plugin-dir /Users/entreprenurecosystem/kit-native-claude/.kit/releases/2.0.0-alpha.2/kit
```

Use the retained candidate rather than the changing source tree. Native appearance/focus/draft/captured-result acceptance remains pending: computer-use access to Terminal was denied, so no alternate capture or app bypass was attempted. No publication, tag, push or global settings change occurred.

## Actual-use continuation — 2026-10-02

The real same-session Backend → Kit capture → generated Security review test found a defect: native Security completed and read the artifact, but was pruned before its identity appeared in a status lookup. Failed evidence is retained at `.kit/actual-use-reviewed-20261002/failed-acceptance.json`.

The repair observes `agent.spawn` passively: unchanged input to `next`, unchanged result returned, native core trace identity/type/description retained only for successful Kit starts in the same session. Kit never calls spawn, changes permissions, submits prompts or rewrites another agent. Completion answers remain immutable; denied status stays unavailable, and missing verified attempts display Not attached. Unknown identities remain withheld. Delayed metadata can reveal a captured result without restoring an old Running status or clearing a newer status error.

Final shared `--host` check passes: 42 source regressions, 43 native fixtures, four native Rust unit/four CLI integration tests, deterministic packaging, formatting/Clippy and actual four-role/sixteen-skill discovery. Claude naturally updated to 2.1.288 during checks; source loading regenerated declarations from that exact host, and TypeScript noEmit passed again. Exact alpha.3 headless task/review/result acceptance then passed, with 22 independent assertions and successful native process exit. The previous 411-pass/one inherited NO_COLOR TUI failure remains outside this Mod scope. Native visible composer/focus/paint is still unverified because Terminal computer-use access was denied.

Current owner launch:

```sh
claude --plugin-dir /Users/entreprenurecosystem/kit-native-claude/.kit/releases/2.0.0-alpha.4/kit
```

Exact alpha.4 extraction repeated the same-session real-use flow successfully; package/loader and archive integrity checks pass after its README/version correction. Both answers are discoverable as Needs review; code passed 22 owner assertions. The initial Security report omitted explicit TypeError discussion; that failure is retained and a supplemental native read-only assessment closed the written coverage gap on the unchanged artifact. Both native processes exited 0; all results remain Needs review, with no automatic acceptance. See NATIVE-RELEASES.md for exact artifact/evidence boundaries.
