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
| `accent` | `#00E6CC` | Brand, focus, RUNNING, selected rail |
| `success` | `#39FF9E` | PASS, done good |
| `danger` | `#FF3B4E` | FAIL, errors, kill |
| `warn` | `#FFBA3D` | QUEUED, GATING, UNCONFIGURED |
| `fail_wash` | `#2A1216` | FAIL row background tint |

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
| `NO_COLOR` | Monochrome theme (modifiers only) |
| `KIT_MOTION=off` | No blink/pulse; theme may still color unless NO_COLOR |
| `KIT_THEME=high` | High-contrast palette |

---

## Explicit non-goals (1.0 surface craft)

- Nerd Font icons as required glyphs
- Purple gradients / glassmorphism / marketing dashboard chrome
- Progress % columns in Control Room
- Animated mascot (ASCII mark optional later)
