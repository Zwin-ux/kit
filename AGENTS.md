# AGENTS.md — Living Context for Coding Agents

You are working on **Kit**.

Kit is a terminal-native Agent Skills platform.
The only interface is a pixel-art TUI.
There is no web app.

## Core Identity
- Portable skills that work across Claude Code, Grok Build, Codex, and others
- Pure black-and-white silhouette visual language
- Free accounts from day one (Railway later)
- Offline-first local engine
- Strict validation

## Current State (update this section when major progress happens)

**Done**
- Monorepo structure (pnpm)
- Skill schema parser + strict validator
- Local skill library (install / list / remove)
- TUI shell + kit-idle live player (same 6 frames as GIF)
- Library + Packs screens (offline)
- Starter packs: essentials, web-app, library + pack CLI/API
- First-run (`kit init` + TUI pack picker) and config.json
- Home shows packs, skills, apply/install actions
- Alpha 1 mascot assets + kit-idle.gif
- Basic docs and architecture

**Done (also)**
- Cross-harness path map: `kit paths` + `kit link --write`
- Skill/pack test runner: `kit test`
- Health check: `kit doctor`

**In Progress**
- Full-stack quality polish / CI

**Next Priority**
1. GitHub Actions CI (test + doctor + pack gate)
2. Workshop screen (validate / edit later)
3. Railway accounts + publish (later)

## How to Maintain This File
When you complete a meaningful piece of work:
- Update the "Current State" section
- Keep language short and factual
- Do not write long status reports here

## Key Documents
- ARCHITECTURE.md
- ROADMAP.md
- docs/SKILL_SCHEMA.md
- docs/LOCAL_LIBRARY.md
- docs/STARTER_PACKS.md
- docs/HARNESS_PATHS.md
- docs/TESTING.md
- docs/PIXEL_ART.md
- docs/TUI_SCREENS.md
- assets/pixel/README.md
- packs/README.md

## Visual Rules (Non-Negotiable)
- Pure black silhouette only for TUI assets
- High contrast
- Must read at 16×16 and 32×32
- No color inside the TUI for the mascot
- Alpha 1: laying-down fox, no wrench, 6-frame tail wag
- Same six frames drive TUI animation and `kit-idle.gif`

## Working Style
- Small, clear commits
- Prefer simple solutions
- Update docs when behavior changes
- Ask only when a decision has large product impact

Grok Build is the preferred coding agent for this repository.
