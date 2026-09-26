# Skills Posture + Protocol Direction

## Progressive Skill Model

### Recommended (default)

High-signal software engineering skills only:

- `spec-driven-development`
- `planning-and-task-breakdown`
- `incremental-implementation`
- `test-driven-development`
- `code-review-and-quality`
- `debugging-and-error-recovery`
- `security-and-hardening`
- `observability-and-instrumentation`
- `api-and-interface-design` (contextual)
- `shipping-and-launch`
- `using-agent-skills` (meta)

### Vibe (explicit escape hatch)

- `using-agent-skills` (meta)
- Lightweight instruction: “Make the gate pass. If it fails, diagnose from the gate output and fix the real problem.”

**Rules:**
- Gate **always** runs in both modes.
- Vibe is available but never the advertised default.
- Mode should be recorded on the run / receipt.
- After the first successful non-vibe receipt, soft-prompt toward the full recommended pack is acceptable.

## Core Domain Types (direction)

These should become clean and consistent over time:

- `RunId`, `Run`, `RunState`, `RunDelta`
- `AgentKind`
- `GateOutcome` / `GateStatus`
- `Receipt` (or `ReceiptSummary`)
- `EngineCommand`
- `EngineEvent`
- `KitError` (typed, stable codes)

The TUI should not invent state. It only sends commands and renders events.

## Protocol Principle

Even while the Engine stays in-process:

- Design `EngineCommand` + `EngineEvent` (or equivalent) as if they could one day cross a socket or MCP boundary.
- This preserves optionality without paying the daemon tax yet.

## Receipt Minimum (target)

Every finished run should leave structured proof that includes at least:

- schemaVersion
- id, repo, agent, task, mode
- state, timestamps, duration
- gate status + firstError + checks
- paths to output log and optional diff

The Control Room and CLI should both be able to surface the important fields without the user digging.
