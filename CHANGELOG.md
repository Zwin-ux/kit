# Changelog

All notable changes to Kit. Versions follow [Semantic Versioning](https://semver.org/): the public API is the `kit` command, `KIT.toml` / `kit.toml`, `kit.lock`, `--json` output and receipts. The Rust library crates carry no semver promise of their own.

## Unreleased

Kit 2.0.0 is the first release of the Rust `kit` as the default install everywhere. It sets your coding agents up for one job, then proves what they do. (0.1.x was the Node workbench; 1.0.0-alpha.1 was an npm-only preview of the Control Room.)

### Highlights

- **Kits.** A kit is a bundle for one job: skills, rules, MCP servers and hooks, pinned to exact versions. `kit setup` asks which agents (Claude Code, Codex, Grok) and which focus, shows everything it will install, and installs only after you say yes. Starter kits: Essentials, Frontend Design, Full-stack Design, Backend Engineer and LLM Engineer.
- **Install, change your mind, undo.** `kit add`, `kit remove` and `kit list` install and remove kits exactly as added; hand edits are kept and reported. A failed install rolls back. Kit refuses to overwrite files it did not write.
- **Share a setup.** `kit.lock` records what a repo uses; `kit sync` installs it for a teammate, a new machine or CI through the same plan and yes, and `kit sync --check` fails CI when a machine drifts. `kit search` finds kits, `kit add github:owner/repo` installs a kit from GitHub at a pinned commit, and `kit new` starts your own.
- **Proof.** `kit run` runs one agent in its own git worktree, then your repo's checks from `kit.toml`, and writes a receipt for every ending, Ctrl-C included. `kit land` puts a proven run on a new branch. `kit init` writes the checks for Rust, Go, Node and Python repos.
- **Control Room.** `kit` opens one table of every run: status, the first error line of a failure, and runs waiting in the queue.
- **One-line install** on macOS, Linux and Windows, from npm, or with `cargo install kitctl`. Every archive has a SHA-256 checksum and a GitHub build attestation.

### Upgrading

- From 0.1.x (npm `@mzwin/kit`, the Node workbench): `npm install -g @mzwin/kit` now installs the Rust binary. Your 0.1 skills stay where they are; `kit doctor` points out the old app if it is still on `PATH`.
- From 1.0.0-alpha.1: run `npm install -g @mzwin/kit@latest` (the `alpha` tag stays on alpha.1), or rerun the installer. Receipts under `~/.kit/runs/` are kept.
- `cargo install kit-cli` installs someone else's project. The crate is `kitctl`; the command is still `kit`.

### Breaking changes

- `kit doctor --json` no longer has the `skillsPack` field.
- `KIT_SKILLS_DIR` is removed. `kit run` no longer copies skill packs or writes `AGENTS.md` into the run's worktree; the agent uses the skills your kits installed.
- A run sees only committed files. If a kit is installed into the repo (not `--global`) and the agent should use it, commit the kit's files before `kit run`.
- A run with no checks (no `kit.toml` and nothing Kit can infer) is recorded with state `unconfigured` and `gatePassed: false` in its receipt, where it used to say `pass`; old receipts read the same way. The Control Room shows `GATE UNCONFIGURED`, and `kit land` refuses such a run without `--force`.

### Details

#### Added
- `kit land <id>` commits a passed run's changes on a new branch `kit/<id>` at the run's base commit. It does not switch your branch or change your files. `--apply` edits the working tree instead (clean tree only), `--branch` names the branch, `--json` returns the `land` envelope. It refuses fail/error/killed runs, UNCONFIGURED gates and empty diffs; `--force` overrides with a warning and a note in the commit. A second land says `already landed` (trailer `Kit-Receipt: <id>`). The kept run worktree goes once it is landed
- `kit run` and `kit receipt show` print `Next: kit land <id>` after a proven PASS with changes
- The run dir has `base.txt`: the commit the run started from
- `kit init` proposes a gate and writes `kit.toml`. It detects Rust, Go, Node (package manager from `packageManager` or the lockfile) and Python (ruff, black, mypy, pytest only when configured). It uses only scripts that exist and do not write: a `format` script that runs `prettier --write` is not the format check. `--print` only prints, `--force` replaces, `--check` runs each command once and keeps the ones that pass, `--json` returns the `init` envelope. Mixed repos use the root toolchain and name the rest
- `kit run` (on stderr) and `kit doctor` point to `kit init` when the repo has no `kit.toml`; `kit doctor --json` has `kitToml`
- npm: `@mzwin/kit` is a launcher plus one binary package per platform (Windows x64, macOS arm64/x64, Linux x64/arm64 glibc 2.17+). Prereleases publish to their own tag (`alpha`, `beta`, `rc`); releases move `latest`
- `scripts/npm-smoke.mjs` installs the packed tarballs and runs `kit` through the npm shim; CI runs it on Linux, macOS and Windows
- `kit doctor` shows how kit was installed and warns only about the old 0.1 Node app, not its own npm shim
- GitHub Release per tag: `kit-<version>-<target>.tar.gz` / `.zip` and `SHA256SUMS`
- `scripts/install.sh` (Linux glibc, macOS) and `scripts/install.ps1` (Windows x64): download from the GitHub Release, stop on a SHA-256 mismatch, install without sudo and without changing `PATH`. CI tests both against a local release (`installers.yml`)
- Release: the launcher is published only after every platform package is visible on the registry; a new job installs the published version on Linux, macOS and Windows and runs it
- The receipt (and so `kit land`) holds only what the agent changed. Files the gate writes, such as coverage or reports, stay out, and the run log says when the gate changed files

#### Changed
- `kit init --check` no longer drops a failing check in silence (the gate could then PASS on lint alone). It stops and names the checks; `--drop-failing` makes that choice explicit
- Agent output that reaches the 8 MiB cap inside a multi-byte character no longer crashes the run, and nothing is appended after the cut
- Live gate inference and `kit init` share one detector. Inference no longer uses a `format` script that writes files or runs `lint` as the typecheck; `lint` is an `extra` check. A pnpm, yarn or bun repo is no longer checked with npm when that tool is missing

#### Fixed
- The receipt diff now holds new files, binary files (`--binary`) and commits the agent made. It used to be `git diff HEAD`: a new file was missing, and an agent that committed its work left an empty diff and a worktree that Kit removed as clean, with the commit in it
- A gate check whose program is not found (or cannot start) now FAILS the gate. It used to be "skipped", which counted as passed: a typo in kit.toml gave a PASS receipt with no check run
- `kit run` from a subdirectory uses the repo root, so it reads the same `kit.toml` the gate runs against
- `kit doctor` says `not ready` (not `missing`) for an installed agent that is logged out or has no model, and `--json` agents gain `installed`
- A run whose agent is not installed stops before any worktree. It used to fall back to a dry run, which could PASS the gate on an unchanged tree
- On Windows, a missing agent no longer shows as ready (`cmd /C` "is not recognized" output was read as a version)
- A `kit.toml` that does not parse stops the run with the file path and the parse error. It used to switch the gate off without a message
- `ollama` is ready only when its server answers and the model is pulled
- `--json` errors print a `{ ok: false, error }` envelope on stdout and exit 2
- `kit` without a terminal stops with a message instead of drawing into a pipe and waiting
- `receipt show` labels a zero-check gate `UNCONFIGURED`, the same as `kit run`
- No `\\?\` prefixes in Windows paths; git worktree chatter is quiet; states print in lowercase; help points to the README, not repo-only docs

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
- `--demo` includes a GATING row; header says `GATING` never `0 GATED`; README first paint is the FAIL still-frame, not the fox
- Selected FAIL keeps wash (not reverse); 60-col run detail keeps `GATE FAIL` + `[r]etry`; demo flash fits the 80-col header; help overlay covers the frame; `[b]oard` stays at 80
- Gate/run children isolate `CARGO_TARGET_DIR` to the worktree (parent cargo artifacts stay honest)
- `KIT_THEME` unchanged; without truecolor, Control Room falls back to ANSI16 (FAIL wash still paints)
- Node CI path-filtered to `packages/**`; 0.1 keep-alive is workflow_dispatch only

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
