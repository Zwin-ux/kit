---
name: kit-ci-triage
description: Get build and test proof for Kit and diagnose CI failures, especially when working on Windows. Use before claiming a Kit change works, when a check fails on a PR, or when a local cargo build behaves oddly on Windows.
---

# Kit CI triage

## Where proof comes from
| Check | Workflow / job | Runs on | Proves |
|---|---|---|---|
| fmt, clippy `-D warnings`, `cargo test --workspace` | `rust.yml` → `quality` | ubuntu, macos, windows | every crate, the TUI snapshots, the CLI integration tests |
| Packaging | `rust.yml` → `crates.io package` | ubuntu | `cargo package --workspace --locked`; version copies agree |
| Cold start | `rust.yml` → `startup budget` | ubuntu, windows | `kit --version` mean under 100 ms (hyperfine) |
| npm launcher | `rust.yml` → `npm package` (`scripts/npm-smoke.mjs`) | 3 OSes | the packed tarballs install and `kit` runs through the shim |
| Installers | `installers.yml` | 3 OSes | `install.sh` / `install.ps1` against a local release |
| Legacy 0.1 Node | `ci.yml` | ubuntu | only the TypeScript tree under `packages/` (path-filtered) |

`rust.yml` is path-filtered: a docs-only PR runs no Rust checks. That is not a pass for code; it means
there was no code. Read results with `gh pr checks <n> -R Zwin-ux/kit` and
`gh run view <run-id> -R Zwin-ux/kit --log-failed`. Use the app's PR status tools rather than polling.
Say which rows ran and on which OS before calling anything verified.

## Known traps
- **Windows path length:** a checkout in a deep folder makes `link.exe` fail with
  `LNK1104: cannot open file … build_script_build-….exe`. Set `CARGO_TARGET_DIR` to a short folder.
- **`NoDefaultCurrentDirectoryInExePath=1`** (set on the owner's hardened machine): `cmd /C name.cmd`
  no longer finds `name.cmd` in the current folder. Tests and checks must write `.\name.cmd`.
- **CRLF:** `core.autocrlf=true` checks files out with `\r\n`. Strip `\r` before grepping, and assert the
  count of any scripted replacement. Insta snapshots compare with line endings normalised.
- **Insta:** a changed screen writes `.snap.new` and fails. Run `INSTA_UPDATE=always cargo test -p kitctl-tui`
  only after reading the diff, then commit the `.snap` and read `git diff` of it line by line.
- **Clippy on newer stable:** CI uses `dtolnay/rust-toolchain@stable`, which can be newer than your
  local toolchain. A lint that only CI reports is real; fix it, don't pin the toolchain down.
- **Encoding:** read and write text as UTF-8 explicitly (`encoding="utf-8"` in Python). `©`, `·`, `→`
  and box-drawing characters appear in Kit's output and docs.

## Failure loop
1. Find the failing job and the first real error line, not the summary.
2. Reproduce locally if the toolchain allows; otherwise reason from the log and push a fix to the PR branch.
3. Add or fix the test that would have caught it.
4. Push and read the new run on every OS before saying it is fixed.
