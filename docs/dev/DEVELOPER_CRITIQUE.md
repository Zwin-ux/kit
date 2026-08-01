# Kit critique — developer quickstart for agentic work

## What Kit claims

Point at a repo. Install portable skills. Wire Claude / Codex / Grok. Run one job. Keep the proof.

## What a developer actually needs (day 0)

```text
cd my-repo
kit tui
→ see: this project, which agents are dark, one next step
→ one confirm: install pack + apply + link
→ open Claude/Codex and the skills are there
```

Time budget: **under two minutes**. Not a tour of four lanes.

## Harsh critique (current product)

| Problem | Why it hurts developers |
|---------|-------------------------|
| **Lanes bury the job** | Skills / Agents / Services / Ops is a product org chart, not a setup flow. Day-0 user needs **Setup**, not a game hub. |
| **Ready is dry-run by default then “y”** | Correct for safety, but the UI does not scream “this is the main path.” Feels like admin tooling. |
| **Agents before wiring** | Running Codex with an empty library is a dead end. Order should be wire → then run. |
| **Feedback is uneven** | Some keys flash; many clicks only change selection with no LOG line. Developers need **every action to leave a trace**. |
| **Menu icons > outcomes** | Feels like a launcher skin. Developers pay for **linked agents + proof**, not glyphs. |
| **Home vs Terminal split** | Two surfaces. The default must be the setup console. |
| **Services (trading) compete with setup** | Fine later; on first open it dilutes the agentic-dev story. |
| **First-run packs without “why”** | “Pick 1–7” without “this repo looks like web-app” is random. |

## Developer journey we optimize

1. **Empty machine / empty library** → Quickstart: essentials or recommended pack → link all agents.  
2. **New repo, library exists** → Ready for this folder (apply + link).  
3. **Chaos skill dumps** → Unify plan → write.  
4. **Daily** → Doctor / status strip → one agent job → proof in `~/.kit/runs`.

## Design rules (iteration)

1. **One NEXT verb** always visible (from situation story).  
2. **Every key and click** → flash + log line (STE, ≤12 words).  
3. **Press effect** on buttons (brief inverse / pulse).  
4. **Setup before run** — if library empty or agents dark, default lane = Ops.  
5. **No silent select** — selection also flashes focus name.

## Non-goals this pass

Skill marketplace, Workshop editor, auto-start Ollama on boot, social login.
