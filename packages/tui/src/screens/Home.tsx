import React from "react";
import { Box, Text } from "ink";
import type {
  AppliedPackRecord,
  InstalledSkill,
  PackListItem,
  SkillRecommendation,
  ToolkitRecommendation,
  UserStory,
} from "@mzwin/kit-core";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { Footer, Header, StatusLine } from "../components/Chrome.js";
import { ScreenShell } from "../components/ScreenShell.js";
import { ActionRail } from "../components/ActionRail.js";
import { MenuButton } from "../components/MenuButton.js";
import {
  CountUp,
  ErrorLine,
  ProgressBar,
  Spinner,
  SuccessLine,
} from "../components/Motion.js";
import { ToolkitPicker } from "../components/ToolkitPicker.js";
import {
  ActionFlash,
  BlinkCursor,
  fixedLine,
  type SelectDirection,
} from "../motion/index.js";

export type HomeConfirm =
  | "none"
  | "ready-write"
  | "unify-write";

export interface HomeProps {
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
  skills: InstalledSkill[];
  packs: PackListItem[];
  applied: AppliedPackRecord[];
  selectedPackIndex: number;
  selectTick: number;
  selectDirection?: SelectDirection;
  recommended: ToolkitRecommendation[];
  skillRecs: SkillRecommendation[];
  topPick: string | null;
  targetProject: string;
  recommendSummary?: string;
  pointingProject?: boolean;
  pointDraft?: string;
  userLogin?: string;
  doctorSummary?: string;
  libraryError?: string;
  packsError?: string;
  statusMessage?: string;
  statusIsError?: boolean;
  celebrateCount?: number;
  actionFlash?: string;
  actionNonce?: number;
  busy?: boolean;
  progress?: { current: number; total: number; skillName: string };
  /** e.g. claude:ok · codex:x · grok:ok */
  agentStatusLine?: string;
  /** Product story for this project (from detectSituation). */
  story?: UserStory;
  /** Pending write confirmation after a dry-run plan. */
  confirmAction?: HomeConfirm;
  /** Multi-line plan preview (ready/unify steps). */
  planLines?: string[];
}

function shortPath(p: string): string {
  const home = process.env.USERPROFILE ?? process.env.HOME ?? "";
  if (home && p.startsWith(home)) {
    return `~${p.slice(home.length).replace(/\\/g, "/")}`;
  }
  return p.replace(/\\/g, "/");
}

function storyPrimaryKey(story: UserStory | undefined): "r" | "u" | "w" {
  if (!story) return "r";
  if (story.id === "chaos-cleanup") return "u";
  if (story.primary.includes("workbench") || story.primary.includes("tui")) {
    return "w";
  }
  if (story.primary.includes("unify")) return "u";
  return "r";
}

