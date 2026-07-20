# Changelog

## 0.1.5 — Animated TUI + visual README

### TUI
- **Status icons** — 8×8 pure-black glyphs (ok/fail/warn, skill, pack, link, agents, spinner…)
- **Mascot variants** — idle · scan · success (placeholder + optional PNG masters)
- **Icon spinner** on load/install/doctor/paths
- List rows get type glyphs; Doctor checks get pass/warn/fail icons
- `KIT_REDUCED_MOTION=1` freezes mascot + spinners

### README
- Motion demos: `demo-unify.gif`, `demo-ready.gif`, `demo-link.gif`, `kit-success.gif`
- `packs-strip.png` + ad cards in visual scroll

```bash
kit tui
```

## 0.1.4 — Product stories: `kit` home + `kit ready`

### Why
Vibe coders don’t want a command encyclopedia. They want the next right move for *their* mess.

### Features
- **`kit` (no args)** — situation-aware home: library size, harness skill estimate, picks a user story, prints exact next commands
- **`kit ready`** — one-shot make-this-repo-agent-ready: recommend → install pack → apply → optional unify → link → doctor
- **`kit ready --unify`** — also adopt personal skill keepers while wiring the project
- User stories baked into product routing: chaos-cleanup, new-repo-agents, multi-agent-sync, empty-start, already-solid
- npm + GitHub README rewritten around those stories

### Loop
```bash
npm i -g @mzwin/kit
kit
kit ready --write
kit unify --write --link
```

## 0.1.3 — `kit unify` (skill OS)

### Feature
- **`kit unify`** — scan Claude/Codex/Grok skill dumps, normalize to Kit schema, dedupe, rank, adopt keepers into `~/.kit`
- **Noise filter (default on)** — bulk `*-automation` / thin stubs filtered; not auto-adopted
- **Honest grades** — S/A keepers require structure or multi-agent presence; normalize-only is not free S-tier
- **Safe `--write`** — adopts S/A keepers only (default top 25, min-score 70)
- **`--write --link`** — adopt + broadcast into project harness folders
- **`--all` / `--json`** — power-user escape hatch + machine report

### Product loop
```bash
npm i -g @mzwin/kit
kit unify
kit unify --write
kit unify --write --link
```

## 0.1.2 — npm under @mzwin/*

Published CLI as `@mzwin/kit` (user scope; `@kit-skills` org not available).

## 0.1.0 — Public npm + import + keep-alive

### Install
- Publish `@mzwin/kit` (bin: `kit`) with `@mzwin/kit-core`, `shared`, `tui`, `catalog`
- Bundled packs/skills via `@mzwin/kit-catalog` so global install works without cloning

### Capture / link
- `kit import --from claude-code|codex|grok-build|all` — harness → Kit library (dry-run default)
- Existing `kit link` remains Kit → harness

### Keep-alive
- `catalog/queue/*` curated skill backlog
- `scripts/keep-alive.mjs` + weekly GitHub Action opens PRs (one promotion per run)

### Claude plugin
- `pnpm plugin:claude` builds `dist/claude-plugin` for local Claude Code plugin install

## 0.1.0-alpha — GitHub alpha

First public alpha of **Kit**: portable agent skills, seven starter packs, pixel TUI.

### Skills engine
- Strict `SKILL.md` parse + validate
- Local library: install / list / remove
- Pack system with `extends` (dependency packs merge skills)
- Cross-harness `paths` + `link` (Claude Code, Codex, Grok Build)
- `test` and `doctor` health checks

### Seven starter packs
essentials · web-app · library · cli-tool · api-service · full-stack · data-ml

### TUI
- kit-idle 6-frame mascot
- Splash, first-run (1–7), Home, Packs, Library, Explore, Doctor, Paths
- Point at a project (`o`) → auto-recommend packs + skills

### CLI & registry
- Full CLI surface + Windows `kit.cmd` / `kit.ps1` shims
- GitHub device-flow `login` / `whoami` / `logout`
- `explore` against Railway public catalog
- `recommend --dir <project>`
