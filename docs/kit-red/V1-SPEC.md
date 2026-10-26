> REJECTED / SUPERSEDED: The user rejected this expanded design on 2026-10-01. Do not implement it. Read README.md for the minimal native-Claude direction.

# V1 specification: Kit Red

Status: implementation target, not built. This is the first release requested by the user.

## Product promise

“Make Claude Code feel like Kit: red, recognizable, readable, and easy to turn off.”

V1 is visual customization with working appearance controls. It does not read Kit's installed kits, receipts, or agent state. It works when the Kit binary is absent. Standalone Kit adopts the same red brand through its existing palette functions.

## Scope lock

Included:

- Red Kit signature in the shared above-prompt band.
- Working/status accent composed beside the complete native spinner output where the supported API permits it.
- A polished Appearance pane opened by `/kit` and `/kit appearance`.
- Saved appearance preferences with immediate redraw and honest save errors.
- Dark/light/terminal/monochrome treatment, keyboard access, narrow layout, no-color precedence.
- Matching standalone Kit brand accent, existing semantic colors and accessible modes retained.
- Native mod tests, real terminal verification, optional separately proven Desktop UI support, MIT distribution.

Excluded until V2:

- Overview/Kits/Runs pages, readiness badges, check summaries, install counts, receipt viewers.
- Process calls, filesystem scans, network requests, model calls, prompt submissions, permission hooks.
- Install/update/remove buttons, run launch/retry/stop, branch creation.
- Continuous mascot animations, resource gauges, dashboards, empty “coming soon” tabs.

The only persistent write in V1 is to the plugin's own appearance preference store through Claude's API. Environment reads are an explicit small allowlist for color/motion behavior, not a general environment inventory.

## Supported visual footprint

1. **Band:** compact `KIT · /kit`, red KIT and neutral hint. Keep other mods' content and ordering. No button hotkeys that intercept prompt typing.
2. **Appearance pane:** red title/section emphasis, restrained neutral body, clear selected values and keyboard focus, optional existing fox mark only if it fits without adding rendering complexity.
3. **Working treatment:** red Kit adjunct plus the original native tree. Never reconstruct a host spinner from incomplete props. If adjunct composition breaks native details or layout on a surface, leave the native spinner and retain the other two visual surfaces; record that platform limit.
4. **Standalone Kit:** replace the existing primary fox-orange accent with approved Kit red in current theme/brand consumers; preserve functionality and semantic state colors.

The native prompt editor, permission prompt, transcript, code blocks, and tool results stay under host control. Documentation calls this a Kit visual mod, not a complete replacement theme for every Claude pixel.

## Interaction

- `/kit` opens Appearance. `/kit appearance` does the same and is the stable command when V2 arrives.
- Repeating either command focuses the existing owned pane instead of duplicating it.
- Closing returns focus to Claude input. No process is started or stopped by pane actions.
- Color and visibility changes apply immediately and persist if storage succeeds.
- Saving fails: keep the selected value for this session, show “Changed for this session. Could not save.” Retry is explicit; do not promise persistence.
- Reset appearance restores defaults after an explicit action and updates only Kit's preference record. It does not reset Claude settings, other plugins, terminal colors, or Kit installations.
- Unknown arguments return a bounded help message with the two supported forms. No guess at a Kit CLI command.

## Preferences and compatibility

Use one store key, `appearance`, with `{schemaVersion:1,palette:"terminal",compactMark:true}` as defaults. Accepted palette values are `terminal`, `dark`, `light`, `high`, and `mono`. These are the only two appearance controls. Do not store a new independent setting for every incidental style choice.

Precedence: `NO_COLOR` overrides all stored color choices. In its absence, the stored choice controls the palette. Unknown/corrupt/older unsupported preference values fall back safely and do not crash rendering. Denied environment access uses conservative Mono with a distinct access-unavailable message; it is not presented as detected NO_COLOR. High contrast and Mono both use inherited color-free foreground; High contrast emphasizes focus/selection, without promising a higher measured contrast on arbitrary themes. A user override is available for terminal backgrounds; never probe the terminal by starting a child process.

`KIT_MOTION=off` is honored. V1 itself schedules no animation timers regardless of this value. Claude's own animation remains its responsibility; do not claim the mod disables native motion.

A stored preference is shared among sessions of this plugin. Last successful save wins; other open sessions adopt saved values on reopen/reload. No cross-session synchronization service. Read the latest record before changing an individual preference so unrelated recent fields are not needlessly overwritten; a remaining race is documented rather than disguised as an atomic transaction.

V1 must use settings keys and command names that V2 can keep. Theme changes cannot depend on a Kit engine schema or CLI version.

## Data and state boundaries

- Module state: loaded preferences, transient save error, pane state, current initialization/refresh generation.
- Plugin store: appearance preference record only.
- Host conversation: not read, rewritten, forwarded, or persisted.
- Environment: only explicitly documented color/motion variables.
- Kit installation: never accessed by the mod.

All mods API calls are literal and located in the entry module, or top-level helpers in that same module. Imported pure render helpers receive resolved UI primitives and data, never `$`. Event names are literals. This is required by Claude's static validator.

## Lifecycle

Startup/reload loads preferences and registers the command once for that module lifecycle. An unavailable store uses defaults and an actionable message in the pane, not a blocking error at every prompt.

Conversation clear/resume/branch handles the documented `classic.SessionStart` events and reloads appearance state where needed. No old conversation-specific text is retained because V1 stores none.

Unloading/disabling removes the mod's rendering and handlers. V1 has no timers or child processes to clean up. Re-enabling restores saved appearance. Uninstall may leave the host-managed plugin store until its normal retention policy removes it; document this precisely rather than claiming it is erased.

## Success criteria

- Installing V1 requires no Kit binary, login probe, model request, external network call by mod code, or custom dependency.
- Red identity is obvious in the band and pane and works with the supported native working treatment.
- Every appearance control works using keyboard alone; selected and focused are visibly distinct.
- No-color and narrow-terminal behavior remain fully legible.
- Other mods' band content, native spinner details, typing, and permission interactions remain intact.
- Reload, reset, save failure, corrupt storage, clear/resume/branch, and disable behave as specified.
- Standalone Kit retains real flows and semantic states while adopting the same accent.
- Exact tested builds and rendering limitations are documented. V1 source checks do not claim V2 data/action support.

See [design](V1-DESIGN.md), [build instructions](V1-BUILD.md), and [acceptance checklist](V1-ACCEPTANCE.md).
