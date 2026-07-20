# QA: TUI selection stability (2026-07-20)

## Issue
P0: Down/up selection glitched the whole TUI (list jump / flicker).

## Root causes fixed
1. Library/Explore `wrap="wrap"` under selection (variable row count)
2. SelectPulse double paint (160ms hot phase)
3. Ambiguous Unicode list glyphs + N PackIcon timers
4. Action lines changing height/length without pad

## Changes
- `fixedLines` / ASCII cursor / static pack glyphs
- Library + Explore fixed 2–3 line detail
- ToolkitPicker no list anim; fixed detail lines
- Home/Packs always 1 action line
- Unit tests: selection-stability

## Automated
`pnpm --filter @mzwin/kit-tui test` — selection-stability + mascot + motion

## Manual (user / Windows Terminal)
- [ ] Home hold ↓ all packs — no jump
- [ ] Packs filter + ↓
- [ ] Library long vs short skill desc
- [ ] Explore long desc
- [ ] KIT_REDUCED_MOTION=1 still clear selection
- [ ] Spam ↑↓ 10s

## Verdict
Code-level geometry contract shipped. Manual Windows Terminal verification required by user.
