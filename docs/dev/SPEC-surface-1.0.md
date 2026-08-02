# Spec: Kit 1.0 Control Room surface

**Status:** Implementation contract for `kit-tui` remake.  
**Product PRD:** `docs/dev/PRD-1.0.md`  
**Visual system:** `docs/dev/DESIGN-tui.md`  
**Owner:** Grok (Power). Contracts in `kit-core` / `event.rs` remain Claude-only.

---

## Objective

Ship a **production-grade** terminal control room: one glance shows every parallel agent run, gate failures surface first, and every screen speaks the same interaction language. The skill pack and marketplace are not the product — the **run, gate, and receipt** are.

**User:** developer running ≥2 coding agents daily.  
**Success:** `kit --demo` looks intentional (not a scaffold); FAIL is unmissable; empty state tells you what to press; `NO_COLOR` remains usable.

---

## Commands

```text
cargo run -p kit-cli -- --demo
cargo test -p kit-tui
cargo clippy -p kit-tui --all-targets -- -D warnings
# Windows PowerShell monochrome:
$env:NO_COLOR=1; cargo run -p kit-cli -- --demo
```

---

## Screens

### Control Room (default)

**Purpose:** Live table of runs. Sorted by state priority then age. Selection stable across re-sorts.

**Columns (fixed):** `REPO | AGENT | TASK | STATE | GATE`

**Layout:**
```
HEADER   KIT / CONTROL ROOM          N RUNNING  M FAIL  K GATED
BODY     bordered table (+ FAIL annotation lines)
FOOTER   key hints (dim) · optional flash/status right
```

**Keys:**

| Key | Action |
|-----|--------|
| `j`/`k` or arrows | Move selection |
| `Enter` | Open run detail (stream) |
| `g` | Open run detail (gate pane) |
| `d` | Dispatch |
| `b` | Board |
| `k` | Kill selected active run |
| `r` | Retry selected FAIL only |
| `?` | Help overlay |
| `q` | Quit |

**Empty:** "No runs yet — press **d** to dispatch" (centered in body).  
**FAIL row:** danger text + optional fail_wash bg; annotation `^ first error` under row.  
**GATE labels:** `PASS` success · `FAIL` danger · `UNCONFIGURED` warn · `--` muted.

**Snapshots:** 80×14 populated, 60×12 narrow, empty 80×12.

### Run detail

**Purpose:** Stream, gate log, diff for one run.

**Keys:** `Esc` back · `1`/`2`/`3` or `Tab` panes · `a` attach stub · `k` kill · `r` retry · `End` follow stream.

**Wide (≥100 cols):** optional stream left + diff right.  
**Attached:** intentional "PTY attach ships 1.0.1" panel; Esc detaches; `q` disabled.

### Dispatch

**Purpose:** Fan-out repos × agents × one task.

**Keys:** `Tab` field · `Space` toggle · type in task · `Enter` submit · `Esc` back.

**Focus:** accent border on focused panel.

### Board

**Purpose:** Prefill-only task list (CEO: no pull-queue in 1.0).

**Keys:** `n` new · `Enter` prefill Dispatch · `Space` toggle done · `x` remove · `Esc` back.

### Doctor / Library

CLI (`kit doctor`) for 1.0; TUI screens optional later.

---

## Interaction grammar (all screens)

1. **Header** — brand/title left · stats right · flash inline if present  
2. **Body** — one primary surface  
3. **Footer** — `[key] action` list, dim; never empty of discoverability  
4. **Esc** — always back one level (never quit from nested)  
5. **q** — quit only from Control Room (and disabled while attached)

---

## Boundaries

**Always:**
- Single clock: `AppEvent::AnimationTick` only
- Snapshot tests for major screens
- Honor `NO_COLOR`
- Failures surface first error line in table

**Ask first:**
- New screens beyond CR/Detail/Dispatch/Board
- Contract changes (`event.rs`, kit-core)
- Mouse hit-testing (F6)

**Never:**
- Marketplace / registry UI
- Timer outside event loop
- Edit contracts as Grok
- Fake RUNNING without engine proof

---

## Success criteria

- [ ] Theme tokens live; monochrome path works
- [ ] Control Room empty + populated match design density
- [ ] FAIL wash + annotation visible in demo fixture
- [ ] All screens use shared chrome
- [ ] `cargo test -p kit-tui` green
- [ ] SPEC + DESIGN committed

---

## Open questions

- Split stream|diff on wide detail: implement in remake phase 4 if free
- Help overlay `?`: ship with remake phase 6
