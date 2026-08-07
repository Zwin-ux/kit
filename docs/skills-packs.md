# Skill packs

Kit injects a skill pack into every live agent worktree. Skills are markdown
workflows (`name/SKILL.md`), not a marketplace product surface.

## Defaults

| Pack | Path | Role |
|------|------|------|
| [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills) | `.agents/skills` | Default coding lifecycle for Kit itself and general repos |

## Optional: Harness

[Harness skills](https://github.com/harness/harness-skills/tree/main/skills) are
Claude Code skills for Harness.io (pipelines, security, feature flags, SLOs…).

They are **compatible** with Kit's loader (`*/SKILL.md`), but:

1. Most skills require the **Harness MCP v2** server and API keys.
2. They are a **domain pack** — do not replace the coding default unless the
   repo's job is Harness operations.

### Use with Kit

```bash
git clone https://github.com/harness/harness-skills.git
export KIT_SKILLS_DIR="$(pwd)/harness-skills/skills"

# Claude is the natural agent for this pack
cargo run -p kit-cli -- run --agent claude --task "debug pipeline X in org Y"
```

Or clone/copy `skills/` into the target repo root (Harness layout). Kit resolves:

1. `KIT_SKILLS_DIR`
2. `<repo>/.agents/skills`
3. `<repo>/skills` (if it contains at least one `*/SKILL.md`)
4. Walk up from cwd

### MCP (host, not Kit)

Configure Harness MCP on the host agent runtime (Claude Code settings, etc.).
Kit does not host MCP; it only injects the skill markdown + prompt routing.

## Custom packs

Any directory of `skill-name/SKILL.md` folders works. Point `KIT_SKILLS_DIR` at it.
Generic packs get a “pick the matching SKILL.md” preamble; packs that include
`using-agent-skills` get the coding lifecycle routing table.
