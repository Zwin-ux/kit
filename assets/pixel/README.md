# Pixel Assets – Kit Mascot

## Final Locked Style (Alpha 1)
- Pure black silhouette only
- Cute anime / cartoon young fox
- Laying down / curled resting pose
- No wrench
- No gray, no anti-aliasing, no internal detail
- High contrast for terminal
- Optimized for 16×16 and 32×32 display (masters may be higher-res)

## Animation: 6-Frame Tail Wag

Body stays completely locked. Only the tail moves.

```
assets/pixel/
├── kit-frame-1.png   # Rest (master)
├── kit-frame-2.png   # Tail rising
├── kit-frame-3.png   # Tail high peak
├── kit-frame-4.png   # Tail coming down
├── kit-frame-5.png   # Tail lower
├── kit-frame-6.png   # Tail near rest
└── kit-idle.gif      # Looping social / README preview
```

## Rules for Grok Build (TUI)
- Load the six frames from this folder
- Cycle slowly for splash and idle (≈ 5–6 fps, ~180 ms)
- Scale cleanly with nearest-neighbor for TUI height (~24 px)
- Do not add color or effects in code
- Missing files fall back to a built-in laying-down silhouette

## GitHub GIF
`kit-idle.gif` is the main visual advertisement for Alpha 1.

Recommended settings (already applied when regenerating):
- Size: 256×256
- Frame delay: 180 ms
- Loop forever
- Pure black on white background

## Current Status
- [x] Master pose locked: laying-down cartoony fox, no wrench
- [x] Six PNG frames committed
- [x] `kit-idle.gif` generated
- [x] TUI wired for 6-frame cycle with downscale
