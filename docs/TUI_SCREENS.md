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

Mascot frames:
- Prefer `assets/pixel/kit-frame-1.png` … `kit-frame-4.png`
- Pure black silhouette on white or transparent
- If PNGs are missing, the TUI uses a built-in placeholder silhouette
- Cycle delay is about 220 ms per frame

Keys:
- Splash: any key → Home, `q` → quit
- Home: `s` → Splash, `q` → quit
