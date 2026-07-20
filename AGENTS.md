# AGENTS.md — Living Context for Coding Agents

You are working on **Kit**.

Kit is a portable Agent Skills platform with a pixel TUI and CLI.
Offline-first local engine.

## Core Identity
- Portable skills across Claude Code, Grok Build, Codex, and others
- Pure black-and-white silhouette visual language
- GitHub sign-in for registry identity
- Offline-first local engine
- Strict validation

## Current State (Alpha v1)

**Shipped**
- Monorepo (pnpm): `@kit-skills/core`, `cli`, `tui`, `shared`, `registry-api`
- Skill schema, library, packs with `extends`, paths/link, test, doctor
- **7 packs:** essentials, web-app, library, cli-tool, api-service, full-stack, data-ml
- Pack silhouettes in `assets/pixel/packs` + TUI `PackIcon`
- TUI: splash, first-run 1–7, home, packs, library, explore, doctor, paths
- kit-idle 6-frame loop; motion primitives; Enter install; path write approval
- Point project (`o` / `KIT_PROJECT_DIR` / config) → auto-recommend
- GitHub login + Railway explore
- CI, README, LICENSE, CHANGELOG for public alpha

**Next**
1. TUI login screen
2. Workshop
3. Publish API
4. Durable catalog

## Key Documents
- README.md, ROADMAP.md, ARCHITECTURE.md, CHANGELOG.md
- docs/SKILL_SCHEMA.md, STARTER_PACKS.md, TUI_SCREENS.md, HARNESS_PATHS.md
- assets/pixel/README.md, assets/pixel/packs/README.md

## Visual Rules (Non-Negotiable)
- Pure black silhouette only for TUI assets
- High contrast; readable at 16×16
- Alpha 1 mascot: laying-down fox, no wrench, 6-frame tail wag
- Pack icons match the same language

## Working Style
- Small, clear commits
- Prefer simple solutions
- Update this file when major progress ships
