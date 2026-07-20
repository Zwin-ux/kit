import React, { useEffect, useState } from "react";
import { Text } from "ink";
import { motionEnabled } from "./motionEnabled.js";

export type SelectDirection = "up" | "down" | "none";

/**
 * Fixed-width (2 col) selection cursor.
 * Directional glyph on move, then rest mark — never shifts list text.
 */
export function SelectPulse(props: {
  selected: boolean;
  /** Bump when selection moves to re-trigger pulse. */
  tick: number;
  /** Last move direction for ↑↓ feedback. */
  direction?: SelectDirection;
}): React.ReactElement {
  const direction = props.direction ?? "none";
  const [hot, setHot] = useState(false);

  useEffect(() => {
    if (!props.selected || !motionEnabled()) {
      setHot(false);
      return;
    }
    setHot(true);
    const t = setTimeout(() => setHot(false), 160);
    return () => clearTimeout(t);
  }, [props.tick, props.selected]);

  // Always 2 columns — layout stability
  if (!props.selected) {
    return <Text>{"  "}</Text>;
  }

  if (!motionEnabled()) {
    return <Text>{"› "}</Text>;
  }

  if (hot && direction === "up") {
    return <Text bold>{"↑ "}</Text>;
  }
  if (hot && direction === "down") {
    return <Text bold>{"↓ "}</Text>;
  }
  if (hot) {
    return <Text bold>{"» "}</Text>;
  }
  return <Text>{"› "}</Text>;
}
