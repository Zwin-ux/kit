# Kit 1.0: Product and Design review

Reviewed 2026-09-26 on `feat/npm-platform-dist` @ `0403696`, the tip that
stacks PR #15 with `kit init`, `kit land`, install channels, proof-integrity
fixes and npm packaging. Every finding marked **seen** was reproduced by running
the debug binary against a fresh `cargo new`-style repo; the rest are read from
code and say so.

Kit's public API is not a Rust library. It is the `kit` command, `kit.toml`,
the `--json` envelope, the receipt directory, and the `kit/<id>` branches that
`kit land` makes. That is what this review treats as the interface.

---

## 1. Product review

### User problem

A developer already has Claude Code or Codex. Asking one to do a task works,
but the result lands in their working tree, unproven: they must read the diff,
run the tests, and clean up when it is wrong. Running three agents at once is
worse, because they fight over the same checkout.

Kit's job, in one sentence: **run any coding agent in its own worktree, prove
the result with the repo's own checks, and only then put it on a branch.**
Parallel dispatch in the Control Room is the multiplier for people who already
trust that loop. It is not the first thing a new user touches.

### Why someone picks Kit instead of scripting it

A shell script can make a worktree and run `cargo test`. What it will not do,
and what Kit must do better than anyone:

1. Use whichever agent you already have, with its existing login.
2. Refuse to call work done unless the repo's own gate passed (UNCONFIGURED is
   never PASS).
3. Never touch your branch or working tree until you say `kit land`.
4. Leave a receipt that says exactly what ran, what changed, and why it failed.
5. Do all of that for eight runs at once, in one screen.

### Primary workflow (the 80% path)

```console
$ npm i -g @mzwin/kit          # or curl | sh, or cargo install
$ cd my-repo
$ kit init
Rust project (Cargo.toml). Checking the gate on your current code…
  format     cargo fmt --all --check                         ok   0.4s
  typecheck  cargo clippy --workspace --all-targets -D ...    ok  12.1s
  test       cargo test --workspace                          ok   8.7s
Wrote kit.toml. Commit it, then: kit run "describe a task"

$ kit run "add a test for parse_duration"
kit run · claude · my-repo · 01M3F4517VJ7
  … agent output …
PASS  format ✓  typecheck ✓  test ✓   2 files changed
next  kit land 01M3F4517VJ7

$ kit land 01M3F4517VJ7
Branch kit/01M3F4517VJ7 has the change. Your branch and files are untouched.

$ kit                            # Control Room: dispatch many, watch, retry fails
```

Five verbs and a default: `kit`, `init`, `run`, `land`, `receipt`, `doctor`.
That is the whole product. Nothing else ships in 1.0.

### What a new user hits today (seen)

| # | Finding | Why it matters |
|---|---------|----------------|
| P1 | `kit run "task"` defaults to **codex** even when only **claude** is installed. `doctor` knows claude is ready; `run` exits 2 with "codex is not installed". | The first real command fails for anyone who is not a Codex user. Claude Code users are half the audience. |
| P2 | `kit init` writes a gate **without running it**. On a repo whose baseline is red (seen: unformatted `main.rs`), every later run FAILs and the user blames the agent. `--check` exists but is opt-in. | A red baseline makes Kit useless and looks like Kit's fault. |
| P3 | `[firewall] mode = "block"` is parsed, documented, and set in Kit's own `kit.toml`, but **never enforced**: the runner builds `KitGate::new()` and nothing calls `screen()`; `firewall_blocks` is always empty (`crates/kit-cli/src/engine/runner.rs:274`). | A safety setting that does nothing contradicts "nothing ships unproven". |
| P4 | `kit run --help`, `kit init --help`, `kit land --help` all print the full global help. | Per-command help is the table stakes Codex and Claude Code set. |
| P5 | A PASS or FAIL ends with state lines but no next step. PASS does not say `kit land <id>`; FAIL does not say how to see why. | Both reference tools always tell you what to do next. |
| P6 | Error remedies are followed by the raw cause: "…Make a first commit, then run again: no HEAD commit in /tmp/kt". | The remedy should be the last thing read. |
| P7 | The binary crate is named `kit-cli`, which is taken on crates.io by another project. `kitrun`, `kitctl`, `kit-run`, `kit-room` are free (checked today). Library crates are publishable by default. | `cargo install <name>` is how Rust users expect to try a Rust tool. |
| P8 | Invocation aliases nobody needs: `tui`, `ui`, `control-room`, `demo`, `-d`, `receipts`, `--live`, `--no-dry-run`, `KIT_DEMO`. | Every alias is surface to document and keep. |

### Scope of this update

1. **Real CLI grammar**: per-command help, "did you mean", one exit-code
   contract, shell completions (P4, P8).
