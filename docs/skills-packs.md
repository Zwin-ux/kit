# Skills in runs

Kit does not copy a skill pack into run worktrees, and it adds no routing
text to the prompt. A run's worktree is a clean checkout of your repository
at `HEAD`, and the agent sees what that checkout holds plus its own config.

Skills reach an agent two ways:

1. **Kits.** `kit add <kit>` installs a kit's skills, rules, MCP servers and
   hooks into each agent's own config: your home directory for user scope (`--global`),
   the repository for repo scope. The agent loads them the way it loads any
   skill.
2. **Committed repo skills.** Skills committed in the repository (for example
   `.agents/skills/` or `.claude/skills/`) arrive with the worktree, because
   they are part of the checkout.

## Commit repo-scope files before `kit run`

A run sees only committed files. If `kit add` wrote kit files into the
repository and the agent should use them in a run, commit them first.
Uncommitted files stay in your checkout and never reach the worktree.

## Domain skills such as Harness

Any `name/SKILL.md` skill works this way. [Harness skills](https://github.com/harness/harness-skills/tree/main/skills)
also need the Harness MCP v2 server and API keys, configured in the agent's
own settings. Kit does not host MCP servers.
