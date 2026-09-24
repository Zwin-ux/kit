<p align="center">
  <strong>Dispatch many agents. Watch them in one place. Nothing ships unproven.</strong><br />
  Local Control Room for Codex, Claude, Grok, and Ollama — worktrees, gates, receipts.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-1a1a1a?style=for-the-badge" alt="MIT" /></a>
  <img src="https://img.shields.io/badge/status-1.0%20alpha-00E6CC?style=for-the-badge" alt="1.0 alpha" />
</p>

`--demo` lands on FAIL. `r` retries with the gate context. That is the product.

```
KIT / CONTROL ROOM                    [ALL]  1 RUNNING  1 GATING  1 FAIL
┌ runs ─────────────────────────────────────────────────────────────────┐
│  kit          codex·eng  port guard.js            RUN 2m      --      │
│  kit          grok·eng   frame clock              GATING 2m   --      │
│▶ trenchwire   codex·eng  fix red CI               DONE        FAIL    │
│^ tsc: 3 errors — Type 'string' is not assignable                      │
│  guardian     claude·eng 855-case suite           DONE        PASS    │
└───────────────────────────────────────────────────────────────────────┘
 [↑↓] select  [d]ispatch  [enter] open  [g]ate  [k]ill  [r]etry  [?]help
```

---

## 30 seconds

```bash
# From this repo — lands on FAIL, r retries
cargo run -p kit-cli -- --demo
```

Repo shims (`.\kit.cmd` / `.\kit.ps1`, or `. .\scripts\use-rust-kit.ps1`) launch the Rust binary. Bare `kit` on this Windows PATH is still npm `@mzwin/kit@0.1.3` until you dot-source `scripts/use-rust-kit.ps1` or put `target\release` on PATH.

