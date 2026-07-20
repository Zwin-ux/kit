import React, { useEffect, useMemo, useState } from "react";
import { Box, Text } from "ink";
import { motionEnabled } from "../motion/motionEnabled.js";
import {
  renderFrameLines,
  renderedCellWidth,
  type RenderFrameOptions,
} from "./renderBitmap.js";
import {
  frameDelayForVariant,
  type MascotVariant,
  type PixelFrame,
} from "./types.js";
import { useLayoutScale } from "./useLayoutScale.js";
import type { LayoutScale } from "./layoutScale.js";

export interface MascotPlayerProps {
  frames: PixelFrame[];
  playing?: boolean;
  delayMs?: number;
  /**
   * compact / auto = side rail (single █, hard-capped)
   * hero / full = splash only (still capped)
   */
  size?: "full" | "compact" | "hero" | "auto";
  caption?: string;
  showCounter?: boolean;
  label?: string;
  variant?: MascotVariant;
  scale?: LayoutScale;
}

/**
 * Live terminal playback of the Kit mascot.
 * Rail never uses double-wide cells. Canvas letterboxed so frames don't jump.
 */
export function MascotPlayer({
  frames,
  playing = true,
  delayMs,
  size = "compact",
  caption,
  showCounter = false,
  label,
  variant = "idle",
  scale: scaleProp,
}: MascotPlayerProps): React.ReactElement {
  const liveScale = useLayoutScale();
  const scale = scaleProp ?? liveScale;
  const [index, setIndex] = useState(0);
  const frameCount = frames.length;
  const enabled = motionEnabled();
  const effectiveDelay = delayMs ?? frameDelayForVariant(variant);
  const shouldPlay = playing && enabled && frameCount > 1;

  useEffect(() => {
    if (!shouldPlay) return;
    const timer = setInterval(() => {
      setIndex((current) => (current + 1) % frameCount);
    }, effectiveDelay);
    return () => clearInterval(timer);
  }, [shouldPlay, frameCount, effectiveDelay]);

  useEffect(() => {
    setIndex(0);
  }, [variant, frameCount]);

  const frame = frames[enabled ? index : 0] ?? frames[0];
  const isHero = size === "full" || size === "hero";

  const renderOpts: RenderFrameOptions = useMemo(() => {
    if (isHero) {
      const double = scale.splashCell === "double";
      return {
        cell: double ? "██" : "█",
        empty: double ? "  " : " ",
        tight: true,
        pad: 0,
        fit: {
          width: scale.splashFit.width,
          height: scale.splashFit.height,
        },
      };
    }
    // Rail: always single-width, capped fit from layout scale
    return {
      cell: "█",
      empty: " ",
      tight: true,
      pad: 0,
      fit: {
        width: scale.mascotFit.width,
        height: scale.mascotFit.height,
      },
    };
  }, [isHero, scale]);

  const lines = useMemo(() => {
    if (!frame) return [] as string[];
    return renderFrameLines(frame, renderOpts);
  }, [frame, renderOpts]);

  const width = useMemo(() => {
    if (!frame) return 0;
    return renderedCellWidth(frame, renderOpts);
  }, [frame, renderOpts]);

  const height = lines.length;
  // Never exceed reserved rail on menus
  const maxWidth = isHero
    ? Math.max(width, 1)
    : Math.min(Math.max(width, 1), scale.railCols);

  if (!frame) {
    return (
      <Box>
        <Text dimColor>mascot unavailable</Text>
      </Box>
    );
  }

  return (
    <Box flexDirection="column" flexShrink={0} width={maxWidth}>
      {label ? <Text dimColor>{label}</Text> : null}
      <Box
        flexDirection="column"
        width={maxWidth}
        height={height}
        flexShrink={0}
        overflow="hidden"
      >
        {lines.map((line, i) => (
          <Text key={i} wrap="truncate">
            {line.length > maxWidth ? line.slice(0, maxWidth) : line}
          </Text>
        ))}
      </Box>
      {showCounter ? (
        <Text dimColor>
          {index + 1}/{frameCount}
        </Text>
      ) : null}
      {caption ? <Text dimColor>{caption}</Text> : null}
    </Box>
  );
}
