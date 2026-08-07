# Kit dogfood checklist (H0)

**Goal:** Prefer Kit over four terminals for multi-agent work on Kit itself.  
**Kill criterion:** One full session leaves useful receipts and at least one real gate lesson.

## Before

```bash
cargo build -p kit-cli --release
./target/release/kit doctor
./target/release/kit receipt list --limit 5
```

Note which agents are `ready`. You need at least two for a real fan-out.

## Session A — offline proof (always)

```bash
./target/release/kit run --dry-run --task "dogfood smoke" --json
./target/release/kit receipt list --limit 3
./target/release/kit receipt show <id> --output
```

Expect: receipt under `~/.kit/runs/`, vacuous dry-run may report `gateVacuous: true`.

## Session B — live single agent

```bash
# Pick one ready agent
./target/release/kit run --agent codex --task "Add a one-line comment to docs/dev/DOGFOOD.md noting dogfood date; do not expand scope."
./target/release/kit receipt show <id>
```

Record: did worktree clean? gate PASS / FAIL / UNCONFIGURED?

## Session C — Control Room fan-out (product moment)

```bash
./target/release/kit --demo     # confirm FAIL selected + annotation
./target/release/kit            # empty/live room
```

In the TUI:

1. `d` Dispatch  
2. Enable **kit** repo + two ready agents  
3. Task: short, verifiable (e.g. “list public modules in kit-core and write a 3-bullet note to /tmp is wrong — only edit a new file under docs/dev/dogfood-notes/”)  
4. Submit; watch RUNNING  
5. If FAIL → `r` retry once  
6. `k` kill a RUNNING job once to prove control plane  

## Session D — notes (required)

Create `docs/dev/dogfood-notes/YYYY-MM-DD.md` (or a private note) with:

| Question | Answer |
|----------|--------|
| Agents used | |
| What worked | |
| What broke | |
| Gate caught something real? | Y/N + detail |
| Would you open Kit tomorrow? | Y/N |

## Done when

- [ ] Sessions A–C completed once  
- [ ] Notes written  
- [ ] Bugs filed or fixed  
- [ ] Ready to tag `v1.0.0-alpha.2`  

## Non-goals for dogfood

- Installer perfection  
- PTY attach  
- Marketplace  
