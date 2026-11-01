---
name: kit-evals
description: Run, add to or report Kit's agent evals — the in-repo Kit suite (kit eval), Terminal-Bench via Harbor, a SWE-bench Verified subset and Aider polyglot. Use before claiming the agent got better or worse, when changing the loop, a tool, a prompt or a provider, or before publishing any benchmark number.
---

# Kit evals

The plan is [`docs/PRD.md`](../../../docs/PRD.md) §13.3; the suite is [`evals/README.md`](../../../evals/README.md).

## The rule for numbers
No number goes in the README, a PR description or a post without a committed result file in
`docs/evals/` (date, Kit commit, model id, task set and its pin, pass rate, cost, turns) and a link to the
run. Comparisons name the same model, date and task subset for both sides.

## Kit suite (`kit eval`)
- 33 tasks in `evals/tasks/<id>/`: `files/` (the fixture repo, with a `kit.toml` gate that is the grader
  and a `[gate.scope]` that denies the check files), `task.toml` (`category`, `expect`, `prompt`) and
  `solution.json` (the reference solution as scripted tool calls).
- `kit eval --baseline` (no agent: every gate must fail) and `kit eval --scripted` (every reference
  solution must pass through `kit run --agent kit`) run in CI on three OSes. Run both after adding or
  changing a task.
- A model run: `kit eval --model <id> --budget 0.50 --out docs/evals/<date>-<model>.json`. It costs money:
  the owner's call. Set `KIT_HOME` to a scratch folder when testing locally so runs stay out of `~/.kit`.
- Adding a task: write the fixture, prompt and solution; the gate must fail on the fixture, pass with the
  solution, and finish in under 2 minutes. A reference solution that only works with a blanket
  replace (`replace_all` over a common word) is a bug: real agents make that mistake, and the gate should
  catch it.

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
