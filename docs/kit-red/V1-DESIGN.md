> REJECTED / SUPERSEDED: The user rejected this expanded design on 2026-10-01. Do not implement it. Read README.md for the minimal native-Claude direction.

# V1 Kit Red: visual design and acceptance contract

Status: implementation-ready design specification; no mod is implemented by this document. This document fixes V1's product boundary. Host capabilities still require the capability spike against the exact supported Claude build.

## Product boundary

V1 is a useful, self-contained visual customization: a red Kit identity, a readable working treatment, and a small appearance pane. It works without the Kit CLI and never reads kits, receipts, repository state, agent readiness, or executable checks. Its installation must not imply that Kit has verified the conversation's work.

V2 is the separate functional product, Kit inside Claude, with Overview / Kits / Runs. Do not ship those tabs, disabled run buttons, installation prompts for the CLI, sample run data, fake check badges, progress gauges, or roadmap cards in V1.

Display name: **Kit Red**. Plugin identifier: `kit-red`. Commands: `/kit` and the stable `/kit appearance` alias. V1 opens Appearance from either command. V2 keeps `/kit`, grows the functional workspace, and preserves the same Appearance controls behind a named Appearance action.

## The look

Kit Red is red framing around neutral work. The visual signature is the compact bold `KIT` wordmark, warm red for Kit-owned active elements, a restrained rose selected surface where supported, and warm neutral text. The conversation remains easy to read. A user should recognize Kit Red while typing, while Claude works, and when opening `/kit`; a banner alone is an incomplete release.

Use the existing Kit fox only in the Appearance pane's roomy layout, and only through a proven portable renderer. The baseline identity is text. No new mascot, required image protocol, Nerd Font, custom font, animated fox, colored body paragraphs, or decorative background fill. Fox art is the first content removed at narrow sizes.

### Token contract

| Semantic role | Dark | Light | Use |
|---|---|---|---|
| Brand | `#FF5A5F` | `#B42332` | KIT, pane title, selected appearance value |
| Body | `#F4F1ED` | `#242124` | Pane body on a known background |
| Secondary | `#AAA3A5` | `#62595C` | Supporting explanation and hints |
| Selection background | `#321C22` | `#FBE9EC` | Focused/selected Kit-owned control, if background primitives are supported |
| Live semantic role | `#53D6CF` | `#006B67` | Reserve for engine-confirmed run activity in V2 / standalone Kit; not a V1 agent-readiness claim |
| Success semantic role | `#74D99A` | `#176B3A` | Existing standalone Kit / V2 check results only |
| Warning semantic role | `#F2C66D` | `#805500` | An actual settings-persistence problem, existing standalone/V2 warnings |
| Failure semantic role | `#FF8D96` | `#A0152B` | An actual settings error, existing standalone/V2 failures |

Reference contrast backgrounds: `#151214` and `#FFF8F6`. They are test fixtures, never an automatic host-background override. Text roles meet 4.5:1 on those fixtures; focus/selection remains recognizable without color. Transparent/inherited body styles are used in Terminal mode. Do not claim universal contrast over arbitrary terminal themes.

ANSI16 maps brand to red, live to bright cyan, success to bright green, warning to yellow, and failure to bright red, paired with words. In the mod, High contrast uses inherited foreground with bold/bracket emphasis, sharing color-free rendering with Mono; it cannot guarantee a higher measured contrast on arbitrary terminal palettes. Standalone Kit retains its existing high-contrast palette, including cyan when appropriate. Accessibility takes precedence over preserving red on every surface. Monochrome uses native foreground plus bold, bracketed values, and an explicit `>` focus marker.

## Surface contract

The implementation must determine actual render-site support before painting. Use supported host elements; no private UI patching, terminal-palette mutation, escape-sequence theme injection, or replacement of Claude itself.

| Surface | V1 appearance | Information/control preservation |
|---|---|---|
| AbovePrompt | The exact compact `KIT · /kit` signature, at most one extra line | Preserve downstream mod content and host input; never intercept normal typing |
| Owned `/kit` pane | Red title and selected values, neutral explanations, immediate appearance preview | Render only the owned pane; foreign panes pass through |
| Working/Spinner site | Red Kit adjunct adjacent to native working output, only, composed with the full `next(e)` tree | Keep the complete host drawing, including native activity text, elapsed time, token/effort information; unsupported recoloring means adjunct only |
| PromptHint / SessionMode / TurnDuration | Optional red accent only after a separate preservation proof | No required V1 claim; leave native if full information cannot be preserved |
| Messages, tool rows, tool arguments, errors | Native | No recoloring/replacement in V1 |
| Permission prompts / prompt editor / host security UI | Native, protected | No custom approval, hiding, relabeling, interception, or recoloring claim |
| Claude Desktop Code tab | Same Kit text/hierarchy through supported portable elements | Test visual support separately; terminal art is optional and never required |
| Unsupported visual surfaces | Concise text response only where the command is available | Explain visual unavailability; no promise of theming headless/chat/cloud surfaces |

