---
name: kit-tui-setup-flow
description: How Kit TUI should work for developers — single-purpose setup wizard, not a multi-lane hub.
version: 0.2.0
compatibility:
  - claude-code
  - codex
  - grok-build
---

# Kit TUI setup flow (for agents working on Kit)

## Product truth

Kit is a **quickstart installer for agent skills** on a real repo.

Developer job:

```text
cd my-repo
kit tui
→ see recommended pack
→ Enter plan · y write
→ Claude/Codex/Grok can use the skills
```

## Default screen = Setup (not Workbench)

```
KIT · SETUP
Project: my-repo
Recommended: web-app (why…)
What Kit will do:
  · Install pack
  · Apply to project
  · Link Claude · Codex · Grok
  · Doctor

[ Enter ] Setup this project
p pick pack · a advanced · q quit
```

## Rules for UI changes

1. **One primary action** on first paint. Never four equal lanes.
2. **Show the pack and why** before install.
3. **Checklist** maps 1:1 to `runReady` steps.
4. **y writes** after plan; never silent write.
5. **Advanced** (`a`) opens multi-lane workbench for power users.
6. Every key → flash + log line.
7. Mouse hits must use the same geometry as the Setup screen (full-width buttons).

## Visual system (Kit brand — not generic Clack)

Source of truth:

- `docs/dev/PIXEL_ART.md` — black ink, fox-orange `#C45C2A`, warm paper marketing
- `packages/tui/scripts/generate-readme-assets.py` — ACCENT / INK / MUTED
- Marketing frames: `docs/assets/ad-ready.png`, `readme-terminal.png`
- Tokens: `packages/tui/src/theme.ts`

Hard rules for Setup + Workbench:

1. **Ink console language** — `KIT SETUP --DIR …`, tabular fields, `OK PACK INSTALL`.
2. **Fox-orange accent only** (`#C45C2A`) for arrows, rules, CTA heat — never cyan/magenta Clack skin.
3. **Status marks**: `→` recommend, `OK` / `!` / `·` / `>` — never color alone.
4. **Inverse `KIT` mark** + orange rule; no nested boxes; no size-debug chrome (`80x24`).
5. **One primary CTA** reverse-video near the bottom.
6. Footer always shows `p · a · q` (setup) or context keys (workbench).
7. Geometry for mouse hits stays fixed — do not add rows above list without updating `workbenchGeometry`.

## Anti-patterns (do not ship)

- Game hub as default
- Ollama status as hero chrome
- Pack gallery without “why this pack”
- Asking users to learn lanes before setup works
- Charm/Clack cyan rails, magenta brand swaps, emoji status spam
- Nested boxes-inside-boxes / DOS double borders
- Color-only status without a letter/mark

## Code map

- Default UI: `packages/tui/src/screens/Setup.tsx`
- Engine: `packages/core/src/product/ready.ts` (`runReady`)
- Boot: `packages/tui/src/App.tsx` → screen `"setup"`
- Advanced: `packages/tui/src/screens/Workbench.tsx`
