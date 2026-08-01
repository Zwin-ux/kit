import React from "react";
import { render } from "ink";
import { App } from "./App.js";
import { disableMouse, installMouseCleanup } from "./mouse/enableMouse.js";
import { enterAlternateScreen } from "./terminal/alternateScreen.js";

/**
 * Start the interactive Kit TUI.
 * Default: Setup wizard (install skills + link agents for this repo).
 * Advanced multi-lane: kit tui workbench
 */
export function startTui(
  options: { initialScreen?: "setup" | "workbench" | "home" } = {},
): void {
  if (!process.stdin.isTTY) {
    console.error("kit tui needs an interactive terminal (stdin is not a TTY).");
    console.error("");
    console.error("In PowerShell (repo root, after build):");
    console.error("  pnpm kit tui");
    console.error("  pnpm tui");
    console.error("");
    console.error("Global install:");
    console.error("  kit tui");
    console.error("");
    console.error("Advanced: kit tui workbench");
    process.exit(1);
  }

  // Default: Setup wizard — one job, not a multi-lane hub
  const initialScreen = options.initialScreen ?? "setup";

  const leaveAlternateScreen = enterAlternateScreen();
  installMouseCleanup();
  const cleanup = () => {
    disableMouse();
    leaveAlternateScreen();
  };
  try {
    const instance = render(<App initialScreen={initialScreen} />);
    instance.waitUntilExit().then(cleanup).catch(cleanup);
  } catch (error) {
    cleanup();
    throw error;
  }
}
