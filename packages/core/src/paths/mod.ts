export type {
  HarnessId,
  PathScope,
  HarnessSkillPath,
  PathReport,
  LinkMode,
  LinkPlanItem,
  LinkResult,
  PathsResult,
} from "./types.js";
export {
  resolveHarnessSkillsRoot,
  harnessNotes,
  ALL_HARNESSES,
  LINKABLE_HARNESSES,
} from "./harness.js";
export { describePaths, type DescribePathsOptions } from "./describe.js";
export { linkSkills, type LinkSkillsOptions } from "./link.js";