Do not append pretend activity labels such as Ready, Connected, Protected, Verified, or Running. The native spinner knows Claude's activity; Kit Red contributes appearance only. Its label must not suggest that Kit is supervising this session.

## First session and default behavior

On load, install the compact visual signature and safe working treatment. Do not open a pane, start onboarding, start a model turn, read the CLI, spawn a process, scan a repository, make a network request, or print an unsolicited transcript message.

Defaults: Palette Terminal; Compact mark On. Terminal uses inherited/native text colors with a named ANSI red brand accent. It performs no background detection or probe. V1 owns no timers or animation; Claude retains its own native animation.

Preference precedence is deterministic: a readable `NO_COLOR` presence forces Mono; otherwise use the valid saved Palette choice; otherwise Terminal. Read `NO_COLOR` only through the capability-proven host environment API; never spawn a shell. If environment access is denied or unavailable, use conservative Mono with the distinct notice `Environment access is unavailable; using Mono.` This fallback is not a claim that NO_COLOR is set. A forced Mono pane says `Mono is enforced by NO_COLOR`; changing the saved choice does not override it. V1 owns no motion, so it needs no motion setting. `KIT_MOTION=off` requires no additional behavior; host animation remains host-controlled.

No welcome tour is needed. The compact `KIT · /kit` signature supplies discovery.

## Exact Appearance content

Wide pane, proposed textual layout (the renderer uses native focusable elements, not a literal preformatted screen):

    KIT RED                                      Appearance

    Palette       Terminal   Dark   Light   High contrast   Mono
    Compact mark  On   Off

    Preview       KIT   Selected action   Normal text

    Colors apply to Kit's UI and supported Claude accents.
    Permission prompts and conversation output stay native.

    [Reset appearance]                            [Close]

The selected value has a persistent marker/brackets; keyboard focus has a separate `>` or native focus indicator. Current selection must remain visible after focus moves elsewhere. All choices have text labels. The preview shows appearance examples, not simulated runtime or verification states.

Two settings are sufficient: Palette addresses actual terminal variability; Compact mark lets users reclaim a prompt row. Do not add an accent-color picker, density selector, animation preference, or theme marketplace.

Edits apply immediately and persist to plugin-owned local settings; no Save or confirmation dialog. Closing does not undo changes. Reset appearance restores the two defaults only; it cannot erase host preferences or future V2 kit/run data. Reset is reversible through the same controls and needs no modal confirmation.

A one-line transient acknowledgement may read `Appearance updated`; it is not inserted into the model transcript. Do not add a success badge or celebratory animation.

When Compact mark is Off, AbovePrompt adds no Kit row; `/kit` and supported working accents continue working. This is a deliberate uncluttered mode, not a disabled plugin. Full disabling remains Claude's plugin control and is explained in documentation rather than duplicated as an unsafe custom toggle.

Settings failure: retain the selected appearance for this session, show `Applied for this session. Settings could not be saved.` and a Retry save action. Do not silently revert or claim persistence. Invalid stored settings fall back to defaults with `Some saved appearance settings were invalid; defaults are in use.` Corrupt settings must not prevent host UI rendering. No raw stack traces in the pane.

Unsupported visual command response: `Kit Red appearance is unavailable on this surface. Use Claude Code in a supported terminal or the supported Desktop Code tab.` Such fallback text is model-readable if the host inserts command results into context; document this limitation.

## Geometry and keyboard contract

Layout uses the host's allocated pane dimensions, not the entire screen. Keep one content region with ordinary rows, no nested cards. Maximum content width is 76 terminal columns. At wide widths, labels take 14 columns and values wrap naturally; keep the footer inside the pane. Never make the prompt unreachable to preserve appearance UI.

| Available pane size | Layout |
|---|---|
| Width at least 80 and height at least 16 | Full layout above; optional existing fox beside title if it consumes no extra reading rows |
| Width 60–79 | No fox; palette wraps across two lines; labels/values remain aligned; two-line explanation |
| Width 40–59 | Stack each label over its choices; preview becomes `KIT · Normal text`; shorten explanation to `Kit accents only. Host prompts stay native.`; footer wraps |
| Width below 40 or height below 8 | Compact text: `Kit Red. Enlarge the pane to edit appearance.` plus accessible Close; no clipped editable controls |
| Height 8–15 | Remove preview and decorative spacing first; settings region scrolls if supported; footer remains reachable; otherwise compact fallback |

At terminal sizes 60x20, 80x24, and 120x40, verify actual host allocation and protected input visibility. A 60x20 terminal may allocate less than 60x20 to the pane; the implementation must respond to the allocation. Full-page overlays are not assumed.

