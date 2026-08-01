# Changelog

## Unreleased

### TUI brand pass — ink console (not Clack)
- Vendored TUI skills: `skills/tui-design-skill` (gfargo) + `skills/terminal-ui` (pproenca) for agents.
- **Real Kit identity** from `PIXEL_ART.md` + ads: fox-orange `#C45C2A`, black ink, command STE.
- Theme tokens: `packages/tui/src/theme.ts` (orange accent, OK/`→` marks).
- **Setup** matches `ad-ready` / `readme-terminal`: `KIT SETUP --DIR`, tabular PROJECT/DIR/AGENTS, plan rows, orange CTA.
- **Workbench / Chrome / Flash / FirstRun** share the same mark language; drop size-debug chrome.

### Setup wizard is the product (not the multi-lane hub)
- **`kit tui` opens SETUP** — one screen: project, recommended pack, checklist, Enter/y.
- Flow: Enter plan → y install+link → done. **a** = advanced workbench only.
- Skill for agents: `skills/kit-tui-setup-flow/SKILL.md`.
- Multi-lane menu is power-user path: `kit tui workbench`.

### UI remake — clicks that work
- **Full-width list UI** (no dual-column sidebar) so pack rows are easy to hit.
- **Shared geometry** for render + mouse (`workbenchGeometry`) — hit rows match the screen.
- **Absolute pack indices** in hit map (windowed lists no longer break clicks).
- **Hide OLLAMA LIVE** chrome noise.
- Mouse: prependListener + debounce; tabs labeled PACKS / AGENTS / TOOLS / SETUP.

### Developer quickstart + press feedback
- **Critique** of the product for agentic-dev setup: `docs/dev/DEVELOPER_CRITIQUE.md`.
- **Situation-first boot** — empty / new-repo / chaos open **Ops** with the right next verb.
- **Quickstart** label on Ready (install · apply · link).
- **Every control press** → inverse flash + LOG line (`feedback()`).
- Menu buttons show a short ★ pressed state.
- ActionFlash is stronger and longer so effects are obvious.

### Council usefulness cut
- **Job proof vault** — every finished job/service saves under `~/.kit/runs/` (log + meta).
- **Agents · H History** — reopen a saved proof into the LOG.
- **Ops write in-terminal** — Ready/Unify plan then `y` write (no Home bounce).
- Decision record: `docs/dev/COUNCIL_USEFULNESS.md`.

### Game main menu (click + assets)
- **One icon asset per option** — skills, agents, services, ops, ready, ollama, run, install, …
- **Clickable main menu** — click lane tabs, list rows, and action buttons (Install / Run / Start Ollama).
- **Double-click a selected row** to run that option.
- Lowkey selected-row pulse (not a full-screen fox GIF).
- Copy follows clear, short STE-style instructions.

### Open like Claude / Grok (boot)
- **`kit tui` opens the Action Terminal immediately** — no splash GIF, no typewriter, no fox theater.
- Loading is a static `KIT` + “Starting…” strip only.
- Mascot art is **opt-in** (`KIT_SHOW_MASCOT` / `KIT_MASCOT_ANIM`); default is content-only.
- Optional nostalgia: `KIT_TUI_SPLASH=1`. Classic home: `kit tui home`.
- First-run still offers pack pick when needed, then lands in the terminal.

### Action Terminal (multi-purpose)
- **Four lanes** — Skills · Agents · Services · Ops (not coding-only).
- **Skills** — install/apply packs into the shared action log.
- **Agents** — Codex / Claude / Grok / Ollama jobs with inspect·build.
- **Services** — plugin tasks (e.g. Trenchwire market/health; no wallet).
- **Ops** — Ready plan, Unify plan, Doctor, Paths, Refresh.
- **Local Ollama lifecycle** — `o` start serve, `O` stop kit-managed, `p` pull model.
- Core: `probeOllamaService`, `startOllamaServe`, `stopOllamaServe`, `pullOllamaModel`.
- Dense Codex/T3-class chrome: lane tabs, OLLAMA chip, shared LOG, `>` prompt.

### TUI quality + product surface
- **Home action rail** — situation story + primary keys (`r` Ready, `u` Unify, `w` Terminal).
- **Ready / Unify in TUI** — dry-run plan first, `y` to write (same safety as CLI).
- **Help screen (`?`)** — full keyboard map without leaving the product.
- **Chrome** — product header (`KIT` badge); hide debug layout meta by default.
- **Workbench log** — `PgUp`/`PgDn` scroll (works while a job is running).
- Splash tagline points at the multi-purpose terminal.

### Workbench / plugins (prior unreleased)
- Add local Ollama model discovery through `GET /api/tags`.
- Run Ollama models through its official Codex bridge with the existing
  inspect/build sandbox.
- Move the TUI into an alternate terminal screen and restore the shell on exit.
- Add compact, standard, and wide Workbench layouts.
- Stream runner output and let `Esc` stop a running job.
- Map the Services lane to its selected read-only task.
- Stream and stop service tasks with the same run controls.
- Add explicit run states and context controls that fit small terminals.
- Keep `Q` as text while a prompt or path field is active.
- Upgrade the Hono Node adapter to the patched 2.0.12 release.
- Add a local CLI plugin contract.
- Keep plugin add and remove in dry-run mode by default.
- Store a SHA-256 manifest digest and block changed manifests.
- Start plugin executables without a shell.
- Add a real Kit and Trenchwire proof capture.

## 0.1.5 — Honest ready & safe writes

### Functions (trust cut)
- **`kit ready --write` only succeeds when complete** — pack install, apply, link, and doctor must pass; incomplete → non-zero exit
- **Unify is opt-in** — never auto-runs on chaos story without `--unify`
- **`kit unify --write --link`** links keepers already in the library (not only new adopts)
- **`--link` requires `--write`** (no silent no-op)
- **Link force is honest** — ready/unify default `force: false`, `mode: symlink` (match `kit link`); pass `--force` to clobber
- **CLI exits 1** on link/import partial failures; ready prints notes
- **Atomic `installSkill`** — stage → validate → rename (no half-deleted live skill)
- **Refuse writes into home/Desktop/Downloads** unless `--force`
- **Publish gate** — `publish.mjs` runs prepare-publish and aborts if any `workspace:*` remains
- **CLI argv** — leading `--` stripped (`pnpm kit -- tui` works)
- **`kit status`** — agent wiring strip (claude/codex/grok)

### TUI
- **Menu-first layout** — stack / split / wide; mascot never steals narrow windows
- **Selection stable** — fixed geometry on ↑↓; ASCII cursor; no list reflow
- **A11y (dark terminals)** — no solid █ pack detail blobs; inverse + sticky `sel` focus; denser Home on small viewports
- **Fluid fullscreen** — rail + content width grow with terminal size (no postage-stamp fox on maximize)
- **Click-to-select** — optional mouse SGR; keyboard still primary

### Catalog
- **`deps-hygiene` skill** promoted from queue (keep-alive)

### Version
- All packages + `KIT_PACKAGE_VERSION` → **0.1.5**

```bash
npm i -g @mzwin/kit
kit ready --write
kit unify --write --link
kit tui
```

## 0.1.4 — Product stories: `kit` home + `kit ready`

### Features
- **`kit` (no args)** — situation-aware home
- **`kit ready`** — one-shot recommend → install → apply → link → doctor
- **`kit ready --unify`** — also adopt personal skill keepers

## 0.1.3 — `kit unify` (skill OS)

- Scan Claude/Codex/Grok skill dumps, normalize, dedupe, rank, adopt keepers
- Noise filter default on; `--write --link` for project wire-up
