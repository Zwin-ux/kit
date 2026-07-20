import React from "react";
import { Box, Text } from "ink";
import type {
  AppliedPackRecord,
  InstalledSkill,
  PackListItem,
  ToolkitRecommendation,
} from "@kit-skills/core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header, StatusLine } from "../components/Chrome.js";
import { ProgressBar, Spinner, SuccessLine } from "../components/Motion.js";
import { ToolkitPicker } from "../components/ToolkitPicker.js";

export interface HomeProps {
  frames: PixelFrame[];
  skills: InstalledSkill[];
  packs: PackListItem[];
  applied: AppliedPackRecord[];
  selectedPackIndex: number;
  recommended: ToolkitRecommendation[];
  topPick: string | null;
  userLogin?: string;
  libraryError?: string;
  packsError?: string;
  statusMessage?: string;
  busy?: boolean;
  progress?: { current: number; total: number; skillName: string };
}

export function Home({
  frames,
  skills,
  packs,
  applied,
  selectedPackIndex,
  recommended,
  topPick,
  userLogin,
  libraryError,
  packsError,
  statusMessage,
  busy,
  progress,
}: HomeProps): React.ReactElement {
  const emptyLibrary = skills.length === 0;
  const appliedNames = new Set(applied.map((a) => a.name));

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header
        screen="Home"
        detail={
          busy
            ? "working…"
            : userLogin
              ? `@${userLogin} · offline`
              : "offline"
        }
      />

      {emptyLibrary ? (
        <Box marginTop={1}>
          <MascotPlayer
            frames={frames}
            playing={!busy}
            size="compact"
            caption="kit-idle · pick a toolkit below"
          />
        </Box>
      ) : null}

      <Box marginTop={1} flexDirection="column">
        <Text bold>Installed skills</Text>
        {libraryError ? (
          <Text color="red">{libraryError}</Text>
        ) : emptyLibrary ? (
          <Text dimColor>
            None yet. Choose a toolkit — ★ marks the recommended pack.
          </Text>
        ) : (
          skills.slice(0, 5).map((skill) => (
            <Text key={skill.name}>
              · {skill.name}@{skill.version}
              <Text dimColor> — {skill.description}</Text>
            </Text>
          ))
        )}
        {skills.length > 5 ? (
          <Text dimColor>  …+{skills.length - 5} more (l library)</Text>
        ) : null}
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Skill toolkits</Text>
        {topPick ? (
          <Text dimColor>
            Suggested for this project: {topPick}
          </Text>
        ) : null}
        {packsError ? (
          <Text color="red">{packsError}</Text>
        ) : (
          <ToolkitPicker
            packs={packs}
            selectedIndex={selectedPackIndex}
            recommended={recommended}
            appliedNames={appliedNames}
          />
        )}
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Applied here</Text>
        {applied.length === 0 ? (
          <Text dimColor>None. Press a to apply selected toolkit.</Text>
        ) : (
          applied.map((pack) => (
            <Text key={pack.name}>
              · {pack.title}@{pack.version}{" "}
              <Text dimColor>({pack.skills.length} skills)</Text>
            </Text>
          ))
        )}
      </Box>

      {busy && progress ? (
        <Box marginTop={1}>
          <ProgressBar
            current={progress.current}
            total={progress.total}
            label={`installing ${progress.skillName}`}
          />
        </Box>
      ) : busy ? (
        <Box marginTop={1}>
          <Spinner label="Working" active />
        </Box>
      ) : null}

      {statusMessage && !busy ? (
        <Box marginTop={1}>
          <SuccessLine message={statusMessage.replace(/^✓\s*/, "")} />
        </Box>
      ) : null}

      <StatusLine
        skillCount={skills.length}
        packCount={packs.length}
        {...(statusMessage !== undefined && busy
          ? { message: statusMessage }
          : {})}
      />

      <Footer keys="↑↓ toolkit · i install · a apply · e explore · l library · p packs · s splash · q quit" />
    </Box>
  );
}
