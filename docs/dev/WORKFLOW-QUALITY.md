# Kit long-running quality workflow

**Goal:** keep multi-agent pressure on Kit until 1.0 is honestly shippable — not a one-shot PR.

**Orchestration:** `docs/dev/ORCHESTRATION.md` (CEO / Power / Luna)  
**Runner:** Grok Build workflows (`.grok/workflows/kit-quality-cycle.rhai`)  
**Git copy:** `docs/dev/workflows/kit-quality-cycle.rhai` (`.grok/` is gitignored)

---

## Why this exists

High quality does not come from one long monologue. It comes from:

1. **Truth** — what is actually green vs claimed  
2. **Parallel audit** — many Luna shards reading code  
3. **Skeptics** — fail-closed verification of findings  
4. **Plan** — ordered slices with acceptance + verify  
5. **Implement** — small Power slices (gated)  
6. **Prove** — cargo test/clippy evidence  
7. **Report** — durable brief for the next cycle  

Repeat until kill criteria pass and CEO merges.

---

## Primary workflow: `kit-quality-cycle`

### Modes

| `args.mode` | What it does | Cost |
|-------------|--------------|------|
| `audit` | Truth + parallel audit + skeptics → report | Medium, read-only |
| `plan` | audit path + ordered backlog → report | Medium |
| `implement` | plan + implement N slices + prove | High |
| `full` | entire ladder (default) | Highest |

### Focus lenses

| `args.focus` | Shards |
|--------------|--------|
| `all` | surface + engine + ship + tests + security |
| `surface` | TUI craft + UX keys |
| `engine` | P3 concurrency, kill, gate |
| `ship` | installers, JSON, distribution |

### Other args

| Arg | Default | Meaning |
|-----|---------|---------|
| `max_implement` | `2` | Max slices to implement per cycle |
| `auto_implement` | `false` | If true, skip human gate before implement |

### How to run (Grok Build)

```text
# Cheap continuous audit (recommended on a schedule)
/workflow kit-quality-cycle {"mode":"audit","focus":"all"}

# Plan only (CEO/Power backlog)
/workflow kit-quality-cycle {"mode":"plan","focus":"all"}

# Full ladder — pauses before implement for your OK
/workflow kit-quality-cycle {"mode":"full","focus":"all","max_implement":2}

# Unattended implement of 3 slices (use carefully)
/workflow kit-quality-cycle {"mode":"full","focus":"engine","max_implement":3,"auto_implement":true}
```

Watch: `/workflows`  
Resume after pause: `/workflow resume kit-quality-cycle` (or numbered handle)

Agent budget: raise if needed (full cycle can use 30–60+ logical agents):

```text
# via tool / UI with agent_budget: 128
```

### Outputs

Each run writes scratch report(s):

- `kit-quality-audit.md` (audit mode)  
- `kit-quality-plan.md` (plan mode)  
- `kit-quality-report.md` (full / implement)

Open from the workflow run panel. Optionally paste into `wiki/kit-1.0/outputs/queries/` for durability.

---

## Recommended long-running cadence

### Daily (or every few hours while building)

1. **Audit cycle** (mode=`audit`) — cheap, no edits  
2. Read confirmed findings  
3. If CI red → fix before anything else  

### Feature days

1. **Plan** (mode=`plan`)  
2. CEO/Power pick 1–3 slices  
3. **Full** cycle with `max_implement: 2` and human gate  
4. Push branch; watch PR #11 CI  

### Pre-merge

1. Focus `engine` full cycle  
2. Focus `surface` audit  
3. Prove green on Windows + note Unix CI from GitHub  
4. CEO merges (Power never merges own PR)

### After merge / toward 1.0.0

1. Focus `ship` plan  
2. P5 installers + third-party clean machine  
3. 60s demo of gate catch  

---

## Scheduled automation

In Grok Build, create a **durable** scheduled task (example):

| Interval | Prompt |
|----------|--------|
| every 4h | Run workflow `kit-quality-cycle` with args `{"mode":"audit","focus":"all"}` and summarize top 5 confirmed findings in chat. Do not implement. |
| daily | Run `kit-quality-cycle` mode `plan`, focus `all`; paste plan summary. |

Use `scheduler_create` / UI schedules. Prefer **audit** on a timer so implement stays human-gated.

---

## Companion workflows

| Workflow | Role |
|----------|------|
| `kit-luna-storm` | Older P0–P2 diagnose storm (still useful) |
| `kit-quality-cycle` | **Primary** quality ladder (this doc) |

---

## Success definition (stop when all true)

- [ ] PR #11 green on ubuntu + macos + windows (Rust)  
- [ ] CEO merge landed  
- [ ] P3: 12-dispatch / max-8 concurrent proven in tests  
- [ ] Vacuous never shows as PASS; demo FAIL path exists  
- [ ] JSON envelope documented + used by doctor/run  
- [ ] `cargo test --workspace` + clippy -D warnings  
- [ ] Clean-machine install path (P5) verified by someone who is not you  

Until then: **run another cycle.**

---

## Anti-patterns

- One agent “does everything” for hours  
- Implementing marketplace/mascot before kill/P3  
- Power merging Power PR  
- Treating empty audit findings as green  
- Auto-implement on a 4h schedule without review  
