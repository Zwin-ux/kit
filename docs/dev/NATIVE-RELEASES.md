# Kit native release stages

Approved audience decision, 2026-10-02: **private alpha for the owner → public beta → stable**. The user approved this release plan and swarm execution. Prepare and test the product through these stages; public publication remains a final action after the concrete candidate passes its promotion gates.

Kit should feel like Claude with a small useful addition: Frontend, Backend, Security and Product, four real skills each, the native conversation and agents view, Kit Red, and a restrained fox. Release stages improve confidence in that same product. They do not add another dashboard, dispatch system, model, MCP, account flow or onboarding game.

## Audience and promise

| Stage | Who receives it | What they should get | Distribution |
| --- | --- | --- | --- |
| Alpha candidate | Owner only | A reproducible preview to inspect and critique; native acceptance explicitly incomplete | Local folder/archive and checksum; no pushed release tag |
| Accepted private alpha | Owner only | Comfortable native interaction and at least one evidenced task/review loop | Same explicit local loading; revised alpha snapshots as needed |
| Public beta | People who knowingly opt into testing | A useful, tested native Mod with stated compatibility, known limitations and recovery instructions | Mod-only GitHub prerelease/archive, after approval |
| Stable | Public users | Repeatable install/update, dependable daily task/review behavior and a verified support matrix | Mod-only GitHub release, after approval |

The alpha is where the owner can reject the feel or fox without a public audience. The beta is where clean installations and returning users expose workflow problems. Stable means the supported experience meets the promises below; it does not mean supporting every platform or Claude version.

## Version and channel

The native plugin now uses **`2.0.0-alpha.4`** in `.claude-plugin/plugin.json` and `provenance.json`. It retains the existing 2.0.0 baseline rather than renumbering unrelated Kit components. Retained earlier candidates: `2.0.0-alpha.1`, `2.0.0-alpha.2` and `2.0.0-alpha.3`. Planned sequence: further alpha revisions as needed, `2.0.0-beta.1` for the first public preview, then `2.0.0` after stable acceptance. Each distributed candidate must have immutable bytes, checksums and its actual source revision; do not overwrite a published artifact or retarget its tag.

The Kit CLI's Cargo version is independent and remains 2.0.0. `scripts/version.mjs` updates Cargo/internal dependencies only; it is not the native-plugin version tool. The existing package regression deliberately checks the chosen plugin prerelease and manifest/provenance agreement. Update those two metadata files and their assertion together when cutting a later candidate. Do not run the workspace version setter for a Mod-only release.

If a native `2.0.0` artifact has already been publicly distributed by publication time, do not reuse that version with new contents. Check existing native release history and select the next compatible plugin version. No registry/release-history check or publication is claimed here.

Reserve **`kit-claude-2.0.0-beta.1`** and **`kit-claude-2.0.0`** as Mod-only tag names. The existing `.github/workflows/release-npm.yml` listens to every `v*` tag and publishes the entire Kit product to public npm and crates.io. An alpha npm dist-tag is still public. That workflow also publishes an existing draft release. It is unsuitable for this private alpha or the first Mod-only beta.

Mod-only public distribution should initially reuse `scripts/sync-claude-plugin.mjs`, the full licenses/provenance and a reviewed archive. A GitHub archive is sufficient; no new registry, global installer, updater or publishing workflow is needed to try the product. An artifact release does not imply Claude marketplace listing or persistent plugin registration.

## Alpha review

The current candidate has verified source regressions, package integrity, real local agent/skill loading, generated-type checking and native Rust checks. An enabled Claude 2.1.287 runner now passes 43 native fixtures, including repaired engine composition. Close/reset and structured close-denial behavior retain source-level coverage; host rules prevent direct test close calls absent from the production call list. All four roles completed native tasks on the original alpha.1 archive, and the three writers used native worktrees. Native painting, focus, visible draft routing and the Mod captured-result review loop remain unverified on this revision. **A successful loader log is not owner acceptance.**

Use [the five-minute alpha guide](NATIVE-ALPHA.md) in an allowed, already-authorized terminal. Start with visual/draft review and do not send a model task for that part. Record the candidate checksum, installed Claude version, terminal size and actual native view. Judge: would the owner leave Kit open while coding? What crowds, distracts or confuses them? Fix those observations before adding features.

