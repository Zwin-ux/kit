import React from "react";
import { Box, Text } from "ink";
import { motionEnabled, useIntervalFrame } from "../motion/index.js";
import {
  packIconGlyph,
  renderPackIconLines,
} from "./packIcons.js";

export interface PackIconProps {
  packName: string;
  /**
   * mini = single animated glyph (list rows)
   * detail = small silhouette (selected pack) — always smaller than mascot
   * full = legacy alias for detail
   */
  size?: "full" | "detail" | "mini";
  /** Pixel edge for detail mode (6–8). From layout scale. */
  detailEdge?: number;
  /** Animate glyph / silhouette. Default true when motion enabled. */
  animate?: boolean;
}

/**
 * Silhouette icon for a starter pack — same pure black language as kit-idle.
 * Detail size is intentionally smaller than the mascot rail so logos never dwarf the fox.
 */
export function PackIcon({
  packName,
  size = "mini",
  detailEdge = 8,
  animate = true,
}: PackIconProps): React.ReactElement {
  const anim = animate && motionEnabled();
  const frame = useIntervalFrame(4, 280, anim);

  if (size === "mini") {
    return <Text>{packIconGlyph(packName, anim ? frame : 0)}</Text>;
  }

  // detail / full — fixed edge, animated pulse, single-width cells
  const edge = Math.max(6, Math.min(10, detailEdge));
  const lines = renderPackIconLines(packName, {
    edge,
    frame: anim ? frame : 0,
  });

  return (
    <Box flexDirection="column" width={edge} height={edge} flexShrink={0}>
      {lines.map((line, i) => (
        <Text key={i} wrap="truncate">
          {line}
        </Text>
      ))}
    </Box>
  );
}
