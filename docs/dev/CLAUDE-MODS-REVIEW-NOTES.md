# Installed Claude 2.1.287 review notes

Source: locally generated `crates/kit-cli/claude-plugin/.claude-plugin/types/claude-code/index.d.ts`, whose first line is `// Written by Claude Code 2.1.287.` Full generated declarations remain excluded from Git. These abbreviated excerpts preserve the relevant field types.

## AbovePrompt

At generated lines 9537–9600, `RenderPropsOf.AbovePrompt` has:

```ts
AbovePrompt: {
  hasSurvey: boolean;
  isWorking: boolean;
  maxRows: number;
  bodyColumns: number;
  scroll: SiteScroll;
  view: SiteView;
};
```

The declarations describe `maxRows` and `bodyColumns` as read-only. `maxRows` is the available band height (the bottom slot includes the native prompt); a taller tree scrolls through `scroll.bodyRows`. `bodyColumns` becomes the transcript column width while a pane docks beside it. Rewriting/dropping `view` is refused. `RenderResultOf` (line 9648) maps each component to `RenderElement`.

[Official interface documentation](https://code.claude.com/docs/en/plugins/mods/interface#pick-where-to-draw) explains that returning a tree replaces downstream mods; composition includes `await next(e)` as a Box child. Our fix calls downstream with the original event, never rewrites the host's height budget, and avoids clipping opaque downstream controls. The core AbovePrompt drawing is empty; known plain-text/vertical-box trees receive conservative row bounds. Unknown layouts retain the whole site unchanged. Kit is still available through `/kit open`.

## Session lifecycle

Generated lines 10326–10369:

```ts
export type SessionEndInput = {
  reason: SessionEndReason;
  sessionId: string;
  resume: SessionResume;
};
export type SessionEndReason = ClassicHookInputs['SessionEnd']['reason'];
export type SessionEndResult = { sessionId: string };
```

The declaration says a `clear` can leave the process running under another session ID without another `session.start`. It documents clear/resume/logout/prompt_input_exit/other reasons. The [official Mods reference](https://code.claude.com/docs/en/plugins/mods/reference#session), rather than a separate branch enum, specifies that `/branch` reports `resume`.

The generation guard is therefore invalidated by `session.end`, not only by startup. The read→fill lock uses a unique transaction token so completing an obsolete read cannot clear a newer action's lock. No cancelled callback writes a saved draft back into the new session.

## Evidence limits

Offline tests hold `prompt.read` pending, fire `session.end`, then resolve the old read and assert no fill is issued. They also exercise all draft command paths, double-clicks, and an old transaction completing while the new session is already preparing a draft. Host-native focus/paint/cancellation still needs Mason's real terminal QA. No model calls, auth changes, prompt submission, or permission interception are involved.