To accept a usable private alpha, also complete a deliberately scoped task in a disposable repository: actual specialist invocation, visible artifact/checks, native isolated worktree for a writer, observed completion marked Needs review, and a Security review bound to that exact result. The owner explicitly sends each request. Kit must not auto-submit, auto-dispatch, fabricate acceptance or edit native permissions.

## Public beta promotion

All of these are required before public distribution:

1. **Owner acceptance:** real native views pass the alpha walkthrough; useful actions stay reachable and the interface still feels like Claude. No unresolved draft-loss, wrong-agent/result, permission/isolation or inaccessible-action defect.
2. **Native interaction:** inspect 80/120/160 columns plus 40/32 and short heights, keyboard focus/Escape, surveys, another AbovePrompt contribution, dark/light/Kit Red, paused/reduced motion and NO_COLOR. Check actual color suppression and fox rendering, not only source flags. An unsupported environment is documented as unsupported, not silently counted as passed.
3. **Four useful roles:** each approved role completes one appropriate native task. Frontend, Backend and Product demonstrate their requested worktree isolation; Security stays Read/Glob/Grep. A completed artifact and its exact-result Security review have inspectable evidence. Skills loading alone does not prove external scanners, components or services exist.
4. **Real failure/recovery:** observed status refusal and session reset remain truthful; repeated preparation does not duplicate an observed mention; draft typing survives. The known lack of atomic native fill/session cancellation is disclosed. Fresh-directory updates preserve previous installs and user settings; ordinary launch without Kit restores native Claude.
5. **Candidate checks:** source/package tests, strict validation, installed-version types and the actual native hooks suite pass on the candidate in an allowed host. A rollout-off refusal blocks automated-host acceptance; it is not a passing skip. Claimed platforms pass their own native matrix. CI must test the exact committed candidate, not just remote PR37.
6. **Artifact evidence:** Node and Rust exports agree byte-for-byte, all skill hashes and licenses are present, generated types/development tests are excluded, and clean extraction/loading of the exact proposed public archive works. Publish only after inspecting the concrete archive and release notes.

Proposed first supported beta target: **macOS terminal with the exact Claude version that passes native acceptance**. Current local host is 2.1.287, but its version alone does not establish Mods access. Linux, Windows and Desktop require separate evidence before joining the support claim. A minimum-version check is not proof that future host API changes work.

Current beta blockers: native owner/matrix acceptance, candidate-specific visible draft/captured-result review, and CI on the unpushed candidate. Original alpha.1 four-role execution/isolation and a separate exact-artifact Security review are supporting evidence; they do not prove the revised Mod review action. The full workspace also has a reproduced inherited-NO_COLOR TUI failure; it is outside the Mod-only surface but must remain explicit. Do not claim the whole workspace is green.

## Stable promotion

Keep the beta surface stable while verifying repeated clean installs, updates and recovery, plus representative returning-session task/review loops. Resolve all defects involving lost work, agent routing, result attribution, permissions, isolation, inaccessible primary actions and misleading completion. Every advertised platform, host version, theme and input mode needs native evidence. Favor completing this matrix over expanding the feature list.

Promotion requires an exact release-candidate commit, passing candidate CI, clean extracted-artifact acceptance, matching version/provenance/checksums and owner approval of the actual release notes/artifact. Record any supported limitations without turning failed supported paths into exclusions after the fact. Remove prerelease wording only after those gates pass.

## Packaging, update and recovery

Development preparation uses existing commands:

```sh
bash scripts/check-native.sh
pnpm exec tsc --noEmit -p crates/kit-cli/claude-plugin/tsconfig.json
claude plugin validate crates/kit-cli/claude-plugin --strict --json
node --test scripts/claude-plugin.test.mjs
claude plugin test crates/kit-cli/claude-plugin
python3 scripts/package-native-alpha.py --out .kit/releases/2.0.0-alpha.4
```

The last command requires Python >=3.9 and a fresh destination. It writes the extracted candidate there, with the archive and checksum beside it. Repeated builds in the same packaging environment with the same source/guide/base revision produce identical archive bytes. BUILD.json records commit time for reproducible archive metadata, not an acceptance or actual build timestamp. Native automated tests pass on the current host; interactive acceptance and exact candidate CI remain public promotion gates.

Users load the extracted `kit` directory explicitly with `claude --plugin-dir /absolute/path/to/kit`, then select Kit Red through `/theme`. No persistent registration/settings edit occurs. For an update, extract into another versioned directory, end the current Claude session normally, and launch the new path. Do not overwrite an active bundle. Session-local results are not migrated: inspect/save needed artifacts before ending a session. Recovery means a normal Claude launch without `--plugin-dir`, or relaunching the retained previous candidate. Theme preference is owned by Claude and can be changed through its own picker.

