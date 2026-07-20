# Changelog

## Unreleased

### Public surface
- Product README only (install → packs → TUI → commands)
- Engineering docs moved under `docs/dev/`
- User pack guide: `docs/packs.md`

## 0.1.0 — Alpha (GitHub v1)

First public alpha of **Kit**: portable agent skills, seven starter packs, pixel TUI.

### Skills engine
- Strict `SKILL.md` parse + validate
- Local library: install / list / remove
- Pack system with `extends` (dependency packs merge skills)
- Cross-harness `paths` + `link` (Claude Code, Codex, Grok Build)
- `test` and `doctor` health checks

### Seven starter packs
essentials · web-app · library · cli-tool · api-service · full-stack · data-ml

Each pack ships a pure black 16×16 silhouette icon.

### TUI
- kit-idle 6-frame mascot (same frames as GIF)
- Splash, first-run (1–7), Home, Packs, Library, Explore, Doctor, Paths
- Point at a project (`o`) → auto-recommend packs + skills
- Enter installs · Paths requires folder approval before write
- Restrained motion: TypeLine, SelectPulse, ActionFlash (`KIT_REDUCED_MOTION=1`)

### CLI & registry
- Full CLI surface + Windows `kit.cmd` / `kit.ps1` shims
- GitHub device-flow `login` / `whoami` / `logout`
- `explore` against Railway public catalog
- `recommend --dir <project>`

### Not in v1 (honest)
- Workshop editor, publish API, social/following, global npm binary
