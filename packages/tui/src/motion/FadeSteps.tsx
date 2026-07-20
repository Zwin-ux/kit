import React, { useEffect, useState } from "react";
import { Text } from "ink";
import { motionEnabled } from "./motionEnabled.js";

/**
 * One-shot title emphasis: dim → bold → normal over a few frames.
 * Restarts when `triggerKey` changes (e.g. screen name).
 */
export function FadeSteps(props: {
  text: string;
  triggerKey: string;
}): React.ReactElement {
  const enabled = motionEnabled();
  const [step, setStep] = useState(enabled ? 0 : 2);

  useEffect(() => {
    if (!enabled) {
      setStep(2);
      return;
    }
    setStep(0);
    const t1 = setTimeout(() => setStep(1), 80);
    const t2 = setTimeout(() => setStep(2), 200);
    return () => {
      clearTimeout(t1);
      clearTimeout(t2);
    };
  }, [props.triggerKey, enabled]);

  if (step === 0) {
    return <Text dimColor>{props.text}</Text>;
  }
  if (step === 1) {
    return <Text bold>{props.text}</Text>;
  }
  return <Text>{props.text}</Text>;
}
