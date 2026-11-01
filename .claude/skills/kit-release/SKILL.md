---
name: kit-release
description: Prepare and ship a Kit release (vX.Y.Z) to the GitHub Release, npm and crates.io. Use when asked to cut, bump, tag or check a Kit release, or to read a failed Release npm run.
---

# Kit release

The full procedure is [`docs/dev/RELEASING.md`](../../../docs/dev/RELEASING.md). This is the short path
and the rules that keep it safe.

## Rules
- **The owner (Mazen) pushes the tag.** A tag push publishes to npm, crates.io and the GitHub Release,
  and npm and crates.io versions can never be reused. Prepare everything, then ask. Never tag, never run
  **Release npm** with `publish` checked, never `npm publish` or `cargo publish` by hand.
- Never print or echo `NPM_TOKEN` or `CARGO_REGISTRY_TOKEN`. `gh secret list -R Zwin-ux/kit` shows the
  names and dates only, and that is all you need.
- `main` must be green on Ubuntu, macOS and Windows (`rust.yml`) at the commit that gets the tag.

## The bump PR
1. Branch from `main`: `git switch -c release/vX.Y.Z origin/main`.
2. `node scripts/version.mjs --set X.Y.Z`, then `cargo update -w`, then `node scripts/version.mjs --check`.
3. In `CHANGELOG.md`, rename `## Unreleased` to `## X.Y.Z — <title>`. The release fails without it.
4. `node scripts/release-notes.mjs X.Y.Z` and read the notes it prints.
5. Update the date and release line in `docs/dev/CURRENT.md`.
6. Push, open the PR, wait for `rust.yml` on all three OSes and the `crates.io package` job.

Optional dry run of the whole pipeline (publishes nothing): Actions → **Release npm** → Run workflow on
the PR branch with `publish` unchecked.

## After the owner merges and tags
Watch `release-npm.yml` to the end: `build` → `verify` → `github-release` → `publish` → `npm-check`,
`crates-io`, `install-check`. Then confirm from outside:

```sh
npm view @mzwin/kit dist-tags                     # latest = X.Y.Z (a prerelease moves only its own tag)
curl -s -A kit-check https://crates.io/api/v1/crates/kitctl | python -I -c "import json,sys;print(json.load(sys.stdin)['crate']['max_version'])"
gh release view vX.Y.Z -R Zwin-ux/kit
```

A rerun of a failed job skips what is already published (`gh run rerun <id> -R Zwin-ux/kit --failed`).
Report which of the three channels you saw with your own command, not from the run summary alone.
