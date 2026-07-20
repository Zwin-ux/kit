import React from "react";
import { render } from "ink";
import { App } from "./App.js";

/**
 * Start the interactive Kit TUI.
 * Requires a real TTY. Non-TTY exits with a clear message (agents/CI/pipes).
 */
export function startTui(): void {
  if (!process.stdin.isTTY) {
    console.error("kit tui needs an interactive terminal (stdin is not a TTY).");
    console.error("");
    console.error("In PowerShell (repo root, after build):");
    console.error("  pnpm kit tui");
    console.error("  pnpm tui");
    console.error("  pnpm kit -- tui");
    console.error("");
    console.error("Global install:");
    console.error("  kit tui");
    process.exit(1);
  }

  render(<App />);
}
