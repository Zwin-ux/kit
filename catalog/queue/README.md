# Skill queue

Curated skills waiting to land in `skills/`.

The **keep-alive** cron promotes **one** queue skill per successful run:

1. Oldest folder under `catalog/queue/<name>/` with a valid `SKILL.md`
2. Moves it to `skills/<name>/`
3. Syncs `skills/README.md`
4. Opens a PR for human review

## Add to the queue

```text
catalog/queue/my-skill/SKILL.md
```

Same schema as live skills (`docs/dev/SKILL_SCHEMA.md`).  
Keep instructions short, concrete, and multi-agent friendly.

## Rules

- No auto-merge to `main` — PRs only
- One promotion per keep-alive run
- Skip if `skills/<name>` already exists
- Queue order = directory sort order (name)
