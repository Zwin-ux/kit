# Session B retry — 2026-09-11 (still blocked, not a receipt)

Factory, branch `factory/opus5-session-b` (base `815e5d7`). Fixture: fresh git repo, one commit with `README.md` and

```toml
[gate]
test = 'echo ok'
```

## Result

No new receipt. Live grok run `01M2A5KZ7G5ARQ0A06S4V3WB8N` did the task: its worktree `README.md` gained `session-b ok` at 06:42:25Z. Grok exited about 06:42:35Z. Kit then spun until `timeout 120` killed it (exit 124, 2m0s). `~/.kit/runs/01M2A5KZ7G5ARQ0A06S4V3WB8N` does not exist. `KIT_FULL_AUTO` was not set.

```sh
timeout 120 target/debug/kit.exe run --agent grok --live --json --repo target/session-b-fixture \
  --task "Append exactly one line 'session-b ok' to README.md. It is a one-line docs edit: do not run tests or explore."
```

The run is no longer silent. Stderr now streams deltas and reports silence (`8eb9784`):

```
kit: state running
Preparing worktree (detached HEAD 65e1a9b)
kit: installed 39 skills for grok
kit: spawning grok -p --cwd C:\Users\mzwin\.kit\worktrees\01M2A5KZ7G5ARQ0A06S4V3WB8N --always-approve
event:available_commands
thought: The
…
text: … confirm the last line is `session-b ok`.
event:usage
event:end
kit: still running (grok), no output for 30s — Ctrl-C to abort
kit: still running (grok), no output for 60s — Ctrl-C to abort
```

## Root cause 1 — engine exit poll starves (blocks Session B; engine-owned, not edited)

`crates/kit-cli/src/engine/runner.rs`, `live_agent` (`815e5d7`, lines 478-538): `tokio::select! { biased; … maybe = local_rx.recv() => { … None => {} } _ = poll.tick() => try_wait … }`.

When the agent exits, its pipes close and the adapter's stream tasks drop every sender. From then on `recv()` is ready with `None` on every poll. `biased` picks it before `poll.tick()`, so `try_wait` never runs. The loop spins until `Bounds::default().timeout` (30 min).

Evidence:

- grok 28556 was a direct child of kit 27864. Its log (`~/.grok/logs/unified.jsonl`) shows `shell.handle_prompt.done` at 06:42:33Z and `session_end.worker_join` at 06:42:34Z. It was gone by 06:43:01Z. kit 27864 was still alive with no children and never printed `kit: grok exited with code …`.
- The same stall happens with a local agent (no credits): `KIT_OLLAMA_MODEL=llama3.1`, scratch `KIT_HOME`. Ollama answered and was gone by 06:47:01Z. kit 2944 stayed alive with 0 children and burned 2.33 s of CPU per 3 s of wall time (51 s of CPU by 06:47:30Z).
- A standalone tokio program with the same loop shape (scratch, not committed):

  ```
  runner.rs as-is      : TimedOut: exit poll never ran after 2.0000276s; recv() returned None 1803438 times
  with pipes_open guard: exit poll ran (try_wait would reap the child) after 54.1666ms; recv() returned None 1 times
  ```

- Session B (22:34 and 22:41 local) is consistent with this. Both grok processes logged `handle_prompt.done` and `worker_join` (05:37:38Z, 05:43:45Z), and kit never wrote a receipt.

Suggested fix for the engine owner: `let mut pipes_open = true;` and `maybe = local_rx.recv(), if pipes_open => match maybe { Some(delta) => …, None => pipes_open = false }`.

## Root cause 2 — `cmd /C` cut the prompt at its first newline (fixed, `3150cd8`)

On Windows, `command_for` runs `cmd /C <binary> …`, and cmd.exe ends a `/C` command line at the first LF. A Rust child spawned the same way received `argc=2`: `-p` and `# Kit Control Room — agent run`.

So Session B's grok got only the prompt title. It never received `--cwd`, `--always-approve` or `--output-format`.

`--output-format streaming-json` was never at fault. `grok -p "…" --cwd <fixture> --always-approve --output-format streaming-json </dev/null` exited 0 in 11.8 s, with NDJSON ending `{"type":"end","stopReason":"end_turn",…}`.

The fix spawns the native `grok.exe` directly. Still open: codex (`codex.rs:68`) and claude (`claude.rs:49`) pass the same multi-line prompt through `cmd /C`.

## Also found

- `--json` stdout is not a single envelope. `git worktree add` inherits kit's stdout (`engine/worktree.rs`, `.status()`), so stdout starts with `HEAD is now at 65e1a9b init fixture`.
- An `execute` error prints no `--json` envelope; the error goes to stderr only.
- Ollama's default `llama3.2` is not pulled on this host. The run now fails in 0.2 s with the remedy (`063336c`).

## Retry once the engine fix lands

```sh
cargo build -p kit-cli
timeout 120 target/debug/kit.exe run --agent grok --live --json --repo <fixture> --task "<one-line edit>"
target/debug/kit.exe receipt show <new id> --json   # gate.checks must list test: echo ok
```
