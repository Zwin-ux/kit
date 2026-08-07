# Index — Kit 1.0 Product

> Knowledge base for shipping a **high-quality Kit 1.0.0** control room: every concept, surface, and workstream fully specified with honest status.

## Navigation

- [[#Concepts]] · [[#Entities]] · [[#Summaries]] · [[#Open Questions]]

## Concepts

### Product
- [[concepts/product/index|Product]] — what Kit 1.0 is
    - [[concepts/product/job-to-be-done|Job to be done]] — user, pains, wedge
    - [[concepts/product/principles|Principles]] — six laws
    - [[concepts/product/non-goals|Non-goals]] — cut list
    - [[concepts/product/definition-of-done|Definition of done]] — ship checklist

### Run (core primitive)
- [[concepts/Run/index|Run]] — the one primitive
    - [[concepts/Run/lifecycle|Lifecycle]] — state machine
    - [[concepts/Run/bounds|Bounds]] — timeout, caps, scope
    - [[concepts/Run/worktree|Worktree]] — git isolation
    - [[concepts/Run/delta|RunDelta]] — UI stream events
- [[concepts/Receipt|Receipt]] — immutable proof on disk

### Gate (differentiator)
- [[concepts/Gate/index|Gate]] — definition of done
    - [[concepts/Gate/kit-toml|kit.toml]] — config schema
    - [[concepts/Gate/firewall|Firewall]] — blast radius
    - [[concepts/Gate/vacuous-vs-real|Vacuous vs real]] — empty config policy
    - [[concepts/Gate/outcome|GateOutcome]] — result shape

### Skills
- [[concepts/Skill|Skill]] — substrate workflows + injection

### Surface (TUI)
- [[concepts/surface/index|Product surface]] — screen map
    - [[concepts/surface/control-room|Control Room]] — live table
    - [[concepts/surface/run-detail|Run Detail]] — stream/gate/diff
    - [[concepts/surface/dispatch|Dispatch]] — fan-out form
    - [[concepts/surface/board|Board]] — queue / orchestrator
    - [[concepts/surface/attach|Attach]] — PTY takeover
    - [[concepts/surface/doctor|Doctor]] — readiness
    - [[concepts/surface/library|Library]] — local skills browser

### Engine
- [[concepts/engine/index|Engine]] — execution home
    - [[concepts/engine/pipeline|Pipeline]] — execute steps
    - [[concepts/engine/adapters|Adapters]] — codex/claude/grok/ollama
    - [[concepts/engine/handle-registry|Handle registry]] — kill/retry

### CLI
- [[concepts/cli/index|CLI]] — commands and env
    - [[concepts/cli/json-contract|JSON contract]] — automation

### Roadmap
- [[concepts/roadmap/index|Roadmap to high-quality v1.0]] — north star
    - [[concepts/roadmap/status-matrix|Status matrix]] — S/P/L/C
    - [[concepts/roadmap/workstreams|Workstreams]] — P0–P5 detailed
    - [[concepts/roadmap/release|Release (M5)]] — distribute 1.0.0

## Entities

### Product / crates
- [[entities/Kit]] — the product
- [[entities/kit-core]] — contracts crate
- [[entities/kit-agents]] — adapters crate
- [[entities/kit-gate]] — gate crate
- [[entities/kit-tui]] — surface crate
- [[entities/kit-cli]] — binary + engine host

## Summaries (sources)

- 2026-08-01 — [[summaries/prd-1.0]] — Product requirements
- 2026-08-01 — [[summaries/current-impl]] — Implementation truth
- 2026-08-01 — [[summaries/b2-adapters]] — Adapter + skills spec

## Open Questions

- Vacuous gate: PASS+warn vs UNCONFIGURED vs infer defaults? → [[concepts/Gate/vacuous-vs-real]]
- Board: prefill-only OK for 1.0 or must pull-queue? → [[concepts/surface/board]]
- Attach: 1.0 blocker or 1.0.1? → [[concepts/surface/attach]]
- npm dual-publish with 0.1 forever?
- Auth probe without credential custody — how deep?

## How to use this wiki

1. Read `CLAUDE.md` + this index at session start  
2. Deep-dive concept pages for feature work  
3. Execute [[concepts/roadmap/workstreams|Workstreams]] in order  
4. File corrections in `audit/` (llm-wiki audit op)  
5. Query answers go to `outputs/queries/`  
