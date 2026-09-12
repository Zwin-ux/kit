# Session B retry — 2026-09-11/12 (receipt `01M2A7DVMF9N1RX7XYQGX2A5Q4`)

Factory, branch `factory/opus5-session-b` (base `815e5d7`). Fixture: fresh git repo, one commit with `README.md` and

```toml
[gate]
test = 'echo ok'
```

## Receipt (2026-09-12, after `e613bfb`)

One live grok run, `KIT_FULL_AUTO` not set, exit 0 in 24.7 s:

```sh
timeout 120 target/debug/kit.exe run --agent grok --live --json --repo target/session-b-fixture \
  --task "Append exactly one line 'session-b ok' to README.md. It is a one-line docs edit: do not run tests or explore."
```

| Fact | Evidence |
|------|----------|
| New `~/.kit/runs` id | `01M2A7DVMF9N1RX7XYQGX2A5Q4` (07:13:37Z → 07:14:01Z) |
| `gate.checks` non-empty | `{"label":"test","command":"echo ok","status":"pass","exit_code":0}`, `gate.passed: true` |
| `output.log` has spawn and exit | L2 `kit: spawning grok -p --cwd C:\Users\mzwin\.kit\worktrees\01M2A7DVMF9N1RX7XYQGX2A5Q4 --always-approve`; L207 `kit: grok exited with code 0` |
| Not a dry-run | `output.log` starts with `kit: installed 39 skills for grok` |

The diff adds `session-b ok` to `README.md`.

Stdout carried git's `HEAD is now at 65e1a9b init fixture` before the envelope. It is stripped below; the `engine/worktree.rs` fix belongs to factory-queued.

```json
{
  "command": "run",
  "data": {
    "gatePassed": true,
    "gateVacuous": false,
    "id": "01M2A7DVMF9N1RX7XYQGX2A5Q4",
    "receiptDir": "C:\\Users\\mzwin\\.kit\\runs\\01M2A7DVMF9N1RX7XYQGX2A5Q4",
    "state": "pass",
    "worktreeRemoved": false
  },
  "error": null,
  "ok": true,
  "schemaVersion": 1,
  "warnings": []
}
```

Stderr shows the whole run: `kit: state running`, skills, `kit: spawning grok …`, grok's `thought:` / `text:` stream, `kit: grok exited with code 0`, `kit: state gating`, `kit: state pass`.

Zero-credit second proof against the same bar: `KIT_OLLAMA_MODEL=llama3.1`, task `Reply with the single word ok.`, exit 0 in 18.7 s.

- New id `01M2A7H6HX5PYN18RV4EP0A4EC`; `gate.checks` holds the same `echo ok` pass.
- `output.log` L2 `kit: spawning ollama run llama3.1 (cwd …)`, L25 `kit: ollama exited with code 0`; first line `kit: installed 39 skills for ollama context`.
- Envelope: `state: pass`, `gatePassed: true`, `gateVacuous: false`.
- The diff is empty, yet `worktreeRemoved` is false: kit's own untracked `.agents/` and `AGENTS.md` keep the worktree from counting as clean.

## First retry (2026-09-11, blocked)

No new receipt. Same command as above. Live grok run `01M2A5KZ7G5ARQ0A06S4V3WB8N` did the task: its worktree `README.md` gained `session-b ok` at 06:42:25Z. Grok exited about 06:42:35Z. Kit then spun until `timeout 120` killed it (exit 124, 2m0s). `~/.kit/runs/01M2A5KZ7G5ARQ0A06S4V3WB8N` does not exist.

The run was no longer silent. Stderr streamed deltas and reported silence (`8eb9784`):

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

## Root cause 1 — engine exit poll starved (fixed, `e613bfb`)

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

Fix: `let mut pipes_open = true;` and `maybe = local_rx.recv(), if pipes_open => match maybe { Some(delta) => …, None => pipes_open = false }`.

`live_agent_sees_exit_after_output_pipes_close` drives `live_agent` with a real child that exits at once. It failed on the old loop (`exit poll starved … Elapsed(())` after 2.00 s) and passes with the guard.

## Root cause 2 — `cmd /C` cut the prompt at its first newline (fixed, `3150cd8`)

On Windows, `command_for` runs `cmd /C <binary> …`, and cmd.exe ends a `/C` command line at the first LF. A Rust child spawned the same way received `argc=2`: `-p` and `# Kit Control Room — agent run`.

So Session B's grok got only the prompt title. It never received `--cwd`, `--always-approve` or `--output-format`.

`--output-format streaming-json` was never at fault. `grok -p "…" --cwd <fixture> --always-approve --output-format streaming-json </dev/null` exited 0 in 11.8 s, with NDJSON ending `{"type":"end","stopReason":"end_turn",…}`.

The fix spawns the native `grok.exe` directly. Still open: codex (`codex.rs:68`) and claude (`claude.rs:49`) pass the same multi-line prompt through `cmd /C`.

### codex and claude (zero-credit probe, not fixed)

Same `cmd /C` launch and argv as the adapters, with stand-ins first on PATH that print what they receive. On this host `codex` resolves to the npm `codex.cmd` shim, which forwards `%*`. `claude` resolves to the native `~/.local/bin/claude.exe`; an npm `claude.cmd` is also installed.

```
codex:  argc=9  exec -C <wt> -s workspace-write --json --color never "# Kit Control Room — agent run"
claude: argc=2  -p "# Kit Control Room — agent run"    (--dangerously-skip-permissions dropped)
```

Codex gets its flags but only the prompt title. That fits the historical codex receipt `01KYZZ06CZZ3TJCJ70457JYNJ9` (empty diff, vacuous gate). A `.cmd` shim cannot take a multi-line argument directly, so both likely need the prompt on stdin or in a file.

## Ollama prompt on stdin could deadlock spawn (fixed, `aab7e4f`)

`spawn_streaming_with_stdin` wrote the whole prompt before starting the stdout/stderr readers, and the runner arms the run timeout only after spawn returns.

Zero-credit repro through the real `OllamaAgent::spawn`, with a stand-in `ollama.cmd` that writes stderr before reading stdin:

```
256 KiB prompt,  5,000 stderr lines: spawn returned after 121 ms, child exit 0
8 MiB prompt,   50,000 stderr lines: DEADLOCK: spawn still blocked after 20.0 s
after aab7e4f, 8 MiB / 50,000:       spawn returned after 32 ms, child exit 0 after 1.4 s
```

On Windows, tokio buffers up to 2 MiB of a stdin write; on Unix a 64 KiB pipe is enough to block. The fix starts the readers first and writes the prompt from its own task.

`large_stdin_prompt_does_not_deadlock_on_early_child_stderr` failed on the old code (timeout after 30 s) and passes with the fix.

## Grok leader / daemon

No grok leader or daemon outlived the run. At 06:43:01Z grok 28556 had no child processes, and `grok leader list` reported none.

Kit's delta channel had closed, meaning both pipe readers hit EOF. With the channel still open, `poll.tick()` would have seen grok's exit. That closure is what sent `live_agent` into its spin.

## Also found

- `--json` stdout is not a single envelope. `git worktree add` inherits kit's stdout (`engine/worktree.rs`, `.status()`), so stdout starts with `HEAD is now at 65e1a9b init fixture`.
- An `execute` error prints no `--json` envelope; the error goes to stderr only.
- Ollama's default `llama3.2` is not pulled on this host. The run now fails in 0.2 s with the remedy (`063336c`).
- `~/.kit/runs` holds `01KILLQ0006` (codex, repo `kit-bare-repo-…`, task `KILLQ task 0006`): a test wrote into the real kit home.
