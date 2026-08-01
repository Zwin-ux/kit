export interface QuitShortcutState {
  input: string;
  busy: boolean;
  enteringText: boolean;
  awaitingChoice: boolean;
}

/**
 * Q quits only from a stable navigation state.
 * Text fields and confirmation prompts own the key while they are active.
 */
export function shouldQuitWithQ(state: QuitShortcutState): boolean {
  return (
    state.input === "q" &&
    !state.busy &&
    !state.enteringText &&
    !state.awaitingChoice
  );
}
