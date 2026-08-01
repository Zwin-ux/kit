# ADR-0003: Run Ollama through the official Codex bridge

## Status

Accepted

## Context

Kit needs a local-model option for coding work. A direct call to
`POST /api/generate` can return text, but it does not give the model safe file,
search, test, or patch tools. Building that agent loop inside Kit would duplicate
tool policy, sandbox, approval, and recovery logic.

Ollama provides an official Codex integration through `ollama launch codex`.
Codex also supports the read-only and workspace-write sandboxes that Workbench
uses.

## Decision

Kit reads installed model names from Ollama's local `GET /api/tags` endpoint.
When the user selects one, Kit starts the official bridge with:

```text
ollama launch codex --model <name> --yes -- exec
```

The child receives an isolated `CODEX_HOME` under `~/.kit/runtime`. This prevents
an unrelated global plugin or skill catalog from consuming the local model's
small context window. Project instructions still come from the selected repo.

Kit keeps its existing mode contract:

- `inspect` uses the read-only sandbox;
- `build` uses the workspace-write sandbox and needs confirmation;
- jobs remain bounded, streamed, and cancellable.

The default endpoint is `http://127.0.0.1:11434`. A user can set
`KIT_OLLAMA_HOST` to an explicit HTTP or HTTPS origin.

## Consequences

### Positive

- A local model gets real coding tools.
- Kit stores no model key and sends no work to a hosted model.
- The same inspect/build policy applies to local and hosted runners.
- Kit does not need its own model tool protocol.

### Negative

- Ollama coding jobs require both Ollama and Codex.
- Model capability varies. Small models can fail at tool use.
- Changes to Codex's local-provider flags can require an adapter update.

### Neutral

- Ollama remains the model server. Codex remains the coding-agent harness.
- Ollama owns local-provider setup. Kit remains the local supervisor and
  terminal interface.

## References

- [Workbench architecture](../dev/WORKBENCH_ARCHITECTURE.md)
- [Ollama: list models](https://docs.ollama.com/api/tags)
- [Ollama: generate a response](https://docs.ollama.com/api/generate)
