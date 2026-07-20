import React from "react";
import { Box, Text } from "ink";
import { motionEnabled, useIntervalFrame } from "../motion/index.js";
import { packIconGlyph, renderPackIconLines } from "./packIcons.js";
import { LAYOUT_CAPS } from "./layoutScale.js";

export interface PackIconProps {
  packName: string;
  /**
   * mini = single animated glyph (list rows) — default everywhere
   * detail = tiny silhouette (≤4 rows) under selection
   * full = alias for detail
   */
  size?: "full" | "detail" | "mini";
  /** Pixel edge for detail (clamped 3–4). */
  detailEdge?: number;
  animate?: boolean;
}

/**
 * Pack logo — glyph-first. Detail is intentionally tiny (≤ half the fox).
 */
export function PackIcon({
  packName,
  size = "mini",
  detailEdge = LAYOUT_CAPS.packDetailMax,
  animate = true,
}: PackIconProps): React.ReactElement {
  const anim = animate && motionEnabled();
  const frame = useIntervalFrame(4, 280, anim);

  if (size === "mini") {
    return <Text>{packIconGlyph(packName, anim ? frame : 0)}</Text>;
  }

  // Hard clamp — never 6–10 again
  const edge = Math.max(3, Math.min(LAYOUT_CAPS.packDetailMax, detailEdge));
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
