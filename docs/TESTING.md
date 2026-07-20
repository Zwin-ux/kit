# Testing Skills and Packs

## Goal
Catch broken skills before install or apply.
Keep official packs green.

## Commands

```sh
# All official packs
kit test --all-packs

# One pack
kit test essentials
kit test pack web-app

# One skill folder
kit test ./skills/add-readme
kit test skill ./skills/fix-bug
```

## What is checked

### Skill
- SKILL.md present and valid (schema v0)
- Non-empty instruction body
- Compatibility list present
- Optional: `tests/` folder (info/warn only)

### Pack
- PACK.md valid
- Every listed skill resolves and passes skill tests

## Doctor

```sh
kit doctor
kit doctor --dir ./my-app
```

Checks:
- Kit home + config
- First-run state
- Library skill count
- packs/ and skills/ discovery
- Harness path map
- Mascot frames + kit-idle.gif

Exit code `1` when any check fails.
Warnings alone still exit `0`.

## CI suggestion

```sh
pnpm build
pnpm test
pnpm kit -- test --all-packs
pnpm kit -- doctor
```
