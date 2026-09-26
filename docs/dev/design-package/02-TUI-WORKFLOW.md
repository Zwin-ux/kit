# Kit TUI Engagement Workflow (Deep Intent)

This document describes **how a human is supposed to feel and move through the Control Room**.  
It is the most important intent document for any coding agent working on the TUI.

If the agent does not understand this, it will optimize the wrong things.

---

## 1. Mental Model

Kit is not “a chat with agents.”

It is a **live operations board for parallel software work**.

The user should feel:

- “I can see everything that is happening.”
- “I can trust the state.”
- “When something fails, the tool helps me fix it instead of making me dig.”
- “I can kill or retry without fear.”
- “When I come back later, the receipts still make sense.”

The Control Room is closer to a good process manager + CI view + git worktree dashboard than it is to a chatbot.

---

## 2. Core Daily Engagement Loop

### A. Entry

User types `kit` or `kit --demo`.

**First paint must answer quickly:**
- Am I in an empty room or are there live runs?
- Which agents are available?
- What can I do right now?

Empty state should feel inviting, not dead:

> “No runs. Press `d` to dispatch.”

### B. Dispatch

User presses `d`.

They choose:
- One or more repos
- One or more agents
- One task
- (Optionally) mode: recommended / guided / vibe

On confirm, the Engine immediately creates runs. The table updates **instantly** with new rows in `QUEUED` → `RUNNING`.

**Emotional beat:** “I just spun up real work and I can already see it.”

### C. Live Monitoring (the main state)

The table is the primary surface.

User scans:
- Who is still running
- Who failed
- Who passed the gate
- How long things have been going

**FAIL rows must be visually loud** (wash + first error annotation).  
The user should not need to open a run to know *why* it failed at a high level.

**Emotional beat:** “I can manage this without opening six terminals.”

### D. Intervention

While runs are live the user can:

| Key        | Action              | Intent                          |
|------------|---------------------|---------------------------------|
| `↑↓` / j/k | Move selection      | Navigate                        |
| `Enter`    | Open run detail     | Inspect stream / gate / diff    |
| `k`        | Kill                | Stop something going wrong      |
| `r`        | Retry               | Re-dispatch with previous gate failure context automatically included |
| `g`        | Focus gate info     | Understand proof status         |
| `?`        | Help                | Discoverability                 |
| `q`        | Quit (top level only) | Leave cleanly                 |

**Retry is the product differentiator.**  
One key. The new task should contain the previous failure so the agent can actually self-correct.

### E. Inspection (Run Detail)

User hits `Enter` on a run.

Surfaces:
- **Stream** — live or historical output (error lines highlighted)
- **Gate** — what ran, what failed, first error
- **Diff** — what actually changed

`Esc` returns to the board without killing the run.

**Emotional beat:** “I can go deep when I want, but I don’t have to.”

### F. Completion & Receipt

When a run finishes:
- State becomes `PASS` / `FAIL` / `KILLED` / `ERROR`
- A receipt is written
- The table stays useful (user can still open it, retry it, or move on)

Later the user can browse receipts with `kit receipt list` / `kit receipt show`.

**Emotional beat:** “There is proof. I can come back to this.”

---

## 3. Multi-Run Reality (power-user flow)

Typical serious usage:

1. User dispatches the same task across 3 agents or 3 repos.
2. One fails fast on the gate.
3. User hits `r` on the failed one while the others continue.
4. User opens the still-running ones occasionally to check progress.
5. User kills one that is clearly stuck.
6. At the end they have a mix of PASS receipts and a couple of useful FAIL receipts that already contain diagnosis.

The TUI must make this feel **calm and controllable**, not chaotic.

---

## 4. Emotional Design Targets

| Moment              | Desired Feeling                  | Design Implication                          |
|---------------------|----------------------------------|---------------------------------------------|
| First open          | “This is fast and clear”         | Instant paint, good empty state             |
| After dispatch      | “Work is happening”              | Immediate row creation + state              |
| Seeing a FAIL       | “I understand why”               | Wash + first error on the board             |
| Hitting retry       | “The tool is helping me”         | Automatic failure context                   |
| Killing a run       | “Safe and clean”                 | Reliable kill + state update                |
| Opening a receipt later | “This is still useful”       | Structured, readable proof                  |
| Leaving             | “I can come back”                | No lost state, clean quit                   |

If any of these feelings are missing, the TUI is failing its job.

---

## 5. Progressive Disclosure

- **Level 0 (board):** State, gate, first error, identity of the run
- **Level 1 (detail):** Full stream, full gate log, diff
- **Level 2 (receipts / CLI):** Historical proof, scripting, CI

Most of the time the user should stay at Level 0 and only drop into Level 1 when needed.

---

## 6. Edge Cases That Matter

- Narrow terminal (60–80 columns) still truthful
- Many runs (10–16) without the table becoming unusable
- Run finishes while user is in detail view
- Kill during gating
- Retry of a killed vs failed run
- No agents available (doctor should be obvious)
- Demo mode must showcase the FAIL → retry loop immediately

---

## 7. First-Run / Demo Intent

`kit --demo` exists to create the “holy shit” moment:

1. Land on a board that already has a FAIL.
2. User sees the wash + first error.
3. User hits `r`.
4. New run starts with context.
5. User feels the loop.

This is how people decide to keep the tool.

---

## 8. Density Rules (table)

| Column              | Priority | Notes                                      |
|---------------------|----------|--------------------------------------------|
| State + Gate        | Highest  | Must be readable at a glance               |
| First error (on FAIL) | Highest | Visible without opening the run            |
| Repo + Agent        | High     | Identity of the work                       |
| Task                | High     | Truncate intelligently                     |
| Age / duration      | Medium   | Helps prioritization                       |
| Extra metadata      | Low      | Only if space remains                      |

---

**This is the intent. Optimize the real Control Room toward these feelings and flows.**
