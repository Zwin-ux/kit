import React from "react";
import { Box } from "ink";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";

export interface ScreenShellProps {
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
  /** compact rail (default) or hero splash */
  mascotSize?: "compact" | "hero" | "full";
  children: React.ReactNode;
  /** Hide mascot (rare). */
  hideMascot?: boolean;
}

/**
 * Full-screen-safe row layout: fixed mascot rail + flexible content.
 * Content shrinks/wraps; mascot never gets clipped by menu width.
 */
export function ScreenShell({
  frames,
  mascotVariant = "idle",
  mascotSize = "compact",
  children,
  hideMascot,
}: ScreenShellProps): React.ReactElement {
  const scale = useLayoutScale();

  if (hideMascot) {
    return (
      <Box flexDirection="column" flexGrow={1} flexShrink={1}>
        {children}
      </Box>
    );
  }

  return (
    <Box flexDirection="row" flexGrow={1}>
      <Box
        flexDirection="column"
        marginRight={2}
        flexShrink={0}
        width={scale.railCols}
      >
        <MascotPlayer
          frames={frames}
          playing
          size={mascotSize}
          variant={mascotVariant}
        />
      </Box>
      <Box flexDirection="column" flexGrow={1} flexShrink={1} minWidth={24}>
        {children}
      </Box>
    </Box>
  );
}