## Licenses and the separate full-Kit release

The native bundle includes MIT Kit/Frontend/Backend/Product material and CC-BY-SA-4.0 Trail of Bits Security knowledge, with exact attribution, full notices and adapted-source hashes. Top-level LICENSE covers Kit-authored material; the manifest's mixed license expression and per-skill notices define the full bundle. Animate UI component code is not bundled; its separate terms are documented. Checksums establish byte integrity, not publisher authentication or functional acceptance.

Do not use full-Kit binary/npm/crates distribution yet: `scripts/npm-stage.mjs` still labels platform packages MIT and copies only the root license, while the binary embeds the CC-BY-SA resources. The full-Kit release archive path similarly copies root notices only. Correct and test those distribution metadata/notices before any full-product release. This is an explicit blocker on that distribution path, not a reason to add a second Mod installer.

## Next work

| Order | Deliverable | Existing files/path | Acceptance |
| --- | --- | --- | --- |
| 1 | Truthful version-only check and explicit alpha metadata | `crates/kit-cli/src/native.rs`, `tests/native_cli.rs`, plugin manifest/provenance, existing package regression | Real CLI text/JSON regression and package tests pass; no activation/execution claim |
| 2 | Local private-alpha folder/archive/checksum | Existing Node exporter and `.kit/releases` | Extracted payload matches source; strict validation/real loader pass; no tag/publication |
| 3 | Owner visual/draft review | Alpha guide and native checkpoint | Actual views/focus/draft evidence; preserve current design until measured feedback |
| 4 | Scoped four-role execution and native matrix | Existing native fixtures/check scripts | Actual runner, isolation, artifact and exact-result review evidence on claimed hosts |
| 5 | Public beta candidate and reviewed release notes | Same exporter/provenance and chosen `kit-claude-*` channel | Exact committed/CI-tested artifact satisfies all beta gates; explicit publication approval |
| 6 | Stable candidate | Same native package, no new surface | Returning-session, update/recovery and all supported-matrix gates pass |

No calendar promise substitutes for passing a gate. Alpha revisions continue privately until the experience earns public beta.

## Original alpha.1 evidence — 2026-10-02

Local-only archive: `.kit/releases/kit-claude-2.0.0-alpha.1.tar.gz`, 292,414 bytes. SHA-256: `04ffff12fc997e0830a6a4f679671a3519bb498bcf6c493028d37ae43d45b93e`. Its adjacent `.sha256` and 231 member checksums pass using the guide's actual `shasum` commands. The archive contains 229 native payload files plus START-HERE.md, BUILD.json and FILES.sha256. At the original checkpoint every payload byte matched source; its Node export and real Rust CLI installation matched. Subsequent UI changes do not alter this retained archive or inherit its acceptance evidence. Clean extraction, strict validation and the actual extracted native loader succeed with four agents/sixteen skills and zero turns/API milliseconds/cost. No native interactive acceptance is inferred.

Shared checks pass: twenty lifecycle regressions, package integrity, four native Rust unit tests, three native CLI integration tests on macOS, formatting and workspace Clippy. The new readiness integration case fails on the previous overclaim and passes on the correction; it uses a version-only fake host and is Unix-specific. The other two native CLI integration cases remain cross-platform. Exact installed-version TypeScript and both package/loader tests pass. Cargo/pnpm lockfiles are unchanged. `kit claude check --json` now reports `versionCompatible: true`, `modsEnabled: "unknown"`, `loginChecked: false` and `executionVerified: false`; the unverified `mods: true` claim is removed.

At that original packaging checkpoint `claude plugin test` exited 1 on the rollout switch and no model task had run. The subsequent approved swarm ran four genuine native roles against this exact alpha.1 extract and later passed 38 native fixtures at a separate source checkpoint. Later engine diagnostics reproduced a UI restriction before rollout went off again. No bypass, interactive launch, push, tag or publication occurred. Cloud proof remains the remote PR37 baseline, not this uncommitted alpha. The earlier full-workspace run's standalone TUI NO_COLOR failure remains recorded; this turn did not claim a new fully green full-workspace run.

## Retained alpha.2 evidence — 2026-10-02

