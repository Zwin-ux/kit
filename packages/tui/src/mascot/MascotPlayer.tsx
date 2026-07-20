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

export interface MascotPlayerProps {
  frames: PixelFrame[];
  /** Play the same loop as assets/pixel/kit-idle.gif (or variant GIFs). */
  playing?: boolean;
  /** Delay between frames in ms (default matches variant). */
  delayMs?: number;
  /**
   * compact = side rail (single-width blocks, smaller)
   * full = splash (double-wide cells)
   */
  size?: "full" | "compact";
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
}

/**
 * Live terminal playback of the Kit mascot.
 *
 * Renders each row as its own Text inside a fixed-width Box so the fox
 * doesn’t wrap mid-sprite and get “cut off” in narrow layouts.
 * KIT_REDUCED_MOTION=1 freezes on frame 0.
 */
export function MascotPlayer({
  frames,
  playing = true,
  delayMs,
  size = "full",
  caption,
  showCounter = false,
  label,
  variant = "idle",
}: MascotPlayerProps): React.ReactElement {
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

  // Reset to first frame when variant / frame set length changes
  useEffect(() => {
    setIndex(0);
  }, [variant, frameCount]);

  const frame = frames[enabled ? index : 0] ?? frames[0];

  const renderOpts: RenderFrameOptions = useMemo(() => {
    if (size === "compact") {
      return { cell: "█", empty: " ", tight: true, pad: 1 };
    }
    return { cell: "██", empty: "  ", tight: true, pad: 1 };
  }, [size]);

  const lines = useMemo(() => {
    if (!frame) return [] as string[];
    return renderFrameLines(frame, renderOpts);
  }, [frame, renderOpts]);

  const width = useMemo(() => {
    if (!frame) return 0;
    return renderedCellWidth(frame, renderOpts);
  }, [frame, renderOpts]);

  if (!frame) {
    return (
      <Box>
        <Text dimColor>mascot unavailable</Text>
      </Box>
    );
  }

  return (
    <Box flexDirection="column" flexShrink={0}>
      {label ? <Text dimColor>{label}</Text> : null}
      <Box flexDirection="column" width={Math.max(width, 1)} flexShrink={0}>
        {lines.map((line, i) => (
          <Text key={i}>{line}</Text>
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
