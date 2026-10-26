# Kit Red: independent product and design review

Review scope: proposed 2026-10-01 spec, current upstream README, DESIGN-tui.md and theme.rs at c35f60855a62534f2ed71edac3d1dc52835b87a3. No implementation or visual runtime proof. The recommendations below are proposed requirements, not statements about working mod behavior.

## Product decision

Make Kit Red a recognizable workspace for answering three questions: what is installed here, what work actually ran, and what evidence supports its result? Red gives the workspace its identity; useful evidence and clear next actions justify keeping it installed. Retain Overview / Kits / Runs. Avoid a dashboard of counters and a second settings application.

Primary scene: a developer alternates between Claude's prompt and a terminal pane during long coding sessions, on either a dark terminal at night or a light terminal in daylight. Respect the host background and user contrast settings. This forces both theme variants and rules out an all-red canvas.

Critical corrections to the existing spec:

1. Add a live token and explicit RUNNING text. The proposed palette omits this role; upstream deliberately distinguishes running work from red branding and failure.
2. Replace “Kit Connected” with “Kit CLI available.” There is no persistent connection. “Claude Ready” becomes the exact known condition: detected / authentication unknown / probe failed. Presence on PATH is not proof of authenticated launch capability.
3. A run result must identify its evidence and scope: check count, receipt, tested commit, and time. “PASS” alone invites users to assume their current Claude edits were tested.
4. Split historical lifecycle status from current liveness. An old receipt saying RUNNING reads “Last recorded: RUNNING; current activity unknown” until an authoritative source confirms it.
5. All-project history must be explicit and off by default. An unrelated green run must never occupy the current repository's success summary.
6. Narrow terminal behavior must protect the prompt, permission UI, and focus. “Stacked content” alone is not an acceptance criterion.
7. A release with only a colored banner should be called the visual preview. The full Kit Red release needs a useful Kits/Runs read experience.

## Visual identity

Use one compact Kit signature: existing fox when supported, otherwise bold KIT; red section titles; red active-tab text paired with brackets or underline; neutral content; a clearly marked selected row. Do not replace other agents' identifying marks with Kit branding. Keep third-party marks only as attribution/identification.

Minimum visual footprint: compact AbovePrompt signature, a fully designed /kit pane, and a working treatment wherever supported without removing host information. A decorative fox is never allowed to displace a failure or permission message. At narrow sizes, remove art before content. No persistent mascot animation in the mod's first release.

Recommended tokens retain the spec's red values. Add live dark #53D6CF / light #006B67. Reference backgrounds for verification: dark #151214, light #FFF8F6. Use these as test fixtures, not a forced terminal palette. Calculated WCAG luminance ratios against these references: brand 6.10/6.20; main text 16.53/15.18; secondary 7.52/6.45; live 10.53/6.07; success 10.75/6.25; warning 11.59/6.22; failure 8.42/7.59. Brand on the proposed selected wash is 5.19 dark / 5.57 light. These calculations establish only reference-palette contrast. Inherited backgrounds and customized ANSI colors require real captures and an explicit light/dark/mono override.

Never use red alone to distinguish a destructive action, selected item, or failure. Brand KIT, focus >, FAIL, and Stop are different textual/structural roles. Focus is visible independently of selection; in a selected failure row both the focus marker and FAIL remain. Avoid DIM on actionable errors and important provenance. ANSI16 may collapse reds; labels must still distinguish the states. High-contrast mode may sacrifice red branding for legibility.

Use existing terminal typography: monospace, bold titles and labels, neutral body, compact aligned lists. No extra display font, card grids, border nesting, shadows, or percentage-complete inventions. Desktop should preserve this hierarchy through supported portable primitives rather than introducing a different web dashboard.

## Hierarchy and journeys

### 1. First open, visual-only installation

