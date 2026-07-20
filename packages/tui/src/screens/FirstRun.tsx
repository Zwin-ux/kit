import React from "react";
import { Box, Text } from "ink";
import { FIRST_RUN_PACK_OPTIONS } from "@mzwin/kit-core";
import { PackIcon } from "../mascot/PackIcon.js";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { Footer, Header } from "../components/Chrome.js";
import { ScreenShell } from "../components/ScreenShell.js";
import { ErrorLine, Spinner, SuccessLine } from "../components/Motion.js";
import { StaggerLines } from "../motion/index.js";

export interface FirstRunProps {
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
  busy?: boolean;
  statusMessage?: string;
  errorMessage?: string;
}

export function FirstRun({
  frames,
  mascotVariant = "idle",
  busy,
  statusMessage,
  errorMessage,
}: FirstRunProps): React.ReactElement {
  const optionNodes = FIRST_RUN_PACK_OPTIONS.map((option) => (
    <Box key={option.name} flexDirection="column">
      <Text>
        <Text bold>{option.key}</Text>
        <Text> </Text>
        <PackIcon packName={option.name} size="mini" animate />
        <Text> {option.title}</Text>
      </Text>
      <Text dimColor wrap="truncate">
        {"  "}
        {option.blurb}
      </Text>
    </Box>
  ));

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1} width="100%">
      <Header screen="First run" detail="starter pack" />

      <Box marginTop={1} width="100%">
        <ScreenShell frames={frames} mascotVariant={mascotVariant}>
          <Text bold>Pick a starter</Text>
          <Text dimColor>
            7 kits · each ships dependency skills via essentials
          </Text>

          <Box marginTop={1} flexDirection="column">
            <StaggerLines stepMs={45}>{optionNodes}</StaggerLines>
          </Box>
        </ScreenShell>
      </Box>

      {busy ? (
        <Box marginTop={1}>
          <Spinner label="Installing" active style="icon" />
        </Box>
      ) : null}

      {statusMessage && !busy ? (
        <Box marginTop={1}>
          <SuccessLine message={statusMessage.replace(/^✓\s*/, "")} />
        </Box>
      ) : null}

      {errorMessage ? (
        <Box marginTop={1}>
          <ErrorLine message={errorMessage} />
        </Box>
      ) : null}

      <Footer keys="1–7 install · s skip · q quit" />
    </Box>
  );
}
