> REJECTED / SUPERSEDED: The user rejected this expanded design on 2026-10-01. Do not implement it. Read README.md for the minimal native-Claude direction.

# Kit Red V1: implementation-ready technical design

Date: 2026-10-01. Planning artifact only. No plugin built, installed, or runtime-validated by this review.

## Version boundary

**V1 is Kit Red: visual-only. V2 is Kit inside Claude: functional integration.** V1 adds a persistent Kit identity and an appearance pane. It never discovers Kit, reads repository files, shells out, starts agents, fetches receipts, or implies a run passed. V1 can be installed on a machine with no Kit binary.

The existing standalone Kit red-theme changes can ship alongside V1 as a separate Rust change. They are not a runtime dependency of the mod.

## Documented facts versus implementation probes

Documented as of Claude Code 2.1.287:

- A plugin manifest plus hooks/modules entry loads ES-module hooks.
- session.start registers an immediate command through $.command.register.
- A command handler can open a pane and return {}, causing no command transcript text or model turn.
- ui.render can draw AbovePrompt, Pane and Spinner. await next(e) preserves the downstream drawing; another plugin earlier in the chain may still replace ours.
- $.ui.resolve(e) supplies supported elements; Box, Text, Button are portable between terminal and Desktop.
- $.store is a plugin-owned persistent JSON store; $.env.get reads literal-named environment variables.
- $.ui.invalidate('ui.render') requests repaint. There is no need for a timer in V1.
- ui.close identifies person/plugin/unload closure; session.start does not rerun for clear/resume/branch; classic.SessionStart source clear/resume/fork covers those operations.
- UI mount tests check trees and callbacks, not physical terminal painting.

Probe before implementation is considered ready:

1. Generate declarations from the exact installed build by loading the development directory. The reviewed public GitHub types were 2.1.277 and disagree with current docs in places.
2. Confirm Text color/bold props, Box children composition, and Spinner adjunct tree are accepted by that build.
3. Verify actual rendering and line height of a red adjunct beside/above the preserved native Spinner. Host elapsed/tokens/effort must remain visible. If composition is unsuitable, keep Spinner unchanged and rely on the always-visible band; describe that release honestly.
4. Verify command collision behavior and useful initialization diagnostics.
5. Test actual Desktop painting separately. V1 has no Desktop process-access dependency.

## Smallest file map

```
mods/kit-red/
  .claude-plugin/plugin.json
  hooks/hooks.json
  hooks/register.js
  tests/visual.test.ts
  README.md
```

One runtime module is enough for V1. Do not create kit.js, actions.js, a store abstraction, event bus, schema registry, npm package, or generic theme engine. If pure palette/render code becomes shared with V2, extract it then without changing its observable contract.

Manifest:

```json
{"name":"kit-red","version":"0.1.0","description":"A red Kit identity and appearance controls for Claude Code","license":"MIT"}
```

Hooks manifest:

```json
{"modules":["./register.js"]}
```

Validate the manifest against the installed build before fixing release metadata. Keep the repository license in the release artifact according to the existing packaging practice.

## Public product contract

- Plugin name: kit-red. Keep it stable across V2 upgrades so preferences do not move.
- Command: /kit, with /kit appearance as its stable argument alias. V1 opens Appearance. V2 reuses the same command/pane with an Appearance entry inside its larger workspace.
- Pane id: kit-red. Only this pane is handled.
- Pane heading: KIT RED. Supporting sentence: “Appearance for your Claude Code workspace.”
- Pane content: color mode choice, static preview, explanation of host-controlled regions, and Close.
- No Overview/Kits/Runs placeholders, fake connection state, disabled Run buttons, or sample receipts in normal V1 UI.
- Static band: “KIT · /kit”, shown when Compact mark is On (the default). It carries no ready/running/pass implication.
- Working adjunct, if proven: “KIT” in brand styling plus the entire downstream host tree. Do not recreate or summarize the native spinner.
- Branding uses an accessible KIT wordmark initially. A fox may be added only through a supported portable glyph with checked width; no font-dependent emoji as the only identity.

