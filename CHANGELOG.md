# Changelog

## 1.0.0-alpha.1 — Control Room

Not 1.0.0. npm `@mzwin/kit` is still the 0.1 workbench. This is the Rust Control Room: dispatch, gate, receipt.

### Added
- Control Room TUI (dispatch, FAIL wash + first-error line, kill/retry, filter, help)
- Isolated git worktrees and immutable receipts under `~/.kit/runs/`
- `kit.toml` on this repo (fmt + clippy + `cargo test --workspace`, 15m)
- `kit doctor`, `kit run`, `kit receipt list/show`
- Repo shims (`kit.cmd` / `kit.ps1`) so this checkout launches the Rust binary

### Fixed
- `kit --demo` opens the Control Room (was `unknown command: --demo`)
- Control Room FAIL annotation is full-width so `^ tsc: 3 errors` stays readable at 60×12
- Dispatch footer includes `↑↓`; one-repo layouts no longer leave a vacant left well
- Run detail / attach headers use `vendor·role` (`codex·eng`), matching Control Room and Board
- RUNNING/GATING rows breathe with a Braille spinner on the one animation clock; `KIT_MOTION=off` stays still

### Changed
- README is Control Room first. 0.1 npm workbench is documented as legacy.

### Not yet
- Clean-machine installer, PTY attach, live Session B dogfood, tagging 1.0.0

## Unreleased (0.1 workbench)

- Add `completeness-qa` to essentials: inventory public functions, flag stubs, and name the next SWE skill. Live runs overlay catalog skills onto `.agents/skills` and require completeness-qa before claiming done.
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
