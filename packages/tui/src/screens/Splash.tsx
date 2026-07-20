import React from "react";
import { Box, Text } from "ink";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import type { PixelFrame } from "../mascot/types.js";
import { Header, Footer } from "../components/Chrome.js";

export interface SplashProps {
  frames: PixelFrame[];
  usingFiles: boolean;
}

/**
 * Splash is the in-terminal kit-idle experience.
 * Same frames as assets/pixel/kit-idle.gif.
 */
export function Splash({
  frames,
  usingFiles,
}: SplashProps): React.ReactElement {
  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header screen="Splash" detail="kit-idle" />

      <Box marginTop={1}>
        <MascotPlayer
          frames={frames}
          playing
          showCounter
          label={
            usingFiles
              ? "kit-idle · assets/pixel (live)"
              : "kit-idle · built-in placeholder"
          }
          caption="Same loop as assets/pixel/kit-idle.gif"
        />
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Kit</Text>
        <Text dimColor>Portable Agent Skills · terminal only</Text>
      </Box>

      <Footer keys="any key continue · q quit" />
    </Box>
  );
}
