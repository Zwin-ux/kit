# Kit visuals

Every image and GIF of Kit in the README and the GitHub Release is a recording of the real `kit` binary, made by [VHS](https://github.com/charmbracelet/vhs) from the tapes in [`tapes/`](tapes). No mockups. Regenerate them all from a release build:

```sh
cargo build --release -p kitctl
scripts/record-media.sh                  # all tapes
scripts/record-media.sh --only run doctor
```

Each tape runs in a throwaway home (`/home/demo`, recreated per tape) with a small git repo at `~/code/shop`. The agents are the ones on your `PATH`, so `kit setup`, `kit doctor` and `kit run` show a real Claude Code (or Codex) run. The one exception is `fleet`: 12 real agent runs would be slow and costly, so it uses [`fakeagent.sh`](fakeagent.sh), a scripted stand-in, and its caption must say so. Inside a Claude Code session a real agent cannot run, so `--fake` records every tape with the stand-in; the 2.0.0 set was made that way, and the README captions the run as scripted. Kit itself, the worktrees, the checks and the receipts are real in every tape.

## Rules

These keep the set looking like one product, and like the product as designed (`crates/kit-tui/src/theme.rs`, `docs/dev/DESIGN-tui.md`).

**Colour.** Kit's palette, in the `Kit` VHS theme in [`tapes/kit-theme.tape`](tapes/kit-theme.tape):

| Token | Hex | Where |
|---|---|---|
| bg | `#0B0E12` | Terminal background (the TUI paints none of its own) |
| fg | `#F0F1E3` | Text |
| muted | `#6B7280` | Footer, column headers, the prompt path |
| accent | `#00E6CC` | Title, focus, selection rail, RUN, setup's cursor |
| success | `#39FF9E` | PASS, setup's `?` and ticks |
| danger | `#FF3B4E` | FAIL, errors |
| warn | `#FFBA3D` | QUEUED, GATING |
| fail wash | `#2A1216` | Background of a failed row |

`COLORTERM=truecolor` is set in the theme tape. VHS leaves it empty, and without it Kit falls back to 16 colours. Never record with `NO_COLOR` (monochrome) for marketing. No stock theme and no 0.1-era paper-and-orange art. The only mascot is the muted fox from `crates/kit-tui/src/fox.rs`, which Kit itself draws in the empty Control Room and at the end of `kit setup`.

**Type.** DejaVu Sans Mono, 16 px, line height 1.0 (box drawing breaks at larger heights). It has every glyph Kit prints except the Braille spinner, which falls back cleanly with no column shift. Liberation Mono lacks the `▶` selection rail.

**Size.** Pick the terminal size from the content, never from a default:

| Tape | Terminal | Why |
|---|---|---|
| `control-room` | 120×16 | Header keeps its flash and counts; four rows plus the error line fill the table |
| `control-room-empty` | 120×26 | The fox shows only in an empty table at about 19 rows or more |
| `fleet` | 120×18 | 10–12 visible rows plus `↓ N more below` |
| `setup` | 110×44 | The whole plan and prompts; the ending (`undo`, `Try it`) scrolls the top off |
| `add` | 110×32 | Backend Engineer's licence column (`Apache-2.0 AND CC-BY-SA-4.0`) reaches 105 columns |
| `doctor` | 100×32 | Widest line is about 80 columns |
| `run` | 110×32 | Worktree paths are about 75 columns plus the home |

**Timing.** Kit animates on one 20 Hz clock; the spinner turns once every 0.8 s and a flash lasts 2 s. Record at 30 fps. Hold a moving frame at least 1.6 s (two spinner turns) and a still frame (a plan, a receipt) 3–4 s. Type at 45 ms per key. Always sleep 300 ms or more after `Escape`, or the terminal merges it with the next key. Wait on the screen (`Wait+Screen /…/`) for anything that depends on the network or an agent, not a fixed sleep.

**Content.** Show what a person would do, in order, with a realistic repo name and task. Never press `r` or `k` on `--demo` rows before #23 lands (they start real runs there). Keep `kit search` out until the kit index is public.

## What each recording is for

| File | Shows | Used in |
|---|---|---|
| `control-room.gif` / `.png` | Every run in one table; the failed run leads with its first error; detail, gate log, diff | README hero, Release |
| `control-room-empty.gif` / `.png` | Bare `kit` before the first run: the empty Control Room and the fox | README "Prove what agents do" |
| `setup.gif`, `setup-plan.png`, `setup-done.png` | The first five minutes: agents found, three questions, the full plan, yes, done | README "Get started" |
| `add.gif`, `add-plan.png` | The plan as the trust surface: paths, pins, licences, what runs code | README "Kits" |
| `doctor.gif` / `.png` | Agents ready or missing, the repo's gate, installed kits still intact | README "Check" |
| `run.gif` / `.png` | A real agent in its own worktree, the repo's checks, the receipt, `kit land` | README "Proof", Release |
| `fleet.gif` / `.png` | 12 runs (3 agents x 4 roles), 8 at a time, the rest queued (scripted agents) | README "Many agents" |
