# Kit 1.0 concepts

These are concepts for the `1.0.0-alpha.1` source tree on 2026-09-23. The SVGs show intended user moments. They do not prove that the npm alpha has been published or that a demo fixture wrote a receipt. See `docs/dev/CURRENT.md` and `docs/dev/RELEASING.md` for the release state.

## Design concept: the first five minutes

Use one visual language: terminal black `#0B0E12`, clear type, one `#00E6CC` focus color, and `#FF3B4E` only for failure. Put the action beside its result. Pair every color with a word such as `FAIL`. Keep the Control Room table readable at 80 by 14 and 60 by 12 cells. The four SVGs are editorial views of the CLI and TUI, not pixel-perfect screenshots.

| Step | User sees and does | Screen must prove |
| --- | --- | --- |
| 0:00, npm page | `@mzwin/kit`, the `@alpha` install line, five supported targets, and a short statement of the worktree, gate, and receipt loop. | This is a local CLI. The page states the exact alpha tag and system support before installation. `04-npm-page.svg` shows the proposed hero. |
| 0:30, install | `npm i -g @mzwin/kit@alpha` installs the launcher and the one optional package for the host. | `kit` resolves to the Rust binary. A plain install remains on 0.1.x until stable 1.0.0. The launcher must give a fix when a binary package is absent. |
| 1:00, doctor | `kit doctor` prints version, binary path, `installed via`, engine status, agent rows, and `try:` commands. | The user can identify the installed binary and see which agent CLI is on PATH. Today's hosted-agent `ready` flag does **not** prove authentication. The screen must state that limit. |
| 2:00, demo | `kit --demo` opens the Control Room with RUNNING, GATING, FAIL, and PASS examples. The selected FAIL row shows its first error. | The gate can stop a run. The demo rows are fixtures and are not real executions. `01-first-run.svg` and `02-fail-retry.svg` show this moment. |
| 3:00, gate and retry | Enter or `g` opens the gate detail. `r` on the failed row queues a **new** run with the prior gate summary in the task. | The user can find the failed check and its command. Retry preserves the original failure and does not imply a pass. The current code accepts retry only when the row state is `Fail`. |
| 4:00, own repo | In a git repo, run `kit run --agent codex --task "fix the failing test"`. Add `[gate]` commands to `kit.toml` when inference cannot find safe checks. | A live run gets a worktree, then a gate result, then a receipt. `--dry-run` exercises plumbing and is not agent proof. A live vacuous gate exits nonzero unless `--allow-vacuous` is set. |
| 5:00, receipt | `kit receipt list`, then `kit receipt show <id>`; use `--output` for the log tail or `--json` for automation. | The result has an ID, state, agent, repo, task, gate checks, and a run directory. `03-receipt.svg` follows the text output layout. A receipt records a result; its present file write is not immutable storage. |

### Error message grammar

Use **problem → effect → fix**. Name the bad value or missing tool. Name the command, file, or next action that repairs it. Keep the first error line visible in the table; keep the full command and log in the gate pane. Example: `typecheck failed: tsc --noEmit; src/client.ts:42 Type 'string' is not assignable. Fix the type error, then press r to retry.` This is proposed copy. The fixture currently shows `tsc: 3 errors — Type 'string' is not assignable` and `r` is the actual retry key.

For an absent optional package, retain the launcher's direct remedy but preserve the user's tag: `@mzwin/kit-win32-x64 is not installed. Reinstall with npm i -g @mzwin/kit@alpha and allow optional dependencies.` For an invalid `kit.toml`, name the parse error and path; do not silently use defaults. For a missing agent, name its CLI and the command to install or sign in. A non-interactive TUI request already points to `kit run --task "…" --json`.

## Backend concept: distribution and updates

**Install path.** `scripts/npm-stage.mjs` reads the version from `[workspace.package]` in `Cargo.toml`. It stages `@mzwin/kit` with a dependency-free Node 18+ launcher and five exact-version optional dependencies from `npm/platforms.json`. npm selects a package by `os`, `cpu`, and Linux `libc`. The launcher selects the matching package, finds `bin/kit` or `bin/kit.exe`, and forwards arguments and exit status. `KIT_BINARY_PATH` selects a user-built binary instead. The launcher sets `KIT_LAUNCHER=npm`; `kit doctor` displays `installed via npm` from that marker. An absent optional package produces an error. The documented source fallback is `cargo install --git https://github.com/Zwin-ux/kit kit-cli`.

**Targets.** The staged set is Windows x64, macOS arm64, macOS x64, Linux x64 glibc, and Linux arm64 glibc. The release guide gives glibc 2.35+ for Linux. There is no musl, Windows arm64, or Linux arm32 package in `npm/platforms.json`. The package page must state this before install.

**Release path.** `.github/workflows/release-npm.yml` builds all five Rust targets on a `v*` tag or a manual run. It runs each binary's `--version`, downloads the artifacts, checks that a tag matches Cargo's version, stages packages, then publishes platform packages before the launcher. An unchecked manual run does an npm dry run. A publish run needs `NPM_TOKEN`; the publish job requests OIDC for provenance. The script skips a package version that npm already has. `scripts/npm-smoke.mjs` packs and installs a host package in a temporary project; `.github/workflows/rust.yml` runs this on Windows, macOS, and Linux. A successful workflow is evidence for its tested packages, not proof of a live registry install or every user's environment.

