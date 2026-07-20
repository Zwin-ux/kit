import React from "react";
import { Box, Text } from "ink";
import type { PixelFrame } from "../mascot/types.js";
import { renderFrame } from "../mascot/renderBitmap.js";

export interface SplashProps {
  frame: PixelFrame;
  frameIndex: number;
  frameCount: number;
  usingFiles: boolean;
}

export function Splash({
  frame,
  frameIndex,
  frameCount,
  usingFiles,
}: SplashProps): React.ReactElement {
  const art = renderFrame(frame);

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Text>{art}</Text>
      <Box marginTop={1} flexDirection="column">
        <Text bold>Kit</Text>
        <Text dimColor>Portable Agent Skills · terminal only</Text>
        <Text dimColor>
          mascot {frameIndex + 1}/{frameCount}
          {usingFiles ? " · assets/pixel" : " · placeholder silhouette"}
        </Text>
        <Box marginTop={1}>
          <Text>Press any key to continue · q to quit</Text>
        </Box>
      </Box>
    </Box>
  );
}
