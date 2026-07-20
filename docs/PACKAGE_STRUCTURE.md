# Package Structure

## Goal
Keep the code easy to understand and easy to extend.

## Recommended Layout

```
kit/
├── packages/
│   ├── core/                 # Skill parser, validator, local library
│   ├── tui/                  # Pixel-art terminal interface
│   ├── cli/                  # Command line entry points
│   └── shared/               # Shared types and utilities
├── apps/
│   └── registry-api/         # Railway backend service
├── docs/
├── skills/                   # Shared skill catalog
├── packs/                    # Starter packs (PACK.md + skill lists)
├── assets/
│   └── pixel/                # Pixel art files
├── AGENTS.md
├── README.md
├── ARCHITECTURE.md
└── ROADMAP.md
```

## Package Responsibilities

### packages/core
- Parse SKILL.md
- Validate skills
- Manage local skill library
- Normalize paths for different agents
- Run skill tests

### packages/tui
- Draw the pixel-art interface
- Handle keyboard input
- Show all main screens
- Call the core package and the API

### packages/cli
- Provide simple commands for scripts and automation
- Example: kit install, kit validate, kit test

### apps/registry-api
- User accounts
- Skill publish and search
- Library sync
- Run on Railway

## Package Names

| Folder | npm name |
|--------|----------|
| `packages/shared` | `@kit-skills/shared` |
| `packages/core` | `@kit-skills/core` |
| `packages/cli` | `@kit-skills/cli` |
| `packages/tui` | `@kit-skills/tui` |
| `apps/registry-api` | `@kit-skills/registry-api` |

Use pnpm workspaces. Install from the repo root with `pnpm install`.

## Rules
- Do not put TUI code in the core package.
- Do not put business logic in the TUI package.
- Keep each package focused on one job.
