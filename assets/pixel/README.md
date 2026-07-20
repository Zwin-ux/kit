# Pixel Assets – Kit Mascot

## Style
- Pure black silhouette
- White or transparent background
- No gray, no anti-aliasing, no internal detail
- Optimized for 16×16 and 32×32 TUI display
- High contrast for terminal

## Animation Frames (Idle / Setup)

Use these exact filenames so the TUI can load them easily:

```
assets/pixel/
├── kit-frame-1.png   # Neutral pose
├── kit-frame-2.png   # Tail up + slight head tilt
├── kit-frame-3.png   # Small shift
└── kit-frame-4.png   # Return / ready pose
```

## Rules for Grok Build
- Load the four frames from this folder
- Cycle them slowly for splash and loading states
- Keep the animation simple (4–6 frames per second)
- Scale cleanly to 16×16 and 32×32
- Do not add color or effects in code

## Current Status
Master direction is locked: pure black silhouette only.
Once the four PNG files exist with the exact names above, the TUI can wire them immediately.