## Appearance preferences and accessibility

Persistent key: `appearance`. Exact schema/default:

```json
{"schemaVersion":1,"palette":"terminal","compactMark":true}
```

Allowed palette values: terminal, dark, light, high, mono. Unknown schema or malformed fields normalize to defaults with an inline invalid-settings message; the read itself never overwrites stored content. Persist only this small appearance object, never future V2 state. Multiple sessions use last successful object write; no compare-and-swap or live sync is claimed. Re-read on startup/reload and conversation reset, and on /kit open only when there is no unsaved session-only choice. A failed save must not be discarded just because the pane reopened.

Compact mark Off removes only Kit's AbovePrompt row; /kit and the working adjunct remain. Reset appearance restores terminal/true only; it does not delete the store or any V2 data. It applies and saves like every other preference edit.

Modes:

| Mode | Brand treatment | Body treatment |
|---|---|---|
| terminal | Native named red; follows terminal palette | Inherited |
| dark | #FF5A5F, confirmed on dark reference background | Inherited |
| light | #B42332, confirmed on light reference background | Inherited |
| high | Inherited foreground + bold KIT label | Inherited; explicit selected text |
| mono | Inherited foreground + bold KIT label | Inherited; explicit selected text |

Terminal palette red is user-configurable and cannot guarantee a contrast ratio. Explain dark/light/high choices in the pane. Never infer a background by reading arbitrary config files or invoking terminal commands. Do not force a background. Explicit token modes receive reference contrast tests and actual terminal checks.

NO_COLOR presence, including an empty value, overrides the effective mode to mono. Keep the saved choice unchanged so it returns when the environment override is absent. Show “Mono is enforced by NO_COLOR” rather than pretending color buttons changed the display. V1 has no animation or timer in any mode, so KIT_MOTION=off requires no read or extra behavior. Do not claim to suppress Claude's own animation; this setting affects Kit-owned visuals only.

Use text “(selected)” for the active choice. Color is never selection's sole indicator. Changes apply immediately. On store.set failure retain the session appearance, show “Applied for this session. Settings could not be saved.”, and offer Retry save. Saving is guarded against concurrent double presses; pending controls must explain the pending state. No hidden file write or host settings mutation.

## Event and state lifecycle

Module variables: appearance object, NO_COLOR flag, initialization/save error, save-in-flight, unsaved-choice flag, initial supported interactive surface. They hold only appearance state. Reload recomputes from store/environment. No $.state declaration is necessary for V1, because all durable state is one small appearance object and all transient state is safe to reset.

| Event/action | Behavior |
|---|---|
| session.start | Read appearance and literal NO_COLOR; register immediate /kit; next(e). Catch recoverable read/registration failures independently. |
| command.run kit | Re-read appearance preferences unless a save failed and a session-only choice remains. If a supported interactive drawing surface is available, open pane with focus:true and closeOnEscape:true, then return {}. Otherwise return short static V1 capability text. |
| classic.SessionStart clear/resume/fork | Re-read preference/env, refresh appearance unless a session-only unsaved choice remains, invalidate UI, then next(e). Never register command twice. |
| ui.render AbovePrompt | Resolve elements; compose Kit band and await next(e). No store/env reads or mutation in render. |
| ui.render Pane | Foreign requestId returns next(e); owned pane draws appearance controls. |
| ui.render Spinner | Proven adjunct plus await next(e), or unconditional passthrough if capability failed. No synthetic activity inference. |
| Button onPress | Apply a validated appearance immediately, invalidate, then persist; on failure retain it for this session and expose Retry save. Reset restores only appearance defaults. Close button calls ui.close. |
| Pane close | No process or timer exists to clean up. Do not intercept/refuse host close. |
| reload/disable/uninstall | Host removes drawings/commands. No runtime activity persists. Store preference may remain under Claude cleanup policy; uninstall documentation must say so. |