2. **Default agent = the one you have** (P1).
3. **Every run ends with a next step** (P5), remedy-last errors (P6).
4. **`kit init` proves the baseline by default** (P2).
5. **Honest firewall**: stop advertising it until it is enforced (P3).
6. **Package boundary**: one installable crate with a free name; the
   library crates are `publish = false` (P7).
7. **README for strangers**: install, the four commands above, a real
   Control Room capture. No links into `docs/dev/`.

### Non-goals (intentionally not building)

- A public Rust library. No one consumes `kit-core` outside this workspace;
  publishing it would put every `pub` field under a semver promise for no user.
- Kit's own agent loop, model keys, or provider config. Kit drives agents;
  it does not replace them.
- Per-agent settings in `kit.toml` (models, flags). `KIT_OLLAMA_MODEL` and
  `KIT_FULL_AUTO` stay as env vars until someone needs more.
- Fan-out from the CLI (`kit run --agents a,b,c`). The Control Room does
  fan-out; the CLI stays one run, scriptable.
- Firewall enforcement. Worth doing, but it is its own milestone.
- Skills marketplace, plugins, web UI.
- Security fixes and install channels: owned by the "Land the 1.0 dogfood
  branch" thread.

### Public contract (what users may depend on)

- Commands and flags in §2, with exit codes **0** proven, **1** ran but not
  proven (FAIL, UNCONFIGURED, killed), **2** could not run (usage, missing
  agent, not a git repo).
- `--json` prints exactly one envelope on stdout, `schemaVersion: 1`, errors
  included (`docs/json-contract.md`).
- `kit.toml` keys: `[gate] format | typecheck | test | extra | timeout`,
  `[gate.scope] allow | deny`. Unknown keys are an error that names the key.
- Kit never writes to your checkout except `kit init` (kit.toml) and
  `kit land --apply`.
- Receipts live at `$KIT_HOME/runs/<id>/receipt.json` and are write-once.
- Any unique prefix of a run id works wherever an id is asked for.

### Complexity check

- Could `init` be skipped by inferring the gate on every run? It already
  infers when `kit.toml` is empty, but inference cannot be reviewed or
  committed. Keep `init`; make it do one more thing well (the baseline check)
  rather than adding flags.
- Could `receipt` fold into `kit run --show`? No: `receipt show` is how you
  read a run from the Control Room or CI later. Keep, but `kit receipt` with no
  subcommand lists.
- `--drop-failing`: keep, it is the only way forward on a red baseline you
  cannot fix today; `--check` becomes the default and disappears.

---

## 2. Design review

### CLI proposal

```text
kit                                   Open the Control Room
kit --demo                            Control Room with sample runs

kit init   [--print] [--force] [--no-check] [--drop-failing]
kit run    <TASK> [-a, --agent <AGENT>] [--dry-run] [--allow-vacuous]
kit land   <RUN>  [-b, --branch <NAME>] [--apply] [--force]
kit receipt [list] [-n, --limit <N>]
kit receipt show <RUN> [--output]
kit doctor
kit completions <bash|zsh|fish|powershell|elvish>

Global:  -C <DIR>   run as if started in DIR (like git -C)
         --json     one JSON envelope on stdout
         -h, --help / -V, --version
```

Kept for compatibility, hidden from help: `run --task/-t`, `--repo`,
`init --check`, `receipts`. Everything else in P8 is removed.

### Ergonomics review

- **Guessable.** `kit run "…"` reads like `codex exec "…"`. The task is
  positional because it is the one thing every run needs.
- **`-C` is global**, as in git and cargo, instead of a per-command
  `--repo`; a user learns it once.
- **`--json` is global** and behaves the same in every command, including
  on errors.
- **Short ids.** Human output prints the shortest unique prefix of the ULID,
  never fewer than 12 characters. The first 10 characters of a ULID are the
  millisecond timestamp, so runs dispatched together share them; 8 would
  collide in every fan-out. Prefix lookup already exists. JSON keeps the full
  id.
- **Help is per command** and each ends with one example line.
- **Next step, always.** Every run ends with one `next` line:
  PASS → `kit land <id>`; FAIL → `kit receipt show <id> --output`;
  UNCONFIGURED → `kit init`; missing agent → `kit doctor`.
- **Progress without noise.** Replace `kit: state running / gating / fail`
  with one header line (`kit run · claude · repo · id`), agent output, then
  the verdict line. State changes still appear in `--json` consumers via the
  receipt.

### Type review (kit-cli internals)

