# Recruiter Engineering Roadmap

KIT already has a real monorepo, core tests, a published CLI package, packs, adapters, and a TUI. The next releases should prove production behavior: safe writes, cross-platform execution, machine-readable output, and a reproducible release.

## Release 1 — Cross-platform end-to-end harness

### Goal
Prove the built CLI works in clean Windows, macOS, and Linux environments without touching a developer's real home directory.

### Build
- `packages/cli/tests/e2e/` harness that spawns the compiled CLI.
- Create isolated temporary values for `HOME`, `USERPROFILE`, `KIT_HOME`, project directory, and agent directories.
- Cover `kit ready`, `status`, `recommend`, `pack apply`, `link`, `unify`, and `doctor`.
- Fixtures for a web app, API service, CLI, monorepo, and data/ML project.
- Test paths containing spaces, Unicode, long names, and Windows separators.
- Test repeated commands for idempotency.
- Expand CI to an OS matrix with supported Node 20 and current Node 22.

### Done when
- The same E2E suite passes on Ubuntu, macOS, and Windows.
- Tests never read or mutate the runner's actual agent configuration.
- A second run produces no unintended changes.
- Failures print the command, exit code, stdout, stderr, and isolated fixture path.

## Release 2 — Transactional filesystem writes

### Goal
Make every mutating command previewable, atomic where practical, conflict-aware, and recoverable.

### Build
- Shared `Plan` model describing create, update, link, unlink, skip, and conflict operations.
- Dry-run renderer used by `ready`, `apply`, `link`, `import`, and `unify`.
- Write through temporary files followed by atomic rename for normal file updates.
- Refuse unsafe traversal outside approved roots.
- Detect existing files, directories, symlinks, broken symlinks, and cross-device limitations.
- Create a rollback journal before multi-step writes; add `kit rollback <operation-id>` if an operation partially fails.
- Add `--force` only for explicitly documented conflicts.
- Test simulated permission errors, interrupted writes, malformed manifests, and link collisions.

### Done when
- Every write command can emit its plan without changing disk state.
- Partial failure leaves the workspace unchanged or provides a tested rollback path.
- Conflicts identify the exact source and destination paths.
- Symlink behavior is tested on all supported operating systems.

## Release 3 — Stable JSON API and error taxonomy

### Goal
Make KIT useful to CI pipelines and other agents, not only humans in a terminal.

### Build
- Shared response envelope: `schemaVersion`, `command`, `ok`, `data`, `warnings`, and `errors`.
- Stable typed error codes such as `INVALID_ARGUMENT`, `PACK_NOT_FOUND`, `PATH_CONFLICT`, `PERMISSION_DENIED`, `AUTH_REQUIRED`, and `PARTIAL_FAILURE`.
- `--json` for `ready`, `status`, `doctor`, `recommend`, `pack`, `link`, `import`, and `unify`.
- Keep stdout valid JSON in JSON mode; diagnostics go to stderr.
- Document exit codes and compatibility guarantees.
- JSON Schema files plus snapshot and contract tests.
- Add `--no-color` and honor `NO_COLOR`.

### Done when
- Each major command has a golden JSON contract test.
- JSON output contains no ANSI sequences or prose outside the envelope.
- Exit codes are consistent across platforms.
- A small example script can call KIT and make decisions without parsing human text.

## Release 4 — Release integrity and install proof

### Goal
Prove that what is tested is what users install.

### Build
- CI creates an npm tarball with `pnpm pack`.
- Smoke-install that tarball into a fresh temporary environment and execute `kit --version`, `kit ready`, and `kit doctor`.
- Verify package contents and reject missing runtime files or accidental large assets.
- Add changelog enforcement and release notes generated from conventional commits or a documented manual process.
- Add npm provenance where supported and document package ownership.
- Add architecture, filesystem safety, adapter contract, and release-process documentation.
- Record startup and common-command performance budgets.
- Produce one concise demo showing install → ready → link → doctor.

### Done when
- CI validates the exact packed artifact.
- A clean global install works on all supported operating systems.
- Package version, CLI version, and changelog agree.
- The README's installation and demo commands are exercised automatically.

## Release 5 — Product clarity

### Goal
Help a recruiter or new user understand the value without reading the whole repository.

### Build
- Above-the-fold explanation: the problem, who experiences it, and the single fastest demo.
- Architecture diagram: CLI → core plans → packs/catalog → agent adapters → filesystem → doctor/TUI.
- `docs/CASE_STUDY.md`: user pain, constraints, technical decisions, edge cases, tests, and results.
- Replace decorative GIF volume with one end-to-end proof GIF and a small number of targeted screenshots.
- Add an explicit limitations section and supported-platform table.

### Done when
- A reviewer can explain KIT accurately after a 60-second README scan.
- Every major claim links to a test, command, architecture section, or release artifact.
