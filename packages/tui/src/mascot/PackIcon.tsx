import React from "react";
import { Box, Text } from "ink";
import {
  PACK_ICON_SIZE,
  packIconGlyph,
  renderPackIconLines,
} from "./packIcons.js";

export interface PackIconProps {
  packName: string;
  /**
   * full = 16-line silhouette (selected detail)
   * mini = single glyph for list rows
   */
  size?: "full" | "mini";
}

/**
 * Silhouette icon for a starter pack — same pure black language as kit-idle.
 */
export function PackIcon({
  packName,
  size = "mini",
}: PackIconProps): React.ReactElement {
  if (size === "mini") {
    return <Text>{packIconGlyph(packName)}</Text>;
  }

  const lines = renderPackIconLines(packName);
  return (
    <Box flexDirection="column" width={PACK_ICON_SIZE} flexShrink={0}>
      {lines.map((line, i) => (
        <Text key={i}>{line}</Text>
      ))}
    </Box>
  );
}
