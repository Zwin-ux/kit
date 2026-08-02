# B2 · Agent adapters + skills-aware TUI dispatch

**Owner:** Grok (integration) · **Crates:** `kit-agents`, `kit-cli`  
**Depends on:** M1 dry-run engine, frozen `Agent` trait  
**Production goal:** Spin agents from the Control Room the way people use `codex exec` / Claude `-p` / Grok `-p` — with high-quality skills in context.

## Product intent

User flow (like Codex / T3 headless workflow, but multi-agent):

1. Open Control Room (`kit`)
2. `d` Dispatch → pick agent(s) × repo(s) × task
3. Kit creates an isolated worktree
4. Kit injects **addyosmani agent-skills** (`.agents/skills`) + routing preamble
5. Kit spawns the real CLI non-interactively
6. Stream shows live output; gate runs; receipt on disk

## Spec

### Adapters (one module each)

| Kind | Binary | Headless invocation (v1) |
|------|--------|---------------------------|
| Codex | `codex` | `codex exec -C <wt> -s workspace-write --json --color never <prompt>` |
| Claude | `claude` | `claude -p <prompt>` with cwd = worktree |
| Grok | `grok` | `grok -p <prompt> --cwd <wt> --always-approve` |
| Ollama | `ollama` | `ollama run <model>` with prompt on stdin (model from `KIT_OLLAMA_MODEL`, default `llama3.2`) |

### Full-auto (optional, dangerous)

When `KIT_FULL_AUTO=1`:

- Codex: add `--dangerously-bypass-approvals-and-sandbox`
- Claude: add `--dangerously-skip-permissions` if supported / documented
- Grok: already `--always-approve`

Default sandbox for Codex remains `workspace-write` (production-safer).

### Skills injection

1. Resolve skills root: `KIT_SKILLS_DIR` → `<repo>/.agents/skills` → `<cwd>/.agents/skills`
2. Copy (or junction) into worktree `.agents/skills` when missing
3. Ensure worktree has a short `AGENTS.md` pointer if absent
4. Prepend **skills preamble** to the user task (see `kit-agents::skills`)

Never ship secrets. Skills are markdown workflows only.

### Engine defaults

| Surface | Default |
|---------|---------|
| `kit run` | **live** if agent on PATH, else dry-run + message |
| `kit run --dry-run` | always dry-run |
| TUI Dispatch | **live** (same auto rule) |
| CI tests | force `--dry-run` |

### Acceptance

- [ ] `kit doctor` shows installed/not for each agent
- [ ] `kit run --agent codex --task "…"` spawns real codex when installed
- [ ] Worktree contains `.agents/skills/using-agent-skills`
- [ ] Prompt includes skill routing preamble
- [ ] TUI Dispatch with codex selected streams output into Run detail
- [ ] Dry-run path still works for CI without CLIs
- [ ] Contracts untouched (`Agent` trait shape unchanged)

## Non-goals (this slice)

- Interactive attach/PTY (B2-pty)
- Kill mid-run (needs handle registry)
- Skill picker UI (always inject full pack routing; selection later)
