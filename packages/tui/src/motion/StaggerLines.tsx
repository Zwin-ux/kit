import React, { useEffect, useState } from "react";
import { Box } from "ink";
import { motionEnabled } from "./motionEnabled.js";

/**
 * Reveal child rows one-by-one once (first-run options, not data lists).
 */
export function StaggerLines(props: {
  children: React.ReactNode[];
  stepMs?: number;
}): React.ReactElement {
  const { children, stepMs = 60 } = props;
  const enabled = motionEnabled();
  const total = children.length;
  const [shown, setShown] = useState(enabled ? 0 : total);

  useEffect(() => {
    if (!enabled || total === 0) {
      setShown(total);
      return;
    }
    setShown(0);
    let n = 0;
    const t = setInterval(() => {
      n += 1;
      if (n >= total) {
        setShown(total);
        clearInterval(t);
        return;
      }
      setShown(n);
    }, stepMs);
    return () => clearInterval(t);
  }, [total, stepMs, enabled]);

  return (
    <Box flexDirection="column">
      {children.slice(0, shown).map((child, index) => (
        <Box key={index} flexDirection="column">
          {child}
        </Box>
      ))}
    </Box>
  );
}
