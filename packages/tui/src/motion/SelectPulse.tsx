import React, { useEffect, useState } from "react";
import { Text } from "ink";
import { motionEnabled } from "./motionEnabled.js";

/**
 * Cursor mark that briefly brightens when selection index changes.
 */
export function SelectPulse(props: {
  selected: boolean;
  /** Bump when selection moves to re-trigger pulse. */
  tick: number;
}): React.ReactElement {
  const [hot, setHot] = useState(false);

  useEffect(() => {
    if (!props.selected || !motionEnabled()) {
      setHot(false);
      return;
    }
    setHot(true);
    const t = setTimeout(() => setHot(false), 120);
    return () => clearTimeout(t);
  }, [props.tick, props.selected]);

  if (!props.selected) {
    return <Text> </Text>;
  }
  return <Text bold={hot}>{hot ? "»" : "›"}</Text>;
}
