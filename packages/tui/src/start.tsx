import React from "react";
import { render } from "ink";
import { App } from "./App.js";
import { disableMouse, installMouseCleanup } from "./mouse/enableMouse.js";

/**
 * Start the interactive Kit TUI.
 * Requires a real TTY. Mouse click-to-select enabled when supported.
 */
export function startTui(): void {
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
    console.error("Disable mouse: KIT_NO_MOUSE=1");
    process.exit(1);
  }

  installMouseCleanup();
  const instance = render(<App />);
  const cleanup = () => {
    disableMouse();
  };
  instance.waitUntilExit().then(cleanup).catch(cleanup);
}