import React, { useEffect, useState } from "react";
import { Text } from "ink";
import { motionEnabled } from "./motionEnabled.js";

/**
 * Brief one-shot status flash after a key action (install, link, test…).
 * Pass a rising `nonce` so the same label can re-trigger.
 */
export function ActionFlash(props: {
  message: string | undefined;
  /** Increment on each action so repeated labels still animate. */
  nonce?: number;
  /** ms to hold the flash. Default 420. */
  holdMs?: number;
}): React.ReactElement | null {
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

  if (!text || !visible) return null;

  return <Text dimColor>▸ {text}</Text>;
}