Local archive: `.kit/releases/kit-claude-2.0.0-alpha.2.tar.gz`, **292,811 bytes**; SHA-256 **`e1909915ebe9f108606a4c424fdd9c98db56cc19b8f337ae8c9969b9da2e64e6`**. Extracted owner preview: `.kit/releases/2.0.0-alpha.2/kit`. The retained alpha.1 checksum is unchanged.

A clean extraction passed the guide's outer checksum command and all 231 member checksums. All 229 native payload files match both current source and the rebuilt real Rust install byte-for-byte; source integrity checks also verify all 214 skill-resource hashes. Strict validation and actual extracted Claude discovery passed with four roles/sixteen skills and zero model turns/API milliseconds/cost. Shared source `--host` checks passed, including 39 native fixtures and the deterministic archive regression. Packaging regression independently passed Python 3.9.6 and 3.12.13; cross-OS native acceptance is not inferred.

BUILD.json records base commit `479d53f0dcdc2cf50945868dfd42fb0553296f33` and **sourceDirty=true**. Per-file/outer checksums identify these uncommitted candidate bytes; this is not committed-candidate CI proof. Native appearance, visible specialist submission and captured-result review remain pending. Computer-use access to Terminal was denied; no alternative app/capture bypass occurred. No public release, push or tag was made. Full workspace remains 411 passes/one inherited NO_COLOR TUI failure, separate from passing native checks.

## Retained alpha.3 actual-use evidence — 2026-10-02

Immutable archive SHA-256 `ed0034e982f87768aa4915cb2633ca339cf3a03f1fc6b7e550b244c84d383214`, 293,456 bytes. This exact extracted candidate passed a real same-session Backend → Kit capture → generated exact-result request → Security → Kit capture loop on naturally updated Claude 2.1.288. Evidence: `.kit/alpha3-final-artifact.json` and `.kit/actual-use-alpha3-20261002/verified-receipt.json`. Backend changed only `parser.mjs` in its native linked worktree, and the unchanged main fixture retained its seed commit. Twelve seeded assertions plus ten independent edge-case assertions passed. Security used only two Read calls, including a successful read of the exact unchanged artifact; both native tasks completed and both answers remained discoverable as Needs review. Native process exited 0.

The headless host refused prompt filling with `no_composer`. A private pass-through companion observed Kit's actual generated review text and preserved that refusal. The owner harness explicitly submitted the unchanged text as its next user message. That proves result binding, native routing and real artifact review; it does not prove visible draft insertion, keyboard/focus or terminal painting. The companion is outside the shipped payload.

Alpha.3's README still carried the prior alpha.2 label. The mismatch was reproduced with a new assertion before correction. Alpha.4 retains identical hooks, agents, skills and theme bytes; only plugin/provenance version and README changed inside the payload. Alpha.3 was retained unchanged.

## Current alpha.4 evidence — 2026-10-02

Owner preview: `.kit/releases/2.0.0-alpha.4/kit`; local archive `.kit/releases/kit-claude-2.0.0-alpha.4.tar.gz`, 293,526 bytes; SHA-256 `6818ac397dc9c8cb1f1778bde615722111f53672d8c140bd8a494f85c923fb3c`. Outer/member integrity, 229-file Rust/Node/source parity and all 214 skill hashes pass. Strict extracted validation and both package/actual loader tests pass on Claude 2.1.288. Runtime hooks/agents/skills/theme are byte-identical to the actual-tested alpha.3; alpha.4 fixes the README label and documentation with a regression that failed before correction.

The exact alpha.4 extraction also completed the same-session headless Backend task → fixed-answer capture → generated Security request → actual read-only Security → both captured results loop. See the exact native session/attempt IDs in `.kit/actual-use-alpha4-20261002/summary.json`. The owner independently ran twelve seeded and ten additional contract assertions; Security read the exact artifact and its unchanged checker, and the artifact/main fixture were unchanged after review. Native process exited 0. Both results remain Needs review.

The Security report substantively addresses input type, whitespace, ASCII digits and safe integers but does not explicitly discuss TypeError. A supplemental native Security turn in the same resumed session read the unchanged exact artifact and explicitly assessed all three TypeError throwing paths at lines 5, 9 and 13. Its new result was captured by Kit; native exit was 0. The original strict report-coverage failure remains retained. The combined reports cover the stated contract, but they are not automatic acceptance or exhaustive security proof. Receipt: `.kit/alpha4-final-artifact.json`. Native visible composer, keyboard/focus/paint and candidate CI remain open, as does the previously recorded unrelated full-workspace TUI failure. No public release was made.
