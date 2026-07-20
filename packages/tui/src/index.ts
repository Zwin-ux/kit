/**
 * Pixel-art TUI for Kit.
 * Drawing and keyboard input only. Business logic stays in @mzwin/kit-core.
 */

export { KIT_PACKAGE_VERSION } from "@mzwin/kit-shared";

/** Package identity for consumers. */
export const TUI_PACKAGE_NAME = "@mzwin/kit-tui" as const;

export { loadMascotFrames, pngToFrame } from "./mascot/loadFrames.js";
export { renderFrame } from "./mascot/renderBitmap.js";
export { getPlaceholderFrames } from "./mascot/placeholderFrames.js";
export {
  FRAME_COUNT,
  FRAME_FILES,
  FRAME_DELAY_MS,
  type PixelFrame,
} from "./mascot/types.js";
export { resolvePixelAssetsDir } from "./mascot/resolveAssetsDir.js";
export { startTui } from "./start.js";
// MascotPlayer is the in-terminal kit-idle loop (same frames as kit-idle.gif).
export { MascotPlayer } from "./mascot/MascotPlayer.js";
