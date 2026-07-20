import React, { useEffect, useMemo, useState } from "react";
import { Box, Text } from "ink";
import { renderFrame } from "./renderBitmap.js";
import { FRAME_DELAY_MS, type PixelFrame } from "./types.js";

export interface MascotPlayerProps {
  frames: PixelFrame[];
  /** Play the same loop as assets/pixel/kit-idle.gif */
  playing?: boolean;
  /** Delay between frames in ms (default matches GIF ~180ms). */
  delayMs?: number;
  /**
   * compact = half-block density for side panels
   * full = default double-wide cells
   */
  size?: "full" | "compact";
  /** Optional caption under the animation */
  caption?: string;
  /** Show frame counter (f1/6) */
  showCounter?: boolean;
  /** Label that this is the live kit-idle loop */
  label?: string;
}

/**
 * Live terminal playback of the Kit idle mascot.
 *
 * Terminals cannot play GIF files inside Ink portably.
 * This player cycles the same six PNG frames used to build kit-idle.gif,
 * so Splash / First-run / empty states match the README animation.
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
  const art = useMemo(() => {
    if (!frame) return "";
    if (size === "compact") {
      return renderFrame(frame, { cell: "█", empty: " " });
    }
    return renderFrame(frame);
  }, [frame, size]);

  if (!frame) {
    return (
      <Box>
        <Text dimColor>mascot unavailable</Text>
      </Box>
    );
  }

  return (
    <Box flexDirection="column">
      {label ? <Text dimColor>{label}</Text> : null}
      <Text>{art}</Text>
      {showCounter ? (
        <Text dimColor>
          kit-idle {index + 1}/{safeFrames.length}
          {playing ? " · playing" : " · paused"}
        </Text>
      ) : null}
      {caption ? <Text dimColor>{caption}</Text> : null}
    </Box>
  );
}
