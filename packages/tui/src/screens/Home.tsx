import React from "react";
import { Box, Text } from "ink";
import type { InstalledSkill } from "@kit-skills/core";
import { KIT_PACKAGE_VERSION } from "@kit-skills/shared";
import type { PixelFrame } from "../mascot/types.js";
import { renderFrame } from "../mascot/renderBitmap.js";

export interface HomeProps {
  frame: PixelFrame;
  skills: InstalledSkill[];
  libraryError?: string;
}

export function Home({
  frame,
  skills,
  libraryError,
}: HomeProps): React.ReactElement {
  // Smaller presence on Home: still silhouette, same frame stream.
  const art = renderFrame(frame);

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Box>
        <Text bold>Kit</Text>
        <Text dimColor>  v{KIT_PACKAGE_VERSION}  ·  Home</Text>
      </Box>

      <Box marginTop={1}>
        <Text>{art}</Text>
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Installed skills</Text>
        {libraryError ? (
          <Text color="red">{libraryError}</Text>
        ) : skills.length === 0 ? (
          <Text dimColor>None yet. Use: kit install ./path/to/skill</Text>
        ) : (
          skills.slice(0, 8).map((skill) => (
            <Text key={skill.name}>
              · {skill.name}@{skill.version}
            </Text>
          ))
        )}
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Quick actions (CLI for now)</Text>
        <Text dimColor>kit list · kit install · kit validate · kit remove</Text>
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text dimColor>Account: offline (sign-in later)</Text>
        <Text>Keys: s splash · q quit</Text>
      </Box>
    </Box>
  );
}