The exact way to query current surfaces for a session that attaches a UI after startup should be checked in generated declarations. Initial session.start surface/isInteractive is sufficient for the ordinary terminal and Desktop acceptance paths. Do not invent a capability method. If current-surface querying is unavailable, document late-attachment fallback and support normal /kit use after starting on a supported surface.

## Build sequence and normative completion requirements

1. Create the five-file plugin listed above and validate its manifest/entry point.
2. Implement preference normalization and command/open lifecycle with native tests.
3. Implement band and owned pane, including the full responsive/focus contract below.
4. Prove Spinner adjunct in the exact host; use native fallback if composition loses details/space.
5. Complete save/error/reset/reload/no-color tests; capture actual terminal layouts.
6. Apply the standalone theme change as a separate existing-TUI-owner slice.
7. Complete V1-ACCEPTANCE.md and release documentation before starting V2.

**Normative requirements missing from the short reference shape below:**

- Include explicit `Palette` and `Compact mark` labels plus the static `Preview` row. The short code is an API example, not the complete pane layout.
- Use allocated `e.props.bodyColumns` and documented placement/scroll fields. Never treat viewport rows as pane rows. Follow V1-DESIGN thresholds; below 40 allocated columns or 8 rows show compact text and reachable Close.
- Use native focusable Buttons as the baseline, with Tab/Shift+Tab and Enter/Space. Arrow-key radio-group navigation is not a V1 requirement unless a native supported Select is adopted and tested. Do not write a custom keyboard listener.
- Initial focus should land on the selected Palette control if the exact-build API supports it; otherwise first Palette control. Native focus must visibly differ from persistent selected text. Preserve focus on redraw/resize through stable keys, without repeated autoFocus.
- During an appearance save, show `Saving appearance…`; prevent a second mutation while preserving navigation/Close. This means a press during pending save is not accepted as a new choice. Test that the accepted choice stays visible and that the user can choose again after completion. Do not queue hidden mutations.
- Environment-read refusal uses conservative Mono with the separate message `Environment access is unavailable; using Mono.` This is not an assertion that NO_COLOR exists.
- High contrast intentionally uses the terminal's native foreground plus bold/bracket markers. It does not promise a higher measured contrast than Mono on arbitrary terminal palettes. The two modes share color-free rendering; High contrast can add stronger focus/selection emphasis. Do not force a black background.
- In the band only `KIT` receives red/bold; ` · /kit` inherits host foreground. Use sibling Text elements, not one red string.

The complete product behavior in V1-SPEC/V1-DESIGN is authoritative over the compact examples. No implementation is complete until these requirements are present and tested.

## Reference hook shapes

These are implementation-ready patterns derived from official documentation, not a claim that this assembled plugin was executed. Complete readPrefs/applyAppearance/error handling and validate the generated plugin before publishing. All helpers taking $ must be top-level in the same runtime file. Callbacks may close over $; do not pass $ to an imported or nested helper.