| Type | Why it exists |
|------|---------------|
| `Cli { global: GlobalArgs, command: Option<Command> }` | clap derive root; `None` = Control Room. |
| `enum Command { Init(InitArgs), Run(RunArgs), Land(LandArgs), Receipt(ReceiptArgs), Doctor, Completions { shell } }` | One variant per verb, so a new verb is a compile error until handled. |
| `enum AgentArg { Claude, Codex, Grok, Ollama }` (`ValueEnum`) | Lives in kit-cli and converts to `kit_core::AgentKind`, so clap never enters the contract crate. Gives completion and "possible values" for free. |
| `enum Exit { Proven = 0, Unproven = 1, CannotRun = 2 }` | One place that maps outcomes to process codes; every command returns it. |
| `struct UserError { what, remedy }` | Expected failures print `kit: <what>. <remedy>` and exit 2, never through anyhow's `Debug`, so `RUST_BACKTRACE=1` cannot dump frames on a typo. |

### Error review

Format: `kit: <what happened>. <what to do>` on one line, remedy last,
cause folded into `what` rather than appended.

| Situation | Message |
|-----------|---------|
| No agent installed | `kit: no coding agent found. Install claude or codex, then run kit doctor` |
| `--agent codex`, not installed | `kit: codex is not installed. Use --agent claude (ready) or install codex` |
| Unknown command | `kit: unknown command 'frob'. Did you mean 'run'? See kit --help` (exit 2) |
| Repo without commits | `kit: my-repo has no commits yet. Make a first commit, then run again` |
| Gate red at `init` | `kit: test fails on your current code. Fix it, or run kit init --drop-failing` |
| `[firewall]` present | warning once per run: `kit: [firewall] in kit.toml is not enforced yet; it has no effect` |

### Default agent

Alternatives:

1. Always `codex` (today). Simple, wrong for half the audience.
2. Ask interactively. Breaks scripts and CI.
3. **First ready agent in a fixed order (claude, codex, grok, ollama)**,
   printed in the header, overridable with `--agent`. Uses the probe `doctor`
   already runs.

Decision: 3. The order is documented and stable, so scripts that care pass
`--agent`.

### Argument parsing

Alternatives:

1. Keep the hand-rolled parser and add per-command help text. Zero new
   dependencies, but completions, suggestions and consistent errors are all
   hand-written and drift (today three parsers each re-implement `--json`).
2. `lexopt` or `argh`: smaller, but no completions or suggestions.
3. **clap 4 (derive) + clap_complete.** What Rust users expect; help, errors,
   suggestions and completions come from the types above.

Decision: 3. Measure release binary size and cold start before and after;
the M0 kill criterion (cold start < 100 ms) must still hold.

### `kit init` baseline check

Alternatives: keep it opt-in (`--check`), or check by default. A check can
take minutes on a large repo, but writing an untested gate is the more
expensive failure. Decision: check by default, stream each check as it
finishes, `--no-check` to skip, `--print` never runs anything.

### Firewall

Alternatives: wire `KitGate::with_firewall_mode(config.firewall.mode)` and
screen gate commands, or stop advertising it. Screening only gate commands
(which the user wrote) protects nothing; the risk is the agent's own tool
calls, which Kit does not see. Decision: keep parsing `[firewall]` so no
existing file breaks, drop it from Kit's `kit.toml` and docs, and warn when
it is set. Enforcement returns as a milestone with its own design.

### Package boundary

- `kit-core`, `kit-agents`, `kit-gate`, `kit-tui`: `publish = false`.
  Their `pub` items are workspace seams, not a public API, so they need no
  semver promise.
- The binary crate gets a free name; the binary stays `kit`. Candidates:
  `kitrun`, `kitctl`, `kit-run`. **Needs Mazen's pick.** Recommendation:
  `kitrun` (reads as what it does, no hyphen to mistype).
- If a library is ever wanted, the shape to aim for is one type and one call,
  not the five contract modules:

  ```rust
  let receipt = kit::Run::new("add a test for parse_duration")
      .repo(".")
      .agent(kit::Agent::Claude)
      .start()
      .await?;
  if receipt.proven() { println!("kit land {}", receipt.id()); }
  ```

  Not in 1.0: nobody has asked for it.

---

## 3. Implementation plan

Thin slices, each its own commit with tests, on
`claude/kit-product-design-uz9njt`. Each slice keeps `cargo fmt`, `clippy
-D warnings` and `cargo test -p kit-cli` green.

1. **clap grammar**: `Cli`/`Command` types, per-command help, hidden
   compat aliases, `Exit` + `UserError`, `completions`. Snapshot tests for each
   `--help`; tests that every old invocation in README and tests still parses.
2. **Default agent**: pick the first ready agent; header shows it.
3. **Run epilogue**: header, verdict line, `next` line; remedy-last errors.
4. **`init` checks by default**, `--no-check`.
5. **Firewall honesty**: warning, remove from Kit's `kit.toml` and README.
6. **Package boundary**: `publish = false` on libraries; rename the binary
   crate once the name is picked; `keywords`, `categories`, `readme`.
7. **README for strangers** with a real Control Room capture.

Open questions for Mazen:

- Crate name for `cargo install` (recommend `kitrun`).
- OK to remove the invocation aliases in P8 (they stay working one release as
  hidden aliases where cheap)?
