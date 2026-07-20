import React from "react";
import { Box, Text } from "ink";
import { StatusIcon } from "../mascot/StatusIcon.js";
import { motionEnabled } from "./motionEnabled.js";
import { useIntervalFrame } from "./useIntervalFrame.js";

const BRAILLE = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

export interface SpinnerProps {
  label?: string;
  active?: boolean;
  /**
   * braille = dense unicode spinner (default, compact)
   * icon = 4-frame status bitmap glyph (same family as StatusIcon)
   */
  style?: "braille" | "icon";
}

/**
 * Busy indicator for load / install / scan.
 * Respects KIT_REDUCED_MOTION=1 (static glyph).
 */
export function Spinner(props: SpinnerProps): React.ReactElement {
  const active = props.active !== false;
  const style = props.style ?? "braille";
  const i = useIntervalFrame(BRAILLE.length, 80, active && motionEnabled());

  if (style === "icon") {
    return (
      <Box>
        <StatusIcon id="spinner" size="mini" active={active} />
        {props.label ? <Text>{` ${props.label}`}</Text> : null}
      </Box>
    );
  }

  return (
    <Text>
      {active && motionEnabled() ? BRAILLE[i] : active ? "…" : "✓"}
      {props.label ? ` ${props.label}` : ""}
    </Text>
  );
}
