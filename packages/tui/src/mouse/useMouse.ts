import { useEffect, useRef } from "react";
import { HitMap, type HitRegion } from "./HitMap.js";
import {
  disableMouse,
  enableMouse,
  installMouseCleanup,
  mouseAllowed,
} from "./enableMouse.js";
import { isPrimaryClick, parseSgrMouseChunk } from "./parseSgrMouse.js";

export type MouseClickHandler = (region: HitRegion) => void;

/**
 * SGR mouse tracking. Uses prependListener so we see clicks before Ink.
 */
export function useMouseClick(
  onClick: MouseClickHandler,
  enabled = true,
): HitMap {
  const mapRef = useRef(new HitMap());
  const handlerRef = useRef(onClick);
  handlerRef.current = onClick;

  useEffect(() => {
    if (!enabled || !mouseAllowed()) return;

    enableMouse();
    installMouseCleanup();

    // Debounce press+release into one click (Windows Terminal sends both).
    let lastFire = 0;
    const onData = (buf: Buffer | string) => {
      const text = typeof buf === "string" ? buf : buf.toString("utf8");
      const ev = parseSgrMouseChunk(text);
      if (!ev || !isPrimaryClick(ev)) return;
      // Prefer release edge when present; always debounce 80ms
      const now = Date.now();
      if (now - lastFire < 80) return;
      lastFire = now;
      const hit = mapRef.current.hit(ev.x, ev.y);
      if (hit) {
        handlerRef.current(hit);
      }
    };

    // Prefer prepend so our handler runs first on Windows Terminal
    const stdin = process.stdin as NodeJS.ReadStream & {
      prependListener?: typeof process.stdin.on;
    };
    if (typeof stdin.prependListener === "function") {
      stdin.prependListener("data", onData);
    } else {
      stdin.on("data", onData);
    }

    return () => {
      stdin.off("data", onData);
      disableMouse();
    };
  }, [enabled]);

  return mapRef.current;
}
