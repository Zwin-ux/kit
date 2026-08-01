---
name: data-check
description: Review data scripts and notebooks for clarity, leakage, and reproducibility.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Inspect data load paths, seeds, and train/test splits for leakage.
2. Prefer small, reproducible samples over silent full-dataset runs.
3. Call out missing schema docs, null handling, and unit assumptions.
4. Keep notebooks linear: clear cells, no hidden state, named outputs.
5. Suggest a minimal check command or test for the critical transform.
6. Summarize risks and what to verify before shipping a model or report.
