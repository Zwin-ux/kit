# CLI End-to-End Tests

These tests spawn the **compiled** KIT CLI (`packages/cli/dist/bin.js`) inside
isolated temporary environments. They never import CLI source code and never
touch the developer's real home directory.

Run them:

```sh
pnpm build                      # e2e runs the compiled bin
pnpm --filter @mzwin/kit test   # argv smoke + vitest e2e
```

## Isolation contract (`harness.ts`)

Every spawned CLI process gets a fresh sandbox under the OS temp dir with:

- `HOME` and `USERPROFILE` → `<sandbox>/home`
- `KIT_HOME` → `<sandbox>/home/.kit`
- `cwd` → `<sandbox>/project` (name configurable, e.g. spaces/unicode)
- `KIT_REGISTRY_URL` → an unroutable local port (fast offline failure)
- `KIT_PACKS` / `KIT_SKILLS` / `KIT_ASSETS` / `HOMEDRIVE` / `HOMEPATH` stripped

A read-only sentinel snapshots the real `~/.kit/skills`, `~/.claude/skills`,
`~/.codex/skills`, and `~/.grok/skills` before each spec and fails the suite
if any new entry appears there afterwards.

On failure, assertions report the command, exit code, stdout, stderr, and
sandbox path.

## Covered scenarios

- `kit ready` dry-run default, `--write` (library + project + 3 harness links),
  idempotent re-run, home-dir write guard, spaces + unicode project paths.
- `kit status` on empty library and `--json` allOk after `ready --write`.
- `kit recommend` top pick for a react project.
- `kit pack list | validate | apply` offline against the repo catalog.
- `kit link` dry-run, `--to all --write`, skip-same idempotency, conflict
  without/with `--force`, broken-link repair-or-report, `--scope personal`.
- `kit import` missing root, dry-run, `--write`, skip-exists, invalid folders.
- `kit unify` dry-run, `--json` counts, `--write` keeper adoption + noise
  filtering, idempotent re-run, `--write --link`, `--link` guard.
- `kit doctor` fresh sandbox, KIT_HOME reporting, post-`ready` health, `--dir`.
- `--version` (matches package.json), `--help`, unknown-command exit codes.
- JSON contract (schemaVersion "1" envelope) for `doctor --json` and
  `ready --json`: pure-JSON stdout, exact envelope keys, typed error codes,
  `--no-color`, and non-regression of the raw `status`/`unify --json` shapes.

The same suite runs on Ubuntu, macOS, and Windows in CI (junction links on
Windows need no admin rights; path assertions normalize separators).

## Fixtures (`fixtures.ts`)

`SKILL.md` fixtures mirror `packages/core/tests/fixtures/valid-add-readme`.
Library seeding goes through the real CLI (`kit install`) so tests never poke
library internals.
