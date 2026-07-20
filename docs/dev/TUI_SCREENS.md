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

Mascot frames (Alpha 1):
- Prefer `assets/pixel/kit-frame-1.png` … `kit-frame-6.png` (tail wag)
- Pure black silhouette; laying-down fox; no wrench
- High-res masters are downscaled for the terminal
- If PNGs are missing, the TUI uses a built-in 6-frame placeholder
- Cycle delay is about 180 ms per frame (~5–6 fps)
- Animate mascot on Splash and First-run only (Home stays calm)

### kit-idle in the TUI

Terminals cannot play GIF files inside Ink on all platforms.
Kit plays the **same six frames** as `assets/pixel/kit-idle.gif` via `MascotPlayer`.

- Splash: full live kit-idle loop
- First-run / Packs / empty Library: compact live loop
- Home: compact loop only when the library is empty

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
- **kit-idle** mascot: always playing compact on main menus; full on Splash
- **Enter (↵)** installs the selected toolkit on Home, Packs, Explore (`i` still works)
- **SelectPulse** (`›`→`»`) on ↑↓ selection change
- **ActionFlash** (`▸ …`) on every meaningful key (nav, install, link, validate, test)
- **TypeLine**: splash tagline once; success messages; empty-library tip
- **BlinkCursor**: after splash typewriter; packs filter while typing
- **Spinner / ProgressBar**: load and install (calm, no competing mascot on Home during busy)
- **SuccessLine**: types once after install, then holds
- **CountUp**: skill count flash after install/apply
- **ErrorLine**: brief `!` pulse, then static red
- **StaggerLines**: first-run pack options once
- **FadeSteps**: header screen name on change
- **Never animate**: pack/skill list rows, footer keys, doctor dumps
- **Reduced motion**: `KIT_REDUCED_MOTION=1` → final frames immediately
- Primitives live in `packages/tui/src/motion/`; re-exported from `components/Motion.tsx`
