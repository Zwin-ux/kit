import React from "react";
import { Box, Text } from "ink";
import { FIRST_RUN_PACK_OPTIONS } from "@kit-skills/core";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import type { PixelFrame } from "../mascot/types.js";
import { Footer, Header } from "../components/Chrome.js";

export interface FirstRunProps {
  frames: PixelFrame[];
  busy?: boolean;
  statusMessage?: string;
  errorMessage?: string;
}

export function FirstRun({
  frames,
  busy,
  statusMessage,
  errorMessage,
}: FirstRunProps): React.ReactElement {
  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header screen="First run" detail="choose a starter pack" />

      <Box marginTop={1}>
        <MascotPlayer frames={frames} playing={!busy} size="compact" />
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Welcome. Install a starter pack to begin.</Text>
        <Text dimColor>
          One pack gives your agents a strong default skill set.
        </Text>
      </Box>

      <Box marginTop={1} flexDirection="column">
        {FIRST_RUN_PACK_OPTIONS.map((option) => (
          <Box key={option.name} flexDirection="column">
            <Text>
              <Text bold>{option.key}</Text>
              <Text> {option.title} </Text>
              <Text dimColor>({option.name})</Text>
            </Text>
            <Text dimColor>  {option.blurb}</Text>
          </Box>
        ))}
      </Box>

      {busy ? (
        <Box marginTop={1}>
          <Text>Installing pack…</Text>
        </Box>
      ) : null}

      {statusMessage ? (
        <Box marginTop={1}>
          <Text>{statusMessage}</Text>
        </Box>
      ) : null}

      {errorMessage ? (
        <Box marginTop={1}>
          <Text color="red">{errorMessage}</Text>
        </Box>
      ) : null}

      <Footer keys="1 essentials · 2 web-app · 3 library · s skip · q quit" />
    </Box>
  );
}
