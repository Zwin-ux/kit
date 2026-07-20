import React from "react";
import { Box, Text } from "ink";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";

export interface ScreenShellProps {
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
  children: React.ReactNode;
  /** Force-hide brand art (tests / dense modes). */
  hideMascot?: boolean;
  /** Optional focus line under header area (a11y: "3/7 web-app"). */
  focusLabel?: string;
}

/**
 * Menu-first shell:
 * - stack  → full-width menu; optional tiny top brand
 * - split/wide → compact left rail + primary menu column
 *
 * Menu always owns the interactive surface. Mascot never steals narrow windows.
 */
export function ScreenShell({
  frames,
  mascotVariant = "idle",
  children,
  hideMascot,
  focusLabel,
}: ScreenShellProps): React.ReactElement {
  const scale = useLayoutScale();
  const place = hideMascot ? "hidden" : scale.mascotPlacement;

  const menu = (
    <Box
      flexDirection="column"
      flexGrow={1}
      flexShrink={1}
      minWidth={scale.contentMinCols}
      width={place === "rail" ? undefined : "100%"}
    >
      {focusLabel ? (
        <Text bold wrap="truncate">
          {focusLabel}
        </Text>
      ) : null}
      {children}
    </Box>
  );

  // --- stack: menu first, full width ---
  if (place === "hidden") {
    return (
      <Box flexDirection="column" flexGrow={1} width="100%">
        {menu}
      </Box>
    );
  }

  if (place === "top") {
    return (
      <Box flexDirection="column" flexGrow={1} width="100%">
        <Box
          flexShrink={0}
          height={scale.railRows}
          width={Math.min(scale.railCols, scale.columns - scale.padX * 2)}
          overflow="hidden"
          marginBottom={1}
        >
          <MascotPlayer
            frames={frames}
            playing
            size="compact"
            variant={mascotVariant}
          />
        </Box>
        {menu}
      </Box>
    );
  }

  // --- split / wide: fluid rail (grows on fullscreen) + primary menu ---
  const railGap = scale.mode === "wide" ? 3 : 2;
  return (
    <Box flexDirection="row" flexGrow={1} width="100%">
      <Box
        flexDirection="column"
        marginRight={railGap}
        flexShrink={0}
        width={scale.railCols}
        height={scale.railRows}
        overflow="hidden"
      >
        <MascotPlayer
          frames={frames}
          playing
          size="compact"
          variant={mascotVariant}
        />
      </Box>
      {menu}
    </Box>
  );
}
