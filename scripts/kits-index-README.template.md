# Kit index

This repo is the list of kits that `kit search` shows and `kit add <name>` installs.
[Kit](https://github.com/Zwin-ux/kit) sets up your coding agents (Claude Code, Codex,
Grok) for one job: a kit is a folder with a `KIT.toml` that bundles the skills,
rules, MCP servers and hooks that job needs.

```console
$ kit search design
$ kit add frontend-design --global
```

Kit reads `index.toml` at the root of this repo. It caches the file under `~/.kit/index/`
and refreshes it once a day, or when you run `kit search --refresh`.

## What is here

- `index.toml`: every listed kit, each pinned to a full commit sha.
- `kits/`: the kits that ship with Kit ({KITS}). They also ship inside the `kit`
  binary, and the copy in the binary is the one it installs.

## Getting a kit listed

1. Put your kit in a public GitHub repo. `kit new my-kit` scaffolds one, and
   `kit add ./my-kit --print` shows exactly what it would install.
2. Open a pull request here that adds one entry to `index.toml`:

   ```toml
   [[kit]]
   name    = "my-kit"                 # lowercase letters, digits and dashes
   summary = "What the kit is for, in one line"
   source  = "github:you/my-kit"
   path    = ""                       # folder holding KIT.toml; empty for the repo root
   rev     = "<full 40-character commit sha>"
   level   = "index"
   tags    = ["ios", "design"]
   ```

3. A reviewer checks four things before merging:
   - Every skill is pinned to a commit.
   - Each licence is stated and allows the ship mode the kit uses.
   - Anything that runs code (hooks, MCP servers) is in the plan `kit add` shows.
   - The kit does what its summary says.

A listed kit shows as **Index** in `kit search`. Only kits maintained with Kit are
listed as `official`. To move a kit to a new version, open a pull request that
changes its `rev`.

Anyone can install a kit that is not listed with `kit add github:owner/repo`.
Kit marks that kit as not reviewed.

## Licence

MIT, see [LICENSE](LICENSE). Skills that a kit points to keep their own licences,
and each kit's `KIT.toml` lists them.