```javascript
const PANE = 'kit-red'
const MODES = ['terminal', 'dark', 'light', 'high', 'mono']
const LABELS = { terminal: 'Terminal', dark: 'Dark', light: 'Light', high: 'High contrast', mono: 'Mono' }
const DEFAULTS = { schemaVersion: 1, palette: 'terminal', compactMark: true }
let appearance = { ...DEFAULTS }
let unsaved = false
let noColor = false
let colorNotice = ''
let canDraw = false
let saving = false
let error = ''

function validAppearance(value) {
  return value?.schemaVersion === 1 && MODES.includes(value.palette) && typeof value.compactMark === 'boolean'
}

function brandStyle() {
  const mode = appearance.palette
  if (noColor || mode === 'mono' || mode === 'high') return { bold: true }
  const color = mode === 'dark' ? '#FF5A5F' : mode === 'light' ? '#B42332' : 'red'
  return { color, bold: true }
}

// Top-level helper in register.js: statically discoverable API calls.
async function readPrefs($) {
  try {
    if (!unsaved) {
      const saved = await $.store.get('appearance')
      appearance = validAppearance(saved) ? saved : { ...DEFAULTS }
      error = saved !== undefined && !validAppearance(saved)
        ? 'Some saved appearance settings were invalid; defaults are in use.' : ''
    }
  } catch {
    error = 'Appearance could not be loaded. Try opening /kit again.'
  }
  // Handle each read independently in final code: a denied env read must not
  // prevent command registration or accidentally disable the whole plugin.
  try {
    noColor = (await $.env.get('NO_COLOR')) !== undefined
    colorNotice = noColor ? 'Mono is enforced by NO_COLOR' : ''
  } catch {
    noColor = true
    colorNotice = 'Environment access is unavailable; using Mono.'
  }
}

async function applyAppearance($, value) {
  if (saving || !validAppearance(value)) return
  appearance = value
  unsaved = true
  saving = true
  $.ui.invalidate('ui.render')
  try {
    await $.store.set('appearance', value)
    unsaved = false
    error = ''
  } catch {
    error = 'Applied for this session. Settings could not be saved.'
  } finally {
    saving = false
    $.ui.invalidate('ui.render')
  }
}

export function register(on) {
  on('session.start', async ($, e, next) => {
    canDraw = e.isInteractive && (e.surface === 'terminal' || e.surface === 'desktop')
    await readPrefs($)
    try {
      await $.command.register({
        name: 'kit', description: 'Open Kit Red appearance', immediate: true,
      })
    } catch {
      error = '/kit could not be registered. Check for a command-name conflict.'
      // Add a bounded host diagnostic outside render if needed, then validate.
    }
    return next(e)
  })

  on('classic.SessionStart', async ($, e, next) => {
    if (['clear', 'resume', 'fork'].includes(e.source)) {
      await readPrefs($)
      $.ui.invalidate('ui.render')
    }
    return next(e)
  })

  on('command.run', { command: 'kit' }, async ($, e) => {
    if (e.args.trim() && e.args.trim() !== 'appearance') return { text: 'Use /kit or /kit appearance.' }
    if (!canDraw) return { text: 'Kit Red appearance is unavailable on this surface. Use Claude Code in a supported terminal or the supported Desktop Code tab.' }
    await readPrefs($)
    await $.ui.open({ id: PANE, title: 'Kit Red', focus: true, closeOnEscape: true })
    return {}
  })

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const { Box, Text } = $.ui.resolve(e)
    const downstream = await next(e)
    if (!appearance.compactMark) return downstream
    return Box({ flexDirection: 'column', children: [
      Box({ flexDirection: 'row', children: [
        Text({ ...brandStyle(), children: ['KIT'] }),
        Text({ children: [' · /kit'] }),
      ] }), downstream,
    ] })
  })

  on('ui.render', { component: 'Pane' }, async ($, e, next) => {
    if (e.requestId !== PANE) return next(e)
    const { Box, Text, Button } = $.ui.resolve(e)
    return Box({ flexDirection: 'column', children: [
      Text({ ...brandStyle(), children: ['KIT RED'] }),
      Text({ children: ['Appearance for your Claude Code workspace.'] }),
      Text({ children: ['Palette'] }),
      ...MODES.map(value => Button({
        key: 'mode-' + value,
        label: LABELS[value] + (appearance.palette === value ? ' (selected)' : ''),
        onPress: () => applyAppearance($, { ...appearance, palette: value }),
      })),
      Button({ key: 'compact-mark', label: 'Compact mark: ' + (appearance.compactMark ? 'On' : 'Off'),
        onPress: () => applyAppearance($, { ...appearance, compactMark: !appearance.compactMark }) }),
      Text({ children: ['Preview'] }),
      Text({ ...brandStyle(), children: ['KIT'] }),
      Text({ children: ['Selected action · Normal text'] }),
      Text({ children: [colorNotice || 'Host prompt and permission controls keep their original appearance.'] }),
      Text({ children: [saving ? 'Saving…' : error] }),
      ...(unsaved && !saving ? [Button({ key: 'retry-save', label: 'Retry save', onPress: () => applyAppearance($, appearance) })] : []),
      Button({ key: 'reset', label: 'Reset appearance', onPress: () => applyAppearance($, { ...DEFAULTS }) }),
      Button({ key: 'close', label: 'Close', onPress: () => $.ui.close({ id: PANE }) }),
    ] })
  })
}
```

