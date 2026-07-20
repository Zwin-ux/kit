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
 * Enable SGR mouse tracking and invoke handler when a registered hit is clicked.
 * Hit map is mutated via the returned HitMap ref each render (caller rebuilds).
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

    const onData = (buf: Buffer | string) => {
      const text = typeof buf === "string" ? buf : buf.toString("utf8");
      const ev = parseSgrMouseChunk(text);
      if (!ev || !isPrimaryClick(ev)) return;
      const hit = mapRef.current.hit(ev.x, ev.y);
      if (hit) handlerRef.current(hit);
    };

    // Ink also reads stdin; append listener carefully
    process.stdin.on("data", onData);

    return () => {
      process.stdin.off("data", onData);
      disableMouse();
    };
  }, [enabled]);

  return mapRef.current;
}
