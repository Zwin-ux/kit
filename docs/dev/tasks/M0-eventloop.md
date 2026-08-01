# M0 · Event loop and frame clock

**Owner:** Grok Build · **Crate:** `kit-tui` · **Branch:** `m0-eventloop`

Implement the single event loop that drives Kit's Control Room. This is the
architectural spine — every screen lands on top of it, so it is built first and
alone.

## Why this task exists

Kit 0.1's motion reads as jitter for a structural reason. Eight components
(`Motion`, `MascotPlayer`, `CountUp`, `StaggerLines`, `TypeLine`,
`useIntervalFrame`, `ActionFlash`, `FadeSteps`) each owned a private
`setInterval` and called `setState` independently. Nothing shared a beat, and
every tick re-rendered a 3,192-line React tree.

1.0 has exactly one clock. Fixing this is the whole point of the rewrite.

## Reference implementation

`C:/Users/mzwin/Documents/fennec/crates/fennec-tui/src/app.rs`

That is a working ratatui + crossterm + tokio loop with a single
`AppEvent::AnimationTick`, written for this same machine and toolchain. Follow
its architecture. Do not invent a different one.

## What to build

1. **`kit-tui/src/loop.rs`** — one `tokio::select!` merging exactly three sources:
   - `crossterm::event::EventStream`
   - one `tokio::time::interval(TICK_INTERVAL)`
   - an `mpsc::Receiver<(RunId, RunDelta)>` for run updates

   Each arm maps into the frozen `AppEvent` enum. No other source of events exists.

2. **`kit-tui/src/app.rs`** — application state holding a `Clock`, plus a pure
   reducer `fn update(&mut self, event: AppEvent) -> Action`. Keep the reducer
   pure and synchronous so it is trivially testable without a terminal.

3. **Terminal lifecycle** — alternate screen, raw mode, mouse capture, and a
   panic hook that restores the terminal before unwinding. A panic must never
   leave the user's shell in raw mode.

4. **Redraw policy** — redraw on `is_redraw_worthy()` events. On
   `AnimationTick`, redraw only when motion is enabled *and* something animated
   is actually on screen. An idle Kit must sit under 1% CPU.

5. **A placeholder Control Room frame** so there is something to snapshot —
   the header, an empty run table with column headers, and the footer key hints
   from `docs/dev/PRD-1.0.md` section 4.2. Real rows come later.

## Hard boundaries

- **Never edit** `crates/kit-core`, `crates/kit-gate`, `crates/kit-cli`
- **Never edit** `crates/kit-tui/src/event.rs` — it is a frozen contract. If you
  need it changed, stop and say so in your final message rather than editing it.
- **No timer anywhere outside the event loop.** Animation asks `Clock` for its
  phase. This is the one rule that must not bend.
- Hit-testing must read the same `ratatui` rects the renderer drew. Do not build
  a parallel geometry module — that mirroring bug is what broke mouse input in 0.1.

## Acceptance

- `cargo test --workspace` passes, including new `insta` snapshots of the frame
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo fmt --all --check` clean
- A deterministic headless test drives a scripted sequence of `AppEvent`s
  through the reducer and asserts the resulting state — no terminal required
- A test proves `AnimationTick` alone does not mark the frame dirty when motion
  is disabled
- Green on ubuntu, macos, **and windows** in CI

## When you finish

Open a PR against `main`. Do not merge it — Claude reviews every crate-boundary
change. In your final message, list anything you wanted to change in a contract
file and could not.