Before using this shape, confirm the validator accepts calling the top-level applyAppearance helper from the button callback. This is consistent with the documented top-level helper rule; the actual validation run is the acceptance authority. Do not work around a rejection by hiding API calls. Inline the three store/state operations in the callback if needed.

Spinner adjunct is deliberately a separate capability-controlled addition:

```javascript
on('ui.render', { component: 'Spinner' }, async ($, e, next) => {
  const { Box, Text } = $.ui.resolve(e)
  const downstream = await next(e)
  return Box({ flexDirection: 'column', children: [
    Text({ ...brandStyle(), children: ['KIT'] }), downstream,
  ] })
})
```

This preserves the downstream tree instead of reimplementing word/message/suffix/mode. Physical layout remains a probe. Do not enable a bulky two-line adjunct if the real terminal loses needed vertical space; preserving the host spinner plus a red band is the required fallback.

## Native test patterns

Register every stub before the first API call. Raise session.start explicitly when testing startup-dependent behavior. A passed-through event needs a terminal stub. Mods API stubs return {value}; core-event stubs use their event result. Denial uses {deny}, not a thrown stub. Tests use claude-code/testing supplied by Claude, not a package installed from npm.

Startup and no-work test:

```typescript
import { expect, mock, test } from 'claude-code/testing'

test('/kit opens Appearance without a model turn or process', async ($, on) => {
  mock.store(on, {})
  mock.env(on, {})
  const registered: unknown[] = []
  const opened: unknown[] = []
  const forbidden: string[] = []
  on('command.register', (_, e) => { registered.push(e); return { value: undefined } })
  on('ui.open', (_, e) => { opened.push(e); return { value: { isPlaced: true } } })
  on('session.start', () => ({ cwd: '/work' }))
  on('model.complete', () => { forbidden.push('model'); return { deny: 'forbidden in V1' } })
  on('process.run', () => { forbidden.push('process'); return { deny: 'forbidden in V1' } })
  on('prompt.submit', () => { forbidden.push('prompt'); return { text: '' } })

  await $.session.start({ surface: 'terminal', isInteractive: true, cwd: '/work' })
  expect(registered[0]).toMatchObject({ name: 'kit', description: 'Open Kit Red appearance', immediate: true })
  const answer = await $.command.run({ command: 'kit', args: '' })
  expect(answer).toEqual({})
  expect(opened.length).toBe(1)
  expect(forbidden).toEqual([])
})
```

Static validate inventory is also required: the three forbidden-call stubs are defense in depth and do not enumerate every possible forbidden API.

Portable pane test:

```typescript
const PANE = {
  plugin: 'kit-red', component: 'Pane', requestId: 'kit-red',
  viewport: { columns: 80, rows: 24 },
  props: {
    title: 'Kit Red', isFocused: true, bodyColumns: 60,
    placement: 'inline', scroll: { offset: 0, bodyRows: 12 }, view: {},
  },
} as const

test('mode control saves and remains explicit on both surfaces', async ($, on) => {
  const saved = new Map<string, unknown>()
  on('store.set', (_, e) => { saved.set(e.key, e.value); return { value: undefined } })
  for (const surface of ['terminal', 'desktop'] as const) {
    const ui = await $.ui.mount({ ...PANE, surface })
    await ui.press({ key: 'mode-light' })
    expect(await ui.find({ type: 'Text', text: 'KIT RED' })).toBeDefined()
    expect(saved.get('appearance')).toEqual({ schemaVersion: 1, palette: 'light', compactMark: true })
    const selected = await ui.find({ key: 'mode-light' })
    expect(selected?.props).toMatchObject({ label: 'Light (selected)' })
    await ui.unmount()
  }
})
```

