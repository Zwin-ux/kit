# Quality anchors (binding)

Kit's promise is *nothing ships unproven*. Kit is held to the same promise: a change is done when it is
proven, not when it looks right. Every change is checked against the people and budgets below.

The design rules are [`docs/dev/DESIGN-tui.md`](dev/DESIGN-tui.md) (palette, floors, motion, the fox).
They are binding too. Where this file and an older spec disagree, this file wins.

## Who we build for (walk every change past these)

1. **Riya, runs Claude Code all day on a MacBook.** Wants one kit for frontend work, installed in a
   minute, and wants it gone exactly when she changes her mind. Reads every plan Kit prints before saying yes.
2. **Tomás, Windows 11, PowerShell, no WSL.** Kit has to install, draw and run agents there with no
   Unix tools. Paths have spaces and can be long. A Windows-only failure is a release blocker.
3. **Sam, uses a screen reader and `NO_COLOR`.** Every state is a word (`PASS`, `FAIL`, `RUN`,
   `UNCONFIGURED`), never a colour alone. Nothing moves when motion is off.
4. **Lee, on a light terminal over SSH in tmux.** Kit must never send a background query there, and
   the light palette must keep every state readable.
5. **A CI job.** Reads `--json`, exit codes and receipts. A shape change without a CHANGELOG entry
   breaks it. Piped output never carries colour, cursor codes or a picker.

## Budgets

| Metric | Budget | Where it is measured |
|---|---|---|
| Cold start, `kit --version` | < 100 ms mean on Linux and Windows | `rust.yml` job `startup budget`, every PR |
| Idle Control Room CPU | < 1% of one core | by hand; no CI check yet (`tasks/todo.md` P3) |
| Smallest drawn terminal | 60×12; below that, the "too small" screen | `too_small_snapshot` |
| Help and footer contract | every key visible at 80×14 | `help_overlay_shows_its_last_line` |
| Redraws while idle | only on the fox's tail frames | `fox_wags_on_the_clock_and_never_moves_the_copy` |

## Testing standard (what "tested" means here)

| Layer | Tool | Required for |
|---|---|---|
| Core logic | `cargo test` unit tests in each crate | every engine, gate, kit or config change; the failure paths too |
| CLI behaviour | integration tests in `crates/kit-cli/tests/` with fake agents and temp repos | every command, flag, exit code and `--json` shape |
| Screens | `insta` snapshots in `crates/kit-tui/src/ui/` | every TUI change; review each `.snap` diff line by line |
| Themes and motion | tests under `NO_COLOR`, `KIT_MOTION=off`, `KIT_THEME=high` | every change to a colour, a state style or motion |
| Packaging | `cargo package --workspace --locked`, `scripts/npm-smoke.mjs`, `installers.yml` | every release, and every change to `npm/`, `scripts/` or a manifest |
| Platforms | `rust.yml` on Ubuntu, macOS and Windows | every PR |

Tests are deterministic: fixed fixtures, temp directories, fake agent scripts, no network and no real
agent logins. A test that needs a real agent is a dogfood step, not a test.

## Definition of done (every PR)

- Meets the task's acceptance criteria and the budgets it touches.
- Tests at the right layer above; `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
  and `cargo test --workspace` pass; `rust.yml` is green on all three operating systems.
- For screens: the changed snapshots are in the PR and were read, at 80 columns and at the narrow floor.
- No stub, "coming soon" or unreachable key on any screen (`no_stub_or_version_copy_anywhere`).
- A public contract change (`--json`, `KIT.toml` / `kit.toml`, `kit.lock`, receipts, `kit-core` public types)
  has a CHANGELOG entry and a test that pins the new shape.
- Walk the five people above through the change and note anything that got worse.

## Credit

Kit is made by Westside Web Studio (westsidewebstudio.com). `kit --help`, the TUI help panel, the README
footer and `LICENSE` carry the credit and `© 2026 Westside Web Studio`. `kit --version` stays one bare
line (`kit X.Y.Z`): the installers and the npm smoke test compare it exactly.
