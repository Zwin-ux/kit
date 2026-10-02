# Kit private alpha — 2.0.0-alpha.4

This is an unpublished owner-review candidate. Four roles, four skills each, native Claude, Kit Red and the current fox. The previous alpha demonstrated four native role tasks and three isolated writer worktrees. This revision repairs lost completion capture when status fails or foreground agents disappear, using observed native identity. Native appearance and visible draft routing still require owner acceptance. Headless execution evidence is recorded separately in NATIVE-RELEASES.md.

## Open the preview

Use an allowed terminal and your already-authorized Claude installation. Current acceptance uses Claude 2.1.288; the native minimum remains 2.1.287. A compatible version number does not prove Mods rollout access. If native Kit cannot load/register, record preview unavailable; do not bypass host restrictions or count a screenshot from another renderer as native acceptance.

For the local release archive:

```sh
(
  set -e
  # Run from the directory containing the archive and its .sha256 file.
  shasum -a 256 -c kit-claude-2.0.0-alpha.4.tar.gz.sha256
  mkdir kit-alpha-review
  tar -xzf kit-claude-2.0.0-alpha.4.tar.gz -C kit-alpha-review
  cd kit-alpha-review
  shasum -a 256 -c FILES.sha256
  claude --plugin-dir "$PWD/kit"
)
```

Use a fresh extraction folder; choose another name if `kit-alpha-review` exists. The archive contains the complete `kit` payload, this START-HERE guide, BUILD.json and FILES.sha256. BUILD.json records the base commit, candidate version and uncommitted-source status. The per-file checksums identify the actual candidate bytes; the base commit alone does not identify uncommitted changes.

Alternatively launch the already-exported folder explicitly:

```sh
claude --plugin-dir /absolute/path/to/extracted/kit
```

Claude owns host agreements, sign-in, permissions, theme selection and task submission. No global installation or persistent plugin registration is needed.

## Five-minute visual review

**Do not send a model request in this walkthrough.**

| Time | Try | Look for |
| --- | --- | --- |
| First minute | In the preview session opened above, choose Kit Red in `/theme` | It still feels like Claude; conversation and ordinary typing are comfortable |
| Second minute | `/kit open`; inspect Frontend, Backend, Security, Product | Four understandable skills per role; selection starts nothing; draft action reachable; fox looks intentional at actual size |
| Third minute | Type `Review the existing login form; do not change files.` Prepare Frontend twice; do not send; press Escape | Original text retained, one visible agent mention, repeated preparation makes no duplicate, prompt focus usable |
| Fourth minute | Resize to roughly 80 columns, then a short window and 40 columns | Conversation, prompt and actions stay usable; no stranded/truncated controls; native content keeps its space |
| Fifth minute | Pause/hide the fox; try your usual theme; close Kit and type normally | Motion stops, controls describe their effect, text/focus readable, ordinary Claude interaction remains intact |

Record the installed Claude version, terminal size, archive checksum and actual native view. Feedback: **Would you keep Kit open while coding? What feels crowded, unclear or distracting?** A rejected visual goes back to a measured small correction, not another dashboard design.

## After visual approval

Use a disposable repository and an explicitly scoped task to check real specialist routing, writer worktree isolation, inspectable artifact/checks and Needs review status. Send an exact-result Security review separately and inspect its evidence. Loading four agents/sixteen skills is not this execution proof. Public beta additionally needs all four roles, the native layout/theme/input matrix and the actual hooks test runner.

Draft actions append and never replay or replace task text. They refuse an observed existing agent mention. The host has no atomic draft/session guard: newer typing survives, but an already-submitted fill may append after a reset. Always inspect the visible draft before sending. Captured task/answer mentions remain escaped JSON data; decoded content is unchanged.

## Leave, update or recover

End Claude normally and launch it without `--plugin-dir` to use ordinary Claude. For an update, retain this folder, extract the next candidate to a fresh folder and launch its path. Do not overwrite a running candidate. Local task/results reset when the Mod/session ends; save any needed artifacts first. Choose your preferred native theme through `/theme`. No separate daemon, updater or settings cleanup is required.

Known release limits: actual native visual/draft/result-review acceptance is pending. The source and native-host checks are recorded in NATIVE-ENVIRONMENT.md; close/reset behavior has source-level coverage because the host forbids direct test close calls absent from the production call list. Four real role tasks and writer isolation were checked on alpha.1, not this archive; the unrelated standalone TUI retains one reproduced NO_COLOR failure after 411 passing workspace tests. Linux, Windows, Desktop and future Claude versions have no native acceptance claim from this alpha. Nothing has been publicly released by this preparation.
