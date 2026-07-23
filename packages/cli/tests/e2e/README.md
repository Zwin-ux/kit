# CLI End-to-End Tests

These tests should spawn the compiled KIT CLI inside isolated temporary environments.

Each test must override at least:
- `HOME`
- `USERPROFILE`
- `KIT_HOME`
- the fixture project directory
- agent-specific skill/config directories

Initial scenarios:
1. `kit ready --write` on a small web project.
2. `kit status` after setup.
3. `kit link --to all --write` without touching the developer's real home directory.
4. `kit doctor` after a valid setup.
5. Repeat the commands to prove idempotency.
6. Exercise a project path containing spaces and Unicode.
7. Report command, exit code, stdout, stderr, and fixture path on failure.

The same suite must run on Ubuntu, macOS, and Windows in CI.
