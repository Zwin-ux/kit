# Kit multi-agent orchestration

**Claude = CEO** (judgment, gates, merges, sequence)  
**Grok = Power** (implementation volume, TUI, PTY, parallel fan-out)  
**Codex = Factory** (ports, fixtures, CI scaffolding)  
**Luna threads = parallel worker agents** (cheap/high-volume read + write shards; burn token budget on concurrent work, not serial monologues)

## Roles

| Role | Who | Owns | Does not |
|------|-----|------|----------|
| **CEO** | Claude | Contracts, merge/no-merge, vacuous-gate policy, PR review, milestone kill criteria, “what ships next” | Bulk screen code, bulk adapter glue |
| **Power** | Grok | `kit-tui`, PTY, engine wiring in practice, wiki maintenance, **spawning Luna threads** | Changing frozen contracts without CEO |
| **Factory** | Codex | Mechanical ports, test oracles, CI matrix | TUI interaction language |
| **Luna** | Many short agents | One-shard tasks: diagnose CI leg, draft handle-registry design, grep kill paths, write one test file | Multi-hour open-ended ownership |

## CEO cadence (Claude)

Every cycle:

1. Read `wiki/kit-1.0/wiki/index.md` + `CURRENT.md`
2. Pick active phase (P0 → P1 → …) from workstreams
3. Issue a **Power brief** (≤15 lines): goal, non-goals, acceptance, crates allowed
4. Review Power PR for contract/boundary violations
5. Merge only when kill criteria for that slice are met

## Power cadence (Grok)

Every cycle:

1. Accept CEO brief (or infer from workstreams if CEO silent)
2. **Fan out Luna threads** (parallel subagents / workflow) for:
   - diagnosis (CI, compile, grep)
   - design shards (one file one concern)
   - verification (tests, clippy)
3. Integrate Luna outputs surgically
4. Open/update PR; report status to CEO in PR body + wiki log

## Luna thread rules

- **One job per thread.** No “do P1 entirely.”
- **Self-contained prompt.** Cold start; no chat memory.
- **Capability minimum.** Prefer read-only unless writing.
- **Cap fan-out.** Prefer 8–24 threads per wave; budget up to workflow max when CEO authorizes a “token burn” sprint.
- **Fail closed on evidence.** Empty results ≠ green.
- **No contract edits.** File CEO issue instead.

## Token burn mode (“luna storm”)

When user or CEO says **luna storm** / **burn tokens**:

1. Power launches a workflow with large parallel panels
2. Each thread targets one wiki workstream item or one CI failure
3. Synthesis thread consolidates into `wiki/kit-1.0/outputs/queries/` + PR checklist
4. Power implements only CEO-approved shards next

## Channels

| Channel | Use |
|---------|-----|
| PR description | CEO-facing status |
| `wiki/kit-1.0/log/` | Durable ops log |
| `docs/dev/tasks/*` | Work orders |
| Frozen contracts | CEO-only |

## Anti-patterns

- Power merging own PRs
- Luna threads editing contracts
- Serial “one agent does everything” when work parallelizes
- Shipping polish (mascot) before P1 kill
