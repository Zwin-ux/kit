const ENTER_ALTERNATE_SCREEN = "\x1b[?1049h\x1b[2J\x1b[H";
const LEAVE_ALTERNATE_SCREEN = "\x1b[?1049l";

export interface TerminalWriter {
  isTTY?: boolean;
  write(value: string): unknown;
}

/**
 * Give Kit its own terminal buffer, like a native coding-agent TUI.
 * The original shell and scrollback return when Kit exits.
 */
export function enterAlternateScreen(
  output: TerminalWriter = process.stdout,
): () => void {
  if (
    process.env.KIT_NO_ALT_SCREEN === "1" ||
    output.isTTY !== true
  ) {
    return () => {};
  }

  let active = true;
  output.write(ENTER_ALTERNATE_SCREEN);

  const leave = () => {
    if (!active) return;
    active = false;
    try {
      output.write(LEAVE_ALTERNATE_SCREEN);
    } catch {
      // The terminal can already be closed during process shutdown.
    }
    process.off("exit", leave);
  };
  process.on("exit", leave);
  return leave;
}

export const alternateScreenCodes = {
  enter: ENTER_ALTERNATE_SCREEN,
  leave: LEAVE_ALTERNATE_SCREEN,
} as const;
