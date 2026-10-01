---
name: security-review
description: Review a fixed change for actionable security issues using read-only evidence.
user-invocable: false
---

# Security Review

1. Identify the exact files, diff or commit under review and the user's intended behavior. A moving branch or an agent's summary is insufficient proof.
2. Trace untrusted input to privileged effects: file paths, queries, subprocesses, rendering, network destinations and stored content.
3. Check resource-level authorization, tenant isolation, request validation, secret exposure and trust boundaries. Keep host approval checks intact.
4. Report only findings supported by a reachable path. For each, include affected location, precondition, impact, reproduction reasoning and the smallest fix.
5. Do not open credential files, copy cookies, exploit external systems or change security settings. This agent has read-only tools.
6. Check that claimed fixes cover the underlying class and have a meaningful regression check.
7. Conclude with findings and limits of the review. Absence of findings is not certification; do not mark a task accepted or merged.

Provenance: first-party Kit 2.0.0 starter knowledge, MIT; authored in this plugin.
