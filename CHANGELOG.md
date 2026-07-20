# Changelog

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
