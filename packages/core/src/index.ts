/**
 * Local skills engine for Kit.
 * Parse, validate, and manage skills. No TUI code here.
 */

export { KIT_PACKAGE_VERSION } from "@mzwin/kit-shared";

/** Package identity for consumers. */
export const CORE_PACKAGE_NAME = "@mzwin/kit-core" as const;

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
  normalizeSkillMd,
  writeNormalizedSkill,
  slugify,
  scoreUnifyCandidate,
  looksKitShaped,
  runUnify,
  type NormalizedSkillMd,
  type ScoreInput,
  type ScoreResult,
  type SkillGrade,
  type UnifyOptions,
  type UnifyResult,
  type UnifyReport,
  type UnifyCandidate,
  type UnifySourceHit,
} from "./unify/mod.js";

export {
  USER_STORIES,
  pickStory,
  detectSituation,
  runReady,
  runStatus,
  type StoryId,
  type UserStory,
  type SituationSnapshot,
  type KitSituation,
  type DetectSituationOptions,
  type ReadyOptions,
  type ReadyReport,
  type ReadyResult,
  type StatusOptions,
  type StatusReport,
  type StatusResult,
  type HarnessStatusRow,
  type HarnessLinkState,
} from "./product/mod.js";

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

export {
  getPluginsIndexPath,
  addPlugin,
  doctorPlugin,
  listPlugins,
  removePlugin,
  runPlugin,
  runPluginTask,
  type AddPluginOptions,
  type AddPluginReport,
  type KitPluginManifest,
  type KitPluginTask,
  type PluginDoctorReport,
  type PluginRegistry,
  type PluginRegistryEntry,
  type PluginResult,
  type PluginRunReport,
  type PluginTaskRunReport,
  type RegisteredPlugin,
  type RunPluginOptions,
} from "./plugin/mod.js";

export {
  detectCodingRunners,
  listOllamaModels,
  planCodingJob,
  runCodingJob,
  findOllamaExecutable,
  probeOllamaService,
  pullOllamaModel,
  startOllamaServe,
  stopOllamaServe,
  saveRun,
  listRuns,
  loadRun,
  type CodingInvocation,
  type CodingJobMode,
  type CodingJobReport,
  type CodingJobRequest,
  type CodingRunnerId,
  type CodingRunnerStatus,
  type LocalModel,
  type OllamaDiscoveryOptions,
  type OllamaPullOptions,
  type OllamaServeOptions,
  type OllamaServeState,
  type OllamaServiceReport,
  type RunnerCommand,
  type RunCodingJobOptions,
  type SavedRunKind,
  type SavedRunRecord,
  type SavedRunStatus,
  type SavedRunSummary,
  type WorkbenchResult,
} from "./workbench/mod.js";

export { getRunsDir } from "./library/paths.js";