/kit -> show repository identity and “Kit CLI not found” -> show the documented install command as inert selectable text -> user installs outside the pane -> Refresh -> populate the pane. Never auto-download on open. The mod continues to provide visual identity while unavailable. The empty Runs tab explains “Runs started by Kit appear here. This Claude session is separate.”

### 2. Returning to current work

/kit opens Overview. Header: KIT, repository basename plus expandable full path, last successful refresh age. Then show one meaningful next action, followed by the most relevant current-repository run. Prefer an authoritatively active owned run, otherwise latest recorded run. Small setup facts sit below, not three equally weighted status cards.

Suggested summary:
KIT / project-name                         Updated 8s ago
[Overview] Kits Runs
Latest Kit run: Fix form validation
FAIL · typecheck · receipt a12b · tested 7f31c2a
1 of 3 configured checks failed · completed 12m ago
[View failure]   [Refresh]
This Claude session is not a Kit run.

When checks pass: “3/3 configured checks passed” and View checks. Never “Safe”, “Approved”, “Verified code”, or “Ready to merge” solely from test success. Add New run only after launch is proven and available; its explanatory disabled state is discoverable.

### 3. Understand and fix a failure

Runs -> select run -> detail -> first failing check visible without opening raw output -> inspect command, exit result, recorded time, source commit -> expand bounded output -> Retry with editable original task and failure context. Retry creates a new run with a visible parent relationship. The previous failure remains intact. Raw output is selectable plain text, sanitized and marked when truncated. A check command is shown as evidence, never executed merely because it is viewed.

Detail order: task and run ID; lifecycle and check evidence; source repo/base/tested commit; first failure; changes summary; remaining checks; optional output tail. Keep run navigation and Back above details. At wide widths an optional detail split is useful; at narrow widths preserve one reading order.

### 4. Understand installed capabilities

Kits -> project scope selected -> list actual installation records -> select one -> show agent, scope, source/revision/license and files/capabilities -> show “Modified locally” next to affected content. Distinguish Installed here, Installed globally, and Listed in project lockfile but not installed. The latter is not an installed kit. Empty state offers an exact setup command. Do not represent a missing record as zero installed when reading failed.

Management extension: choose source -> preview resolved contents -> choose project/global and agent explicitly -> show code-running entries and conflicts -> confirm -> execute that exact preview -> receipt and undo instructions -> refreshed installation. Skills/rules-only is a first-class choice. A new source resolution or scope change invalidates the preview. Remove distinguishes removed files from retained manual modifications. No force button.

### 5. Launch, stop, and land

Launch extension: task -> repository/committed base -> ready agent -> review -> start once. Dirty-worktree warning is concrete: “This run starts from commit <sha>; uncommitted edits are excluded.” A denied permission ends with “Not started”; it never appears as a run in progress. Pending launch shows a disabled Start control and a local submission state distinct from engine RUNNING.

Stop shows the owned task and run ID. An explicit stop action can be confirmed inline; the result remains Stopping until confirmed. Closing the pane never implies stop. Host session and background Kit run get separate labels.

Land should be labelled “Create result branch” with secondary wording “Kit land.” Review shows target repository, original base, checks and exact resulting branch. Success says “Created branch kit/<id>”; it never claims merge, deployment, push, or PR creation. Unsupported or refused landing preserves the receipt and shows next action.

## Keyboard and size contract

Tab/Shift+Tab move among controls; arrow keys navigate a focused tablist/list; Enter activates selected control; Escape returns from detail then closes pane. Do not hijack ordinary prompt typing or bind letter shortcuts when a text field owns focus. Standalone Kit's existing shortcut vocabulary remains unchanged. Mod shortcuts may adopt those letters only while its pane owns focus and after conflict testing.

Opening the pane focuses its primary navigation or current item. Refresh preserves selection by stable ID, scroll and focus; disappearance of the selected row moves focus to a safe list position and announces the change where supported. Async output must never steal focus or auto-scroll a user reading history. Closing returns focus to the host input. Confirmation defaults to Cancel, not mutation.

