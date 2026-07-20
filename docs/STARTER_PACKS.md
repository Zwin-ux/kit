# Starter Packs

## Goal
Give a project a strong set of skills in one step.
A pack is a curated list of validated skills for a project type.

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
---
```

Rules:
- `name` uses the same rules as skill names
- `title` is a short display name
- `description` is one or two sentences
- `version` is semver
- `skills` is a non-empty list of skill names
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

Agents and future path-normalization can read `.kit/skills/` as a project skill source.

## CLI

```sh
kit pack list
kit pack show essentials
kit pack install essentials
kit pack apply essentials --dir .
kit pack validate essentials
```

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
