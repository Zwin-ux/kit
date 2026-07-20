import React, { useEffect, useMemo, useState } from "react";
import { Box, Text } from "ink";
import {
  renderFrameLines,
  renderedCellWidth,
  type RenderFrameOptions,
} from "./renderBitmap.js";
import { FRAME_DELAY_MS, type PixelFrame } from "./types.js";

export interface MascotPlayerProps {
  frames: PixelFrame[];
  /** Play the same loop as assets/pixel/kit-idle.gif */
  playing?: boolean;
  /** Delay between frames in ms (default matches GIF ~180ms). */
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
}

/**
 * Live terminal playback of the Kit idle mascot.
 *
 * Renders each row as its own Text inside a fixed-width Box so the fox
 * doesn’t wrap mid-sprite and get “cut off” in narrow layouts.
 */
export function MascotPlayer({
  frames,
  playing = true,
  delayMs = FRAME_DELAY_MS,
  size = "full",
  caption,
  showCounter = false,
  label,
}: MascotPlayerProps): React.ReactElement {
  const [index, setIndex] = useState(0);
  const safeFrames = frames.length > 0 ? frames : [];

  useEffect(() => {
    if (!playing || safeFrames.length <= 1) return;
    const timer = setInterval(() => {
      setIndex((current) => (current + 1) % safeFrames.length);
    }, delayMs);
    return () => clearInterval(timer);
  }, [playing, safeFrames.length, delayMs]);

  const frame = safeFrames[index] ?? safeFrames[0];

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
          {index + 1}/{safeFrames.length}
        </Text>
      ) : null}
      {caption ? <Text dimColor>{caption}</Text> : null}
    </Box>
  );
}
