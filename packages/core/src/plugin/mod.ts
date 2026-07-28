export { getPluginsIndexPath } from "./paths.js";
export {
  addPlugin,
  doctorPlugin,
  listPlugins,
  removePlugin,
  runPlugin,
} from "./plugin.js";
export type {
  AddPluginOptions,
  AddPluginReport,
  KitPluginManifest,
  PluginDoctorReport,
  PluginRegistry,
  PluginRegistryEntry,
  PluginResult,
  PluginRunReport,
  RegisteredPlugin,
  RunPluginOptions,
} from "./types.js";
