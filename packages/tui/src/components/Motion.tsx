import React, { useEffect, useState } from "react";
import { Box, Text } from "ink";

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
  const [i, setI] = useState(0);

  useEffect(() => {
    if (!active) return;
    const t = setInterval(() => setI((n) => (n + 1) % SPINNER.length), 80);
    return () => clearInterval(t);
  }, [active]);

  return (
    <Text>
      {active ? SPINNER[i] : "✓"}
      {props.label ? ` ${props.label}` : ""}
    </Text>
  );
}

/** Soft pulse mark for status lines. */
export function Pulse(props: { label?: string }): React.ReactElement {
  const [i, setI] = useState(0);
  useEffect(() => {
    const t = setInterval(() => setI((n) => (n + 1) % PULSE.length), 320);
    return () => clearInterval(t);
  }, []);
  return (
    <Text dimColor>
      {PULSE[i]}
      {props.label ? ` ${props.label}` : ""}
    </Text>
  );
}

/** Animated trailing dots. */
export function WorkingDots(props: { label: string }): React.ReactElement {
  const [i, setI] = useState(0);
  useEffect(() => {
    const t = setInterval(() => setI((n) => (n + 1) % DOTS.length), 400);
    return () => clearInterval(t);
  }, []);
  return (
    <Text>
      {props.label}
      {DOTS[i]}
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

/** One-shot success flash (static check — no flicker after done). */
export function SuccessLine(props: { message: string }): React.ReactElement {
  return <Text>✓ {props.message}</Text>;
}
