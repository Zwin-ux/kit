/**
 * Local skills engine for Kit.
 * Parse, validate, and manage skills. No TUI code here.
 */

export { KIT_PACKAGE_VERSION } from "@kit-skills/shared";

/** Package identity for consumers. */
export const CORE_PACKAGE_NAME = "@kit-skills/core" as const;

export {
  KNOWN_AGENTS,
  type KnownAgent,
  type Skill,
  type SkillParseResult,
  type ValidationIssue,
} from "./types.js";

export { parseSkillMd, type ParsedSkillMd, type SkillFrontMatterRaw } from "./parse/skillMd.js";
export { validateSkill } from "./validate/skill.js";
export {
  loadSkill,
  parseAndValidateSkillMd,
  formatIssues,
} from "./loadSkill.js";

export {
  getKitHome,
  getSkillsDir,
  getLibraryIndexPath,
  getConfigPath,
  installSkill,
  listSkills,
  removeSkill,
  type InstallOptions,
  type ListOptions,
  type RemoveOptions,
  type InstalledSkill,
  type LibraryIndex,
  type LibraryIndexEntry,
  type LibraryResult,
} from "./library/mod.js";

export {
  parsePackMd,
  validatePackFrontMatter,
  resolvePacksRoot,
  resolveSkillsCatalogRoot,
  loadPack,
  listPacks,
  validatePack,
  installPack,
  applyPack,
  detectMissingGitRoot,
  readProjectAppliedPacks,
  type PackLoadOptions,
  type InstallPackOptions,
  type ApplyPackOptions,
  type SkillPack,
  type ResolvedPackSkill,
  type LoadedPack,
  type PackResult,
  type PackListItem,
  type InstallPackResult,
  type ApplyPackResult,
  type AppliedPackRecord,
  type AppliedPacksFile,
} from "./pack/mod.js";

export {
  DEFAULT_KIT_CONFIG,
  FIRST_RUN_PACK_OPTIONS,
  readConfig,
  writeConfig,
  updateConfig,
  getFirstRunStatus,
  completeFirstRun,
  isFirstRunPackName,
  listFirstRunPackOptions,
  type KitConfig,
  type FirstRunPackName,
  type FirstRunStatus,
} from "./config/mod.js";

export {
  resolveHarnessSkillsRoot,
  harnessNotes,
  ALL_HARNESSES,
  LINKABLE_HARNESSES,
  describePaths,
  linkSkills,
  importSkillsFromHarness,
  type DescribePathsOptions,
  type LinkSkillsOptions,
  type ImportSkillsOptions,
  type HarnessId,
  type PathScope,
  type HarnessSkillPath,
  type PathReport,
  type LinkMode,
  type LinkPlanItem,
  type LinkResult,
  type ImportPlanItem,
  type ImportResult,
  type PathsResult,
} from "./paths/mod.js";

export {
  testSkill,
  testPack,
  testAllPacks,
  type TestSkillOptions,
  type TestPackOptions,
  type SkillTestReport,
  type PackTestReport,
  type MultiPackTestReport,
  type TestResult,
  type CheckResult,
  type CheckLevel,
} from "./test/mod.js";

export {
  runDoctor,
  type DoctorOptions,
  type DoctorReport,
} from "./doctor/mod.js";

export {
  DEFAULT_REGISTRY_URL,
  getRegistryUrl,
  getAuthPath,
  readAuthSession,
  writeAuthSession,
  clearAuthSession,
  loginWithDeviceFlow,
  getLoggedInUser,
  logout,
  type KitAuthUser,
  type KitAuthSession,
  type AuthResult,
  type LoginProgress,
  type LoginOptions,
  type DeviceStartPayload,
} from "./auth/mod.js";

export {
  exploreListPacks,
  exploreShowPack,
  exploreListSkills,
  exploreSearch,
  type ExploreResult,
  type ExploreOptions,
  type RegistryPackSummary,
  type RegistrySkillSummary,
} from "./explore/mod.js";

export {
  recommendToolkits,
  type ToolkitRecommendation,
  type SkillRecommendation,
  type RecommendReport,
  type RecommendResult,
} from "./recommend/mod.js";