At 120x40: run list plus optional detail split; secondary provenance visible. At 80x24: single-column detail and compact list. At 60x20: eliminate mascot, decorative borders, secondary columns; preserve state, task, selected item and actions. Path truncation keeps the full value available on detail. Do not truncate status labels. Under the supported floor use a concise text summary and command handoff rather than clipped controls. The mod must fit the host's allocated pane, not assume the entire terminal is available. CJK/wide glyphs and plain ASCII fallback are included in layout proof.

## State and acceptance matrix

| Scenario | Required visible result | Test evidence |
|---|---|---|
| Loading first data | Loading Kit data, stable layout, no fake zeros | Delayed CLI fixture + real pane capture |
| Refresh after good data fails | Last good data retained; Stale, timestamp and reason | Good response then timeout/malformed output |
| Agent only detected | Detected; authentication/launch not claimed | PATH-only agent fixture |
| No Kit binary | Visual identity works; install command and Refresh | PATH excluding Kit |
| No repo / non-Git folder | No repository selected; scope explicit; run disabled | Open from plain directory |
| No installed kits | Helpful setup action | Valid empty CLI response |
| Lockfile-only kit | Not installed; no invented capabilities | Actual lockfile/record mismatch |
| No receipts in this repo | No Kit runs here; optional explicit all-project link | Other-project receipts exist |
| Empty checks | UNCONFIGURED; Configure checks action | Real empty-check run |
| Checks passed | N/N checks, tested commit, receipt and time | Real passing run |
| Failed check | FAIL and first failure remain visible without color | Real failed run + NO_COLOR |
| Old RUNNING receipt | Last recorded RUNNING; liveness unknown | Stale receipt, absent authoritative process |
| Current Claude edits | No inherited PASS from unrelated Kit run | Separate session and receipt |
| Permission refused | Not started / operation denied, safe retry | Host refusal interaction |
| Duplicate Start | One run and one submission result | Rapid keyboard/click repeat |
| Stop | Stopping then confirmed final status; unrelated session survives | Owned process and child lifecycle proof |
| Close/reopen/reload | No unintended restart, duplicate run or false stop | Real host reload sequence |
| Manual install edits | Conflict and retained files explicit | Modified recorded skill file |
| Unsupported host site | Host content preserved; truthful fallback | Real terminal/Desktop coverage |
| Monochrome / light / dark | Focus, brand, RUNNING and FAIL distinguishable | Captures + palette checks |
| CJK and 60x20 | No overlapping rows, obscured actions or lost prompt | Wide-glyph and resize recordings |
| Keyboard only | All read and permitted write journeys complete | End-to-end key walkthrough |
| Disable/uninstall | Host UI restored; no plugin refresh activity | Before/after session proof |

Prioritize visual identity plus reliable read journeys as the first useful release. Management and launch are subsequent gated slices. Defer swarm dispatch UI, shared dashboards, custom themes marketplace, counters/analytics, mascot customization, broad message repainting and speculative AI summaries. None is needed to deliver the requested red Kit workspace with useful improvements.

Sources:
- Existing local proposal: /Users/entreprenurecosystem/kit/docs/superpowers/specs/2026-10-01-kit-red.md
- https://github.com/Zwin-ux/kit/blob/c35f60855a62534f2ed71edac3d1dc52835b87a3/crates/kit-tui/src/theme.rs
- https://github.com/Zwin-ux/kit/blob/c35f60855a62534f2ed71edac3d1dc52835b87a3/docs/dev/DESIGN-tui.md
- https://github.com/Zwin-ux/kit/blob/c35f60855a62534f2ed71edac3d1dc52835b87a3/README.md

Skill: Impeccable product guidance. Root PRODUCT.md/DESIGN.md absent; explicit spec-only/no-modification task takes precedence over the skill's setup-file workflow. No relevant memory fact was used. Claude mod interface URL could not be fetched by web tool in this lane, so supported API assertions remain the parent technical lane's responsibility.
