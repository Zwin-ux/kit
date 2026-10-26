# Kit kickoff — 2026-10-26

- **Owner:** Mazen Zwin (Zwin-ux)
- **Status:** Kickoff spec for the next working session. Docs only. Nothing here was built, tagged or published.
- **Base:** `origin/main` = `c35f608` (2026-09-26), the `v2.0.0` release line.
- **Reads:** `docs/superpowers/specs/2026-09-24-kit-v2-design.md` (on trunk, revised 2026-09-26), branch `docs/kit-v2-design` (`3e7cad0`, the unrevised draft), the kit-red spec and its three reviews (`2026-10-01-kit-red*.md`, only on `archive/archive/wip-kit-20261026`), `docs/kit-red/` (same branch), PRs #37 and #38, `AGENTS.md`, `README.md`, `CHANGELOG.md`, `docs/dev/RELEASING.md`, the Cargo and npm manifests.

Kit's pitch stays: *Dispatch any agent. Trust none. Land only what's proven.*

---

## 0. Summary

1. **Kit 2.0.0 shipped on 2026-09-27**, three weeks before this kickoff. The V2 draft's "Foundation" step (merge the stack, fix land, ship a release pipeline) is done. npm `@mzwin/kit@2.0.0` is `latest`, and the GitHub Release `v2.0.0` has five archives with checksums and attestations. **crates.io never got it:** the release job failed with `A verified email address is required to publish crates to crates.io`. The README still tells people to `cargo install kitctl`, so that install line is broken.
2. **Most of the 24 Sep P0 list is already fixed on trunk.** Still open: the agent output reader stops at invalid UTF-8, and the gate's scope check fails open and cannot see committed changes. Today's stable Clippy probably fails on trunk: PR #37 had to bump `async-trait` for Rust 1.99. Two RustSec "unsound" advisories have patched versions available.
3. **Four design directions exist.** The revised V2 spec on trunk is the umbrella. kit-red, PR #37 and PR #38 are frontend and driver tracks under it. Section 2 sets out what supersedes what.
4. **D11–D13 were settled by shipping.** The owner only needs to ratify them. Owner actions are still open on crates.io (verify the email), npm (deprecate 0.1.x), the `kit` name clash, and which 2.x track comes after M0.
5. **M0 (1–2 days):** dependency and toolchain baseline, the two open proof bugs, honest install docs, then `v2.0.1` once the owner says go.

---

## 1. Where Kit stands today

### 1.1 Branches and local checkout

