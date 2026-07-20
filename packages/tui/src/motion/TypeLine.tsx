import React, { useEffect, useRef, useState } from "react";
import { Text } from "ink";
import { BlinkCursor } from "./BlinkCursor.js";
import { motionEnabled } from "./motionEnabled.js";

export interface TypeLineProps {
  text: string;
  /** Characters per second. Default 28 → short lines finish under ~1.2s. */
  cps?: number;
  /** Show blinking cursor after complete. Default false (success lines stay calm). */
  cursor?: boolean;
  dimColor?: boolean;
  bold?: boolean;
  onDone?: () => void;
}

/**
 * Reveal a string left-to-right once. Does not restart on parent re-render
 * while `text` is unchanged. When reduced motion is on, shows full string.
 */
export function TypeLine(props: TypeLineProps): React.ReactElement {
  const {
    text,
    cps = 28,
    cursor = false,
    dimColor,
    bold,
    onDone,
  } = props;
  const enabled = motionEnabled();
  const [visible, setVisible] = useState(enabled ? 0 : text.length);
  const doneRef = useRef(false);
  const onDoneRef = useRef(onDone);
  onDoneRef.current = onDone;

  useEffect(() => {
    doneRef.current = false;
    if (!enabled || text.length === 0) {
      setVisible(text.length);
      if (!doneRef.current) {
        doneRef.current = true;
        onDoneRef.current?.();
      }
      return;
    }

    setVisible(0);
    const intervalMs = Math.max(16, Math.round(1000 / Math.max(1, cps)));
    let n = 0;
    const t = setInterval(() => {
      n += 1;
      if (n >= text.length) {
        setVisible(text.length);
        clearInterval(t);
        if (!doneRef.current) {
          doneRef.current = true;
          onDoneRef.current?.();
        }
        return;
      }
      setVisible(n);
    }, intervalMs);

    return () => clearInterval(t);
  }, [text, cps, enabled]);

  const complete = visible >= text.length;
  const shown = text.slice(0, visible);

  const textProps = {
    ...(bold ? { bold: true as const } : {}),
    ...(dimColor ? { dimColor: true as const } : {}),
  };

  return (
    <Text {...textProps}>
      {shown}
      {cursor && complete ? <BlinkCursor /> : null}
      {cursor && !complete && enabled ? <Text>▌</Text> : null}
    </Text>
  );
}
