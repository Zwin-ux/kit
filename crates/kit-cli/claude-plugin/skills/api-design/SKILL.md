---
name: api-design
description: Implement a minimal API contract with validation, authorization and predictable errors.
user-invocable: false
---

# Api Design

1. Read the existing route, schema, callers and tests before proposing a contract. Keep compatibility unless the task explicitly changes it.
2. State accepted input, output, error shape, authentication and resource authorization. Never trust a caller-supplied owner or tenant identifier by itself.
3. Validate input at the boundary. Parameterize queries and restrict returned fields. Keep credentials and private payloads out of logs.
4. Define idempotency and transaction boundaries for mutations; consider concurrent requests and retries. Do not add a queue or framework without a concrete need.
5. Implement one complete request path with the smallest change. Preserve existing public behavior outside the task.
6. Test success and relevant malformed, unauthorized, missing-resource, duplicate and dependency-failure cases.
7. Return the contract change, files, test evidence and migration requirements. Never deploy or mutate production data merely to verify a local change.

Provenance: first-party Kit 2.0.0 starter knowledge, MIT; authored in this plugin.