| Item | State | Evidence |
|---|---|---|
| Trunk | `main` on GitHub, `c35f608` "Merge pull request #36" (2026-09-26 19:58 -0700). `v2.0.0` tag on `a2938fc` (PR #35 merge). | `gh repo view Zwin-ux/kit --json defaultBranchRef`; `git -C ~/kit log -1 origin/main` |
| Local `main` | Stale: `fb0ddf1` (2026-08-14), **286 commits behind** `origin/main`. Both the V2 draft and kit-red were written against this stale base. | `git -C ~/kit for-each-ref --format='%(refname:short) %(upstream:track)' refs/heads` |
| Main checkout | `/Users/entreprenurecosystem/kit` is on `archive/wip-kit-20261026` (`7671d55`), clean. | `git -C ~/kit status --short` (empty) |
| Fetch refspec | `+refs/heads/main:refs/remotes/origin/main` only, so git never sees any other remote branch. | `git -C ~/kit config --get-all remote.origin.fetch` |
| Other worktrees | `~/kit-native-claude` (`codex/kit-native-prep-20261002`, clean). `~/kit-npm-recovery` (`codex/rust-npm-install` = `fb0ddf1`, with **untracked** `distribution/`, `scripts/pack-rust-npm.mjs` and `scripts/tests/rust-npm.test.mjs` from 2026-09-23). `~/kit/.claude/worktrees/kit-v2-design` (clean). This kickoff's worktree is `~/kit/.claude/worktrees/kit-kickoff`. | `git -C ~/kit worktree list`; `git -C <wt> status --short` |
| Remote heads | 39 branches on GitHub before this kickoff branch, mostly PR heads (`claude/*`, `feat/*`, many already merged) plus `archive/*`, `factory/*`, `ceo/*` and `keep-alive/catalog`. | `git -C ~/kit ls-remote --heads origin` |
| Branch protection | None on `main`. | `gh api repos/Zwin-ux/kit/branches/main/protection` returns 404 "Branch not protected" |

#### The "3 commits still local-only"

The commits are `7671d55`, `cb6f2bf` and `3e7cad0`. **They are not local-only.** `git log --branches --not --remotes` lists them only because the narrow fetch refspec hides every remote branch except `main`. `git ls-remote` shows each one on GitHub, at the same SHA:

| Commit | What it is | On GitHub as | Does it matter? |
|---|---|---|---|
| `7671d55` (2026-10-26) "archive: uncommitted work before removing from work Mac" | 53 files, +3,471. The kit-red spec, its three reviews and its plan, the `docs/kit-red/` build pack (V1/V2 specs, handoff, concept PNGs), the `mods/kit-red/` JS plugin draft, and `tasks/plan.md` / `todo.md`. | `archive/archive/wip-kit-20261026` (the archive prefix is doubled) | **Yes, as history.** It is the only copy of the kit-red docs and of `mods/kit-red`. Keep it, but don't merge it. PR #37 replaces `mods/kit-red` (see §2). |
| `cb6f2bf` (2026-10-02) "Checkpoint Kit workspace, native integration, and verified execution evidence" | On top of the PR #37 head (`479d53f`). 305 files, +30,549, mostly vendored third-party skills with licences. It also has **real engine fixes**: `kit-gate` scope (+199 lines), `tests/run_scope.rs` (4 tests: committed denied paths, rename sources, the gate erasing a denied change, signed-out agent), `kit-agents/src/connection.rs`, and TUI `accept.rs`. | `codex/kit-native-prep-20261002` and `archive/codex/kit-native-prep-20261002` | **Yes, as salvage.** M0-4 takes its scope tests and fix. The rest stays archived; PR #38 rejected merging it whole. |
| `3e7cad0` (2026-09-24) "docs: Kit V2 umbrella design spec" | The unrevised V2 draft. | `docs/kit-v2-design` and `archive/docs/kit-v2-design` | **No.** Trunk has the same document plus the owner's 2026-09-26 revision (`e943f4f`, `10bf6a7`, `6cbc01e`). The two copies differ by 18 diff lines, all from that revision. Don't merge it. |

The untracked files in `~/kit-npm-recovery` are **not** on GitHub. `npm/` and `scripts/npm-stage.mjs` on trunk, which shipped 2.0.0, supersede them. They can be deleted, with the owner's OK.

### 1.2 Open PRs

| PR | Title | State | Checks |
|---|---|---|---|
| #37 | Native Claude Mods: specialist workflows, red theme, and animated fox (`feat/native-claude-mods-setup-20261002`, 6 commits, +2,107/−82, 46 files) | Draft, mergeable. Its own text says to keep it draft until real-terminal QA is done. | Latest Rust and Node CI green (runs 36974896690 / 36974896696). The three runs before that were red on Rust. |
| #38 | docs: land post-2.0 ACP harness design (`docs/design-harness`, +473, 3 docs files) | Open, mergeable | Sourcery only (docs) |

`gh pr list -R Zwin-ux/kit --state open`

### 1.3 CI

- `main`: the last runs (2026-09-27, PR #36 merge) are green on `Rust` and on `CI` (Node). `gh run list -R Zwin-ux/kit --branch main --limit 6`
- Release `v2.0.0` (run 36287580003): every job passed except `crates.io`. `gh run view 36287580003 -R Zwin-ux/kit --json jobs`
- Nothing has run on `main` since 2026-09-27. PR #37 needed `async-trait` 0.1.92 because Rust 1.99 Clippy rejects code generated by 0.1.91. Trunk pins 0.1.91 (`Cargo.lock`), so **`main` is probably red on today's stable**. This is an inference; M0-1 re-runs CI to check.
- Local baseline, measured for this spec on `c35f608` with Homebrew rustc 1.96.0: `cargo build -p kitctl` OK (1m18s) and `cargo test --workspace` **404 passed, 0 failed**.

### 1.4 Published packages

| Channel | State | Evidence |
|---|---|---|
| npm `@mzwin/kit` | `latest` = `2.0.0` (the Rust launcher plus platform packages), `alpha` = `1.0.0-alpha.1`. Versions: 0.1.0–0.1.3, 1.0.0-alpha.1, 2.0.0. 0.1.x is **not** deprecated. | `npm view @mzwin/kit dist-tags versions` |
| GitHub Release | `v2.0.0` "Kit 2.0.0" (Latest), 2026-09-27. Below it are the 0.1.x Node releases. | `gh release list -R Zwin-ux/kit` |
| crates.io | **Not published.** `kitctl`, `kitctl-core`, `kitctl-gate`, `kitctl-agents` and `kitctl-tui` all return "does not exist". The failure log says `failed to publish kitctl-core v2.0.0 … A verified email address is required`. `kit` and `kit-cli` belong to other projects. | `curl -s https://crates.io/api/v1/crates/kitctl`; `gh run view 36287580003 -R Zwin-ux/kit --log-failed` |
| Kit index | `Zwin-ux/kits` is public, last pushed 2026-09-26. | `gh repo view Zwin-ux/kits` |
| Repo secrets | `NPM_TOKEN` (set 2026-09-24) and `CARGO_REGISTRY_TOKEN` (set 2026-09-26). The cargo token authenticated: the error was about the account email, not the token. | `gh secret list -R Zwin-ux/kit` |

### 1.5 Workspace

- Five crates, all `version = "2.0.0"`, edition 2024, `rust-version = "1.90"`, MIT:
  - `kitctl` (`crates/kit-cli`, binary `kit`)
  - `kitctl-core`, `kitctl-agents`, `kitctl-gate`, `kitctl-tui`
  - Internal pins are exact (`=2.0.0`), and each library keeps its Rust name through `[lib] name` (`kit_core`, …).
- Size: `main.rs` 1,313 lines, `runner.rs` 1,516, `kit-tui/src/app.rs` 3,355. The engine still lives in the binary crate (V2 §4.1 is not started).
- Legacy TypeScript 0.1 is still on trunk: `packages/`, `apps/`, root `package.json` (`"version": "0.1.0"`), the `CI` workflow (pnpm build/test/typecheck), `keep-alive.yml`, `catalog/`. No `Zwin-ux/kit-legacy` repo exists.
- `kit.toml` at the root is Kit's own gate: fmt, clippy `-D warnings`, workspace tests, 15 minutes, and `[gate.scope]`.
- `cargo audit` reports two "unsound" warnings, both with patched versions:
  - `lru` 0.18.1, RUSTSEC-2026-0253. Comes in through `ratatui-core`; fixed in ≥ 0.18.2.
  - `tokio` 1.53.1, RUSTSEC-2026-0386. Fixed in ≥ 1.53.3.

### 1.6 The 24 Sep P0 list against trunk

The V2 audit was taken on `fb0ddf1`. Re-checked on `c35f608`:

| # | Defect (V2 §3) | Trunk status | Evidence |
|---|---|---|---|
| 1 | `kit --demo` "unknown command" | **Fixed** | `main.rs:70-77`; test `cli.rs` `bare_kit_and_demo_open_the_control_room`. `kit --demo </dev/null` exits 2 with the no-terminal message. |
| 2 | Demo `r` starts a live run | **Fixed** | `app.rs:1506,1526` guard `row.demo` → `DEMO_ROW_FLASH` |
| 3 | Skill packs / `AGENTS.md` copied into worktrees | **Fixed** | CHANGELOG 2.0.0 "Breaking changes" (`KIT_SKILLS_DIR` removed) |
| 4a | Empty diff can PASS | **Fixed** | State `NO CHANGES` (CHANGELOG) |
| 4b | Vacuous gate PASS | **Fixed** | State `unconfigured`, `gatePassed: false`; `tests/run_json_stdout.rs` `vacuous_run_receipt_is_unconfigured_not_pass` |
| 4c | Scope check fails open | **Open** | `kit-gate/src/lib.rs:129-130` `scope_violations(…).unwrap_or_default()`. It reads only `git status`, so a denied path in an agent **commit** is invisible, and `porcelain_paths` skips rename sources (`lib.rs:1245-1268`). → M0-4 |
| 5 | Adapter permission modes | **Mostly fixed** | Claude runs with `--permission-mode acceptEdits` (`claude.rs:115`). Grok needs `KIT_FULL_AUTO=1` and is never the default. |
| 6 | Receipts drop new files; non-atomic writes | **Fixed** | CHANGELOG "Fixed"; `engine/store.rs:54-56` temp file + rename |
| 7 | Firewall never sees agent commands | **Open** | `lib.rs:137` `firewall_blocks: Vec::new()`. Not in M0; it belongs to the 2.1 policy work. |
| 8 | Windows `cmd /C` | **Partly open** | Grok spawns directly. Claude and Codex npm shims still go through `cmd /C` with escaping (`process.rs:34`, `claude.rs:92-135`). 2.1. |
| 9 | Ctrl-C / `q` orphans | **Fixed** | `tests/run_endings.rs` `ctrl_c_ends_the_run_as_killed_with_a_receipt`; TUI asks before quitting (`app.rs:953-972`) and `loop.rs:159-168` calls `stop_active_runs` |
| 10 | Engine errors over the TUI | **Fixed as far as checked** | No `eprintln!` in `supervisor.rs`. There is still no `tracing` or log file (V2 §10), which goes to 2.1. |
| 11 | Retry drops lineage | **Open** | No parent id in `kit-core` run types. 2.1. |
| 12 | Layering (engine in binary, protocol in UI) | **Open** | V2 §4.1 skeleton not started. Not M0 (see §2). |
| 13 | Runner spins until timeout | **Fixed** | `runner.rs:692-707` `if pipes_open` guard |
| 14 | `--json` stdout not valid JSON | **Fixed** | `tests/run_json_stdout.rs` `run_json_stdout_is_exactly_one_json_value` |
| 15 | Missing check tool counts as pass; `kit.toml` typo swallowed | **Fixed** | CHANGELOG "Fixed" (both) |
| 16 | `kit run --task help` prints help; no TTY crash | **Fixed** | clap derive. `kit run --task help --dry-run --json` runs the task (`task: help`), and `tests/no_tty.rs` covers the no-TTY case. |
| 17a | Output stream stops on invalid UTF-8 | **Open** | `kit-agents/src/process.rs:170-184` uses `while let Ok(Some(line)) = lines.next_line()`. Tokio's `Lines` returns `Err` on invalid UTF-8, so the reader task ends, the read end of the pipe drops, the rest of the output is lost, and the agent's next write gets EPIPE. → M0-3 |
| 17b | Output cap panics on a multi-byte boundary | **Fixed** | CHANGELOG "Changed" (8 MiB cap) |
| — | `--repo` nonexistent / fallback | **Fixed** | `kit -C /nonexistent-dir run … --json` returns `ok:false` with "no such folder", exit 2 |
| — | RUSTSEC-2026-0253 (`lru`) | **Open** (plus a new tokio advisory) | `cargo audit`. → M0-2 |

### 1.7 What kit-red changed compared with V2

kit-red (1 Oct) is not an engine plan. It proposes **Kit as a Claude Code mod**: a red identity on the native Claude surfaces plus a `/kit` pane over the existing CLI. Compared with V2:

| Topic | V2 (24 Sep, revised 26 Sep) | kit-red (1 Oct) and successors |
|---|---|---|
| Product centre | Cross-vendor proof, policy and landing. Standalone Control Room. | Kit inside Claude Code. Claude stays the conversation; Kit adds identity, kit choices and run evidence. |
| Accent | Fox orange `#FF5A1F` dark / `#E04814` light (shipped in 2.0.0) | Kit red `#FF5A5F` dark / `#B42332` light, plus a separate teal "live" token (`#53D6CF` / `#006B67`) |
| Architecture | Planned: `kitd` daemon, SQLite event log, local HTTP/WS API, `kit mcp` (M4) | "No daemon, HTTP server, new database". The mod calls the CLI's `--json` contract through bounded argv calls. |
| New CLI contracts | Receipts v2 in `kit-proto` (base SHA, versions, exit codes, RFC 3339) | Additive v1 fields: `doctor --passive` (never runs checks), receipt list `repoPath` / `hasMore` / `--current-repo` filter before limit, `remove --print` with no writes, a reviewed-plan digest binding preview to apply, opt-in JSONL run events |
| Land | D9: local fast-forward by default | "Create result branch" wording. Trunk already ships branch mode (`kit land` → `kit/<id>`), so **trunk diverged from D9**. |
| Status | Pillars M1–M4 are the 2.x roadmap | The 1 Oct spec marks itself superseded by `docs/kit-red/README.md`. On the same day that README narrowed the scope to "one conversation, four job kits (frontend-design, fullstack-design, backend-engineer, llm-engineer), session-only guidance" and marked the expanded V1/V2 docs **rejected**. PR #37 (2 Oct) rebuilt this as `kit claude check/install/launch` with a plugin packaged in the Rust binary (`crates/kit-cli/claude-plugin/`). `cb6f2bf` extended PR #37 further. |

Lineage, oldest first:

1. V2 draft `3e7cad0` (09-24)
2. 2.0.0 ships kits plus the existing proof engine, and V2 is revised on trunk (09-26)
3. kit-red spec and reviews (10-01)
4. kit-red README narrows the scope (10-01)
5. PR #37 native Claude Mods (10-02)
6. `cb6f2bf` checkpoint (10-02)
7. PR #38 ACP harness design (10-07)
8. Archive (10-26)

---

## 2. Reconciled direction

### 2.1 What is current

1. **The revised V2 spec on trunk is the umbrella.** It covers pitch, architecture direction, quality bar and the 2.x pillars. Its 2026-09-26 revision note is binding:
   - 2.0.0 = kits plus the proof engine that exists today.
   - M1–M4 are the 2.x roadmap.
   - `dist` and `release-plz` are not adopted; `release-npm.yml` covers those channels.
   - D11/D12 are recorded as shipped.
2. **Branch `docs/kit-v2-design` (`3e7cad0`) is superseded.** It is the unrevised draft. Don't merge it.
3. **The kit-red spec (10-01) is planning history.** Its own header says so, and the kit-red README rejects its expanded V1/V2 docs.
4. **PR #37 supersedes `mods/kit-red/`.** The JS draft lives only in `7671d55`. PR #37 is the packaged, CI-tested native path (`kit claude …`, 37 offline mod tests, green 3-OS CI). If the native track continues, it continues from PR #37.
5. **PR #38 (ACP harness) is the concrete design for V2 §5.1's ACP driver** and adds "steer" on top. Where they differ, PR #38 wins because it is newer and more specific. It replaces the PTY "Attached" view with a Steer view. It is explicitly post-2.0 and changes no behaviour.
6. **kit-red's CLI contract asks are adopted as additive 2.1 work** whatever happens to the mod. They make the CLI more honest for every client: passive doctor, repo-scoped receipt listing with `hasMore`, `remove --print`, a preview/apply digest. They fit within envelope `schemaVersion: 1`.

### 2.2 Conflicts and recommendations

| Conflict | Options | Recommendation |
|---|---|---|
| **What 2.x leads with** | (a) V2 M1 proof engine (drivers, receipts v2, `kit verify`). (b) Native Claude (PR #37 / kit-red). (c) ACP harness (PR #38: Claude, Codex and Cursor inside Kit's room). | **M0 first; it benefits all three.** Then **2.1 = proof integrity**: scope, firewall wiring, receipt fields, lineage, kit-red's additive contracts, tracing. That is what the pitch promises, and it is smaller than any driver rewrite. Choose (b) or (c) for 2.2 after 2.1. They point in opposite directions (Kit inside Claude, or Claude inside Kit), so don't run both. |
| **Accent colour** | Keep the shipped fox orange, or move the standalone TUI to Kit red | Keep fox orange until the 2.2 choice. Red applies only inside the Claude mod if (b) is chosen. |
| **Daemon / database** | V2 D10 `kitd` auto-start plus SQLite, or kit-red's "no daemon" | **Defer D10** to whenever M4 actually starts. Until then nothing adds a daemon, an HTTP server or a database. Receipt folders stay the store, which is fast enough: V2 measured 400 receipts listing in 39 ms. |
| **Land default** | V2 D9 local fast-forward, or branch mode as shipped | **Ratify branch mode** (`kit/<id>`, never touches the user's branch). It matches kit-red's "Create result branch" and the shipped CHANGELOG. |
| **Crate restructure (V2 §4.1, `kit-proto` etc.)** | Now or later | **Later.** Pick it up when a second client, the mod or `kit mcp`, actually needs a shared contract crate. It is not part of M0. |
| **Legacy TS archive (D3)** | Now, in 2.1, or never | **2.1.** Tag a snapshot and delete it from trunk. **Do not rewrite history.** `npm deprecate` 0.1.x happens in M0, since it needs only the owner's npm login. |

---

## 3. Decisions the owner must make

Each decision has a recommended default. Anything marked "ratify" already shipped; a yes records it.

| # | Decision | Recommended default |
|---|---|---|
| K1 | Canonical direction | **Ratify §2.1.** The trunk V2 spec (revised) is the umbrella. Retire `docs/kit-v2-design` and the kit-red 10-01 spec as history. PR #37 is the native path, PR #38 refines the V2 drivers, and 2.1 = proof integrity. |
| K2 (D11) | Published crate names | **Ratify as shipped:** binary crate `kitctl` (installs `kit`), libraries `kitctl-core`, `kitctl-agents`, `kitctl-gate`, `kitctl-tui`. Drop V2's proposed `kit-proto` / `kit-engine` / … names until such a crate exists. |
| K3 (D12) | Library versioning | **Ratify as shipped:** lockstep with the product, exact `=X.Y.Z` pins, no semver promise for the libraries (CHANGELOG header and RELEASING.md say so). Revisit a `0.x` split only if a library is ever meant for third parties. |
| K4 (D13) | npm package | **Ratify:** `@mzwin/kit` ≥ 2.0 is the Rust launcher. **Plus (owner, npm login):** `npm deprecate @mzwin/kit@"<1.0.0" "Kit 0.1 (Node) is retired; npm i -g @mzwin/kit@latest installs the Rust kit 2.x"` and the same for `1.0.0-alpha.1`. Leave the `alpha` dist-tag. |
| K5 | `kit` binary name clashes with KitOps' `kit` (CNCF) and with crates.io `kit-cli`, which also installs `kit` | **Keep `kit`** (published, docs, muscle memory). In 2.1, add a `kitctl` alias binary, and have `kit doctor` warn when another `kit` comes first on `PATH`. The installers already avoid touching `PATH`. Renaming the binary costs every 2.0 user and gains little. |
| K6 | crates.io channel | **Verify the email on the crates.io account** (crates.io → Settings → Profile), then re-run the failed job: `gh run rerun 36287580003 -R Zwin-ux/kit --failed`. It re-runs only the `crates.io` job at the `v2.0.0` tag, and that job publishes all five crates in order, since none are on crates.io yet. If the owner would rather not use crates.io, M0-5 removes `cargo install kitctl` from README/RELEASING instead. |
| K7 | Fate of PR #37 (native Claude Mods) and `mods/kit-red` | **Keep PR #37 in draft; don't merge it in M0.** Retire `mods/kit-red` (archive only). Decide (b) or (c) for 2.2 after 2.1. If (b), PR #37 needs the terminal QA it lists, a rebase, and its `async-trait` bump (M0-2 already takes that). |
| K8 | PR #38 (ACP harness design, docs only) | **Merge**, as a post-2.0 design that refines V2 §5.1. |
| K9 | Standalone accent: fox orange or Kit red | **Fox orange** (shipped) until K7/2.2 decides. |
| K10 | Legacy TypeScript 0.1 | **2.1:** tag `legacy-0.1-final` on the last commit that contains it, then delete `packages/`, `apps/`, `catalog/`, the Node `CI` workflow and `keep-alive.yml` from trunk, plus the root pnpm files if nothing else uses them. No `git filter-repo`, no new repo unless wanted. |
| K11 | Land default (V2 D9) | **Ratify branch mode** as shipped. |
| K12 | Daemon / SQLite (V2 D10) | **Defer** until M4 starts. No daemon, server or database before then. |
| K13 | Ship `v2.0.1` after M0 | **Yes.** It carries the M0 fixes, and if K6 is done first, `cargo install kitctl` works. Only the owner pushes the tag. |
| K14 | Repo hygiene | **Yes.** Protect `main`: require the `Rust` checks, no force pushes. Delete remote heads whose PRs merged (list in M0-1). Keep `archive/*`, `codex/kit-native-prep-20261002` and PR heads. Delete the untracked `~/kit-npm-recovery` files and remove the three stale local worktrees once nothing in them is unpushed. |
| K15 | `AGENTS.md` multi-agent ownership model (Grok owns TUI, Codex owns engine, "contract files are Claude-only", a Windows reference path) | **Replace** it with plain rules: any agent may edit any crate. A `kit-core` public type or `--json` shape change needs a CHANGELOG entry and a contract test. Point to this spec. The current table makes agents refuse legitimate fixes. |

---

## 4. First milestone (M0): foundation, 1–2 focused days

Rules for every item:

- Branch from `origin/main`.
- TDD: write the failing test first and record that it fails. A fake agent is a shell script named `claude` or `codex` on a test-local `PATH`; follow `tests/run_endings.rs:100-140`.
- One PR per item, or two at most.
- Kit's own gate must pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- No tags, publishing or version bumps without the owner.

### M0-1 Repo hygiene and real CI status (≈30 min, no code)

1. Widen the local fetch refspec and refresh:
   - `git -C ~/kit config remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'`
   - `git -C ~/kit fetch --prune origin`
2. Fast-forward local `main`, which the main checkout does not have checked out: `git -C ~/kit branch -f main origin/main`.
3. Re-run CI on trunk with today's stable: `gh workflow run rust.yml -R Zwin-ux/kit --ref main`, then `gh run list -R Zwin-ux/kit --workflow rust.yml --limit 1`.
4. List merged remote heads for K14: `git -C ~/kit branch -r --merged origin/main`. Don't delete anything without the owner.

**Accept:**
- `git -C ~/kit branch -r | wc -l` shows about 40 heads (39 plus `docs/kit-kickoff-20261026`).
- Local `main` == `c35f608`.
- The `rust.yml` result is recorded in the M0 PR description, green or with the failing step.

### M0-2 Dependency and toolchain baseline (≈1 h)

1. `cargo update -p async-trait --precise 0.1.92` or newer. This is the Rust 1.99 Clippy fix PR #37 carried.
2. `cargo update -p lru --precise 0.18.2` or newer, staying within `ratatui-core`'s requirement (RUSTSEC-2026-0253).
3. `cargo update -p tokio`, to ≥ 1.53.3 (RUSTSEC-2026-0386).
4. Upgrade the local toolchain to the CI stable (`rustup update stable`, or `brew upgrade rust`; local is 1.96.0) so local Clippy matches CI.

**Accept:**
- `cargo audit` prints no vulnerabilities and no warnings.
- fmt, clippy `-D warnings` and tests are green locally on current stable.
- `rust.yml` is green on Linux, macOS and Windows for the PR.
- `cargo package --workspace --locked` still passes (CI job `crates.io package`).

### M0-3 Agent output survives invalid UTF-8 (≈2–3 h)

Bug: `crates/kit-agents/src/process.rs:167-185` `spawn_stream_tasks` reads with `BufReader::lines()`. The first invalid UTF-8 line ends the reader. The rest of the output is lost and the pipe closes under the agent.

Fix:
- Read bytes with `read_until(b'\n', …)` and convert each line with `String::from_utf8_lossy`.
- Bound one line, for example 64 KiB. Keep the first part, add a `…[line truncated]` marker, and keep draining until the newline, so a newline-free flood cannot grow memory.
- Apply the same fix to stdout and stderr; both use the same helper today.
- `prettify_line` stays as it is.

Test, in `crates/kit-cli/tests/`, unix:
- A fake `claude` prints `before`, then `printf '\377\376 bad\n'`, then 2,000 lines `line N`, then a final `after`.
- It changes one file and exits 0. The repo has `kit.toml` `[gate] test = "git --version"`.

**Accept:**
- `kit run --agent claude --json` ends with `state` `pass`, not `error`, in under 10 s.
- `kit receipt show <id> --output` contains `after` and the U+FFFD replacement.
- The test fails on `c35f608`.
- One more unit test: a 1 MiB line with no newline gives one bounded line plus the marker.

### M0-4 Scope check sees commits and renames, and fails closed (≈3–4 h)

Bug: `crates/kit-gate/src/lib.rs`.

- `:129-130` `scope_violations(…).unwrap_or_default()` turns any git error into "no violations".
- `:1225-1243` lists only `git status --porcelain` paths. If the agent **commits** its work, which the 2.0.0 receipt diff now includes, denied paths escape scope. The run PASSes and `kit land` accepts it.
- `:1245-1268` skips the rename source path.

Fix:
- List changed paths from the run's base: `git diff --name-status -z <base>` plus untracked (`git ls-files --others --exclude-standard -z`). Include both sides of renames and copies.
- Pass the base SHA in. The runner already has it (`base.txt`).
- A scope-listing error is a FAIL with reason `scope check could not run: …`.
- Salvage, don't rewrite: `git -C ~/kit show cb6f2bf -- crates/kit-gate/src/lib.rs crates/kit-cli/tests/run_scope.rs`. Take the scope part and its tests only.

Tests (from `cb6f2bf`'s `run_scope.rs`):
- `committed_denied_paths_and_rename_sources_fail_and_cannot_land`
- `allowed_agent_commit_passes_and_lands_without_changing_parent`
- `gate_cannot_erase_a_denied_receipt_change_before_scope_check`
- New: a scope-listing failure (for example a removed `.git` file in the worktree) fails the gate.

**Accept:**
- The three salvaged tests fail on `c35f608` and pass after the fix.
- `kit land <id>` refuses the denied-commit run.
- Kit's own `kit.toml` `[gate.scope]` still passes a normal `crates/**` change (dogfood: `cargo run -p kitctl -- run --dry-run --task smoke --json` in this repo).

### M0-5 Install docs match reality (≈30 min, plus owner action K6)

- If K6 is done and `curl -s -o /dev/null -w '%{http_code}' https://crates.io/api/v1/crates/kitctl` returns `200`, nothing changes.
- Otherwise: in `README.md:280` and the `docs/dev/RELEASING.md` channel table, replace `cargo install kitctl --locked` with `cargo install --git https://github.com/Zwin-ux/kit kitctl --locked --tag v2.0.0` and say why. Add the same note to the `CHANGELOG.md` `## Unreleased` section.

**Accept:** every install line in the README works today when copied and pasted on macOS. Check the cargo line with `cargo install --git … --root /tmp/kitcheck`.

### M0-6 Docs truth (≈1 h)

1. `docs/dev/CURRENT.md` (dated 2026-08-16, says `1.0.0-alpha.1`): replace it with a short 2.0.x state that links to this spec and to `CHANGELOG.md`.
2. `AGENTS.md`:
   - Apply K15: drop the Grok/Codex/Claude ownership table, the "contract files are Claude-only" rule and the `C:/Users/mzwin/...` reference path.
   - Keep the Commands block. Remove "Legacy Node packages … 1.0 is the Rust binary".
   - Point to this spec as the current plan.
3. `CHANGELOG.md`: an `## Unreleased` section listing M0-2…M0-5.

**Accept:**
- Every command in the `AGENTS.md` Commands block runs on macOS from a fresh clone.
- `grep -n "1.0.0-alpha.1" docs/dev/CURRENT.md AGENTS.md` finds nothing.

### M0-7 Release `v2.0.1` (owner go, ≈1 h of watching)

Only after M0-2…M0-6 are merged and `main` is green on all three operating systems.

1. On a branch, run:
   - `node scripts/version.mjs --set 2.0.1 && cargo update -w && node scripts/version.mjs --check`
   - Rename `## Unreleased` to `## 2.0.1 — Proof fixes`
   - `node scripts/release-notes.mjs 2.0.1`
2. PR, green, **owner merges**.
3. **The owner pushes the tag** on that `main` commit: `git tag v2.0.1 <sha> && git push origin v2.0.1`.
4. Watch `release-npm.yml` until every job is green, including `npm-check`, `install-check` and `crates-io`.

**Accept:**
- `npm view @mzwin/kit version` → `2.0.1`.
- The `gh release view v2.0.1 -R Zwin-ux/kit` assets list five archives plus `SHA256SUMS`.
- `cargo install kitctl --locked --version 2.0.1` works, if K6 is done.

### Explicitly not in M0

- The crate restructure (V2 §4.1).
- `tracing` and a log file.
- Firewall wiring (`firewall_blocks`).
- Retry lineage.
- Windows `cmd /C` removal.
- The `kitctl` alias and the KitOps doctor warning (K5).
- Legacy TS removal (K10).
- kit-red's additive CLI contracts.
- Exit-code semantics for UNCONFIGURED. Observed: `kit run --dry-run --json` on an unconfigured repo prints `ok: true`, `state: unconfigured` and exits 0. V2 §5.2 wants a non-zero exit. Decide in 2.1.
- Anything in PR #37 or #38.

These are the 2.1 backlog, in roughly that order.

---

## 5. Run, test and release locally

All commands are taken from the repo (`AGENTS.md`, `docs/dev/RELEASING.md`, `.github/workflows/*.yml`). Run them from the repo root of a checkout of `origin/main`.

### 5.1 Build and run

```sh
cargo build -p kitctl                         # debug binary at target/debug/kit
cargo run -p kitctl -- --version              # "kit 2.0.0"
cargo run -p kitctl -- doctor                 # agents ready?, gate?, install method
cargo run -p kitctl -- --demo                 # Control Room with the fixture (needs a TTY)
cargo run -p kitctl                           # empty Control Room (first run opens setup)
cargo run -p kitctl -- show                   # installed kits
cargo run -p kitctl -- run --dry-run --task "smoke" --json   # worktree + gate + receipt, no agent
KIT_HOME=/tmp/kit-home cargo run -p kitctl -- …               # keep receipts out of ~/.kit
```

### 5.2 Test (what CI runs, `.github/workflows/rust.yml`)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                        # 404 tests on c35f608
node scripts/version.mjs --check              # version copies agree
cargo package --workspace --locked            # crates.io packaging check
cargo build --release -p kitctl && node scripts/npm-smoke.mjs   # npm shim on this host
cargo audit                                   # not in CI today; run by hand
```

Narrow runs: `cargo test -p kitctl --test run_endings`, `--test run_json_stdout`, `--test land_cli`, `--test run_diff_scope`, `--test kits_cli`, `--test marketplace_cli`, `--test no_tty`, `--test init_cli`, and `cargo test -p kitctl-tui` (insta snapshots).

Startup budget (CI job `startup budget`): `cargo build --release -p kitctl && hyperfine -N --warmup 3 --runs 50 "target/release/kit --version"`. The budget is under 100 ms.

Installers against a fake local release: `docs/dev/RELEASING.md` § "GitHub Release and installers" has the exact `install.sh` / `install.ps1` steps and the corrupted-archive refusal check.

Legacy Node (until K10): `pnpm install --frozen-lockfile && pnpm build && pnpm test && pnpm typecheck`.

PR #37 track only: `node scripts/claude-plugin.test.mjs`, `claude plugin validate ./crates/kit-cli/claude-plugin`, `claude plugin test ./crates/kit-cli/claude-plugin` (Claude Code ≥ 2.1.287).

### 5.3 Release

Full procedure: `docs/dev/RELEASING.md`. Short form:

```sh
node scripts/version.mjs --set X.Y.Z && cargo update -w && node scripts/version.mjs --check
# rename "## Unreleased" → "## X.Y.Z — <title>" in CHANGELOG.md
node scripts/release-notes.mjs X.Y.Z          # fails if the section is missing
# PR → green on 3 OSes → owner merges → owner pushes tag vX.Y.Z on that main commit
```

- Pushing the tag triggers `release-npm.yml`, which runs build (5 targets) → verify → GitHub Release (archives, `SHA256SUMS`, attestations) → npm publish (platforms first, then the launcher) → `npm-check` → crates.io (5 crates in order) → `install-check`.
- Dry run with nothing published: `gh workflow run release-npm.yml -R Zwin-ux/kit --ref main -f publish=false`.
- Re-run only the failed jobs: `gh run rerun <run-id> -R Zwin-ux/kit --failed`.

### 5.4 What needs the owner

| Need | Why | Where |
|---|---|---|
| **Verified email on the crates.io account** | Blocks every crates.io publish (2.0.0 failed on it) | crates.io → Settings → Profile |
| `CARGO_REGISTRY_TOKEN` scopes `publish-new` + `publish-update` | Already set (2026-09-26); worked as far as auth. Re-check scopes if a new error appears. | crates.io → API Tokens; `gh secret set CARGO_REGISTRY_TOKEN -R Zwin-ux/kit` |
| `NPM_TOKEN` (automation, `@mzwin` scope) | Set 2026-09-24. Granular npm tokens expire, so check before tagging. | npmjs.com → Access Tokens |
| npm login as the `@mzwin` owner | `npm deprecate` for 0.1.x / alpha.1 (K4) | local `npm login` |
| Tag push and PR merges | Every release and every merge to `main` | GitHub, account Zwin-ux |
| Branch protection (K14) | Repo admin | GitHub → Settings → Branches |
| Claude Code ≥ 2.1.287 with mods enabled | Only for PR #37 QA | local |
| Agent CLIs logged in (`claude auth status`, `codex login status`) | Live dogfood runs. Kit never stores credentials. | local |

No Apple or Windows signing is involved. The binaries are unsigned by design (RELEASING.md § Signing).

---

## 6. Risks

| Risk | Likelihood / impact | Mitigation |
|---|---|---|
| **Direction sprawl.** Four documents and two PRs pull different ways: proof engine, Kit-in-Claude, Kit-as-harness, red rebrand. | High / high: the work restarts every session. | K1. This spec is the entry point. Only one 2.2 track (K7). |
| **A false PASS through scope.** A committed denied change passes and lands. | Real today / high, because it breaks "Land only what's proven". | M0-4 first among the code items. |
| **Silent output loss on invalid UTF-8.** It can also turn a good run into an error. | Medium / medium | M0-3 |
| **Trunk red on today's stable** (async-trait / Rust 1.99 Clippy) | Likely / medium: blocks every PR | M0-1 verifies, M0-2 fixes |
| **Broken advertised install channel** (`cargo install kitctl`) | Certain until K6 / medium: first impression | K6, or M0-5 edits the docs |
| **`kit` name collision** with KitOps and crates.io `kit-cli` | Medium / medium: the wrong binary runs | K5 in 2.1. The README already warns about `kit-cli`. |
| **No branch protection.** A bad push can reach `main`, and release tags come from `main`. | Low / high | K14 |
| **Expired secrets at tag time** (`NPM_TOKEN`) | Medium / medium: a half-published release | Check expiry before M0-7. The pipeline publishes platforms before the launcher and fails loudly. |
| **Host API churn for the Claude mod** (types 2.1.277 vs docs 2.1.287; rollout switch served "off" once) | High / medium for PR #37 only | Keep PR #37 in draft and gate it on real-terminal QA (K7) |
| **Protocol churn** (ACP 1.x → 2.x, Codex app-server experimental) | High / medium for 2.2(c) | Version-pinned types and recorded-transcript contract tests (V2 §13) |
| **Stale agent instructions** (`AGENTS.md` ownership table, `CURRENT.md` from August) | High / low–medium: agents refuse or misdirect work | M0-6 / K15 |
| **Salvaging from `cb6f2bf` drags in its 30k-line vendored skills** | Medium / medium | Take only `kit-gate` scope and `run_scope.rs`, by path (`git show cb6f2bf -- <paths>`), and review the diff |
| **Legacy TS keeps costing CI minutes and confusing contributors** | Ongoing / low | K10 in 2.1. Deprecating npm 0.1.x now (K4). |
| **Scope creep in M0** | Medium / medium | §4 "Explicitly not in M0". M0 ends at `v2.0.1`. |

---

## Appendix: evidence commands used for this spec (2026-10-26)

```sh
git -C ~/kit status --short
git -C ~/kit branch -a --sort=-committerdate | head -30
git -C ~/kit log --oneline -15 --all --decorate
git -C ~/kit config --get-all remote.origin.fetch
git -C ~/kit ls-remote --heads origin
git -C ~/kit log --oneline --branches --not --remotes
git -C ~/kit worktree list
gh pr list -R Zwin-ux/kit --state open
gh run list -R Zwin-ux/kit --limit 5
gh run list -R Zwin-ux/kit --branch main --limit 6
gh run view 36287580003 -R Zwin-ux/kit --json jobs
gh run view 36287580003 -R Zwin-ux/kit --log-failed
gh release list -R Zwin-ux/kit --limit 5
gh secret list -R Zwin-ux/kit
gh api repos/Zwin-ux/kit/branches/main/protection
npm view @mzwin/kit dist-tags versions
curl -s https://crates.io/api/v1/crates/kitctl
cargo build -p kitctl && cargo test --workspace && cargo audit
target/debug/kit --version; target/debug/kit --demo </dev/null
target/debug/kit -C /nonexistent-dir run --task x --dry-run --json
target/debug/kit run --task help --dry-run --json      # in a scratch git repo with KIT_HOME set
```
