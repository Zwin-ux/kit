# Starter Packs

## Goal
Give a project a strong set of skills in one step.
A pack is a curated list of validated skills for a project type.

## v1 ship set (7 packs)

| Pack | Depends on | Best for |
|------|------------|----------|
| `essentials` | — | Default first install |
| `web-app` | essentials | Apps / sites |
| `library` | essentials | Packages / SDKs |
| `cli-tool` | essentials | CLIs |
| `api-service` | essentials | HTTP backends |
| `full-stack` | essentials | UI + API products |
| `data-ml` | essentials | Data / ML |

Each pack has a pure black 16×16 silhouette in `assets/pixel/packs/` and the TUI picker.

## Why packs matter
Single skills are useful.
Packs make Kit valuable on day one.
A new repo can install a starter pack and get agent workflows that match the work.

## Pack format

A pack is a folder with a `PACK.md` file.

```
packs/
  essentials/
    PACK.md
    skills/           # optional pack-local skills
      my-extra/
        SKILL.md
```

### PACK.md front matter

```yaml
---
name: essentials
title: Essentials
description: Core skills every coding project should have.
version: 0.1.0
tags:
  - starter
  - general
projectTypes:
  - any
skills:
  - add-readme
  - project-setup
  - code-review
  - write-tests
  - fix-bug
  - pr-ready
---
```

Stack packs **extend** Essentials so dependency skills always install:

```yaml
---
name: web-app
title: Web App
description: Full starter for apps and sites. Includes Essentials plus ship and a11y skills.
version: 0.2.0
extends:
  - essentials
skills:
  - ship-checklist
  - a11y-pass
  - pr-ready
---
```

Rules:
- `name` uses the same rules as skill names
- `title` is a short display name
- `description` is one or two sentences
- `version` is semver
- `skills` lists own skills (may be empty only if `extends` is set)
- `extends` is an optional list of base pack names; their skills merge first (de-duped)
- `tags` and `projectTypes` are optional string lists

### Skill resolution order

For each skill name in the pack:

1. `packs/<pack>/skills/<name>/` (pack-local)
2. `skills/<name>/` (shared catalog at repo root)
3. Fail with a clear error if missing or invalid

## Install vs apply

### Install pack (library)

```sh
kit pack install essentials
kit pack install essentials --force
```

- Validates the pack and every skill
- Installs each skill into `~/.kit/skills`
- Records the pack in `~/.kit/installed-packs.json`
- Offline once the pack files are on disk

### Apply pack (project)

```sh
kit pack apply essentials
kit pack apply web-app --dir ./my-app
```

- Installs the pack into the library (uses `--force` for pack skills)
- Writes `.kit/applied-packs.json` in the project
- Copies skill folders into `.kit/skills/` in the project
- Does not require a network

Agents can read `.kit/skills/` directly, or you can link into harness folders:

```sh
kit link --to claude-code --write
```

See [HARNESS_PATHS.md](./HARNESS_PATHS.md).

## CLI

```sh
kit init --pack essentials          # first-run install into library
kit init --pack web-app --apply     # install + apply to project
kit init --skip                     # skip first-run nag
kit pack list
kit pack show essentials
kit pack install essentials
kit pack apply essentials --dir .
kit pack validate essentials
```

First-run state lives in `~/.kit/config.json`.

## Official packs (v0)

| Pack | Project fit | Skills |
|------|-------------|--------|
| `essentials` | Any repo | add-readme, project-setup, code-review, write-tests, fix-bug |
| `web-app` | Apps and sites | essentials + ship-checklist, a11y-pass |
| `library` | Libraries and SDKs | add-readme, api-docs, changelog, write-tests, code-review |

## Core API

Package: `@kit-skills/core`

- `listPacks(options?)`
- `loadPack(packDirOrName, options?)`
- `installPack(packDirOrName, options?)`
- `applyPack(packDirOrName, options?)`
- `validatePack(packDirOrName, options?)`
- `resolvePacksRoot(options?)`
