export { getPluginsIndexPath } from "./paths.js";
export {
  addPlugin,
  doctorPlugin,
  listPlugins,
  removePlugin,
  runPlugin,
  runPluginTask,
} from "./plugin.js";
export type {
  AddPluginOptions,
  AddPluginReport,
  KitPluginManifest,
  KitPluginTask,
  PluginDoctorReport,
  PluginRegistry,
  PluginRegistryEntry,
  PluginResult,
  PluginRunReport,
  PluginTaskRunReport,
  RegisteredPlugin,
  RunPluginOptions,
} from "./types.js";
