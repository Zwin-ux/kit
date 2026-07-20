import React, { useEffect, useState } from "react";
import { Text } from "ink";
import { motionEnabled } from "./motionEnabled.js";

/**
 * One-shot status flash after a key action.
 * Always reserves one terminal line so mount/unmount never jumps the layout.
 */
export function ActionFlash(props: {
  message: string | undefined;
  /** Increment on each action so repeated labels still animate. */
  nonce?: number;
  /** ms to hold the flash. Default 420. */
  holdMs?: number;
}): React.ReactElement {
  const { message, nonce = 0, holdMs = 420 } = props;
  const [visible, setVisible] = useState(false);
  const [text, setText] = useState<string | undefined>();

  useEffect(() => {
    if (!message) {
      setVisible(false);
      setText(undefined);
      return;
    }
    setText(message);
    setVisible(true);
    if (!motionEnabled()) {
      return;
    }
    const t = setTimeout(() => setVisible(false), holdMs);
    return () => clearTimeout(t);
  }, [message, nonce, holdMs]);

  // Always one line — empty space when idle (no null → no reflow)
  if (!text || !visible) {
    return <Text> </Text>;
  }

  return <Text dimColor>▸ {text}</Text>;
}