Tab/Shift+Tab traverse controls and actions. Use native focusable Buttons as the baseline; Space/Enter activates. Arrow-key navigation is optional only if a supported native Select is adopted and tested, never claimed for a custom button row. Escape closes the pane and restores host input focus. Enter on Close closes. No global letter/digit shortcuts and no keyboard interception in the compact band. Mouse interaction, if supported, must reach the same controls without being required.

Opening `/kit` or `/kit appearance` focuses Palette's selected value where the exact host API permits it, otherwise the first Palette control. Changing a value does not move focus. Resizing preserves active control and selected value. Closing/reopening preserves settings and starts at Palette again. Refresh/reload must not duplicate registrations or wordmarks. Host permission UI always wins focus and visibility.

Screen readers and assistive terminals: controls expose labels and selection where the API supports them. Do not claim native accessibility-tree coverage without testing. Plain text, meaningful reading order, no color-only content and keyboard reachability are mandatory even if richer semantics are unavailable.

## Required visual/state proof matrix

| Case | Expected result |
|---|---|
| Clean load, Kit CLI absent | Red compact identity and native working treatment; no CLI error, setup nag, process or model call |
| Dark reference background | Red title/accent, neutral readable content, separately visible focus and selected value |
| Light reference background | Dark red accent, dark neutral body, readable hints and focused controls |
| Unknown host background / Terminal | Native/inherited body; conservative ANSI red accent; no invented background detection |
| User-custom ANSI palette | Labels and focus remain understandable even when colors collapse |
| High contrast | Legible native/ANSI contrast, visible focus; red may yield to accessibility |
| NO_COLOR | No color styles or Kit motion; KIT hierarchy and all controls still usable |
| Reduced motion / KIT_MOTION=off | No Kit-added animation; native host state words preserved |
| Compact mark Off | No added AbovePrompt row; `/kit` and working treatment retained |
| Idle session | No fake status, moving mascot, periodic redraw or background work |
| Claude working | Native status/elapsed/token/effort information unchanged; red identity visible through proven treatment |
| Host permission request | Native prompt and decision controls remain fully visible and usable |
| Another mod's AbovePrompt | Both outputs survive composition without replacing one another |
| Foreign pane | Kit does not redraw it |
| Palette changed with keyboard | Immediate preview, persistent selection marker, retained focus |
| Settings write failed | Honest session-only state and retry; host remains usable |
| Invalid stored settings | Safe defaults plus concise explanation on opening Appearance |
| Resize / narrow pane | No clipped controls; meaningful compact fallback below supported floor |
| Reload / clear / resume / fork | One registration/wordmark; preferences retained or re-read; no stale pane focus |
| Disable / uninstall | Native UI restored; no Kit UI or activity left behind; host settings unchanged |
| Desktop Code tab | Native portable Kit appearance tested on actual supported build; no terminal-only dependency |

Proof must include real captures of idle, working, Appearance, keyboard focus, and a host permission request, plus light/dark/mono at supported terminal sizes. Snapshot and pure-tree tests support these captures but do not replace them. Publish precisely which host sites were styled in the tested build.

## Standalone Kit identity

The same red identity applies to standalone Kit's existing semantic theme functions, not a second theme framework. Change the brand token for dark/light themes; preserve live/check/error labels and existing high/mono/ANSI handling. Reuse the existing fox asset. Do not recolor every agent logo red or replace the meaning of PASS/FAIL/RUNNING.

Keep standalone Kit's existing geometry, keyboard controls, theme environment variables and process behavior. V1 plugin settings belong to the plugin; do not write shell profiles or synchronize terminal preferences silently. Visual parity means consistent tokens and identity, not forcing one surface's preference storage into another.

Standalone acceptance: setup and Control Room show the same Kit red; active/failure/selected rows remain distinguishable; plain and JSON output retain their existing no-color behavior; no extra animation timer is added.

## V2 upgrade contract

V2 adds Overview / Kits / Runs only after genuine CLI contracts exist. Existing `/kit` remains the entry point. `/kit appearance` continues to open Appearance directly. Appearance becomes a named action visible in the workspace header, preserving Palette, Compact mark, and Reset appearance semantics. Escape in Appearance returns to the workspace; Escape at the workspace root closes it. The transition is documented in V2 release notes.

Persisted V1 settings remain valid unchanged; new V2 settings get independent keys/defaults. Reset appearance never resets repository scope, deletes receipts, uninstalls kits, stops runs, or changes permissions. V2 can enrich the compact band only with explicitly scoped real data; it must retain an appearance-only option and must never attach a past run's PASS to the current Claude session.

Do not block V1 release on V2 integration. V1 is complete when the visual surface contract, Appearance controls, shared standalone identity, and acceptance matrix pass. V2 is complete only when its separate functional journeys pass.
