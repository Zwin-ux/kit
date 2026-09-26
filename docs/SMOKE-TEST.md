# Kit smoke test

A check anyone can run on a clean machine in about five minutes, before or after a release. Every step says what you should see. If a step shows something else, that is a bug: [open an issue](https://github.com/Zwin-ux/kit/issues) with the command and its output.

You need git, and at least one of Claude Code, Codex or Grok installed and logged in (step 8 only). Nothing here changes files outside the throwaway folder, except `~/.kit` (Kit's own record) and `~/.local/bin/kit`.

## 1. Install

macOS or Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/Zwin-ux/kit/main/scripts/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/Zwin-ux/kit/main/scripts/install.ps1 | iex
```

You should see the archive it downloads and `installed kit <version> to <folder>`. It checks the archive's SHA-256 first and refuses to install on a mismatch. If that folder is not on your `PATH`, it prints the line to add.

Other ways: `npm install -g @mzwin/kit`, or `cargo install kitctl --locked`.

## 2. It runs

```sh
kit --version
```

Prints `kit <version>`, the version from step 1.

## 3. It sees your agents

```sh
kit doctor
```

Lists Codex, Claude Code, Grok and Ollama as `ready`, `not ready` (installed but logged out) or `missing`, each with what to do next. Exit code 0.

## 4. Browse kits

```sh
kit show
kit show essentials
```

The first lists the starter kits with one line each. The second lists every skill Essentials installs, each with its source repo, the commit it is pinned to, and its licence.

## 5. Set up a throwaway repo

```sh
mkdir kit-smoke && cd kit-smoke && git init
git commit --allow-empty -m init
kit setup
```

`kit setup` finds your agents, asks which to set up, which kit and where (pick Essentials and "this repo"), then shows the plan: every file it will write and every skill with its pin. Nothing is written until you say yes. After yes you should see `Done.`, where the record is, and `undo  kit remove essentials`.

Check: `git status` shows only the new skill folders (for Claude Code, `.claude/skills/`), the rules file (`CLAUDE.md` or `AGENTS.md`) and `kit.lock`.

## 6. It knows what it installed

```sh
kit list
kit doctor
```

`kit list` shows `essentials` with your agent and `ok`. `kit doctor` now has a `kits:` section with `ok` lines. Edit one installed `SKILL.md` by hand and run `kit list` again: it reports the change instead of hiding it.

## 7. The Control Room

```sh
kit --demo
```

Opens a table of sample runs (no agent starts): running, gating, passed and failed rows, a failed row showing its first error line. `?` shows the keys, `q` quits. Resize the window: the table stays readable down to 60 columns.

## 8. A real run (uses your agent)

```sh
kit run "add a README that says hello"
```

Kit makes a git worktree, runs your agent there, then runs this repo's checks. It ends with a verdict and a receipt id. `kit receipt show <id>` shows the diff, what ran and how long it took. Your own working tree is untouched.

## 9. Undo

```sh
kit remove essentials
git status
```

Kit removes the skills and the rules block. The one skill you edited by hand in step 6 is kept, and Kit says so (`kept … changed after install; left in place`). `git status` shows only that folder. `kit list` says `No kits installed.`

## 10. Clean up

```sh
cd .. && rm -rf kit-smoke
```

To remove Kit itself: delete the `kit` binary from step 1 and the `~/.kit` folder.
