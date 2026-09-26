# Kit JSON contract (1.0 thin)

**Status:** Active for headless automation.  
**CEO stamp:** wiki thin envelope — `schemaVersion: 1`, camelCase, `warnings` always present.

## Envelope

Every `kit … --json` payload:

```json
{
  "schemaVersion": 1,
  "command": "run",
  "ok": true,
  "data": {},
  "error": null,
  "warnings": []
}
```

| Field | Type | Notes |
|-------|------|--------|
| `schemaVersion` | integer | Always `1` for this generation |
| `command` | string | `run`, `init`, `doctor`, … |
| `ok` | bool | Process-level success for this command |
| `data` | object | Command-specific payload |
| `error` | string \| null | Human-readable failure when `ok` is false |
| `warnings` | string[] | Present from day one (may be empty) |

## `kit run --json` → `data`

| Field | Type |
|-------|------|
| `id` | string (ULID) |
| `state` | string (`pass`, `fail`, `killed`, …) |
| `receiptDir` | path string |
| `worktreeRemoved` | bool |
| `gatePassed` | bool \| null |
| `gateVacuous` | bool |

Exit code: `0` pass, `1` fail or vacuous (unless `--allow-vacuous` / `--dry-run`), `2` other.

## `kit init --json` → `data`

| Field | Type |
|-------|------|
| `repo` | path |
| `path` | path to `kit.toml` |
| `toolchain` | `rust`, `go`, `node` or `python` |
| `marker` | file that selected the toolchain (`Cargo.toml`, `package.json`, …) |
| `existed` | bool: a `kit.toml` was there before this command |
| `written` | bool: false with `--print` |
| `toml` | string: the exact file text |
| `checks` | `null` without `--check`; else array of `{ label, command, result, exitCode, summary, durationMs }`, `result` is `pass`, `fail`, `missing`, `timeout` or `refused` |
| `skipped` | strings: other projects not in the gate |
| `notes` | strings: why a script or tool was used or left out |

`warnings` names each program the gate needs that is not on PATH.  
Exit code: `0` written or printed. `2` when no check is safe to propose, `kit.toml` exists without `--force`, or a check fails under `--check` without `--drop-failing` (stdout: one `ok: false` envelope; nothing is written).

## `kit land <id> --json` → `data`

| Field | Type |
|-------|------|
| `id` | string (ULID) |
| `mode` | `branch` or `apply` |
| `repo` | path |
| `branch` | string \| null: the new branch (`kit/<first 12 chars of id>` or `--branch`); the branch that holds it when `alreadyLanded`; null in `apply` mode |
| `commit` | sha \| null: the landed commit (null in `apply` mode) |
| `base` | sha \| null: the parent of `commit` |
| `baseSource` | `recorded` (`base.txt`), `worktree` (kept worktree HEAD), `head` (repo HEAD, 3-way apply) or `none` |
| `threeWay` | bool |
| `alreadyLanded` | bool: a branch already has a commit with trailer `Kit-Receipt: <id>` (or, with `--apply`, the tree already has the changes) |
| `forced` | bool |
| `files` | strings: paths the diff changes |
| `worktreeRemoved` | bool: the kept run worktree held exactly this diff and was removed |
| `next` | string: the command to run next (`git merge kit/<id>`) |

`warnings` holds each `--force` override and each base fallback.  
Exit code: `0` landed or already landed. `2` refused (run not `pass`, gate UNCONFIGURED, empty diff, dirty tree under `--apply`, branch exists without this run, diff does not apply): one `ok: false` envelope.

The run dir also holds `base.txt`: the commit the run's worktree started at (one line). `diff.patch` is binary-safe (`git diff --binary`) and holds all changes since that commit, new files and agent commits included.

## Errors

When any `--json` command fails before it has a result (not a git repo, agent not installed, invalid `kit.toml`, unknown command), stdout still holds exactly one envelope: `ok: false`, `data: null`, `error` set. Exit code `2`.

## `kit doctor --json` → `data`

| Field | Type |
|-------|------|
| `version` | string |
| `binary` / `controlRoom` / `gateEngine` / `runEngine` | status strings |
| `kitHome` | path |
| `binaryPath` | path |
| `install` | `npm` (via the `@mzwin/kit` launcher) or `binary` |
| `pathCollisions` | paths of other `kit` programs on PATH (the npm 1.x shim is not one) |
| `agents` | array of `{ agent, installed, ready, version, remedy }` |
| `kitToml` | path to `./kit.toml` \| null |

## `kit receipt list --json` → `data`

| Field | Type |
|-------|------|
| `kitHome` | path |
| `runsDir` | path |
| `count` | integer |
| `receipts` | array of `{ id, state, agent, repo, task, gatePassed, dir }` |

## `kit receipt show <id> --json` → `data`

Receipt object (`version`, `id`, `spec`, `state`, `gate`, `diff`, …) plus:

| Field | Type |
|-------|------|
| `dir` | path to `~/.kit/runs/<id>/` |
| `outputTail` | string (only with `--output`) |

Id may be a unique prefix of the ULID.
