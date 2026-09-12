# Kit quality cycle — surface (2026-09-11)

Source: `kit-quality-cycle` mode=full focus=surface. 12 confirmed findings. Power landed FAIL-moment TUI slices. Factory P1/P3 CARGO_TARGET_DIR work is still uncommitted (kit-cli / kit-agents / kit-gate).

## Verdict

PR #15 is alpha-land only. Do not self-merge. Do not tag 1.0.0. GitHub identity is still 0.1 (human I1).

## Power landed after this report

- Selected FAIL keeps `fail_wash`, not reverse; cyan caret rail
- 60-col run detail keeps `GATE FAIL` and `[r]etry`; `follow` is status
- Demo flash truncated into the 80-col header (`FAIL · enter open · r…`)
- Help overlay Clears the full 80×14 frame
- `[b]oard` kept at 80; filter/enter drop first

## Still Factory / You

- P1/P3 isolation in kit-cli (dirty working tree — do not mix with TUI)
- N1 Node CI / keep-alive
- I1 GitHub About
- I3–I5 Sessions B–D
