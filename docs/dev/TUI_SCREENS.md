# TUI Screens

## Goal
Define every main screen.
Keep navigation simple.
Use the pixel-art style on all screens.

## Main Screens

### 1. Splash
Show the Kit mascot.
Show the name "Kit".
Show a short tagline.
Move to Home after a short time or on key press.

### 2. Home
Show:
- Installed skills (recent)
- Quick actions
- Status of account (logged in or not)
- Short tips

### 3. Explore
Show a list of skills from the registry.
Support search.
Support filters by tag or agent compatibility.
Allow install with one key.

### 4. Workshop
Create a new skill.
Edit an existing skill.
Validate the skill.
Run a test on the skill.
Publish the skill.

### 5. Library
Show all local skills.
Show version of each skill.
Allow update, remove, and open in Workshop.

### 6. Profile
Show user name.
Show published skills.
Show followers and following (later).
Allow logout.

### 7. Settings
Change theme options.
Change default agent paths.
Manage account.
View version of Kit.

## Navigation Rules
- Use clear keyboard shortcuts.
- Show the current screen name.
- Always allow return to Home.
- Keep important actions on one key.

## Visual Rules
- Use the silhouette pixel style.
- Keep high contrast.
- Show the mascot on Splash and empty states.
- Use simple status icons.

## Implementation notes (v0 shell)

Start the TUI:

```sh
kit tui
# or
pnpm --filter @kit-skills/tui start
```

First-run (empty library):
```sh
kit init --pack essentials
# or open the TUI and press 1 / 2 / 3 after Splash
```

Mascot frames:
- **idle** — `kit-frame-1..6.png` (tail wag) · default menus + splash
- **scan** — `kit-scan-1..4.png` (ear tilt) · load / install / doctor run / explore
- **success** — `kit-success-1..4.png` (bob) · healthy doctor, install done, link wrote
- Pure black silhouette; no wrench; NN downscale for terminal
- Missing PNGs → built-in placeholders per variant
- ~140–180 ms/frame; `KIT_REDUCED_MOTION=1` freezes frame 0

### Selection stability (P0)

↑↓ must **never** change frame geometry — only which row is marked.

| Rule | Implementation |
|------|----------------|
| List rows | 1 line each; ASCII mini glyph; no animate timers |
| Cursor | Always 2 cells: `  ` / `> ` / `^ ` / `v ` |
| Detail panel | Always fixed lines (`fixedLines`, never `wrap="wrap"`) |
| Action hint | Always 1 truncated line |
| Tests | `tests/selection-stability.test.ts` |

QA: hold ↓ through Home/Packs/Library — list must not jump.

### Fixed mascot slot (no layout thrash)

Animation may change **pixels only**. Rail width/height and line count never change on frame tick.

```
glyph (2-col cursor) < pack detail (≤4, often off) < fixed rail << content
```

| Mode | Rail slot (cols × rows) | Pack detail | Notes |
|------|-------------------------|-------------|-------|
| narrow | 10 × 9 | off | short terminals |
| normal 80×24 | **12 × 10** | off until `rows ≥ 28` | default product |
| tall (`rows ≥ 32`) | **14 × 12** | ≤4 | more air for fox |
| wide | same as tall/normal art | same | more `listMaxItems` |

- `MascotPlayer` always emits `padSlotLines` → exactly `railRows` lines of length `railCols`.
- `ActionFlash` always reserves 1 line (never `null`).
- ↑↓: fixed-width cursor (`↑ `/`↓ `/`› `) so list text never jiggles.
- Rail frame delay ≥ 210ms (calmer full-screen paint).
- `KIT_REDUCED_MOTION=1` freezes mascot + cursor pulse.

### kit-idle (+ variants) in the TUI

Terminals cannot play GIF files inside Ink on all platforms.
Kit plays pixel frames via `MascotPlayer` (same language as `kit-idle.gif`).

- Splash: capped **hero** idle loop
- Busy work: compact **scan** loop
- Success moments: compact **success** loop
- Otherwise: compact **idle** on all main screens (fixed rail via ScreenShell)

Keys:
- Splash: any key → First-run (if needed) or Home · `q` quit
- First-run: `1` essentials · `2` web · `3` library · `4` cli · `5` api · `s` skip · `q` quit
- Home: `↑↓` · `↵`/`i` install · `a` apply · `k` paths · `d` doctor · `e` explore · `l` library · `p` packs · `q` quit
- Packs: filter by typing · `★ recommended` · progress on install · stack packs show `+essentials`
- Explore: remote catalog · `/` search · `↵` install · `r` refresh
- Library: `↑↓` · `v` validate · `t` test · `r` remove · `k` paths
- Doctor: `r` re-run health checks
- Paths: `↑↓` harness · `↵` link write · `p` plan · `r` refresh

Motion (restrained — explain or reward, never decorate alone):
- **Mascot variants**: idle / scan / success by screen state
- **StatusIcon**: ok · fail · warn · skill · pack · link · agent · spinner (list + doctor)
- **PackIcon**: list = mini glyph; selected detail = ≤4×4 (or hidden if short)
- **Enter (↵)** installs the selected toolkit on Home, Packs, Explore (`i` still works)
- **SelectPulse** (`›`→`»`) on ↑↓ selection change
- **ActionFlash** (`▸ …`) on every meaningful key (nav, install, link, validate, test)
- **TypeLine**: splash tagline once; success messages
- **BlinkCursor**: after splash typewriter; packs filter while typing
- **Spinner** (braille or icon) / **ProgressBar**: load and install
- **SuccessLine**: types once after install, then holds
- **CountUp**: skill count flash after install/apply; doctor pass tally
- **ErrorLine**: brief `!` pulse, then static red
- **StaggerLines**: first-run pack options; empty library tips
- **FadeSteps**: header screen name on change
- **Reduced motion**: `KIT_REDUCED_MOTION=1` → final frames / static icons
- Primitives: `packages/tui/src/motion/` + `mascot/statusIcons.ts`
