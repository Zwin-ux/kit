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
├── kit-frame-1.png   # Neutral pose (master)
├── kit-frame-2.png   # Tail up + slight head tilt
├── kit-frame-3.png   # Small shift
└── kit-frame-4.png   # Return / ready pose
```

## Rules for Grok Build (TUI)
- Load the four frames from this folder
- Cycle them slowly for splash and loading states
- Keep the animation simple (4–6 frames per second)
- Scale cleanly to 16×16 and 32×32
- Do not add color or effects in code

## GitHub GIF
The same four frames must also be usable to create a small animated GIF for the README and GitHub social preview.

Recommended GIF settings:
- Size: 128×128 or 256×256
- Frame delay: 180–250 ms
- Loop forever
- Keep pure black on transparent or white background
- No extra effects

Suggested output filename:
```
assets/pixel/kit-idle.gif
```

Once `kit-idle.gif` exists, it can be embedded directly in the README.

## Current Status
Master direction is locked: pure black silhouette only.
Frame 1 (master) is defined.
Frames 2–4 are described as small controlled changes from the master.
Once the four PNGs exist, both the TUI and the GitHub GIF can be created from the same source.

## TUI wiring
The TUI loads these exact names from this folder.
Missing files fall back to a built-in 16×16 black silhouette (4 frames).
Run: `kit tui`
