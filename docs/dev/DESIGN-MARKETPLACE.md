# Marketplace: `kit search`, `kit sync`, `kit new`, `kit add github:`

Status: design for v0.1, 2026-09-26. Extends `DESIGN-KITS.md` §4 and §6.
Decisions already taken there: the marketplace starts as a public Git
repo of kits (`Zwin-ux/kits`), a server comes later and serves the same
`index.toml`, kits point at upstream skills by pinned commit.

## Product review: what each command is for

| Command | The job | Who |
|---------|---------|-----|
| `kit search [words]` | Find a kit for my job without leaving the terminal | Anyone who ran `kit setup` and wants more than the starter five |
| `kit add github:you/kit` | Install a kit someone linked me | Anyone with a link from a README, a tweet, a teammate |
| `kit new <name>` | Save my own setup as a kit I can share | The person whose agent setup works and whose team keeps asking for it |
| `kit sync` | Make this machine match `kit.lock` | A new teammate after `git clone`; me on a new laptop; CI |

What makes each one good, measured against Codex and Claude Code:

- **One command, no config.** `kit search` works on first run; the index is
  fetched and cached without a setup step. `kit sync` needs no arguments in
  a repo with a `kit.lock`.
- **Offline still works.** The starter kits ship in the binary, so search
  always shows them; a stale index is used with a warning rather than failing.
- **Nothing new to trust.** Everything a remote kit does goes through the
  same plan screen and the same yes as `kit add`, with its source level
  above the plan. A kit from a repo nobody reviewed says so.
- **Exactly the same bits.** `kit.lock` records a pinned source for each
  kit and a content hash for each skill; `kit sync` refuses to install
  anything whose content differs from those hashes.

## Design review: the ideal terminal sessions

### Finding a kit

```console
$ kit search design
frontend-design    Official   UI, design systems, accessibility, browser testing   installed
fullstack-design   Official   Frontend Design plus APIs, data, end-to-end tests
apple-design       Index      HIG and platform design review

next      kit show <kit>   ·   kit add <kit> --global
```

Ranking: exact name, then name prefix, then name contains, then title and
tags, then description. Words are ANDed. No words lists every kit.

```console
$ kit search quantum
No kits match "quantum". kit search lists all 9.
Make your own: kit new quantum
```

### Installing from a link

```console
$ kit add github:ana/ios-kit --global
Fetching  ana/ios-kit … 1a2b3c4
iOS 0.3.0  →  Claude Code, all projects
Direct source, not reviewed by Kit  (github:ana/ios-kit@1a2b3c4)
… the usual plan …
```

`github:owner/repo`, `github:owner/repo/path/to/kit`, and `@rev` (a
commit, branch or tag) at the end. Without `@rev` Kit uses the default
branch and pins the commit it got into `kit.lock`.

### Saving my own kit

```console
$ kit new ios-team
Created ios-team/
  KIT.toml                        what the kit installs (edit this)
  RULES.md                        added to CLAUDE.md and AGENTS.md
  skills/ios-team-house-style/    a skill that lives in the kit
  README.md                       how others install it

Try it    kit show ./ios-team
          kit add ./ios-team --print
Share it  push the folder to GitHub, then: kit add github:<you>/ios-team
```

`kit new ios-team --from frontend-design` starts from an existing kit
(bundled, index or github) with a new name, so "frontend design, but with
our house rules" is a copy and an edit.

### Joining a team

```console
$ git clone git@github.com:shop/web && cd web
$ kit sync
kit.lock pins frontend-design 0.1.0 for this repo  →  Claude Code, Codex
Missing   4 of 16 (3 skills, 1 rules block)
… plan of what is missing …
Continue?  [y] install  [n] cancel
Done. This repo matches kit.lock.

$ kit sync
In sync: kit.lock pins 1 kit and everything is in place.

$ kit sync --check        # CI: exit 1 when anything is missing (2 = could not run), writes nothing
```

New machine, global kits: `kit sync --global --from ~/dotfiles/kit.lock`.

## The index

`index.toml` at the root of the index repo:

```toml
schema = 1

[[kit]]
name    = "apple-design"
summary = "HIG and platform design review"
source  = "github:Zwin-ux/kits"
path    = "kits/apple-design"         # folder in source holding KIT.toml
rev     = "4b7a1d2…"                  # full commit sha
level   = "index"                     # "official" or "index"
tags    = ["ios", "macos", "design"]
```

- Where it comes from: the `KIT_INDEX` environment variable
  (`github:owner/repo`, a folder holding `index.toml`, a file, or `off`),
  else `github:Zwin-ux/kits`. A local folder or file is a private team
  index. (A `config.toml` setting can follow; `kit setup` owns that file.)
- Fetched with `git` over HTTPS like skills (no new network code), cached
  under `~/.kit/index/`, refreshed when older than a day or with
  `kit search --refresh`. A refresh that fails falls back to the cache and
  says how old it is.
- `level = "official"` is honoured only from the default index. Any other
  index can list kits, but they show as Index, never Official.
- A bundled kit wins over an index entry with the same name, so the
  starter kits work offline and cannot be shadowed by an index.

## What is deliberately not built

- **No server.** The same `index.toml` can be served later; clients keep
  working.
- **No publishing command.** Listing a kit is a PR to the index repo;
  `kit new` prints how. A `kit publish` would need accounts.
- **No `kit update`.** Moving a kit to a newer pin is its own design
  (diff, re-ask on code changes) and belongs to the product thread.
- **No ratings, downloads or stars.** Nothing phones home.
- **Remote kits read nothing outside themselves.** `KIT.toml` validation
  now refuses a rules file or skill path that is absolute or climbs out
  with `..`; before this, a kit from a stranger could name
  `~/.ssh/id_rsa` as its rules file and have it copied into `CLAUDE.md`
  (after the yes, but unnoticed in a one-line plan).
- **`kit sync` does not remove kits** that are installed but not in the
  lock. It only adds what is missing; `kit remove` stays the one way to
  take things out.
