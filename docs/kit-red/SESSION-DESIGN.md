# Kit in the coding conversation

Status: concept art provisionally approved; implementation and native behavior unverified. Latest user direction governs: keep the Claude conversation and add a small Kit presence. This document and [FIRST-LAUNCH.md](FIRST-LAUNCH.md) define that behavior. Earlier Appearance panels, dashboards, permanent role rosters and expanded V1/V2 screens remain rejected.

## Fixed product boundary

Preserve Claude's native transcript, prompt editor, tool output, permissions, keyboard controls and conversation continuity. Add compact Kit identity and truthful kit context only at supported render sites. The result should feel like the coding workspace the user already knows.

The concept does not authorize a new chat runtime, independent prompt editor, second agent engine, daemon or database. If a depicted addition cannot be implemented through a supported native surface without losing information or usable space, keep the native surface and record the limitation. Do not recreate the whole screen to match artwork.

Kit's existing setup and install machinery remains authoritative for configuration. Its Control Room remains available for isolated runs; neither receipts from that surface nor installed kits certify ordinary Claude conversation edits.

## Daily interaction

The user reads one native conversation, types into one native prompt and submits work normally. A small red/cream fox or compact Kit mark provides identity; a compact kit label appears only when its meaning can be established. Repository/provider information uses actual host context where available. Avoid duplicating context already clearly shown by Claude.

Reading order: prompt and work; real tool output and errors; kit context; decorative identity. No permanent four-kit catalogue, sidebar, stage tracker, progress checklist, mission banner, counters or achievement panel. Do not add an always-visible row solely to advertise Kit.

Illustrative fixture, not an exact native rendering contract:

```text
> Build a pricing page with three tiers.

I'll check the existing components and styles first.

Read  src/app/page.tsx
Read  src/components/button.tsx

Kit · Frontend Design · /kit
> Add a monthly / annual toggle
```

The role line is a proposed supported Kit addition; the conversation and tool lines belong to the host. These fixture reads are not evidence that a tool ran. The final native placement must be proven at the actual allocated width and height, without replacing the prompt or concealing host content.

## Exact four job identities

| Name | Small visual identity | Meaning |
|---|---|---|
| Frontend Design | Browser window | UI, design systems, accessibility, browser testing |
| Full-stack Design | Stacked layers | Frontend plus APIs, data, security and end-to-end tests |
| Backend Engineer | Server drawers | APIs, databases, security, performance and observability |
| LLM Engineer | Connected model nodes | Prompts, evals, RAG, model APIs, cost and latency |

Use related, intentionally drawn symbols with matching visual weight and distinct silhouettes. Keep full names beside them. They are job kits, not provider identities, model names, autonomous workers or the Control Room's engineering/product/design/QA personas. Several may coexist; Full-stack inherits Frontend, and Essentials is shared base content.

A requested chooser reuses the existing flat multi-select interaction and exact kit IDs. The four lead the experience; existing additional catalogue entries remain reachable. Do not restrict the catalogue to four or imply selecting four kits launches four agents.

The refined fox keeps Kit's recognizable heritage: compact head, pointed ears, cream cheeks, tapered muzzle and clean red silhouette. The provisionally approved artwork may refine the old geometry. A giant mascot, generic emoji or new mascot customization surface is out of scope.

## Context label must say what is known

| Known fact | Permitted label/behavior |
|---|---|
| No authoritative Kit data available | Identity only; do not infer a role from the task or artwork |
| Installation record exists for selected scope/provider | `Frontend Design · installed`; installation is not proof of instructions applied to this conversation |
| Host/integration proves the kit context is applied here | `Frontend Design · in this session` |
| Multiple applied kits | Show names when space permits; otherwise a count with an inspectable full list, without inventing a primary role |
| Kit chosen in picker but not installed | Show it only as a pending selection in the chooser/review, never as installed/applied |
| Installed change requires a new session | Say `Installed. Start a new session to use it.` only when this requirement is established |
| Last known data cannot be refreshed | Mark context stale or omit the effective claim; never keep an unqualified applied label |

The first native visual slice may show identity alone. Integration claims wait for real data. Do not silently execute diagnostic checks, scan repositories or start integrations to populate decorative UI.

## Interaction contract

- Typing remains native. Work starts only after deliberate native submission; viewing identity, opening a chooser or selecting a kit does not start a model turn.
- Any in-session chooser entry must use a supported command/control proven by the implementation. Do not promise a new shortcut merely because it appears in art.
- Opening and closing a Kit-owned chooser preserves the host draft and returns focus to the native input. Native permission UI takes precedence.
- Selecting kits changes a draft selection. Missing kits use the existing installation plan, scope and confirmation before writes or code-running integrations. Cancel preserves installed configuration.
- Changing a label is never treated as applying instructions. Report the real integration outcome and any necessary session restart.
- Display tool state, errors and check outcomes only from actual host/engine evidence. Preserve the original output, arguments and permission decisions; a successful model response does not erase a failed test.
- Keep completion wording scoped: `3/3 configured checks passed` only for the matching check evidence. Do not imply the whole conversation or current dirty tree was verified by another run.
- Close/reload/resume cannot cause automatic installation, task replay, duplicate agents or lost drafts. An old receipt is historical evidence, not proof a process is alive.

## Presentation and accessibility

Use restrained red accents with readable native/neutral text and a flat terminal background. Do not recolor protected host regions or force a terminal palette. The requested red identity does not require every line to be red.

Role/fox assets are optional rendering enhancements. Use them only where the host supports their size and format; a pixel icon is neither literal ASCII nor guaranteed to fit one cell. Plain terminals retain names and a compact text identity. Do not substitute the rejected punctuation logos as the rich visual direction.

At narrow widths, remove artwork before truncating meaningful state or interfering with the prompt. Keep the native scroll behavior; no animated transcript reflow. Respect NO_COLOR and existing reduced-motion handling. Color and motion never carry the only state distinction.

## Acceptance before claiming native support

| Scenario | Required proof |
|---|---|
| Idle and active Claude conversation | Before/after native captures show unchanged prompt/transcript/tool information with only supported Kit addition |
| Typed, unsubmitted draft; open/close Kit control | Exact draft retained; focus returns correctly; no model/process call caused by selection |
| Permission request while Kit UI is visible | Native request and all decisions remain visible and usable |
| One/multiple/missing/pending/stale kits | Labels match authoritative scope and distinguish installed from applied |
| Long transcript and long tool/error output | Native scrolling, full evidence and cancel behavior preserved |
| 60×20, 80×24, 120×40 terminals | Actual host allocation checked; no hidden prompt or clipped required control |
| Plain/NO_COLOR/reduced-motion environment | Names, focus and state remain usable without artwork/color/motion |
| Reload, disable and uninstall | Native UI restored without host-setting mutation or new work |
| Returning/resumed session | Actual context restored; no inferred liveness or replay |

Record exact Claude/Kit versions, terminal/OS, supported render sites, observed results and capture paths. Pure render tests and generated images are separate evidence levels from native behavior.

## Artwork status

The current conversation/identity and first-launch artwork is provisionally approved as visual direction. Do not regenerate or redesign it during implementation without a concrete mismatch. It is conceptual, not a screenshot of an installed mod, working chat transport, accessible control or completed tool run.

Retain the composition and restrained personality. Resolve unsupported placement, selection markers, texture, focus or state labels according to this behavioral contract. Earlier rejected Appearance/dashboard boards remain historical even if stored beside the current assets.
