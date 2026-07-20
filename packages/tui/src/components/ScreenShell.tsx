import React from "react";
import { Box } from "ink";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";

export interface ScreenShellProps {
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
  children: React.ReactNode;
  hideMascot?: boolean;
}

/**
 * Full-screen-safe row layout:
 * fixed small mascot rail + flexible content (gets the rest of the window).
 * Shell is always compact rail — never hero.
 */
export function ScreenShell({
  frames,
  mascotVariant = "idle",
  children,
  hideMascot,
}: ScreenShellProps): React.ReactElement {
  const scale = useLayoutScale();

  if (hideMascot) {
    return (
      <Box
        flexDirection="column"
        flexGrow={1}
        flexShrink={1}
        minWidth={scale.contentMinCols}
        width="100%"
      >
        {children}
      </Box>
    );
  }

  return (
    <Box flexDirection="row" flexGrow={1} width="100%">
      <Box
        flexDirection="column"
        marginRight={1}
        flexShrink={0}
        width={scale.railCols}
        overflow="hidden"
      >
        <MascotPlayer
          frames={frames}
          playing
          size="compact"
          variant={mascotVariant}
        />
      </Box>
      <Box
        flexDirection="column"
        flexGrow={1}
        flexShrink={1}
        minWidth={scale.contentMinCols}
      >
        {children}
      </Box>
    </Box>
  );
}
