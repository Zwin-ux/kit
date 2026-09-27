# Kit TUI design system (1.0)

**Status:** Active for the surface remake.  
**North star:** concept art in `docs/dev/assets/` + fennec-tui craft.  
**Companion:** `docs/dev/SPEC-surface-1.0.md`

---

## Concept art

| File | Intent |
|------|--------|
| `assets/concept-control-room.jpg` | Dense ops table, FAIL row wash, cyan focus rail, stats header |
| `assets/concept-design-system.jpg` | Palette tokens, Swiss industrial + hacker ops |
| `assets/concept-run-detail.jpg` | Breadcrumb, tabs, stream + diff, fatal line |

Concept art is **mood and density**, not a pixel contract. Column set follows the PRD, not invented progress bars.

---

## Palette

Semantic tokens (truecolor default). Map to ANSI16 when truecolor unavailable; drop to modifiers-only when `NO_COLOR` is set.

| Token | Hex | Use |
|-------|-----|-----|
| `bg` | `#0B0E12` | Base background (usually terminal default) |
| `fg` | `#F0F1E3` | Primary text |
| `muted` | `#6B7280` | Footers, metadata, inactive chrome |
| `accent` | `#FF5A1F` | Fox red: brand, focus, RUNNING, selected rail, the fox |
| `success` | `#39FF9E` | PASS, done good |
| `danger` | `#FF2D6F` | FAIL, errors, kill (a pinker red, so it never reads as the accent) |
| `warn` | `#FFBA3D` | QUEUED, GATING, UNCONFIGURED |
| `fail_wash` | `#2A1216` | FAIL row background tint |

### Agent marks

Every list of agents shows each agent the same way (`kit-tui/src/brand.rs`): a two-cell mark in the brand colour, then the id in tables or the product name in prose and prompts. One order everywhere: claude, codex, grok, ollama.

| Agent | Mark | Colour | ANSI16 |
|-------|------|--------|--------|
| Claude Code | `✻` | `#D97757` | red |
| Codex | `>_` | `#7A9DFF` | light blue |
| Grok | `⊘` | `fg` | white |
| Ollama | `◉` | `#C8C8C8` | gray |

In line output (`kit setup`, `kit doctor`) each agent is a three-cell tile: the mark on the brand's own background, like a small app icon (`Brand::tile`, Codex on its violet-to-blue gradient). Any terminal with colour draws it. `kit setup`'s agent picker draws the real logo (`kit-tui/assets/logos/`) in the mark's cells where the terminal speaks the kitty or iTerm2 image protocol, and the text mark everywhere else. Under `NO_COLOR` the mark keeps its glyph and loses its colour. A past run's mark is muted with the rest of its row, and tables under 13 cells for AGENT drop the mark and keep the name.

ANSI16 maps the accent to red and FAIL to bright red.

### The fox in line output

`kit setup` opens with the fox head (`fox::HEAD`, 16×12 pixels drawn in 16×6 half blocks: fox red, cream, ink) with the name and promise beside it, and signs off with the same head beside "You're set". Under 66 columns, piped, or without colour it falls back to text.

The CLI's line output (`setup`, `add`, `doctor`) uses the same tokens through `kit-tui/src/ansi.rs`, only when stdout is a terminal: piped output and `--json` stay plain.

**Never color alone.** Always pair with words: `PASS` / `FAIL` / `RUN` / `UNCONFIGURED`.

---

## Typography (monospace hierarchy)

| Role | Treatment |
|------|-----------|
| Title / brand | Bold + accent |
| Column headers | Bold + muted or bold |
| Primary cell text | Default fg |
| Secondary / elapsed | Muted |
| Selection | Reverse video **or** accent left rail + bold (both monochrome-safe) |
| FAIL annotation | Danger + dim prefix `^ ` |

---

## Density

- **Pack** Control Room and Board (ops scanning).
- **Pad** Dispatch form fields (decision making).
- One border around the primary table — no boxes-inside-boxes.
- Footer always one line of hints; full help behind `?`.

---

## Responsive floors

| Width | Behavior |
|-------|----------|
| ≥ 100 | Optional run-detail split (stream \| diff) |
| 80–99 | Full table; standard snapshots |
| 60–79 | Narrow columns, more truncation |
| < 60 or < 12 rows | "terminal too small — need 60×12" |

---

## Reduced motion / color

| Env | Effect |
|-----|--------|
| `NO_COLOR` | Monochrome theme (modifiers only); motion also off |
| `KIT_MOTION=off` | No spinner; RUNNING stays `RUN 2m` (resting frame) |
| `KIT_THEME=high` | High-contrast palette |
| `KIT_LOGOS=off` | Text marks instead of logo images in `kit setup`; `KIT_LOGOS=kitty` or `iterm` forces a protocol |

## Motion (F5)

Kit 0.1 jittered because eight widgets each owned a timer. 1.0 has one clock (`AppEvent::AnimationTick`, 20 Hz). Motion is a **product signal**, not decoration:

- **What moves:** RUNNING and GATING rows only — a one-cell Braille spinner (`⠋⠙⠹⠸⠼⠴⠦⠧`) plus the elapsed word. FAIL/DONE rows stay still; an idle room moves only the fox's tail. `--demo` includes one GATING row so motion means “checking done,” not “session manager.”
- **Cadence:** spinner frame every 2 ticks (10 Hz). Redraw only then, or while a flash is live. An idle Control Room paints only when the fox's tail frame changes.
- **Reduced motion:** `KIT_MOTION=off` or `NO_COLOR` → no spinner, no dirty-on-tick for live rows. The word `RUN` still carries the state (never color-alone).
- **The fox:** the empty Control Room shows Kit's fox (the 0.1 mascot, `kit-tui/src/fox.rs`), 18×12 half blocks in the muted colour, centred above the message and hint. Its tail flicks once every 8 s (5 frames × 200 ms) off the same clock, and the room redraws only on those 6 frame changes. It rests under `KIT_MOTION=off` / `NO_COLOR`, hides when the terminal is under 19 rows, and never sits over the `No coding agents on PATH` error. `kit setup` prints the resting fox after its last line, in a terminal only.
- **Not motion:** mascot GIF, blink, progress % columns, Nerd Font glyphs.

The spinner is Unicode Braille, not a Nerd Font. Pair it with `RUN` / `GATING` so monochrome and CVD still read.

---

## Explicit non-goals (1.0 surface craft)

- Nerd Font icons as required glyphs
- Purple gradients / glassmorphism / marketing dashboard chrome
- Progress % columns in Control Room
- An animated mascot anywhere but the empty Control Room
