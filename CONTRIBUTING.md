# Contributing

Thanks for helping with Kit.

## Setup

```bash
pnpm install
pnpm build
pnpm test
pnpm typecheck
```

## Run

```bash
pnpm tui
pnpm kit -- --help
pnpm kit -- doctor
```

Windows shims (from repo root after build):

```powershell
.\kit.cmd --help
.\kit.cmd tui
```

## What to change

| Path | Role |
|------|------|
| `skills/` | Skill content people actually use |
| `packs/` | Starter pack definitions |
| `packages/cli` | CLI |
| `packages/tui` | Pixel TUI |
| `packages/core` | Engine |

Prefer small commits. Keep the public README product-focused — put engineering detail in `docs/dev/`.

## More

- [docs/dev/](docs/dev/) — architecture, schema, roadmap
- [CHANGELOG.md](CHANGELOG.md)
- CI: [`.github/CI.md`](.github/CI.md)
