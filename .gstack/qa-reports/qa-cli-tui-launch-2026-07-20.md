# QA: Kit CLI / TUI launch (2026-07-20)

## Scope
CLI launch paths for `kit tui` after user error `unknown command: --` from `pnpm kit -- tui`.

## Root cause
Root script used nested `pnpm --filter @mzwin/kit exec node dist/bin.js`, so
`pnpm kit -- tui` became `node dist/bin.js -- tui` and treated `--` as the command.

## Fixes shipped
1. `normalizeArgv` strips **leading** `--` only (`packages/cli/src/argv.ts`)
2. Root scripts call `node packages/cli/dist/bin.js` directly (no nested pnpm)
3. Non-TTY: clear error instead of Ink raw-mode crash
4. Unit smoke: `packages/cli/tests/argv.test.mjs`

## Smoke matrix

| # | Command | Result |
|---|---|---|
| 1 | `pnpm kit --version` | OK → 0.1.4 |
| 2 | `pnpm kit -- --version` | OK → 0.1.4 |
| 3 | `pnpm kit -- tui` | OK path (non-TTY → exit 1 clear msg) **not** unknown command |
| 4 | `pnpm kit tui` | OK path (non-TTY msg) |
| 5 | `pnpm tui` | OK path |
| 6 | `pnpm kit -- doctor` | OK doctor runs |
| 7 | `node …/bin.js unify --write --help` | OK unify help; flags preserved |
| 8 | `node packages/cli/tests/argv.test.mjs` | 7/7 pass |

## Multi-opinion
- **Explore agent:** strip leading `--` correct; do not strip mid-argv; prefer direct node scripts. Ship-with-rebuild.
- **Codex:** (async) review of same launch surface.

## User action
Open a **real PowerShell/Windows Terminal** window (not agent pipe):

```powershell
cd C:\Users\mzwin\agent-sandbox\projects\kit
pnpm build
pnpm kit tui
```

## Verdict
**DONE** for launch bug. TUI still requires interactive TTY (by design).
