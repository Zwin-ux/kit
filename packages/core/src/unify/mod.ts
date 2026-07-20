export {
  normalizeSkillMd,
  writeNormalizedSkill,
  slugify,
  type NormalizedSkillMd,
} from "./normalize.js";
export {
  scoreUnifyCandidate,
  looksKitShaped,
  type ScoreInput,
  type ScoreResult,
  type SkillGrade,
} from "./score.js";
export {
  runUnify,
  type UnifyOptions,
  type UnifyResult,
  type UnifyReport,
  type UnifyCandidate,
  type UnifySourceHit,
} from "./unify.js";