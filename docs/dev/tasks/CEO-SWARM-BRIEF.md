# CEO brief — Kit alpha-land swarm (2026-09-11)

You are Kit **CEO** (Claude Opus). Judgment only. Do not merge. Do not tag v1.0.0. Do not edit frozen contracts: `crates/kit-tui/src/event.rs`, kit-core run/config/gate, kit-agents trait.

Repo: `C:\Users\mzwin\kit` branch `feat/1.0-dogfood-kit-toml` PR https://github.com/Zwin-ux/kit/pull/15

## Product

Kit is the Control Room: dispatch many agents, watch them, refuse unproven work. Gate is the wedge. Mascot/marketplace/PTY/installer are parked.

## Already landed on this PR (do not redo)

- Control Room FAIL still-frame, GATING row in `--demo`, motion spinner on live work only
- FAIL wash on selected row, GATE FAIL + retry at 60, header flash, help overlay cover, `[b]oard` at 80
- README first paint is FAIL still-frame, not the fox
- Session A receipt exists historically

## Your job this turn

1. Read `AGENTS.md`, `docs/dev/CURRENT.md`, `docs/dev/SPEC-next.md`, `tasks/todo.md`, `wiki/kit-1.0/outputs/queries/2026-09-11-kit-quality-cycle-surface.md`.
2. Inspect uncommitted Factory diffs in `crates/kit-cli`, `crates/kit-agents/src/process.rs`, `crates/kit-gate/src/lib.rs` for P1 `CARGO_TARGET_DIR` isolation. Say merge-as-Factory-commit or revert.
3. Issue a ≤15 line Power/Factory order: next 3 slices, owners, kill criteria.
4. Confirm: Power does not merge PR 15. I1 GitHub About is human. Session B is a live `kit run --agent <ready>` receipt.

Write the brief to stdout. No git push. No merge.
