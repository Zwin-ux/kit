import React from "react";
import { Box, Text } from "ink";
import { motionEnabled, useIntervalFrame } from "../motion/index.js";
import {
  STATUS_ICON_SIZE,
  SPINNER_ICON_FRAMES,
  getStatusIconBitmap,
  renderStatusIconLines,
  statusIconGlyph,
  type StatusIconId,
} from "./statusIcons.js";

export interface StatusIconProps {
  /** Static icon id, or "spinner" for the 4-frame busy cycle. */
  id: StatusIconId | "spinner" | string;
  /**
   * mini = single glyph (list rows)
   * full = 8-line silhouette block
   */
  size?: "full" | "mini";
  /** When id is spinner, control whether frames advance. Default true. */
  active?: boolean;
  /** Optional color for mini glyphs (Ink color names). */
  color?: string;
  dimColor?: boolean;
}

/**
 * Status / type glyph — same pure black language as pack icons and the fox.
 */
export function StatusIcon({
  id,
  size = "mini",
  active = true,
  color,
  dimColor,
}: StatusIconProps): React.ReactElement {
  const isSpinner = id === "spinner";
  const frameIndex = useIntervalFrame(
    SPINNER_ICON_FRAMES.length,
    100,
    isSpinner && active && motionEnabled(),
  );

  const resolvedId: string = isSpinner
    ? motionEnabled() && active
      ? SPINNER_ICON_FRAMES[frameIndex]!
      : active
        ? "spinner-0"
        : "ok"
    : id;

  if (size === "mini") {
    const glyph = statusIconGlyph(resolvedId);
    if (color) {
      return (
        <Text color={color} {...(dimColor ? { dimColor: true as const } : {})}>
          {glyph}
        </Text>
      );
    }
    return (
      <Text {...(dimColor ? { dimColor: true as const } : {})}>{glyph}</Text>
    );
  }

  // Full 8×8: for spinner, re-render from bitmap each frame
  const lines = renderStatusIconLines(resolvedId);
  // touch bitmap length so bad icons fail loudly in tests
  void getStatusIconBitmap(resolvedId);

  return (
    <Box flexDirection="column" width={STATUS_ICON_SIZE} flexShrink={0}>
      {lines.map((line, i) => (
        <Text key={i} {...(dimColor ? { dimColor: true as const } : {})}>
          {line}
        </Text>
      ))}
    </Box>
  );
}
