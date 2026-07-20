import React from "react";
import { Box, Text } from "ink";
import type { InstalledSkill } from "@kit-skills/core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header } from "../components/Chrome.js";

export interface LibraryProps {
  skills: InstalledSkill[];
  selectedIndex: number;
  frames: PixelFrame[];
  confirmRemove: boolean;
  statusMessage?: string;
  errorMessage?: string;
  libraryError?: string;
}

export function Library({
  skills,
  selectedIndex,
  frames,
  confirmRemove,
  statusMessage,
  errorMessage,
  libraryError,
}: LibraryProps): React.ReactElement {
  const selected = skills[selectedIndex];

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header
        screen="Library"
        detail={`${skills.length} skill(s) · offline`}
      />

      {skills.length === 0 ? (
        <Box marginTop={1} flexDirection="column">
          <MascotPlayer
            frames={frames}
            playing
            size="compact"
            caption="Library empty — install a pack from Home or Packs"
          />
          <Box marginTop={1}>
            <Text dimColor>Press p for packs · h for home</Text>
          </Box>
        </Box>
      ) : (
        <Box marginTop={1} flexDirection="column">
          {skills.map((skill, index) => {
            const mark = index === selectedIndex ? "›" : " ";
            return (
              <Text key={skill.name}>
                {mark} {skill.name}@{skill.version}
                <Text dimColor> — {skill.description}</Text>
              </Text>
            );
          })}
        </Box>
      )}

      {selected ? (
        <Box marginTop={1} flexDirection="column">
          <Text bold>Selected</Text>
          <Text>
            {selected.name}@{selected.version}
          </Text>
          <Text dimColor>{selected.description}</Text>
          <Text dimColor>{selected.installPath}</Text>
          <Text dimColor>
            agents: {selected.compatibility.join(", ")}
          </Text>
        </Box>
      ) : null}

      {confirmRemove && selected ? (
        <Box marginTop={1}>
          <Text color="yellow">
            Remove {selected.name}? y confirm · n cancel
          </Text>
        </Box>
      ) : null}

      {statusMessage ? (
        <Box marginTop={1}>
          <Text>{statusMessage}</Text>
        </Box>
      ) : null}
      {errorMessage || libraryError ? (
        <Box marginTop={1}>
          <Text color="red">{errorMessage ?? libraryError}</Text>
        </Box>
      ) : null}

      <Footer keys="↑↓ select · r remove · p packs · h home · s splash · q quit" />
    </Box>
  );
}
