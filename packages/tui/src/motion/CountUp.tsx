import React, { useEffect, useState } from "react";
import { Text } from "ink";
import { motionEnabled } from "./motionEnabled.js";

/**
 * Animate an integer from 0 to `to`, then hold. For post-install skill counts.
 */
export function CountUp(props: {
  to: number;
  suffix?: string;
  /** Total animation duration ms. Default 400. */
  ms?: number;
  dimColor?: boolean;
}): React.ReactElement {
  const { to, suffix = "", ms = 400, dimColor } = props;
  const enabled = motionEnabled();
  const [value, setValue] = useState(enabled ? 0 : to);

  useEffect(() => {
    if (!enabled || to <= 0) {
      setValue(to);
      return;
    }
    setValue(0);
    const steps = Math.min(to, 12);
    const stepMs = Math.max(24, Math.floor(ms / steps));
    let i = 0;
    const t = setInterval(() => {
      i += 1;
      const next = Math.min(to, Math.round((i / steps) * to));
      setValue(next);
      if (i >= steps) {
        setValue(to);
        clearInterval(t);
      }
    }, stepMs);
    return () => clearInterval(t);
  }, [to, ms, enabled]);

  return (
    <Text {...(dimColor ? { dimColor: true as const } : {})}>
      {value}
      {suffix}
    </Text>
  );
}