| Command | What it does |
|---------|----------------|
| `cargo run -p kit-cli -- --demo` / `.\kit.cmd --demo` | Control Room TUI |
| `cargo run -p kit-cli -- run --task "…"` | One isolated run (live agent if on PATH) |
| `cargo run -p kit-cli -- doctor` | Probe codex / claude / grok / ollama + skills pack |
| `cargo run -p kit-cli -- run --dry-run --json` | Offline path (worktree → stream → **this repo's gate** → receipt) |
| `cargo run -p kit-cli -- receipt list` | Browse proof under `~/.kit/runs/` |
| `cargo run -p kit-cli -- receipt show <id>` | One receipt (+ `--output` for log tail) |

**Product loop:** dispatch → table of runs → FAIL wash + first error → `r` retry with gate context → receipt under `~/.kit/runs/<id>/`.

Keys: `↑↓` select · `f` filter · `Enter` open · `g` gate · `d` dispatch · `b` board · `k` kill · `r` retry · `?` help · `q` quit.

This repo has a real [`kit.toml`](kit.toml) (fmt + clippy + `cargo test --workspace`, 15m). A dry-run against Kit is no longer vacuous.

---

## Install (1.0 alpha)

Use one of these lines. Each installs the same `kit` binary.

| Method | Command |
|--------|---------|
| npm (primary) | `npm install -g @mzwin/kit@alpha` |
| Linux, macOS | `curl -fsSL https://raw.githubusercontent.com/Zwin-ux/kit/main/scripts/install.sh \| sh -s -- --prerelease` |
| Windows PowerShell | `& ([scriptblock]::Create((irm https://raw.githubusercontent.com/Zwin-ux/kit/main/scripts/install.ps1))) -Prerelease` |
| Cargo | `cargo install --git https://github.com/Zwin-ux/kit kit-cli --locked` |

- npm without `@alpha` installs the old 0.1 app until 1.0.0.
- The install scripts need a GitHub Release with archives. The first one comes with the release after 1.0.0-alpha.1. Until then, use npm or Cargo.
- The install scripts check the SHA-256 of the download against `SHA256SUMS` and stop if it does not match. They install to `~/.local/bin` (Windows: `%LOCALAPPDATA%\kit\bin`) and do not change `PATH`. If that directory is not on `PATH`, they show the line to add.
- Linux builds need glibc 2.17 or newer. On musl (Alpine), use the Cargo line.
- Do not use `cargo install kit-cli`. That crate on crates.io is a different project.

Then:

```bash
kit doctor
kit --demo
```

Details: [`docs/dev/RELEASING.md`](docs/dev/RELEASING.md).

From source:

```bash
git clone https://github.com/Zwin-ux/kit.git
cd kit
cargo build -p kit-cli --release
./target/release/kit doctor
./target/release/kit --demo
```

Requires Rust stable and (for live runs) at least one of: `codex`, `claude`, `grok`, `ollama` on PATH.  
Kit uses each provider's existing login. It does not store model API keys.

Env flags:

| Env | Effect |
|-----|--------|
| `KIT_FULL_AUTO=1` | Bypass agent approval prompts (dangerous — sandboxes only) |
| `KIT_SKILLS_DIR=…` | Override skill pack root (see Skills) |
| `KIT_OLLAMA_MODEL=…` | Model for Ollama adapter (default `llama3.2`) |
| `NO_COLOR` / `KIT_MOTION=off` | Monochrome / reduced motion (RUNNING stays a still `RUN` label) |
| `KIT_THEME=high` | High-contrast ANSI palette (`high-contrast` / `hc` also work) |

Architecture: [`docs/dev/CURRENT.md`](docs/dev/CURRENT.md) · honest state: [`docs/dev/design-package/00-HONEST-STATE.md`](docs/dev/design-package/00-HONEST-STATE.md) · PRD: [`docs/dev/PRD-1.0.md`](docs/dev/PRD-1.0.md)

---

## Skills

Every live run copies a skill pack into the worktree and prepends routing to the prompt.

**Default:** [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) at `.agents/skills` (coding lifecycle: spec → plan → build → verify → review).

**Any `*/SKILL.md` pack works**, including [Harness skills](https://github.com/harness/harness-skills):

```bash
git clone https://github.com/harness/harness-skills.git
# Point Kit at the skills tree (not the repo root)
export KIT_SKILLS_DIR="$PWD/harness-skills/skills"
cargo run -p kit-cli -- run --agent claude --task "debug my failed pipeline"
```

Or place a `skills/` directory (Harness layout) in the target repo — Kit discovers it after `.agents/skills`.

**Harness note:** those skills expect the [Harness MCP v2 server](https://github.com/harness/mcp-server) and API credentials. They are a **domain pack**, not Kit's default. Without MCP, agents can still read the markdown but cannot call Harness tools.

Resolution order: `KIT_SKILLS_DIR` → `<repo>/.agents/skills` → `<repo>/skills` → walk up from cwd.

---

## Control Room

- Live table of runs (repo · agent · task · STATE · GATE)
- FAIL rows get a danger wash and a `^ first error` annotation
- Dispatch fans out repo × agent (cap 16) with one task
- Board is prefill-only in 1.0 (Enter → Dispatch)
- Gate: empty checks render **UNCONFIGURED**, never silent PASS
- Kill mid-run (`k`) and retry failed (`r`) with gate failure context

```bash
cargo run -p kit-cli -- --demo   # lands on a FAIL so the proof loop is obvious
```

## Receipts

Every run writes `~/.kit/runs/<id>/` (`receipt.json`, `output.log`, optional `diff.patch` / `gate.json`).

```bash
cargo run -p kit-cli -- receipt list
cargo run -p kit-cli -- receipt show 01KZD0… --output
cargo run -p kit-cli -- receipt list --json --limit 10
```

Override data root with `KIT_HOME`. Contract: [`docs/json-contract.md`](docs/json-contract.md).  
Dogfood path: [`docs/dev/DOGFOOD.md`](docs/dev/DOGFOOD.md).

---

## Legacy: Kit 0.1.x npm workbench

Earlier alpha (`npm i -g @mzwin/kit`) was a Node skill/workbench TUI. Still in `packages/` for history; **1.0 is the Rust binary above**. Prefer Control Room commands.

<details>
<summary>0.1 workbench notes</summary>

```bash
npm i -g @mzwin/kit
kit ready --write
kit tui workbench
```

See [Workbench architecture](docs/dev/WORKBENCH_ARCHITECTURE.md) for the archived design.

</details>

---

<p align="center">
  <sub>
    <a href="LICENSE">MIT</a> ·
    <a href="docs/dev/design-package/README.md">design package</a>
  </sub>
</p>
