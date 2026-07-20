# Kit Architecture

## Goals

- Become the default way people create, share, and run portable Agent Skills
- Stay 100% terminal-native (pixel-art TUI is the product)
- Free account system from day one for distribution and future expansion
- High-quality, opinionated, and delightful

## High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                        Kit TUI                              │
│              (Pixel-art, keyboard-first)                    │
└────────────────────────────┬────────────────────────────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
     ┌────────────────┐ ┌────────────┐ ┌────────────────┐
     │  Local Engine  │ │   Auth     │ │    Registry    │
     │  (TypeScript)  │ │  Client    │ │    Client      │
     └────────────────┘ └────────────┘ └────────────────┘
              │                                 │
              │                                 ▼
              │                        ┌────────────────┐
              │                        │ Railway Backend│
              │                        │ - Accounts     │
              │                        │ - Skills DB    │
              │                        │ - Search       │
              │                        │ - Social       │
              │                        │ - Sync         │
              │                        └────────────────┘
              ▼
     Local skill library + cache
```

## Local Engine (`@kit-skills/core` or `kit`)

Responsibilities:
- Parse and strictly validate skills
- Normalize discovery paths across harnesses (Claude Code, Grok Build, Codex, etc.)
- Manage local skill library
- Safe execution of skill scripts
- Test runner (run skill against a fixture directory or current repo)
- Offline-first

## Pixel-Art TUI

- Built for delight and speed
- Limited color palette + high-quality pixel sprites (mascot, skill cards, icons, status)
- Screens: Home, Explore, Workshop (create/edit/test), Library, Profile, Settings
- Auth flows happen inside the TUI
- Animations used sparingly but intentionally (install success, publish, level-up style feedback)

## Account & Backend (Railway)

Free tier with mandatory accounts for publishing and sync.

Core services:
- Auth (email magic link or GitHub, device-code style for TUI)
- User profiles
- Skill registry (metadata + tarball/storage)
- Search & tags
- Follow / collections
- Library sync
- Rate limits & basic moderation

Designed so we can later add:
- Teams / orgs
- Private registries
- Verified publishers
- Usage analytics
- Billing (optional paid tiers)

## Skill Format

Strict, versioned extension of the emerging open Agent Skills standard (SKILL.md family).

Required:
- name, description, version
- compatibility declarations
- clear entrypoints

Strong validation on publish and on install.

## Mascot & Visual Identity

**Kit** — a small, confident fox (baby fox = kit) holding a wrench.

- Pixel art only
- Limited palette (oranges, cream, deep blue/navy, black, small accent)
- Used in: GitHub avatar, TUI splash, loading states, empty states, success animations, skill cards

## Grok Build Integration

Grok Build is the primary coding agent for developing Kit itself.  
The repo will contain clear instructions and skills so Grok Build (and later other agents) can work on Kit effectively.

## Design Principles

1. Product surface is the CLI and pixel TUI
2. Pixel art is part of the product, not decoration
3. Free + accounts = distribution
4. Strict validation over flexibility
5. Offline-first local engine
6. Delight without sacrificing speed or clarity
