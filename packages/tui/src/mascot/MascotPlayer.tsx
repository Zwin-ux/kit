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
import { splashMascotFit, type LayoutScale } from "./layoutScale.js";

export interface MascotPlayerProps {
  frames: PixelFrame[];
  /** Play the same loop as assets/pixel/kit-idle.gif (or variant GIFs). */
  playing?: boolean;
  /** Delay between frames in ms (default matches variant). */
  delayMs?: number;
  /**
   * compact = side rail (layout-scaled, letterboxed)
   * full / hero = splash (larger on wide terminals)
   * auto = compact on rail sizing
   */
  size?: "full" | "compact" | "hero" | "auto";
  /**
   * Optional caption under the animation.
   * Prefer empty for product UI — avoid asset-path / debug strings.
   */
  caption?: string;
  /** Dev-only frame counter (off by default; do not use on product screens). */
  showCounter?: boolean;
  /** Dev-only label above the art (off by default). */
  label?: string;
  /**
   * Visual intent — mainly used for default delay when delayMs omitted.
   * Frames themselves should already match the variant.
   */
  variant?: MascotVariant;
  /** Override layout scale (tests). */
  scale?: LayoutScale;
}

/**
 * Live terminal playback of the Kit mascot.
 *
 * Every frame is letterboxed into a fixed canvas so tail/ear motion never
 * changes the reserved terminal box (no cut-off, no size jump).
 * KIT_REDUCED_MOTION=1 freezes on frame 0.
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

  const renderOpts: RenderFrameOptions = useMemo(() => {
    const isHero = size === "full" || size === "hero";
    if (isHero) {
      const splash = splashMascotFit(scale);
      return {
        cell: splash.cell === "double" ? "██" : "█",
        empty: splash.cell === "double" ? "  " : " ",
        tight: true,
        pad: 0,
        fit: { width: splash.width, height: splash.height },
      };
    }
    // compact / auto rail — fixed fit from terminal scale
    return {
      cell: scale.mascotCell === "double" ? "██" : "█",
      empty: scale.mascotCell === "double" ? "  " : " ",
      tight: true,
      pad: 0,
      fit: {
        width: scale.mascotFit.width,
        height: scale.mascotFit.height,
      },
    };
  }, [size, scale]);

  const lines = useMemo(() => {
    if (!frame) return [] as string[];
    return renderFrameLines(frame, renderOpts);
  }, [frame, renderOpts]);

  const width = useMemo(() => {
    if (!frame) return 0;
    return renderedCellWidth(frame, renderOpts);
  }, [frame, renderOpts]);

  const height = lines.length;

  if (!frame) {
    return (
      <Box>
        <Text dimColor>mascot unavailable</Text>
      </Box>
    );
  }

  return (
    <Box flexDirection="column" flexShrink={0} width={Math.max(width, 1)}>
      {label ? <Text dimColor>{label}</Text> : null}
      <Box
        flexDirection="column"
        width={Math.max(width, 1)}
        height={height}
        flexShrink={0}
        overflow="hidden"
      >
        {lines.map((line, i) => (
          <Text key={i} wrap="truncate">
            {line}
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
