> REJECTED / SUPERSEDED: The user rejected this expanded design on 2026-10-01. Do not implement it. Read README.md for the minimal native-Claude direction.

# V1 acceptance and release checklist

This checklist is runnable release work, not evidence that it has passed. Record the tested commit, Claude version, OS/terminal, date, command, outcome, and artifact location for each completed row. Keep temporary fixtures out of the public artifact.

## A. Package and capability gate

- [ ] Plugin name is `kit-red`, source is under `mods/kit-red`, license is MIT.
- [ ] Only local modules and permitted `claude-code` imports; no new runtime dependency.
- [ ] Load development plugin on the minimum supported build and read its generated `.claude-plugin/types/claude-code/index.d.ts` before relying on a new API.
- [ ] Record exact support for Text colors, Box composition, Select/Button focus, pane width/height, store/env APIs, `/kit` registration, and Spinner composition.
- [ ] `claude plugin validate ./mods/kit-red` exits 0.
- [ ] Static calls inventory contains only necessary UI/command/store/environment operations. No process, file/network, model, prompt, permission, or session messaging capability.
- [ ] Generated MCP type declarations and local configs are absent from the release artifact.

If exact-build types and docs disagree, use exact-build behavior. If a site cannot be styled safely, document and use the specified native fallback; do not patch internal host code.

## B. Automated behavior suite

Run `claude plugin test ./mods/kit-red` and record its test count. Tests must assert outcomes, not just that hooks registered.

| Case | Smallest useful assertion |
| --- | --- |
| Fresh session | `/kit` registered; defaults loaded; zero process/model/prompt calls |
| `/kit`, `/kit appearance` | Same owned pane ID opened, interactive result `{}` |
| Unknown argument | Helpful bounded usage text, no pane/process mutation |
| Appearance paint | Correct labels, stable keys, selected values |
| Palette change | Immediate redraw; valid store record written; focus not reassigned |
| Compact mark off | Downstream AbovePrompt preserved, no added KIT row |
| Another mod in band | Original downstream sentinel element remains |
| Foreign pane | Exactly passed to downstream handler |
| Native Spinner | Downstream sentinel tree and all its content remain, red adjunct separate |
| Missing/invalid stored data | Defaults or valid fields loaded, host usable, concise pane diagnostic |
| Save fails | Chosen value stays for session, Retry save offered, no false Saved state |
| Rapid preference changes | While save is pending new mutations are not accepted; accepted selection remains, navigation/Close work, and controls accept changes after completion |
| Reset | Writes only appearance defaults; leaves unrelated store keys alone |
| NO_COLOR present | No Kit color/background styles; all labels/selection retained |
| Conversation lifecycle | clear/resume/fork/reload have one command/band and valid appearance |
| Store/env refusal | Safe defaults/explicit fallback, bounded error, no retry loop |
| Close/reopen | Preferences retained; no duplicate pane, host focus restored where API supports |
| Unsupported surface | Concise intentional text fallback, no unsupported tree |

Stub every API call before the first test event. Use native `$.ui.mount`/press (and select only if a supported native Select is used) and pure fixture assertions. Tree tests do not establish actual focus painting or terminal escape correctness.

## C. Real terminal verification

Run the plugin with the exact supported Claude build. Do not change the user's normal default plugins/settings to make screenshots look correct. Use a disposable development session and record configuration.

```bash
claude --version
claude --plugin-dir ./mods/kit-red
```

Inside the session: `/kit`, change Palette, toggle Compact mark, Reset, close/reopen, `/kit appearance`, and an unknown `/kit` argument. Capture before/after.

| Terminal window | Required checks |
| --- | --- |
| 120×40 | Full Appearance, keyboard focus, idle band, working adjunct |
| 80×24 | Wrapped choices, readable hints, native prompt remains available |
| 60×20 | Allocated pane geometry honored; no art or clipping displaces controls |
| Less than 40 allocated columns or 8 rows | Compact fallback and reachable Close |

- [ ] Terminal, Dark, Light, High contrast, and Mono choices are distinguishable and usable.
- [ ] Actual light and dark backgrounds match reference contrast expectations; arbitrary themes use explicit overrides.
- [ ] Test a customized ANSI palette where brand/failure colors collapse; meaning remains in labels/focus.
- [ ] Test `NO_COLOR=1` in a fresh session and verify no Kit RGB/ANSI color styles.
- [ ] Test `KIT_MOTION=off`; V1 adds no animation, timer, or periodic redraw.
- [ ] Keyboard only: reach every control with Tab/Shift+Tab, choose values, reset, close. Ordinary input isn't intercepted.
- [ ] While Claude works, native words/details/elapsed/tokens/effort remain exactly as available without the mod; compare real captures.
- [ ] A real host permission request stays readable, focusable, and usable. Do not weaken permission settings for this check.
- [ ] Another simple test mod's band content remains visible and operable.
- [ ] CJK/wide characters in host content do not shift Kit controls or obscure the prompt. KIT itself needs no special font.
- [ ] Changing appearance while working doesn't submit a turn, alter tools, or stop activity.

## D. Lifecycle and persistence

- [ ] Change appearance, reload plugin, confirm saved settings reappear without duplicate wordmarks.
- [ ] `/clear`, `/resume`, `/branch` preserve appearance and don't retain unrelated conversation content.
- [ ] Two sessions use last-successful-save semantics; reopening reads current preferences. Document remaining non-atomic cross-session races.
- [ ] Simulated storage failure shows session-only state; explicit retry recovers.
- [ ] Disable the plugin: native UI restored, no Kit controls or activity remain.
- [ ] Re-enable: saved preference restored.
- [ ] Uninstall: plugin removed, host settings unaffected; store retention behavior documented accurately.

## E. Standalone Kit palette

The standalone change is visual only; current CLI/engine functionality must remain unchanged.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] Current upstream Kit setup and Control Room use the approved dark/light red tokens.
- [ ] Live, success, warning, failure, selection, and focus remain distinct in both color and monochrome.
- [ ] Existing theme variables, ANSI fallback, and one-clock rule are retained.
- [ ] Existing JSON output remains structured and color-free.
- [ ] Fox asset reused; no logo/license regression or new image runtime.
- [ ] Capture actual standalone screens beside mod captures for visual consistency.

## F. Desktop and distribution

Desktop is a separately reported target, not a prerequisite for claiming terminal-only V1 if it cannot be tested. Do not mark it supported without a real capture.

- [ ] Mount portable UI trees in native tests for terminal and Desktop.
- [ ] On an available supported Desktop Code tab, test Appearance controls, band, safe working treatment, persistence, and disable.
- [ ] Document Desktop WSL plugin limitation and unsupported visual surfaces.
- [ ] Install from a tagged marketplace entry in a clean environment; validate plugin actually loads from the artifact, not developer checkout.
- [ ] Reload/update/disable/uninstall instructions tested.
- [ ] README lists exact styled sites, exact tested Claude versions, no Kit binary requirement, preference semantics, and known fallbacks.
- [ ] Public screenshots/recordings are actual implementation captures, not the textual layout mockup.

## Release decision

Release V1 when A–E pass and distribution proof is captured. State Desktop status accurately from F. Do not wait for V2, and do not claim V2 functionality. Record any unsupported host styling as a visible compatibility limit, with the native fallback proven.

A version claim and screenshot are insufficient without the controls and lifecycle checks. Conversely, passing tests are insufficient without real rendering and keyboard proof.
