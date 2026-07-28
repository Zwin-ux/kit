import path from "node:path";

import { getKitHome } from "../library/paths.js";

export function getPluginsIndexPath(kitHome: string = getKitHome()): string {
  return path.join(kitHome, "plugins.json");
}
