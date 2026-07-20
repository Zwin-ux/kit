/**
 * Pixel-art TUI for Kit.
 * Drawing and keyboard input only. Business logic stays in @kit-skills/core.
 */

export { KIT_PACKAGE_VERSION } from "@kit-skills/shared";

/** Package identity for consumers. */
export const TUI_PACKAGE_NAME = "@kit-skills/tui" as const;

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
