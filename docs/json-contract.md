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
| `command` | string | `run`, `doctor`, … |
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

## `kit doctor --json` → `data`

| Field | Type |
|-------|------|
| `version` | string |
| `binary` / `controlRoom` / `gateEngine` / `runEngine` | status strings |
| `kitHome` | path |
| `skillsPack` | path \| null |
| `agents` | array of `{ agent, ready, version, remedy }` |
