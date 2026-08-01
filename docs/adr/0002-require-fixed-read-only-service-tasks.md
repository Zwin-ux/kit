# ADR-0002: Require fixed read-only service tasks

## Status

Accepted

## Context

Raw plugin arguments are useful for experts, but they are hard to discover in
a TUI. Kit also needs a narrow service surface that cannot silently grow into
wallet or trade authority.

## Decision

A plugin may publish named tasks with a description, fixed arguments, and the
access value `read-only`.

Kit will list and run these tasks. Kit will reject other access values in the
current schema. Raw `plugin run` remains available as an expert command.

Trenchwire will publish only:

- `health` for provider checks;
- `market` for public market facts.

## Consequences

### Positive

- The TUI can show useful actions without knowing domain commands.
- A task run is repeatable and easy to prove.
- Trenchwire does not expose trade commands through the Kit task shelf.

### Negative

- The manifest has more fields to validate.
- A false `read-only` claim is still possible in an untrusted plugin.

### Neutral

- Manifest digest review remains the trust gate.
- The plugin remains responsible for its network and write behavior.

## Alternatives considered

### Let Kit classify raw arguments

Rejected. Kit would need domain knowledge and could misclassify a command.

### Expose every command as a task

Rejected. The task shelf is for a small safe surface, not a second help system.

## References

- [Workbench architecture](../dev/WORKBENCH_ARCHITECTURE.md)
- [Local CLI plugins](../plugins.md)

