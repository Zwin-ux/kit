---
name: llm-evals
description: Build a small eval set before changing a prompt, model or retrieval step, and compare results before and after.
---

# LLM evals

Use when changing a prompt, switching models, or changing retrieval.

1. Find existing evals (`evals/`, `tests/evals`, a promptfoo or similar config). Use them if present.
2. If none exist, write 10 to 30 cases that cover the main task, known failures and edge cases. Store them as data (JSONL or YAML), not code.
3. Pick a check per case: exact match, contains, JSON schema, or a rubric graded by a model. Prefer the cheapest check that catches the failure.
4. Run the evals on the current version and record the score.
5. Make the change. Run the evals again.
6. Report both scores, the cases that changed, and the cost and latency difference.
7. Never tune the prompt on the eval cases alone; keep a few held out.
