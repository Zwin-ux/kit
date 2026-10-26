> REJECTED / SUPERSEDED: The user rejected this expanded design on 2026-10-01. Do not implement it. Read README.md for the minimal native-Claude direction.

# Kit Red decisions and risk register

These decisions remove routine ambiguity for the implementation pass. They do not claim unsupported host APIs work.

| Decision | Chosen behavior | Why / change only when |
| --- | --- | --- |
| Release boundary | V1 visual, V2 functional | User explicitly requested this sequence |
| Product identity | Kit Red; plugin `kit-red` | Stable name and preference namespace across versions |
| First entry | V1 `/kit` opens Appearance | Complete small visual product without future placeholders |
| Stable entry | `/kit appearance` retained in V2 | Existing users retain direct access |
| V1 defaults | Terminal palette, Compact mark On | Works without background detection or Kit binary |
| Palette values | terminal/dark/light/high/mono | Real environmental needs; no arbitrary color picker |
| Color override | Presence of NO_COLOR forces Mono | User terminal preference overrides branding |
| Motion | No V1 timers/mascot animation | Native host already conveys activity |
| Native status | Compose full host drawing; don't reconstruct | Exposed props may omit elapsed/token/effort details |
| Brand | Text KIT baseline, existing fox optional | Portable, legible, no new asset dependency |
| Color role | Red brand; explicit separate live/failure words | Avoid “everything red means failure” ambiguity |
| Preference store | `appearance`, schemaVersion 1 | One small object, no new storage layer |
| Save failure | Apply locally, explain session-only, Retry | Do not discard user choice or falsely claim persistence |
| API location | Literal calls in entry module/top-level same-file helpers | Static validator prohibits passing $ across imported modules |
| Runtime dependency | None beyond host mod runtime | A visual plugin does not need npm/React/Ink |
| V2 backend | Existing Rust Kit CLI | Preserve ownership/rollback/check/receipt behavior |
| Automatic diagnostics | Passive config/file inspection only | Current doctor can execute configured checks |
| Repo history | Full recorded canonical path; filter before limit | Same basename is not identity |
| Proof | Recorded checks/base/receipt/time; missing stays unknown | Model completion is not test evidence |
| Mutations | Review digest validated in Rust before writes | Preview and execution must refer to the same content |
| Run launch | Expected base and check plan bound to execution | HEAD/config can change between review and launch |
| Process stop | Owned cooperative cancellation, final receipt | Pane close/PID guessing cannot prove cleanup |
| Land wording | Create result branch | Does not imply merge/push/deploy |
| V2 agent scope | One owned active run per host session initially | Enough for complete workflow; do not build swarm UI yet |
| Desktop support | UI and binary reachability independently proven | Portable tree is not process-context evidence |
| Marketing proof | Concept art labeled, real recordings separately captured | Generated layouts are not runtime screenshots |

## Risks that need actual probes

| Risk | Early probe | Required outcome / fallback |
| --- | --- | --- |
| Host build API differs from website | Generate exact-build declarations | Use that build's supported API; record version |
| Spinner wrapper changes line geometry | Real working terminal before/after | Full native details preserved; otherwise native fallback |
| Custom terminal colors have poor contrast | Reference + customized ANSI captures | Explicit palette/Mono override, text and focus markers |
| Another mod replaces band after Kit | Coexistence test with ordering | Preserve downstream content; document ordering limits |
| Store/env access denied | Native stub + actual managed environment where available | Defaults/Mono and honest pane message; no failure loop |
| Desktop mod visuals unavailable | Real Code-tab test | Terminal-only support claim until proven |
| Existing Kit too old | Passive/capability request fails | Upgrade handoff; V1 still works |
| Approval waiting in nested agent | Disposable real nested run | Supported interaction or disabled adapter handoff |
| Mod unload loses child ownership | Real reload/disable/exit during run | Cooperative cleanup or precise unknown state; no false success |
| Concurrent source/target mutation | Expected-plan race tests | Refuse before changes or existing rollback conflict outcome |
| Corrupt/legacy receipt | CLI + parser fixtures | Unknown proof, visible bounded warning |

## Avoid accidental scope growth

No hosted service, new model, fork of Claude, terminal emulator, theme marketplace, telemetry dashboard, chat sync, agent-session messaging, general process-control RPC, or duplicate installer. The concept art does not authorize features outside these docs.

An uncertainty about one styling site should not turn into a rewrite of all host UI. An uncertainty about one agent's approvals should not turn into a permission bypass. A missing CLI field should become a small documented addition, not a second data system.