Add these cases to the same test file, using the same official patterns:

1. AbovePrompt terminal stub returns HOST BAND; mount verifies HOST BAND, KIT, and the neutral /kit hint all remain.
2. Foreign pane terminal stub returns FOREIGN; Kit leaves it unchanged.
3. Spinner terminal stub returns distinct elapsed/token/effort text; adjunct preserves all text. Real host test remains mandatory.
4. NO_COLOR=1 and separately NO_COLOR="" after session.start: KIT text has no color/backgroundColor prop; selected labels remain.
5. Denied store.set: new mode remains active for this session, error appears, Retry save works; reopening does not discard the unsaved choice.
6. Corrupt stored appearance normalizes to terminal/true with a clear notice and no automatic write.
7. classic.SessionStart clear/resume/fork reloads changed stored preference; no duplicate command registration.
8. Noninteractive startup followed by /kit returns bounded static text; no pane/process/model attempt.
9. Preference load denial leaves usable default controls; env denial uses conservative monochrome.
10. Compact mark Off removes only the Kit band; Reset restores terminal/true, respects NO_COLOR and preserves unrelated store keys. Both /kit and /kit appearance open Appearance.
11. Double press while save pending causes only one write. Resolve the stubbed pending promise; confirm saving state clears.

Use a static inspection of validate output to prohibit V1 fs/process/http/model/prompt/session-message/tool/permission APIs. Allowed inventory should be limited to command.register, store.get/set, env.get, ui.open/close/invalidate and UI resolution. Document the exact inventory emitted by the real validator; element resolution may be reported differently by build.

## Manual acceptance and release commands

```
claude plugin validate ./mods/kit-red
claude plugin test ./mods/kit-red
claude --plugin-dir ./mods/kit-red
```

Capture 60x20, 80x24 and 120x40. Test dark/light/ANSI-custom/high/mono, NO_COLOR, KIT_MOTION=off, keyboard-only navigation, command during active turn, Escape, pane close, and another mod's band. The pane must fit through normal scrolling and maintain reachable Close. Check permission UI remains native. Inspect actual host Spinner before/after with elapsed/token/effort, failure and tool progress, not just a mocked string.

Test reload, clear, resume, branch, disable and uninstall. Preference retention is documented, host UI restored, no timer/process remains. Test Desktop on a real supported Code session; WSL plugin absence is an explicit unsupported case. Record exact Claude version and OS/terminal. Install from a tagged artifact in a clean profile before release. Screenshots of static fixtures are not live execution evidence.

## V2 extension contracts

Keep plugin name, /kit command, /kit appearance alias, pane id and appearance schema/key (`schemaVersion:1`, `palette`, `compactMark`). V2 expands the owned pane, retains Appearance, and keeps theme-only behavior when Kit is absent. Do not register a second /kit or layer a second identity band.

V2 may add pure kit.js parsers and pure actions.js transitions; all API-owning calls/helpers remain in register.js. V1's brandStyle and view construction can be extracted into one pure view.js when shared use actually exists. No public generic renderer interface is needed now.

V1 does not reserve fake readiness fields or run state. V2 introduces actual diagnostics/receipts only after passive inspection, canonical repository identity and schema contracts are proven. The appearance key must never hold Kit receipts, secrets or process ownership. V2 run controls require their own lifecycle proof. Upgrading V1 to V2 must preserve the user's appearance choice and the accessible no-color behavior without requiring a reinstallation or theme migration.

## Sources

- https://code.claude.com/docs/en/plugins/mods/create
- https://code.claude.com/docs/en/plugins/mods/interface
- https://code.claude.com/docs/en/plugins/mods/api
- https://code.claude.com/docs/en/plugins/mods/reference
- https://code.claude.com/docs/en/plugins/mods/test
- https://github.com/anthropics/claude-code/blob/main/mods/types/claude-code.d.ts (reviewed version 2.1.277; not authoritative over installed generated types)
