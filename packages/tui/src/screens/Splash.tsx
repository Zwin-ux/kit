import React from "react";
import { Box, Text } from "ink";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import type { PixelFrame } from "../mascot/types.js";
import { Header, Footer } from "../components/Chrome.js";
import { TypeLine } from "../motion/index.js";

export interface SplashProps {
  frames: PixelFrame[];
}

/**
 * Splash — full kit-idle loop scaled for the terminal (hero on full-screen).
 * No asset-path / debug chrome — product face only.
 */
export function Splash({ frames }: SplashProps): React.ReactElement {
  return (
    <Box flexDirection="column" paddingX={2} paddingY={1} width="100%">
      <Header screen="Splash" />

      <Box marginTop={1} flexShrink={0}>
        <MascotPlayer frames={frames} playing size="hero" variant="idle" />
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Kit</Text>
        <TypeLine text="Portable agent skills" dimColor cursor cps={30} />
      </Box>

      <Footer keys="any key · q quit" />
    </Box>
  );
}
