import { useEffect, useState } from "react";
import { useStdout } from "ink";
import { layoutScaleFromTerminal, type LayoutScale } from "./layoutScale.js";

/**
 * Live terminal size → layout scale (resize-safe).
 */
export function useLayoutScale(): LayoutScale {
  const { stdout } = useStdout();
  const [scale, setScale] = useState<LayoutScale>(() =>
    layoutScaleFromTerminal(stdout?.columns, stdout?.rows),
  );

  useEffect(() => {
    const read = () =>
      setScale(layoutScaleFromTerminal(stdout?.columns, stdout?.rows));

    read();
    if (!stdout || typeof stdout.on !== "function") return;

    const onResize = () => read();
    stdout.on("resize", onResize);
    return () => {
      if (typeof stdout.off === "function") {
        stdout.off("resize", onResize);
      } else if (typeof stdout.removeListener === "function") {
        stdout.removeListener("resize", onResize);
      }
    };
  }, [stdout]);

  return scale;
}
