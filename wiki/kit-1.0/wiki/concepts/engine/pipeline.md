---
title: Engine Pipeline
type: concept
created: 2026-08-01
updated: 2026-08-01
sources: [current]
tags: [engine]
status: partial
---

# Engine Pipeline

## `execute(opts, id?, tx?)` steps

```mermaid
sequenceDiagram
  participant C as Caller
  participant E as Engine
  participant G as git
  participant A as Adapter
  participant Gate as kit-gate
  participant FS as Receipt store

  C->>E: execute
  E->>E: ensure_layout KIT_HOME
  E->>C: State(Running)
  E->>G: worktree add
  E->>C: Worktree(path)
  alt dry-run
    E->>C: Output chunks
  else live
    E->>A: spawn(spec, wt, deltas)
    A->>C: Output chunks
    A-->>E: exit code
  end
  E->>C: State(Gating)
  E->>Gate: evaluate(wt, config)
  Gate-->>E: GateOutcome
  E->>C: Gate(outcome)
  E->>C: State(Pass|Fail|Error)
  E->>G: diff + status
  E->>FS: receipt.json + output.log
  E->>G: remove if clean
  E-->>C: RunResult
```

## Dry-run vs live

| Mode | When | Agent body |
|------|------|------------|
| dry-run | `--dry-run` or binary missing (auto) | Synthetic transcript |
| live | binary on PATH (auto) or `--live` | Real CLI via adapter |

## Error mapping

| Failure | Result state |
|---------|--------------|
| Not git repo | Error (before Running ideally) |
| worktree add fails | Error |
| Agent missing + force live | fallback dry-run or Error |
| Agent exit ≠ 0 | agent_ok false → Error (today) |
| Gate fail | Fail |
| Gate pass | Pass |

## Gaps for v1.0

- [ ] Enforce Bounds.timeout with process kill
- [ ] Firewall wrap of agent-spawned commands (deep)
- [ ] Concurrent scheduler limits (max N runs)
- [ ] Structured error codes in RunResult JSON
