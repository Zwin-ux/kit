# Pixel Art Direction

## Goal
Create pixel art that works well in a TUI.
The art must stay clear at small sizes.
Use strong silhouettes.

## Mascot
Name: Kit
Animal: Young fox (cute anime / cartoon silhouette)
Pose: Laying down / curled rest (Alpha 1)
Object: None (no wrench in Alpha 1)

Style rules:
- Pure black silhouette only (in TUI)
- High contrast
- Simple shapes
- Clear silhouette
- No complex shading
- No anti-aliasing
- No gray
- Body locked; tail-only animation (6 frames)

## Palette
- TUI: black on white/transparent only
- GitHub / marketing assets: **black ink on warm paper** (`#F7F4ED`) so silhouettes stay visible on GitHub light *and* dark themes
- Accent orange (`#C45C2A`) for rules and highlights only — never fill the fox
- Do **not** ship black-on-transparent PNGs for the README (they vanish on dark mode)

## Regenerating README assets

```bash
python packages/tui/scripts/generate-readme-assets.py
# or: pnpm --filter @kit-skills/tui assets:readme
```

Writes: `docs/assets/readme-banner.png`, `kit-idle.gif`, `kit-flow.gif`, `readme-loop.png`, `readme-terminal.png`, pack tiles, social preview.

## Sizes
Primary sizes:
- 16x16
- 32x32
- 64x64

All icons must stay readable at 16x16.

## Usage
- TUI splash
- Status icons
- Skill cards
- Empty states
- Success states
- GitHub avatar

## Rules for new art
1. Start with the silhouette.
2. Add only necessary details.
3. Test at 16x16.
4. Keep the same outline weight.
5. Do not add gradients.
