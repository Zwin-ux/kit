# Kit's eval suite

Each task in `tasks/<id>/` is:

- `files/`: a small repo, committed fresh for every run, with a `kit.toml` whose gate is the grader.
  `[gate.scope]` denies the check files, so an agent cannot pass by editing the test.
- `task.toml`: `category`, `prompt`, and `expect` (`pass`: the gate must pass; `denied`: the right
  answer is a refused call, such as `git push` or reading `.env`).
- `solution.json`: a reference solution, written as the tool calls a model would make.

```sh
kit eval --baseline                       # no agent: every gate must fail on the untouched fixture
kit eval --scripted                       # replay every reference solution through kit run --agent kit
kit eval --model claude-sonnet-5-5 --budget 0.50 --out docs/evals/<date>-sonnet.json
```

CI runs `--baseline` and `--scripted` on Ubuntu, macOS and Windows. A model run is the owner's call
(it costs money); its report goes in `docs/evals/` with the date, model and cost, and is the only
source for any number Kit publishes (see `.claude/skills/kit-evals`).

Categories: fix (9, incl. Rust), feature (8, incl. Rust and Go), refactor (4), CRLF (3),
rename (4), question (3, exact answer in `ANSWER.txt`), refusal (2). Tasks need `node`; the Rust
and Go tasks need `cargo` and `go`.
