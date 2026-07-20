import React from "react";
import { Text } from "ink";
import { useIntervalFrame } from "./useIntervalFrame.js";
import { motionEnabled } from "./motionEnabled.js";

/**
 * Terminal block cursor blink — use after typewriter or on filter prompts.
 */
export function BlinkCursor(props: {
  active?: boolean;
  /** Characters to cycle; default block / empty. */
  frames?: [string, string];
}): React.ReactElement | null {
  const active = props.active !== false;
  const frames = props.frames ?? (["▌", " "] as [string, string]);
  const i = useIntervalFrame(2, 530, active && motionEnabled());

  if (!active) return null;
  if (!motionEnabled()) {
    return <Text>{frames[0]}</Text>;
  }
  return <Text>{frames[i]}</Text>;
}
