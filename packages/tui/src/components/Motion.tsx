import React, { useEffect, useState } from "react";
import { Box, Text } from "ink";
import { motionEnabled, TypeLine, useIntervalFrame } from "../motion/index.js";

const SPINNER = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const PULSE = ["·", "•", "●", "•"];
const DOTS = [".  ", ".. ", "..."];

/**
 * Small terminal spinner for busy states.
 * Pure text — no color gimmicks.
 */
export function Spinner(props: {
  label?: string;
  active?: boolean;
}): React.ReactElement {
  const active = props.active !== false;
  const i = useIntervalFrame(SPINNER.length, 80, active);

  return (
    <Text>
      {active && motionEnabled() ? SPINNER[i] : active ? "…" : "✓"}
      {props.label ? ` ${props.label}` : ""}
    </Text>
  );
}

/** Soft pulse mark for status lines. */
export function Pulse(props: { label?: string }): React.ReactElement {
  const i = useIntervalFrame(PULSE.length, 320, true);
  return (
    <Text dimColor>
      {motionEnabled() ? PULSE[i] : "·"}
      {props.label ? ` ${props.label}` : ""}
    </Text>
  );
}

/** Animated trailing dots. */
export function WorkingDots(props: { label: string }): React.ReactElement {
  const i = useIntervalFrame(DOTS.length, 400, true);
  return (
    <Text>
      {props.label}
      {motionEnabled() ? DOTS[i] : "..."}
    </Text>
  );
}

/**
 * Text progress bar for pack installs.
 * width is character cells for the bar body.
 */
export function ProgressBar(props: {
  current: number;
  total: number;
  label?: string;
  width?: number;
}): React.ReactElement {
  const total = Math.max(1, props.total);
  const current = Math.min(Math.max(0, props.current), total);
  const width = props.width ?? 16;
  const filled = Math.round((current / total) * width);
  const bar = "█".repeat(filled) + "░".repeat(width - filled);

  return (
    <Box flexDirection="column">
      <Text>
        [{bar}] {current}/{total}
      </Text>
      {props.label ? <Text dimColor>{props.label}</Text> : null}
    </Box>
  );
}

/** One-shot success: types the message once, then holds static. */
export function SuccessLine(props: {
  message: string;
  /** When true, typewriter reveal. Default true. */
  animate?: boolean;
}): React.ReactElement {
  const animate = props.animate !== false;
  if (!animate || !motionEnabled()) {
    return <Text>✓ {props.message}</Text>;
  }
  return (
    <Box>
      <Text>✓ </Text>
      <TypeLine text={props.message} cursor={false} cps={32} />
    </Box>
  );
}

/**
 * Error line: brief ! blink, then static red.
 * Prefer readability over drama.
 */
export function ErrorLine(props: { message: string }): React.ReactElement {
  const enabled = motionEnabled();
  const [beats, setBeats] = useState(0);

  useEffect(() => {
    if (!enabled) {
      setBeats(4);
      return;
    }
    setBeats(0);
    let n = 0;
    const t = setInterval(() => {
      n += 1;
      setBeats(n);
      if (n >= 4) clearInterval(t);
    }, 120);
    return () => clearInterval(t);
  }, [props.message, enabled]);

  const mark = !enabled || beats >= 4 || beats % 2 === 0 ? "!" : " ";
  return (
    <Text color="red">
      {mark} {props.message}
    </Text>
  );
}

// Re-export motion primitives for convenient imports from Motion.js
export {
  TypeLine,
  BlinkCursor,
  FadeSteps,
  CountUp,
  StaggerLines,
  ActionFlash,
  SelectPulse,
  motionEnabled,
  useIntervalFrame,
} from "../motion/index.js";
