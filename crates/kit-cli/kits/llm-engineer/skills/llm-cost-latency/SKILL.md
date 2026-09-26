---
name: llm-cost-latency
description: Budget and reduce token cost and latency of model calls without losing quality.
---

# LLM cost and latency

Use when adding a model call, or when calls are slow or expensive.

1. Measure first: input tokens, output tokens, latency (p50 and p95) and cost per call, from logs or a small benchmark.
2. Cut input: remove unused context, cache stable prefixes (prompt caching) where the provider supports it, and retrieve fewer, better chunks.
3. Cut output: ask for the exact format needed and set a max token limit.
4. Pick the smallest model that passes the evals (see the llm-evals skill); route only hard cases to larger models.
5. Batch or stream where the product allows it.
6. Report before and after numbers and the eval score, so a cost cut never hides a quality drop.
