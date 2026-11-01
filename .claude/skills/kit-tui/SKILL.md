---
name: kit-tui
description: Build or change Kit's terminal UI (ratatui + crossterm + tokio in crates/kit-tui): screens, panes, keybindings, themes, motion and insta snapshot tests. Use for any change under crates/kit-tui or anything that draws to the terminal.
---

# Kit TUI

Binding rules: [`docs/dev/DESIGN-tui.md`](../../../docs/dev/DESIGN-tui.md) (palette, floors, motion) and
[`docs/QUALITY.md`](../../../docs/QUALITY.md). Product shape: [`docs/PRD.md`](../../../docs/PRD.md) §7.

## How the code is laid out
- `src/loop.rs`: the only event loop. One `tokio::select!` over terminal input, engine deltas and one
  clock (`AppEvent::AnimationTick`, 20 Hz). **No other timers, threads or sleeps anywhere in the UI.**
- `src/app.rs`: the reducer. `App::update(AppEvent) -> Action`. Navigation changes `Screen`; engine work
  is returned as an `Action` (`DispatchSubmitted`, `KillSelected`, …), never done inside the reducer.
- `src/ui/*.rs`: pure drawing from `&App` into a `Frame`. No state changes while drawing.
- `src/theme.rs`: every colour comes from a `Theme` token (`kit`, `kit_light`, `monochrome`,
  `high_contrast`, `ansi16`). Never write an RGB value in a screen file.
- `src/fox.rs`: the mascot. Motion only on the shared clock, and off under `NO_COLOR` / `KIT_MOTION=off`.

## Rules that keep it good
- Every state is a word as well as a colour (`PASS`, `FAIL`, `RUN`, `GATING`, `UNCONFIGURED`).
- Geometry is fixed under ↑↓: selection never moves columns or the header counts.
- One footer grammar on every screen (`[key] verb`); `?` opens help; `Esc` goes back; `q` quits only from
  the Control Room.
- Floors: 80×24 is the reference, 80×14 must show every help key, below 60×12 draw the "too small" screen.
- Idle costs nothing: redraw only when state changed or a motion frame is due. Budget: idle CPU < 1%.
- Hit-testing (mouse) uses the same `Rect`s the renderer drew; compute layout once per frame.
- No stub copy on any screen: no "coming soon", no unreachable key in a footer or help panel.

## Tests (required for every UI change)
1. Reducer test in `app.rs` for the key or event.
2. Snapshot in `ui/mod.rs` with `render_to_string(&app, w, h)` at 80 columns and at the narrow floor;
   add a monochrome or `KIT_THEME=high` case when a style changed.
3. `INSTA_UPDATE=always cargo test -p kitctl-tui` (PowerShell: `$env:INSTA_UPDATE = 'always'`), then read
   every changed `.snap` line by line before committing it. A snapshot you didn't read is not a test.
4. `cargo clippy -p kitctl-tui --all-targets -- -D warnings`.

For screenshots in a PR, paste the relevant `.snap` frame in a code block; for motion, record with
`scripts/record-media.sh` (VHS) from a release build.
