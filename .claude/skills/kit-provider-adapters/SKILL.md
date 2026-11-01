---
name: kit-provider-adapters
description: Add or change a model provider (Anthropic, OpenAI, xAI, OpenAI-compatible/local) or an external agent driver (Claude Code, Codex, Grok, ACP) in Kit — streaming, tool calls, errors, cost and tests. Use for any work in crates/kit-providers or crates/kit-agents.
---

# Kit provider adapters and harness drivers

The design is [`docs/PRD.md`](../../../docs/PRD.md) §6. Two kinds of adapter, two crates:

| Kind | Crate | Interface | Example |
|---|---|---|---|
| Model provider (Kit's own agent) | `crates/kit-providers` | `Provider` trait: `models()`, `stream(Request) -> EventStream` | `anthropic.rs`, `openai_compatible.rs` |
| External agent driver (harness) | `crates/kit-agents` | `Agent` trait: `probe()`, `spawn()` → `AgentHandle`; events become `KitEvent` | `claude.rs`, `codex.rs`, `grok.rs` |

## Rules
1. **Check the real docs, never guess a flag.** Record the doc URL and the date in a comment above the
   invocation, and the CLI version you tested against. Upstream flags change: `codex --full-auto` and
   `codex mcp-server` were removed in 2026.
2. **One file per provider or driver,** ≤ 600 lines. Shared SSE parsing and retry live in one module.
3. **Prompts never go on argv** (Windows `cmd` injection and length limits). Use stdin, a prompt file the
   tool's published docs list, or the protocol (ACP).
4. **No shells.** Spawn the real executable; on Windows resolve `.cmd` shims to their entry point as the
   existing adapters do.
5. **Auth stays where it is.** Providers: env var → OS keychain (`kit:<provider>`) → `key_command`. Drivers:
   the tool's own login, checked with its status command (`claude auth status`, `codex login status`).
   Never read another tool's credential file. Never log a key; keys never enter a tool's environment.
6. **Errors map to the closed set:** `Auth`, `RateLimited{retry_after}`, `ContextOverflow`, `Transient`,
   `BadRequest`, `Fatal`. The user sees the class in words and the fix.
7. **Cost:** use the provider's reported cost when there is one (xAI ticks, Claude `total_cost_usd`,
   Grok's reported cost); otherwise compute from usage × `prices.toml` and mark it `≈`. Never sum
   Claude `total_cost_usd` across resumes (it already includes earlier spend).
8. **Context window comes from the provider,** never a default guess.
9. **Published docs beat repo docs.** A flag found only in a tool's repo or user guide, not on its published
   CLI reference, is unverified: don't depend on it until a live check against the released CLI passes.

## Tests (required)
- **Recorded streams:** capture real responses once (success, tool call, rate limit, context overflow,
  auth failure), strip keys and personal data, and commit them under the crate's fixtures folder
  (`crates/kit-agents/src/fixtures/` for drivers). CI replays
  them; CI never calls a real API.
- **Live tests** go behind the `live` cargo feature and are run by hand with a budget.
- **Drivers:** replay a transcript through the parser, and use a fake binary (see
  `crates/kit-cli/tests/land_cli.rs`) for exit codes, kill, resume and a version whose stream changed shape,
  which must fail loudly with the version named.
- Run `cargo test -p kitctl-providers -p kitctl-agents` and the 3-OS CI before calling it done.
