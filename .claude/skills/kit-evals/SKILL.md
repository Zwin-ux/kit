---
name: kit-evals
description: Run, add to or report Kit's agent evals — the in-repo Kit suite (kit eval), Terminal-Bench via Harbor, a SWE-bench Verified subset and Aider polyglot. Use before claiming the agent got better or worse, when changing the loop, a tool, a prompt or a provider, or before publishing any benchmark number.
---

# Kit evals

The plan is [`docs/PRD.md`](../../../docs/PRD.md) §13.3. Until `kit eval` lands (milestone A0.1), this skill
describes the target; say so rather than pretending a suite exists.

## The rule for numbers
No number goes in the README, a PR description or a post without a committed result file in
`docs/evals/` (date, Kit commit, model id, task set and its pin, pass rate, cost, turns) and a link to the
run. Comparisons name the same model, date and task subset for both sides.

## Kit suite (`kit eval`)
- 30 tasks in fixture repos under `evals/tasks/<id>/`, each with a `kit.toml` gate as the grader and a
  `task.md` prompt. The gate decides pass/fail; no model grades a model.
- Categories: failing test, feature plus test, refactor, Windows/CRLF bug, multi-file rename, read-only
  question (exact-answer check), refusal (must end in a denied prompt).
- `kit eval --model <id> --budget 5` runs them in worktrees, in parallel, and writes
  `docs/evals/<date>-<model>.json`. In CI, `kit eval --scripted` uses the scripted provider and checks only
  the harness.
- Adding a task: it must fail on the fixture's base commit, pass with a reference patch, and run in under
  2 minutes. Commit the reference patch under `evals/tasks/<id>/solution.patch`.

## External benchmarks (manual or weekly, always budget-capped)
- **Terminal-Bench:** Harbor (Apache-2.0). `uv tool install 'harbor[modal]'`, run the oracle agent first,
  then `harbor run -d terminal-bench/terminal-bench@<pinned> --agent <kit adapter> --model <id>`.
  Version 4.0 is not comparable with 2.x results.
- **SWE-bench Verified:** a fixed 50-task subset, MIT harness, Docker on x86_64 (about 120 GB disk).
  `swebench eval verified -p preds --run-id <id>`. A regression check, not a headline (saturated).
- **Aider polyglot:** `Aider-AI/polyglot-benchmark` in Docker via `benchmark/docker_build.sh`. Use it to
  compare edit formats.

Never run a paid benchmark without the owner's go and a `--budget`. Report what ran, what it cost, and
what failed, including infrastructure failures.
