import React from "react";
import { Box, Text } from "ink";
import type {
  AppliedPackRecord,
  InstalledSkill,
  PackListItem,
} from "@kit-skills/core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header, StatusLine } from "../components/Chrome.js";

export interface HomeProps {
  frames: PixelFrame[];
  skills: InstalledSkill[];
  packs: PackListItem[];
  applied: AppliedPackRecord[];
  selectedPackIndex: number;
  libraryError?: string;
  packsError?: string;
  statusMessage?: string;
  busy?: boolean;
}

export function Home({
  frames,
  skills,
  packs,
  applied,
  selectedPackIndex,
  libraryError,
  packsError,
  statusMessage,
  busy,
}: HomeProps): React.ReactElement {
  const selected = packs[selectedPackIndex];
  const emptyLibrary = skills.length === 0;

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header screen="Home" detail={busy ? "working…" : "offline"} />

      {emptyLibrary ? (
        <Box marginTop={1}>
          <MascotPlayer
            frames={frames}
            playing={!busy}
            size="compact"
            caption="kit-idle · install a pack to fill your library"
          />
        </Box>
      ) : null}

      <Box marginTop={1} flexDirection="column">
        <Text bold>Installed skills</Text>
        {libraryError ? (
          <Text color="red">{libraryError}</Text>
        ) : emptyLibrary ? (
          <Text dimColor>
            None yet. Press p for packs, or 1–3 to install quickly.
          </Text>
        ) : (
          skills.slice(0, 6).map((skill) => (
            <Text key={skill.name}>
              · {skill.name}@{skill.version}
              <Text dimColor> — {skill.description}</Text>
            </Text>
          ))
        )}
        {skills.length > 6 ? (
          <Text dimColor>
            {" "}
            …and {skills.length - 6} more (press l for Library)
          </Text>
        ) : null}
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Starter packs</Text>
        {packsError ? (
          <Text color="red">{packsError}</Text>
        ) : packs.length === 0 ? (
          <Text dimColor>
            No packs found. Run Kit from the repo or set KIT_PACKS.
          </Text>
        ) : (
          packs.map((pack, index) => {
            const mark = index === selectedPackIndex ? "›" : " ";
            return (
              <Text key={pack.name}>
                {mark} {index + 1}. {pack.title}{" "}
                <Text dimColor>
                  ({pack.name} · {pack.skillCount} skills)
                </Text>
              </Text>
            );
          })
        )}
        {selected ? <Text dimColor>  {selected.description}</Text> : null}
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Applied in this project</Text>
        {applied.length === 0 ? (
          <Text dimColor>None. Select a pack and press a to apply.</Text>
        ) : (
          applied.map((pack) => (
            <Text key={pack.name}>
              · {pack.title}@{pack.version}{" "}
              <Text dimColor>({pack.skills.length} skills)</Text>
            </Text>
          ))
        )}
      </Box>

      <StatusLine
        skillCount={skills.length}
        packCount={packs.length}
        {...(statusMessage !== undefined ? { message: statusMessage } : {})}
      />

      <Footer keys="↑↓ pack · i install · a apply · l library · p packs · s splash · q quit" />
    </Box>
  );
}