export function Home({
  frames,
  mascotVariant = "idle",
  skills,
  packs,
  applied,
  selectedPackIndex,
  selectTick,
  selectDirection = "none",
  recommended,
  skillRecs,
  topPick,
  targetProject,
  recommendSummary,
  pointingProject,
  pointDraft = "",
  userLogin,
  doctorSummary,
  libraryError,
  packsError,
  statusMessage,
  statusIsError,
  celebrateCount,
  actionFlash,
  actionNonce = 0,
  busy,
  progress,
  agentStatusLine,
  story,
  confirmAction = "none",
  planLines,
}: HomeProps): React.ReactElement {
  const scale = useLayoutScale();
  const emptyLibrary = skills.length === 0;
  const appliedNames = new Set(applied.map((a) => a.name));
  const selected = packs[selectedPackIndex];
  const variant =
    mascotVariant ??
    (busy ? "scan" : celebrateCount !== undefined ? "success" : "idle");
  const skillShow = scale.listMaxItems;
  const compact = scale.mode === "stack" || scale.rows < 26;
  const showSecondaryLists =
    scale.mode === "wide" || (scale.mode === "split" && scale.rows >= 32);
  const primaryKey = storyPrimaryKey(story);

  const focusLabel =
    selected && packs.length > 0
      ? `${selectedPackIndex + 1}/${packs.length} ${selected.title}`
      : undefined;

  const storyLine = story
    ? `${story.title} — ${story.win}`
    : recommendSummary
      ? `Recommend: ${recommendSummary}`
      : "Point at a project, then Ready or Workbench.";

  const railItems: Array<{
    key: string;
    label: string;
    primary?: boolean;
    disabled?: boolean;
  }> = [
    {
      key: "r",
      label: "Ready",
      primary: primaryKey === "r" && confirmAction === "none",
      disabled: Boolean(busy),
    },
    {
      key: "u",
      label: "Unify",
      primary: primaryKey === "u" && confirmAction === "none",
      disabled: Boolean(busy),
    },
    {
      key: "w",
      label: "Main menu",
      primary: primaryKey === "w" && confirmAction === "none",
    },
    { key: "o", label: "Point" },
    { key: "k", label: "Paths" },
    { key: "?", label: "Help" },
  ];

  const menuIcons: Record<string, string> = {
    r: "ready",
    u: "unify",
    w: "kit",
    o: "point",
    k: "paths",
    "?": "help",
  };

  return (
    <Box
      flexDirection="column"
      paddingX={scale.padX}
      paddingY={scale.padY}
      width="100%"
    >
      <Header
        screen="Home"
        {...(busy
          ? { detail: "working…" }
          : userLogin
            ? { detail: `@${userLogin}` }
            : { detail: "local" })}
      />

      <Box marginTop={compact ? 0 : 1} width="100%">
        <ScreenShell frames={frames} mascotVariant={variant}>
          {/* Project identity — one dense strip, not three status lines */}
          <Text bold>Project</Text>
          {pointingProject ? (
            <Text>
              path: {pointDraft}
              <BlinkCursor active />
            </Text>
          ) : (
            <Text dimColor wrap="truncate">
              {shortPath(targetProject)}
              {topPick ? ` · pack *${topPick}` : ""}
              {doctorSummary ? ` · ${doctorSummary}` : ""}
              {userLogin ? ` · @${userLogin}` : ""}
            </Text>
          )}

          {confirmAction === "none" ? (
            <Box flexDirection="column" marginTop={1}>
              <Text bold wrap="truncate">
                {fixedLine(storyLine, Math.max(40, scale.contentSoftMax)).trimEnd()}
              </Text>
              <Box flexDirection="column" marginTop={0}>
                {railItems.map((item) => (
                  <MenuButton
                    key={item.key}
                    icon={menuIcons[item.key] ?? "kit"}
                    label={item.label}
                    hotkey={item.key}
                    selected={Boolean(item.primary)}
                    disabled={Boolean(item.disabled)}
                    variant="list"
                  />
                ))}
              </Box>
              <Text dimColor>Click a row or press the key.</Text>
            </Box>
          ) : (
            <Box marginTop={1} flexDirection="column">
              <Text bold color="yellow">
                {confirmAction === "ready-write"
                  ? "Write Ready? installs pack, applies, links agents, runs doctor."
                  : "Write Unify? adopts keepers into ~/.kit (optional link after)."}
              </Text>
              <Text bold inverse>
                {" "}
                y write · n cancel{" "}
              </Text>
            </Box>
          )}

          {planLines && planLines.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>Plan</Text>
              {planLines.slice(0, compact ? 3 : 5).map((line, i) => (
                <Text key={`plan-${i}`} dimColor wrap="truncate">
                  {line}
                </Text>
              ))}
            </Box>
          ) : null}

          <Box marginTop={1} flexDirection="column">
            <Text bold>Toolkits</Text>
            {topPick && !compact ? (
              <Text dimColor wrap="truncate">
                * {topPick} for this project
              </Text>
            ) : null}
            {packsError ? (
              <Text color="red">{packsError}</Text>
            ) : (
              <ToolkitPicker
                packs={packs}
                selectedIndex={selectedPackIndex}
                selectTick={selectTick}
                selectDirection={selectDirection}
                recommended={recommended}
                appliedNames={appliedNames}
                dense={compact || confirmAction !== "none"}
              />
            )}
          </Box>

          {showSecondaryLists && skillRecs.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>Suggested</Text>
              {skillRecs.slice(0, skillShow).map((s) => (
                <Text key={s.skillName} dimColor wrap="truncate">
                  {"  "}+ {s.skillName}
                  {s.fromPack ? ` · ${s.fromPack}` : ""}
                </Text>
              ))}
            </Box>
          ) : null}

          {showSecondaryLists ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>Installed</Text>
              {libraryError ? (
                <Text color="red">{libraryError}</Text>
              ) : emptyLibrary ? (
                <Text dimColor>
                  none yet · r ready or enter installs focus
                </Text>
              ) : (
                <>
                  {skills.slice(0, skillShow).map((skill) => (
                    <Text key={skill.name} dimColor wrap="truncate">
                      {"  "}+ {skill.name}
                    </Text>
                  ))}
                  {skills.length > skillShow ? (
                    <Text dimColor>
                      {"  "}+{skills.length - skillShow} more (l)
                    </Text>
                  ) : null}
                </>
              )}
            </Box>
          ) : !emptyLibrary ? (
            <Text dimColor wrap="truncate">
              installed {skills.length}
              {applied.length > 0 ? ` · applied ${applied.length}` : ""}
              {" · l library"}
            </Text>
          ) : (
            <Text dimColor>none installed · r ready · enter installs focus</Text>
          )}

          {showSecondaryLists && applied.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>Applied</Text>
              {applied.slice(0, skillShow).map((pack) => (
                <Text key={pack.name} dimColor wrap="truncate">
                  {"  "}+ {pack.title} ({pack.skills.length})
                </Text>
              ))}
            </Box>
          ) : null}

          <Box marginTop={1} flexShrink={0}>
            <Text dimColor>
              {fixedLine(
                selected && !busy && !pointingProject && confirmAction === "none"
                  ? `enter install ${selected.title} · a apply · r ready · ? help`
                  : pointingProject
                    ? "Enter set path · Esc cancel"
                    : confirmAction !== "none"
                      ? "y write · n cancel"
                      : " ",
                Math.max(32, scale.contentSoftMax),
              )}
            </Text>
          </Box>

          {actionFlash ? (
            <Box marginTop={0}>
              <ActionFlash message={actionFlash} nonce={actionNonce} />
            </Box>
          ) : null}
        </ScreenShell>
      </Box>

      {busy && progress ? (
        <Box marginTop={1}>
          <ProgressBar
            current={progress.current}
            total={progress.total}
            label={progress.skillName}
          />
        </Box>
      ) : busy ? (
        <Box marginTop={1}>
          <Spinner label="Working" active style="icon" />
        </Box>
      ) : null}

      {statusMessage && !busy ? (
        <Box marginTop={1} flexDirection="column">
          {statusIsError ? (
            <ErrorLine message={statusMessage} />
          ) : (
            <SuccessLine message={statusMessage.replace(/^✓\s*/, "")} />
          )}
          {celebrateCount !== undefined && !statusIsError ? (
            <Text dimColor>
              +
              <CountUp to={celebrateCount} suffix=" skills" />
            </Text>
          ) : null}
        </Box>
      ) : null}

      <StatusLine
        skillCount={skills.length}
        packCount={packs.length}
        {...(focusLabel !== undefined ? { focus: focusLabel } : {})}
        {...(agentStatusLine !== undefined ? { agents: agentStatusLine } : {})}
        {...(statusMessage !== undefined && busy
          ? { message: statusMessage }
          : {})}
      />

      <Footer
        keys={
          confirmAction !== "none"
            ? "y write · n cancel · q quit"
            : scale.mode === "stack"
              ? "r ready · u unify · w terminal · enter install · ? help · q"
              : "r ready · u unify · w terminal · o point · enter install · ? help · q quit"
        }
      />
    </Box>
  );
}
