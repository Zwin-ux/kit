---
name: product-spec
description: Turn a concrete user outcome into a small implementation specification.
user-invocable: false
---

# Product Spec

1. Read the task and canonical product context. Identify the user, triggering situation and measurable outcome; distinguish supplied facts from assumptions.
2. Describe the current behavior and the intended change with a concrete example.
3. Define one primary flow, essential edge cases and clear non-goals. Avoid speculative feature expansion.
4. Write acceptance criteria that a reviewer can observe or test. Include failure and recovery paths, accessibility and persistence only where relevant.
5. Identify dependencies, existing interfaces and open decisions. Ask only for information that materially changes the implementation.
6. Break the work into small complete slices with evidence required for each. Keep infrastructure details out of user-facing copy unless they help a decision.
7. Return the specification, assumptions and verification plan. Do not implement unrelated features or create external tickets without authorization.

Provenance: first-party Kit 2.0.0 starter knowledge, MIT; authored in this plugin.
