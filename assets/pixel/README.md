# Pixel Assets – Kit Mascot

## Final Locked Style (Alpha 1)
- Pure black silhouette only
- Cute anime / cartoon young fox
- Laying down / curled resting pose
- No wrench
- No gray, no anti-aliasing, no internal detail
- High contrast for terminal
- Optimized for 16×16 and 32×32

## Animation: 6-Frame Tail Wag

Body stays completely locked. Only the tail moves.

Use these exact filenames:

```
assets/pixel/
├── kit-frame-1.png   # Rest (master)
├── kit-frame-2.png   # Tail rising
├── kit-frame-3.png   # Tail high peak
├── kit-frame-4.png   # Tail coming down
├── kit-frame-5.png   # Tail lower
└── kit-frame-6.png   # Tail near rest
```

## Rules for Grok Build (TUI)
- Load the six frames from this folder
- Cycle them slowly for splash and idle states (≈ 5–6 fps)
- Scale cleanly to 16×16 and 32×32
- Do not add color or effects in code
- Missing files fall back to a simple built-in silhouette

## GitHub GIF (Important for Alpha 1)
The same six frames must be used to create a looping GIF for the README and social preview.

Recommended settings:
- Size: 128×128 or 256×256
- Frame delay: 160–220 ms
- Loop forever
- Pure black on transparent or white background

Output filename:
```
assets/pixel/kit-idle.gif
```

This GIF is the main visual advertisement for the project.

## Current Status
- Master pose locked: laying-down cartoony fox, no wrench
- Animation style locked: simple 6-frame tail wag
- Actual PNG files still need to be added
- Once the six PNGs exist, wire them in the TUI and generate the GIF
