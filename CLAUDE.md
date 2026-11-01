# Kit — instructions for Claude Code

Read [`AGENTS.md`](AGENTS.md) first: what Kit is, the commands, the TUI rules. The product plan is
[`docs/PRD.md`](docs/PRD.md); the bar for every change is [`docs/QUALITY.md`](docs/QUALITY.md).

## Skills: when to use which

Project skills live in `.claude/skills/` (Kit-specific, written for this repo). Engineering skills live in
`.agents/skills/` (vendored from [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills),
MIT, © 2025 Addy Osmani; licence in `.agents/skills/LICENSE-addyosmani-agent-skills`, pins in
`skills-lock.json`).

| When you are… | Use | Where |
|---|---|---|
| Changing anything under `crates/kit-tui` or anything that draws to the terminal | `kit-tui` | `.claude/skills` |
| Adding or changing a model provider or an agent driver (Claude Code, Codex, Grok, ACP) | `kit-provider-adapters` | `.claude/skills` |
| Changing the agent loop, a tool, a prompt or a provider, or quoting a benchmark | `kit-evals` | `.claude/skills` |
| Bumping a version, editing the changelog, tagging, reading a release run | `kit-release` | `.claude/skills` |
| A CI check failed, or you need build/test proof (especially on Windows) | `kit-ci-triage` | `.claude/skills` |
| Starting a new feature or milestone | `spec-driven-development` → `planning-and-task-breakdown` | `.agents/skills` |
| Implementing | `incremental-implementation`, `test-driven-development` | `.agents/skills` |
| A bug, a flaky test, a wrong output | `debugging-and-error-recovery` | `.agents/skills` |
| A public contract (`--json`, `KIT.toml`, `kit.toml`, receipts, traits) | `api-and-interface-design` | `.agents/skills` |
| Sandbox, permissions, keys, hooks, anything security-relevant | `security-and-hardening` | `.agents/skills` |
| Cold start, idle CPU, render cost | `performance-optimization` | `.agents/skills` |
| Before opening a PR | `code-review-and-quality`, `code-simplification` | `.agents/skills` |
| A boundary decision worth recording | `documentation-and-adrs` (ADRs in `docs/adr/`) | `.agents/skills` |

## Always
- Prove it: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, and `rust.yml` green on Ubuntu, macOS and Windows. "Looks right" is not done.
- Never print or commit a key or token. Never tag, publish or run a paid eval without the owner.
- Commit as the owner's identity configured in the repo; end messages with the attribution the session gives.
