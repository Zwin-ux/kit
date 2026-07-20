# Changelog

## 0.1.5 — Honest ready & safe writes

### Functions (trust cut)
- **`kit ready --write` only succeeds when complete** — pack install, apply, link, and doctor must pass; incomplete → non-zero exit
- **Unify is opt-in** — never auto-runs on chaos story without `--unify`
- **`kit unify --write --link`** links keepers already in the library (not only new adopts)
- **`--link` requires `--write`** (no silent no-op)
- **Link force is honest** — ready/unify default `force: false`, `mode: symlink` (match `kit link`); pass `--force` to clobber
- **CLI exits 1** on link/import partial failures; ready prints notes
- **Atomic `installSkill`** — stage → validate → rename (no half-deleted live skill)
- **Refuse writes into home/Desktop/Downloads** unless `--force`
- **Publish gate** — `publish.mjs` runs prepare-publish and aborts if any `workspace:*` remains
- **CLI argv** — leading `--` stripped (`pnpm kit -- tui` works)

### Version
- All packages + `KIT_PACKAGE_VERSION` → **0.1.5**

```bash
npm i -g @mzwin/kit
kit ready --write
kit unify --write --link
kit tui
```

## 0.1.4 — Product stories: `kit` home + `kit ready`

### Features
- **`kit` (no args)** — situation-aware home
- **`kit ready`** — one-shot recommend → install → apply → link → doctor
- **`kit ready --unify`** — also adopt personal skill keepers

## 0.1.3 — `kit unify` (skill OS)

- Scan Claude/Codex/Grok skill dumps, normalize, dedupe, rank, adopt keepers
- Noise filter default on; `--write --link` for project wire-up
