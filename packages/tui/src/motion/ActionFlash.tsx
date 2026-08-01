import React, { useEffect, useState } from "react";
import { Text } from "ink";
import { motionEnabled } from "./motionEnabled.js";
import { theme } from "../theme.js";

/**
 * One-shot status flash after a key or click.
 * Always reserves one terminal line so mount/unmount never jumps the layout.
 * Hot phase = inverse orange (brand). Hold = green OK.
 */
export function ActionFlash(props: {
  message: string | undefined;
  /** Increment on each action so repeated labels still animate. */
  nonce?: number;
  /** ms to hold the flash. Default 700 (readable). */
  holdMs?: number;
  /** error style */
  isError?: boolean;
}): React.ReactElement {
  const { message, nonce = 0, holdMs = 700, isError = false } = props;
  const [visible, setVisible] = useState(false);
  const [text, setText] = useState<string | undefined>();
  const [phase, setPhase] = useState<"hot" | "hold">("hot");

  useEffect(() => {
    if (!message) {
      setVisible(false);
      setText(undefined);
      return;
    }
    setText(message);
    setVisible(true);
    setPhase("hot");
    if (!motionEnabled()) {
      return;
    }
    const hot = setTimeout(() => setPhase("hold"), 120);
    const t = setTimeout(() => setVisible(false), holdMs);
    return () => {
      clearTimeout(hot);
      clearTimeout(t);
    };
  }, [message, nonce, holdMs]);

  // Always one line — empty space when idle (no null → no reflow)
  if (!text || !visible) {
    return <Text> </Text>;
  }

  if (isError) {
    return (
      <Text bold color={theme.error} inverse={phase === "hot"}>
        {" "}
        ! {text}{" "}
      </Text>
    );
  }

  if (phase === "hot") {
    return (
      <Text bold inverse color={theme.accent}>
        {" "}
        → {text}{" "}
      </Text>
    );
  }
  return (
    <Text bold color={theme.success}>
      {" "}
      → {text}{" "}
    </Text>
  );
}
