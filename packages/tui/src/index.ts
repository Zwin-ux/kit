/**
 * Pixel-art TUI for Kit.
 * Drawing and keyboard input only. Business logic stays in @mzwin/kit-core.
 */

export { KIT_PACKAGE_VERSION } from "@mzwin/kit-shared";

/** Package identity for consumers. */
export const TUI_PACKAGE_NAME = "@mzwin/kit-tui" as const;

export {
  loadMascotFrames,
  loadAllMascotFrames,
  pngToFrame,
} from "./mascot/loadFrames.js";
export { renderFrame } from "./mascot/renderBitmap.js";
export { getPlaceholderFrames } from "./mascot/placeholderFrames.js";
export {
  FRAME_COUNT,
  FRAME_FILES,
  FRAME_DELAY_MS,
  SCAN_FRAME_COUNT,
  SUCCESS_FRAME_COUNT,
  type PixelFrame,
  type MascotVariant,
} from "./mascot/types.js";
export { resolvePixelAssetsDir } from "./mascot/resolveAssetsDir.js";
export { startTui } from "./start.js";
// MascotPlayer is the in-terminal kit-idle loop (same frames as kit-idle.gif).
export { MascotPlayer } from "./mascot/MascotPlayer.js";
export { StatusIcon } from "./mascot/StatusIcon.js";
export {
  STATUS_ICON_IDS,
  STATUS_ICON_SIZE,
  getStatusIconBitmap,
  statusIconGlyph,
  levelToStatusIcon,
  harnessToStatusIcon,
  type StatusIconId,
} from "./mascot/statusIcons.js";
export { PackIcon } from "./mascot/PackIcon.js";
export {
  layoutScaleFromTerminal,
  LAYOUT_CAPS,
} from "./mascot/layoutScale.js";
export type { LayoutScale, LayoutMode } from "./mascot/layoutScale.js";
export { padSlotLines } from "./mascot/MascotPlayer.js";
export type { SelectDirection } from "./motion/SelectPulse.js";
export { Spinner } from "./motion/Spinner.js";
