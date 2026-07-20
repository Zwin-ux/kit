/**
 * Product user stories for Kit — the vibe-coding boom.
 * Used by CLI home, ready, and unify next-steps.
 */

export type StoryId =
  | "chaos-cleanup"
  | "new-repo-agents"
  | "multi-agent-sync"
  | "empty-start"
  | "already-solid";

export interface UserStory {
  id: StoryId;
  /** Short label for UI */
  title: string;
  /** Who this is for (1 line) */
  who: string;
  /** Pain they feel */
  pain: string;
  /** What Kit does */
  win: string;
  /** Primary command to run */
  primary: string;
  /** Follow-up commands */
  next: string[];
}

/** Canonical stories we optimize for. */
export const USER_STORIES: Record<StoryId, UserStory> = {
  "chaos-cleanup": {
    id: "chaos-cleanup",
    title: "Clean the skill pile",
    who: "You already installed a mountain of Claude/Codex skills.",
    pain: "Most are noise. None travel between agents. You don't know what to trust.",
    win: "kit unify ranks keepers, filters automation junk, and builds one portable library.",
    primary: "kit unify",
    next: [
      "kit unify --write",
      "kit unify --write --link",
      "kit list",
    ],
  },
  "new-repo-agents": {
    id: "new-repo-agents",
    title: "Make this repo agent-ready",
    who: "You just opened (or cloned) a project and want agents useful in one shot.",
    pain: "You shouldn't have to hand-pick skills every time you start something.",
    win: "kit ready recommends a pack, installs it, applies it, and links your agents.",
    primary: "kit ready --write",
    next: [
      "kit ready --write --unify",
      "kit recommend --dir .",
      "kit tui",
    ],
  },
  "multi-agent-sync": {
    id: "multi-agent-sync",
    title: "Same skills in every agent",
    who: "You bounce between Claude Code, Codex, and Grok on the same work.",
    pain: "Skills live in different folders. Claude has it, Codex doesn't.",
    win: "One library in ~/.kit, then kit link --to all so every agent sees the same set.",
    primary: "kit link --to all --write",
    next: [
      "kit unify --write --link",
      "kit paths",
      "kit doctor",
    ],
  },
  "empty-start": {
    id: "empty-start",
    title: "First install",
    who: "You just ran npm i -g @mzwin/kit and your library is empty.",
    pain: "Blank slate — you need a clear first move, not a wall of commands.",
    win: "Start with essentials or unify if you already have skills elsewhere.",
    primary: "kit init --pack essentials",
    next: [
      "kit ready --write",
      "kit unify",
      "kit tui",
    ],
  },
  "already-solid": {
    id: "already-solid",
    title: "You're set — keep shipping",
    who: "Library has skills, agents look linked, project has a pack.",
    pain: "Nothing broken — you just want the next useful action.",
    win: "Recommend for this repo, doctor, or open the TUI.",
    primary: "kit recommend --dir .",
    next: ["kit doctor", "kit tui", "kit pack list"],
  },
};

export interface SituationSnapshot {
  libraryCount: number;
  harnessSkillEstimate: number;
  keeperEstimate: number;
  noiseEstimate: number;
  hasProjectSignals: boolean;
  recommendedPack: string | null;
  recommendSummary: string | null;
  projectLinkedLikely: boolean;
}

/**
 * Pick the best story from a lightweight situation snapshot.
 */
export function pickStory(s: SituationSnapshot): UserStory {
  // Big harness pile + thin library → chaos cleanup is the product moment
  if (s.harnessSkillEstimate >= 15 && s.libraryCount < 8) {
    return USER_STORIES["chaos-cleanup"];
  }
  if (s.harnessSkillEstimate >= 40 && s.keeperEstimate >= 3) {
    return USER_STORIES["chaos-cleanup"];
  }

  // Empty library
  if (s.libraryCount === 0) {
    if (s.harnessSkillEstimate >= 5) return USER_STORIES["chaos-cleanup"];
    return USER_STORIES["empty-start"];
  }

  // Has library, project exists, not linked
  if (s.libraryCount > 0 && s.hasProjectSignals && !s.projectLinkedLikely) {
    return USER_STORIES["new-repo-agents"];
  }

  // Has library, multi-agent skills elsewhere not synced
  if (s.libraryCount > 0 && s.harnessSkillEstimate >= 10 && s.keeperEstimate >= 2) {
    return USER_STORIES["multi-agent-sync"];
  }

  if (s.libraryCount > 0 && s.projectLinkedLikely) {
    return USER_STORIES["already-solid"];
  }

  if (s.hasProjectSignals) {
    return USER_STORIES["new-repo-agents"];
  }

  return USER_STORIES["empty-start"];
}
