import React from "react";
import { Text } from "ink";
import {
  selectCursorGlyph,
  type SelectDirection,
} from "./fixedLines.js";

export type { SelectDirection };


/**
 * Fixed-width (2 col) selection cursor — one paint per keypress.
 * ASCII only so Windows terminals never reflow on glyph width.
 */
export function SelectPulse(props: {
  selected: boolean;
  /** Kept for API compat; no delayed hot phase (that caused double glitch). */
  tick?: number;
  direction?: SelectDirection;
}): React.ReactElement {
  void props.tick;
  const glyph = selectCursorGlyph(
    props.selected,
    props.direction ?? "none",
  );
  return (
    <Text bold={props.selected}>{glyph}</Text>
  );
}
