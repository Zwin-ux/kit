import React from "react";
import { render } from "ink";
import { App } from "./App.js";

/** Start the interactive Kit TUI. */
export function startTui(): void {
  render(<App />);
}
