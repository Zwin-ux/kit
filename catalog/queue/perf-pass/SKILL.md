---
name: perf-pass
description: Find and fix high-impact performance issues with measurements, not guesses.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
  - codex
---

# Instructions

1. Define the user-visible symptom (slow page, hot endpoint, long CLI, jank).
2. Measure before changing code: timing, profiles, logs, or a simple before/after script.
3. Hunt common wins first: N+1 queries, unbounded work, missing caches, huge payloads, sync I/O on hot paths.
4. Prefer one high-impact fix over many micro-optimizations.
5. Keep behavior identical unless a tradeoff is explicit and accepted.
6. Re-measure after the change.
7. Report:
   - Bottleneck
   - Fix
   - Before / after evidence
   - Residual risks
