import React from "react";
import { Box, Text } from "ink";
import type {
  AppliedPackRecord,
  InstalledSkill,
  PackListItem,
  SkillRecommendation,
  ToolkitRecommendation,
} from "@kit-skills/core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header, StatusLine } from "../components/Chrome.js";
import {
  CountUp,
  ErrorLine,
  ProgressBar,
  Spinner,
  SuccessLine,
} from "../components/Motion.js";
import { ToolkitPicker } from "../components/ToolkitPicker.js";
import { ActionFlash, BlinkCursor } from "../motion/index.js";

export interface HomeProps {
  frames: PixelFrame[];
  skills: InstalledSkill[];
  packs: PackListItem[];
  applied: AppliedPackRecord[];
  selectedPackIndex: number;
  selectTick: number;
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
  skills,
  packs,
  applied,
  selectedPackIndex,
  selectTick,
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
}: HomeProps): React.ReactElement {
  const emptyLibrary = skills.length === 0;
  const appliedNames = new Set(applied.map((a) => a.name));
  const selected = packs[selectedPackIndex];
  const topReasons =
    recommended.find((r) => r.packName === topPick)?.reasons.slice(0, 2) ?? [];

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header
        screen="Home"
        {...(busy
          ? { detail: "installing…" }
          : userLogin
            ? { detail: `@${userLogin}` }
            : { detail: "local" })}
      />

      <Box marginTop={1} flexDirection="row">
        <Box flexDirection="column" marginRight={2} flexShrink={0}>
          <MascotPlayer frames={frames} playing size="compact" />
        </Box>

        <Box flexDirection="column" flexGrow={1}>
          <Text bold>Pointed at</Text>
          {pointingProject ? (
            <Text>
              path: {pointDraft}
              <BlinkCursor active />
            </Text>
          ) : (
            <Text dimColor>{shortPath(targetProject)}</Text>
          )}
          {recommendSummary && !pointingProject ? (
            <Text>★ {recommendSummary}</Text>
          ) : null}
          {topReasons.length > 0 && !pointingProject ? (
            <Text dimColor>· {topReasons[0]}</Text>
          ) : null}
          <Text dimColor>
            {userLogin ? `@${userLogin}` : "not logged in"}
            {doctorSummary ? ` · ${doctorSummary}` : ""}
            {" · o point"}
          </Text>

          <Box marginTop={1} flexDirection="column">
            <Text bold>Toolkits</Text>
            {topPick ? (
              <Text dimColor>★ {topPick} selected for this project</Text>
            ) : (
              <Text dimColor>Start with Essentials on most repos</Text>
            )}
            {packsError ? (
              <Text color="red">{packsError}</Text>
            ) : (
              <ToolkitPicker
                packs={packs}
                selectedIndex={selectedPackIndex}
                selectTick={selectTick}
                recommended={recommended}
                appliedNames={appliedNames}
              />
            )}
          </Box>

          {skillRecs.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>Suggested skills</Text>
              {skillRecs.slice(0, 5).map((s) => (
                <Text key={s.skillName} dimColor>
                  · {s.skillName}
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
              <Text dimColor>None yet · ↵ installs ★ selection (+ deps)</Text>
            ) : (
              <>
                {skills.slice(0, 4).map((skill) => (
                  <Text key={skill.name} dimColor>
                    · {skill.name}
                  </Text>
                ))}
                {skills.length > 4 ? (
                  <Text dimColor>  +{skills.length - 4} more (l)</Text>
                ) : null}
              </>
            )}
          </Box>

          {applied.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>Applied here</Text>
              {applied.map((pack) => (
                <Text key={pack.name} dimColor>
                  · {pack.title} ({pack.skills.length})
                </Text>
              ))}
            </Box>
          ) : null}

          {selected && !busy && !pointingProject ? (
            <Box marginTop={1}>
              <Text dimColor>
                ↵ install {selected.title}
                {" · "}a apply to project · k link
              </Text>
            </Box>
          ) : null}

          {pointingProject ? (
            <Box marginTop={1}>
              <Text dimColor>Enter set · Esc cancel</Text>
            </Box>
          ) : null}

          <Box marginTop={1}>
            <ActionFlash message={actionFlash} nonce={actionNonce} />
          </Box>
        </Box>
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
          <Spinner label="Working" active />
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
        {...(statusMessage !== undefined && busy
          ? { message: statusMessage }
          : {})}
      />

      <Footer keys="o point · ↑↓ · ↵ install · a apply · k paths · d doctor · e explore · l library · q quit" />
    </Box>
  );
}
