import React, { useEffect, useMemo, useState } from "react";
import { Box, Text } from "ink";
import { motionEnabled } from "../motion/motionEnabled.js";
import {
  renderFrameLines,
  type RenderFrameOptions,
} from "./renderBitmap.js";
import {
  frameDelayForVariant,
  type MascotVariant,
  type PixelFrame,
} from "./types.js";
import { useLayoutScale } from "./useLayoutScale.js";
import { LAYOUT_CAPS, type LayoutScale } from "./layoutScale.js";

export interface MascotPlayerProps {
  frames: PixelFrame[];
  playing?: boolean;
  delayMs?: number;
  /**
   * compact / auto = fixed rail slot
   * hero / full = splash (still fixed line count)
   */
  size?: "full" | "compact" | "hero" | "auto";
  caption?: string;
  showCounter?: boolean;
  label?: string;
  variant?: MascotVariant;
  scale?: LayoutScale;
}

/**
 * Pad/truncate to exactly `rows` lines of length `cols` (spaces).
 * Stable React tree → no layout thrash while animating.
 */
export function padSlotLines(
  lines: string[],
  cols: number,
  rows: number,
): string[] {
  const out: string[] = [];
  for (let i = 0; i < rows; i++) {
    const raw = lines[i] ?? "";
    const clipped = raw.length > cols ? raw.slice(0, cols) : raw;
    out.push(clipped.padEnd(cols, " "));
  }
  return out;
}

/**
 * Live mascot in a **fixed** terminal slot.
 * Only characters inside change — never width/height/node count.
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
  const isHero = size === "full" || size === "hero";

  const slotCols = isHero
    ? Math.min(scale.splashFit.width, LAYOUT_CAPS.splashFitMax)
    : scale.railCols;
  const slotRows = isHero
    ? Math.min(scale.splashFit.height, LAYOUT_CAPS.splashFitMax)
    : scale.railRows;

  const effectiveDelay =
    delayMs ??
    (isHero
      ? frameDelayForVariant(variant)
      : Math.max(frameDelayForVariant(variant), LAYOUT_CAPS.railFrameDelayMs));

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

  // tight: false with fit — composition fixed at load (normalizeFrame).
  // Re-tightening each frame was the silhouette "breathe" / TUI thrash bug.
  const renderOpts: RenderFrameOptions = useMemo(() => {
    if (isHero) {
      return {
        cell: "█",
        empty: " ",
        tight: false,
        pad: 0,
        fit: {
          width: scale.splashFit.width,
          height: scale.splashFit.height,
        },
      };
    }
    return {
      cell: "█",
      empty: " ",
      tight: false,
      pad: 0,
      fit: {
        width: scale.mascotFit.width,
        height: scale.mascotFit.height,
      },
    };
  }, [isHero, scale]);

  // Always exactly slotRows lines × slotCols chars
  const lines = useMemo(() => {
    if (!frame) {
      return padSlotLines([], slotCols, slotRows);
    }
    const raw = renderFrameLines(frame, renderOpts);
    return padSlotLines(raw, slotCols, slotRows);
  }, [frame, renderOpts, slotCols, slotRows]);

  if (!frame) {
    return (
      <Box width={slotCols} height={slotRows} flexShrink={0}>
        <Text dimColor>{" ".repeat(slotCols)}</Text>
      </Box>
    );
  }

  // Extra reserved lines for optional chrome (fixed 0 when off)
  const chromeRows = (label ? 1 : 0) + (showCounter ? 1 : 0) + (caption ? 1 : 0);
  const totalRows = slotRows + chromeRows;

  return (
    <Box
      flexDirection="column"
      flexShrink={0}
      width={slotCols}
      height={totalRows}
      overflow="hidden"
    >
      {label ? (
        <Text dimColor wrap="truncate">
          {label.padEnd(slotCols, " ").slice(0, slotCols)}
        </Text>
      ) : null}
      {lines.map((line, i) => (
        <Text key={i}>{line}</Text>
      ))}
      {showCounter ? (
        <Text dimColor>
          {`${index + 1}/${frameCount}`.padEnd(slotCols, " ").slice(0, slotCols)}
        </Text>
      ) : null}
      {caption ? (
        <Text dimColor>
          {caption.padEnd(slotCols, " ").slice(0, slotCols)}
        </Text>
      ) : null}
    </Box>
  );
}
