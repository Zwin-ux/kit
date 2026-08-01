import React from "react";
import { Box, Text } from "ink";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import type { PixelFrame } from "../mascot/types.js";
import { Footer, Header } from "../components/Chrome.js";
import { mascotAnimEnabled, mascotVisible } from "../brand/mascotPolicy.js";

export interface SplashProps {
  frames: PixelFrame[];
}

/**
 * Optional nostalgia gate (KIT_TUI_SPLASH=1 only).
 * Default product path never shows this — kit opens like Claude/Grok.
 * Static by default; multi-frame only with KIT_MASCOT_ANIM=1.
 */
export function Splash({ frames }: SplashProps): React.ReactElement {
  const showArt = mascotVisible() && frames.length > 0;

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1} width="100%">
      <Header screen="Kit" detail="press any key" />

      {showArt ? (
        <Box marginTop={1} flexShrink={0}>
          <MascotPlayer
            frames={frames}
            playing={mascotAnimEnabled()}
            size="hero"
            variant="idle"
          />
        </Box>
      ) : (
        <Box marginTop={1}>
          <Text bold inverse>
            {" "}
            KIT{" "}
          </Text>
        </Box>
      )}

      <Box marginTop={1} flexDirection="column">
        <Text dimColor>Action terminal for skills, agents, and services</Text>
      </Box>

      <Footer keys="any key open · q quit" />
    </Box>
  );
}
