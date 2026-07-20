import { useEffect, useState } from "react";
import { motionEnabled } from "./motionEnabled.js";

/**
 * Cycles 0..length-1 on an interval while active.
 * When motion is disabled or active is false, stays at 0.
 */
export function useIntervalFrame(
  length: number,
  ms: number,
  active = true,
): number {
  const [i, setI] = useState(0);
  const enabled = motionEnabled();
  const run = active && enabled && length > 1 && ms > 0;

  useEffect(() => {
    if (!run) {
      setI(0);
      return;
    }
    setI(0);
    const t = setInterval(() => {
      setI((n) => (n + 1) % length);
    }, ms);
    return () => clearInterval(t);
  }, [run, length, ms]);

  return run ? i : 0;
}
