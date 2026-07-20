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
import { StatusIcon } from "../mascot/StatusIcon.js";
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
  const topReasons =
    recommended.find((r) => r.packName === topPick)?.reasons.slice(0, 2) ?? [];
  const variant =
    mascotVariant ??
    (busy ? "scan" : celebrateCount !== undefined ? "success" : "idle");
  const skillShow = scale.listMaxItems;

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

      <Box marginTop={1} width="100%">
        <ScreenShell
          frames={frames}
          mascotVariant={variant}
          {...(focusLabel !== undefined ? { focusLabel } : {})}
        >
          <Text bold>Project</Text>
          {pointingProject ? (
            <Text>
              path: {pointDraft}
              <BlinkCursor active />
            </Text>
          ) : (
            <Text dimColor wrap="truncate">
              {shortPath(targetProject)}
            </Text>
          )}
          {recommendSummary && !pointingProject ? (
            <Text wrap="truncate">* {recommendSummary}</Text>
          ) : null}
          {topReasons.length > 0 &&
          !pointingProject &&
          scale.mode !== "stack" ? (
            <Text dimColor wrap="truncate">
              · {topReasons[0]}
            </Text>
          ) : null}
          <Text dimColor wrap="truncate">
            {userLogin ? `@${userLogin}` : "local"}
            {doctorSummary ? ` · ${doctorSummary}` : ""}
            {" · o point"}
          </Text>
          {agentStatusLine ? (
            <Text wrap="truncate">agents {agentStatusLine}</Text>
          ) : null}

          <Box marginTop={1} flexDirection="column">
            <Text bold>Toolkits</Text>
            {topPick && scale.mode !== "stack" ? (
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
              />
            )}
          </Box>

          {skillRecs.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>
                <StatusIcon id="skill" size="mini" /> Suggested skills
              </Text>
              {skillRecs.slice(0, skillShow).map((s) => (
                <Text key={s.skillName} dimColor wrap="truncate">
                  <StatusIcon id="skill" size="mini" dimColor /> {s.skillName}
                  {s.fromPack ? ` · ${s.fromPack}` : ""}
                </Text>
              ))}
            </Box>
          ) : null}

          <Box marginTop={1} flexDirection="column">
            <Text bold>Installed</Text>
            {libraryError ? (
              <Text color="red">{libraryError}</Text>
            ) : emptyLibrary ? (
              <Text dimColor>
                <StatusIcon id="folder" size="mini" dimColor /> None yet · ↵
                installs ★ selection (+ deps)
              </Text>
            ) : (
              <>
                {skills.slice(0, skillShow).map((skill) => (
                  <Text key={skill.name} dimColor wrap="truncate">
                    <StatusIcon id="ok" size="mini" dimColor /> {skill.name}
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

          {applied.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>Applied here</Text>
              {applied.slice(0, skillShow).map((pack) => (
                <Text key={pack.name} dimColor wrap="truncate">
                  <StatusIcon id="pack" size="mini" dimColor /> {pack.title} (
                  {pack.skills.length})
                </Text>
              ))}
              {applied.length > skillShow ? (
                <Text dimColor>
                  {"  "}+{applied.length - skillShow} more
                </Text>
              ) : null}
            </Box>
          ) : null}

          {/* Always 1 action line (pad when empty) — no height jump on selection */}
          <Box marginTop={1} flexShrink={0}>
            <Text dimColor>
              {fixedLine(
                selected && !busy && !pointingProject
                  ? `enter install ${selected.title} · a apply · k link`
                  : pointingProject
                    ? "Enter set path · Esc cancel"
                    : " ",
                Math.min(scale.contentSoftMax, 64),
              )}
            </Text>
          </Box>

          <Box marginTop={1}>
            <ActionFlash message={actionFlash} nonce={actionNonce} />
          </Box>
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
