import React from "react";
import { Box, Text } from "ink";
import { motionEnabled, useIntervalFrame } from "../motion/index.js";
import { packIconGlyph, renderPackIconLines } from "./packIcons.js";
import { LAYOUT_CAPS } from "./layoutScale.js";

export interface PackIconProps {
  packName: string;
  /**
   * mini = single static ASCII glyph (list rows) — default
   * detail = small silhouette under selection
   * full = alias for detail
   */
  size?: "full" | "detail" | "mini";
  detailEdge?: number;
  /**
   * Animate detail silhouette only. Mini list icons never animate
   * (N timers caused flicker on ↑↓).
   */
  animate?: boolean;
}

/**
 * Pack logo — list rows are static single-cell ASCII.
 *
 * Prefer size="mini" in UI. Detail/full █ blocks invert to unreadable white
 * masses on dark terminals; ToolkitPicker no longer mounts them.
 */
export function PackIcon({
  packName,
  size = "mini",
  detailEdge = LAYOUT_CAPS.packDetailMax,
  animate = false,
}: PackIconProps): React.ReactElement {
  if (size === "mini") {
    // Always static — never start a timer on list rows
    return <Text>{packIconGlyph(packName, 0)}</Text>;
  }

  // Detail/full: keep API for tests/splash-adjacent callers, but dim the
  // silhouette so dark terminals don't get a solid white blob.
  const anim = animate && motionEnabled();
  const frame = useIntervalFrame(4, 280, anim);
  const edge = Math.max(3, Math.min(LAYOUT_CAPS.packDetailMax, detailEdge));
  const lines = renderPackIconLines(packName, {
    edge,
    frame: anim ? frame : 0,
  });

  return (
    <Box flexDirection="column" width={edge} height={edge} flexShrink={0}>
      {lines.map((line, i) => (
        <Text key={i} dimColor>
          {line}
        </Text>
      ))}
    </Box>
  );
}
