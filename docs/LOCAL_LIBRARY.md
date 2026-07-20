# Local Skill Library

## Goal
Store skills on disk for offline use.
Support install, list, and remove.

## Location

Default home:

```
~/.kit/
  config.json           # first-run + preferences
  library.json
  installed-packs.json
  skills/
    <skill-name>/
      SKILL.md
      ...
```

Override the home directory with `KIT_HOME`.

## Rules
- Validate a skill before install.
- Copy the skill folder into `~/.kit/skills/<name>/`.
- Keep metadata in `library.json`.
- Do not require network access.

## Core API

Package: `@kit-skills/core`

- `installSkill(sourceDir, options?)`
- `listSkills(options?)`
- `removeSkill(name, options?)`
- `getKitHome()`
- `getSkillsDir()`

## CLI

```sh
kit validate ./skills/add-readme
kit install ./skills/add-readme
kit install ./skills/add-readme --force
kit list
kit remove add-readme
kit pack install essentials
kit pack apply essentials --dir .
```

See also [STARTER_PACKS.md](./STARTER_PACKS.md) and [HARNESS_PATHS.md](./HARNESS_PATHS.md).

```sh
kit paths
kit link --to claude-code --write
```
