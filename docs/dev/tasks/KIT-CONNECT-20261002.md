# Kit account connection and owned workspace

Approved direction: user requested the full Kit CLI, with Claude underneath, provider subscription sign-in, and reuse of older Kit work. The native Mod remains a separate preserved integration.

## Outcome

`kit` opens Kit's own workspace. Users can inspect their local agents, connect Claude/Codex through the unmodified provider CLI, return to Kit, dispatch a bounded task using their own login, inspect its live output, gate and diff, and review its exact diff, then explicitly accept a proven result onto a branch through the existing `kit land` path.

## Global constraints

- Reuse Rust Kit's reducer, ratatui surface, adapter, worktree, gate, receipt and land paths. No new renderer or dependencies.
- Never read, copy, store or relay provider credentials. Native sign-in inherits terminal stdio; no custom OAuth.
- No full-auto or permission bypass. Preserve all native authentication options.
- Preserve user-owned dirty native alpha files and existing tasks/plan.md and tasks/todo.md.
- No event.rs or core/Agent trait contract changes. The user's explicit full-UI request authorizes the required narrow TUI updates.
- One animation clock, keyboard access, NO_COLOR/reduced motion, usable narrow layout. Signing in must not hide stop controls for active jobs.
- Readiness must distinguish not installed, not signed in and unchecked authentication. An explicitly signed-out agent must be rejected before a worktree/model starts.
- No real Terminal GUI capture: Computer Use denied that app. TestBackend snapshots are test evidence, never represented as live footage.

## Backlog and acceptance

1. Recover older implementation and establish current Rust baseline.
2. Add shared native login launcher and `kit connect claude|codex`; reject JSON/noninteractive calls and unsupported providers clearly. Test fixed argv, failed/cancelled login and no credential custody.
3. Add Kit Agents screen with accurate status, native sign-in and refresh. Preserve draft and completed runs across login. Prevent sign-in while live runs require stop controls. Render explicit failure/remedy, no success claim from child exit alone: re-probe.
4. Make bare `kit` open workspace without forcing legacy skill installation. Retain `kit setup` explicitly.
5. Expose Frontend, Backend, Security and Product through existing role mechanism while preserving old persona parsing/receipts.
6. Prove a real authenticated Claude run against a disposable useful website task, with an independent gate, changed files, saved receipt, unchanged source checkout, and explicit `kit land` branch acceptance. Add the signed-out failure check. No mocked real-run claim. Use `l` to review and fresh Enter to invoke unforced branch acceptance, retaining explicit cancellation and visible branch/commit or error.
7. Review the scoped diff, run applicable Rust tests, fmt and clippy, build local binary and provide a short working launch command plus exact proof boundaries.

## Verified local evidence

- Older Rust Kit was reused; native alpha and user-owned tasks were preserved.
- Native Claude subscription execution created a useful Kit get-started website. Run `01M3ZG27M1PTKNM2XT2KZQEAB2` passed an independent two-check Node gate and landed as `40b7cd50c5d83204e3c3494c7e544971aaa2d95b`.
- Chrome found real mobile overflow. A second native Claude run `01M3ZG8RQ1D5J45Y922TCRXMZB` fixed the intrinsic grid sizing, passed the unchanged gate and landed as `dc03d867cee77ce8836e191f508ef67136e83cc3`.
- Independent checks ran again on both accepted commits. Parent checkout and acceptance test/config bytes remained unchanged. Chrome verified no page overflow at 320px/390px, visible Copy buttons, working platform/provider selectors and copy feedback.
- Machine-readable run, branch, gate, artifact and browser evidence is retained in `.kit/owned-cli-proof-20261002/`. This directory is local and ignored; it contains no copied credentials.
- Native sign-in regressions prove failed status commands cannot claim Ready, foreground Ctrl-C cancels the provider without terminating Kit, and cancelled status refreshes cannot overwrite fresh login state.
- Landing regressions reject modified saved patches even with force/apply and reject inconsistent PASS receipts with failed gates. In-UI confirmation requires a fresh Enter and the exact reviewed run/diff, remains responsive, and waits for approved acceptance on quit.
- A signed-out CLI regression proves no model invocation, extra worktree, source change or HEAD movement. Scope regressions cover denied agent commits, rename sources, literal Unix backslash filenames and gate commands that erase denied edits.

## Proof boundaries

The generated website proves actual provider execution and accepted code; it is not a recording of Kit's terminal UI. Both providers were already signed in; native fresh-human login was not automated. Fake-child tests cover failed/cancelled sign-in and terminal group interruption. Security is a scoped role prompt, not a separate read-only security boundary. No new npm release, hosted service, domain purchase or public deployment of Kit was made.

Native Kit video/GIF remains unavailable because Computer Use rejected Terminal. TestBackend frames are test evidence only. Final workspace checks: 446 tests passed in both color modes; strict Clippy, fmt and build passed. Sanitized portable evidence and the accepted website are in `docs/dev/evidence/2026-10-02-owned-workspace/`.
