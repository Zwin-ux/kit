# ADR-0001: Use local process adapters for Workbench

## Status

Accepted

## Context

Kit needs to run coding tools and attached CLI services from one TUI. Codex,
Claude Code, Grok Build, and Trenchwire already have local command interfaces.
Kit must not copy their authentication, domain logic, or safety rules.

## Decision

Kit will use small local process adapters.

Each coding runner has a fixed argument builder for `inspect` and `build`.
Each attached service publishes fixed read-only tasks in `kit.plugin.json`.
Kit starts each process with an argument array and `shell: false`.

## Consequences

### Positive

- Kit can use existing provider login and model settings.
- The first release needs no server or background daemon.
- Provider and service rules stay in their own products.
- The process boundary is easy to test with fixture executables.

### Negative

- Provider CLI changes can require adapter updates.
- Output is text, not a rich shared protocol.
- A running job cannot survive the Kit process in the first release.

### Neutral

- Kit is a local supervisor, not an LLM proxy.
- Trenchwire is one service attached through the same public contract.

## Alternatives considered

### Call each model API from Kit

Rejected for the first release. This would add key storage, billing, streaming,
model policy, and a second authentication path.

### Run a local daemon

Rejected for the first release. A daemon adds lifecycle, port, log, upgrade,
and recovery work before the job contract is proven.

### Embed Kit code in every service

Rejected. It couples release cycles and can move domain authority into Kit.

## References

- [Workbench architecture](../dev/WORKBENCH_ARCHITECTURE.md)
- [Local CLI plugins](../plugins.md)