**Version policy.** `1.0.0-alpha.N` uses `alpha`, beta uses `beta`, release candidate uses `rc`, and only plain `1.0.0` uses `latest`. The exact platform package version must match the launcher. While 1.0 is prerelease, `npm i -g @mzwin/kit` continues to install 0.1.x. Do not advertise the prerelease as the default npm command.

**0.1.x migration.** 0.1.x was the Node skill workbench. 1.x is the Rust Control Room. Keep an explicit upgrade line: `npm i -g @mzwin/kit@alpha`, then `kit doctor`, then `kit --demo`. Explain that the old lanes are not 1.x commands. On Windows, doctor already distinguishes the old `dist/bin.js` npm shim from the new `bin/kit.js` shim and warns about a 0.1.x command on PATH. A migration guide must say how to identify and remove an old shim if it still wins. It must not claim that skill state or 0.1 configuration migrates automatically; no migration routine is visible in the CLI or launcher.

**Update concept.** Do not add `kit self-update` until the owner channel is explicit. For an npm install, the safe fix is an npm command with the intended dist-tag, and the launcher should remain the owner of installed files. A source build needs a source upgrade path. An update notice would need an opt-in or local version source, a bounded check interval, a channel-aware comparison, offline behavior, and no token or command execution from remote text. `kit doctor` can show the installed channel and a precise manual update command. The current CLI has no `self-update` command or update notice.

**Cargo install and crates.io.** Git install of `kit-cli` is the present documented source route. Before any crates.io plan, check live availability and ownership of `kit-core`, `kit-agents`, `kit-gate`, `kit-tui`, and `kit-cli`; those names may be taken. Also check each manifest's publish metadata, license files, README, crate graph, and whether the path-only workspace dependencies can be packaged with registry versions. Do not promise `cargo install kit-cli` until the published graph and name ownership are proven. Keep npm as the first release channel until that check is done.

## Ranked gaps between this source tree and a strong package

The rank weights first-run trust, proof integrity, and user recovery. Each observation is tied to current files; proposed fixes are concepts.

| Rank | Verified gap and user effect | Files to change or verify |
| --- | --- | --- |
| 1 | Receipts are described as immutable, but `write_receipt` calls `create_dir_all` and `fs::write` on `receipt.json`, `output.log`, `diff.patch`, and `gate.json`. A repeat write can replace proof. Use create-new or an atomic, no-replace commit and test collision and crash paths. | `crates/kit-cli/src/engine/store.rs`, `crates/kit-cli/src/engine/runner.rs` |
| 2 | A present but malformed or unreadable `kit.toml` becomes `KitConfig::default()` without a diagnostic. A user can lose their intended gate. Return a path-specific parse error and block the run. | `crates/kit-cli/src/engine/store.rs`, `crates/kit-cli/src/engine/runner.rs` |
| 3 | `doctor` reports hosted agents ready after a binary probe: Codex, Claude, and Grok set `authenticated: true` when installed. This cannot prove login or a runnable session. Separate `installed` from verified readiness, or label auth as unchecked. | `crates/kit-agents/src/{codex,claude,grok}.rs`, `crates/kit-cli/src/main.rs` |
| 4 | The release workflow tests built binaries with `--version` and publishes staged packages, but it does not run the packed-install smoke test on those **release artifacts** before publish. Add a release-artifact install gate across target families. | `.github/workflows/release-npm.yml`, `scripts/npm-smoke.mjs`, `scripts/npm-stage.mjs` |
| 5 | The npm launcher covers five targets only. Unsupported hosts must build from source; Linux musl has no package. Add support only after build and installed-package tests, or make the support matrix more prominent. | `npm/platforms.json`, `npm/kit/bin/kit.js`, `npm/kit/README.md` |
| 6 | The launcher's missing-optional-package remedy says `npm install -g @mzwin/kit` without `@alpha`. Before stable 1.0, that installs 0.1.x. Keep the installed channel in the fix. | `npm/kit/bin/kit.js`, `docs/dev/RELEASING.md` |
| 7 | PTY attach is still an honest stub. A user can enter an Attached view, but cannot interact with the live agent there. Complete the PTY path and verify detach and kill on each host before presenting attach as live. | `crates/kit-tui/src/app.rs`, `crates/kit-tui/src/loop.rs`, `docs/dev/CURRENT.md` |
| 8 | Gate inference checks Cargo and package scripts conservatively, but an npm `format` script can be a writer and a project without recognized scripts stays vacuous. Make the inferred command visible before live dispatch and offer a clear `kit.toml` path. | `crates/kit-cli/src/engine/infer.rs`, `crates/kit-cli/src/main.rs`, `crates/kit-tui/src/app.rs` |
| 9 | The human `receipt show` output prints a gate's `PASS` when `g.passed` is true, even for a zero-check vacuous gate. The run command calls this `UNCONFIGURED`. Use one label in both views and the JSON browser. | `crates/kit-cli/src/main.rs`, `crates/kit-cli/src/engine/infer.rs` |
| 10 | README and status still include repo-only run instructions and a dated install/PATH state; the staged npm README is short and has no full migration or troubleshooting route. Review package copy against the published artifact after the first alpha. | `README.md`, `npm/kit/README.md`, `docs/dev/CURRENT.md`, `docs/dev/RELEASING.md` |

## Art files

- `01-first-run.svg`: npm install, doctor, and demo in order.
- `02-fail-retry.svg`: failure on the table, gate detail, and new retry context.
- `03-receipt.svg`: the text receipt as an inspectable proof artifact.
- `04-npm-page.svg`: the package page's install and support information.
