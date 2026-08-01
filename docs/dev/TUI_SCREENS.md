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

## Implementation notes

### Open like Claude / Grok / Codex

```sh
kit tui                 # Action Terminal immediately (no splash GIF)
kit tui home            # optional classic home
kit tui terminal        # same as bare kit tui
```

**Default path:** loading chrome (`KIT` + Starting…) → Action Terminal  
(or FirstRun if starter pack not chosen yet).

**Not default:** Splash, typewriter, continuous fox loop.

| Env | Effect |
|-----|--------|
| `KIT_TUI_SPLASH=1` | Optional static splash gate |
| `KIT_SHOW_MASCOT=1` | Show static brand art on menu shells |
| `KIT_MASCOT_ANIM=1` | Allow idle fox loop (implies show) |
| `KIT_NO_MASCOT=1` | Force hide brand art |
| `KIT_REDUCED_MOTION=1` | Freeze residual motion |

### Motion policy

- Allowed: busy spinner, inverse selection, action flash, progress bar
- Forbidden by default: splash hero loop, boot typewriter, animated list icons, rail fox during menus

### First-run

```sh
kit init --pack essentials
# or kit tui → FirstRun → 1–7 install
```

### Mascot frames (opt-in only)

- **idle** — `kit-frame-1..6.png` · only if `KIT_MASCOT_ANIM=1`
- **scan** / **success** — same gate
- Missing PNGs → placeholders; load is **lazy** and never blocks open

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
- Help (`?`): full key map · Esc back
- Home: situation story + action rail · `r` Ready plan · `u` Unify plan · `y` confirm write · `w` workbench · `o` point · `↑↓` · `↵`/`i` install · `a` apply · `k` paths · `d` doctor · `e` explore · `l` library · `p` packs · `q` quit
- Packs: filter by typing · `★ recommended` · progress on install · stack packs show `+essentials`
- Explore: remote catalog · `/` search · `↵` install · `r` refresh
- Library: `↑↓` · `v` validate · `t` test · `r` remove · `k` paths
- Doctor: `r` re-run health checks
- Paths: `↑↓` harness · `↵` link write · `p` plan · `r` refresh
- Action Terminal (`w` or `kit tui terminal`):
  - `1–4` / `Tab` — Skills · Agents · Services · Ops
  - Skills: `Enter` install · `a` apply
  - Agents: `o` start Ollama · `O` stop kit-managed · `p` pull · `e` job · `m` mode · `Enter` run
  - Services: `Enter` run read-only plugin task
  - Ops: `Enter` Ready / Unify / Doctor / Paths / Refresh
  - `PgUp`/`PgDn` scroll shared log · `Esc` stop or home

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
