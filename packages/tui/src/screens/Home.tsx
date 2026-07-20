import React from "react";
import { Box, Text } from "ink";
import type {
  AppliedPackRecord,
  InstalledSkill,
  PackListItem,
  SkillRecommendation,
  ToolkitRecommendation,
} from "@mzwin/kit-core";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { Footer, Header, StatusLine } from "../components/Chrome.js";
import { ScreenShell } from "../components/ScreenShell.js";
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
}

function shortPath(p: string): string {
  const home = process.env.USERPROFILE ?? process.env.HOME ?? "";
  if (home && p.startsWith(home)) {
    return `~${p.slice(home.length).replace(/\\/g, "/")}`;
  }
  return p.replace(/\\/g, "/");
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
}: HomeProps): React.ReactElement {
  const scale = useLayoutScale();
  const emptyLibrary = skills.length === 0;
  const appliedNames = new Set(applied.map((a) => a.name));
  const selected = packs[selectedPackIndex];
  const variant =
    mascotVariant ??
    (busy ? "scan" : celebrateCount !== undefined ? "success" : "idle");
  // Cognitive a11y: secondary lists are summaries so tools stay in view.
  // Fullscreen (wide / tall) opens them up; small windows stay dense.
  const skillShow = scale.listMaxItems;
  const compact = scale.mode === "stack" || scale.rows < 26;
  const showSecondaryLists =
    scale.mode === "wide" || (scale.mode === "split" && scale.rows >= 32);

  // Sticky footer focus only (ToolkitPicker owns the in-list focus line)
  const focusLabel =
    selected && packs.length > 0
      ? `${selectedPackIndex + 1}/${packs.length} ${selected.title}`
      : undefined;

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
          ? { detail: "installing…" }
          : userLogin
            ? { detail: `@${userLogin}` }
            : { detail: "local" })}
      />

      <Box marginTop={compact ? 0 : 1} width="100%">
        <ScreenShell frames={frames} mascotVariant={variant}>
          <Text bold>Project</Text>
          {pointingProject ? (
            <Text>
              path: {pointDraft}
              <BlinkCursor active />
            </Text>
          ) : (
            <Text dimColor wrap="truncate">
              {shortPath(targetProject)}
              {recommendSummary ? ` · ${recommendSummary}` : ""}
              {userLogin ? ` · @${userLogin}` : " · local"}
              {doctorSummary ? ` · ${doctorSummary}` : ""}
              {agentStatusLine ? ` · agents ${agentStatusLine}` : ""}
            </Text>
          )}

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
                dense={compact}
              />
            )}
          </Box>

          {/* Secondary blocks only when viewport has room — avoid scroll-past-focus */}
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
                  none yet · enter installs focus (+ deps)
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
            <Text dimColor>none installed · enter installs focus</Text>
          )}

          {showSecondaryLists && applied.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>Applied</Text>
              {applied.slice(0, skillShow).map((pack) => (
                <Text key={pack.name} dimColor wrap="truncate">
                  {"  "}+ {pack.title} ({pack.skills.length})
                </Text>
              ))}
              {applied.length > skillShow ? (
                <Text dimColor>
                  {"  "}+{applied.length - skillShow} more
                </Text>
              ) : null}
            </Box>
          ) : null}

          {/* Always 1 action line — no height jump on selection */}
          <Box marginTop={1} flexShrink={0}>
            <Text dimColor>
              {fixedLine(
                selected && !busy && !pointingProject
                  ? `enter install ${selected.title} · a apply · k link`
                  : pointingProject
                    ? "Enter set path · Esc cancel"
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

      {/* Sticky focus + counts — stays at bottom even if list scrolls */}
      <StatusLine
        skillCount={skills.length}
        packCount={packs.length}
        {...(focusLabel !== undefined ? { focus: focusLabel } : {})}
        {...(statusMessage !== undefined && busy
          ? { message: statusMessage }
          : {})}
      />

      <Footer
        keys={
          scale.mode === "stack"
            ? "o point · up/down · enter install · a apply · k link · q quit"
            : "o point · up/down · enter install · a apply · k paths · d doctor · e explore · l library · q quit"
        }
      />
    </Box>
  );
}
