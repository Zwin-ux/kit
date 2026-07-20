import React from "react";
import { Box, Text } from "ink";
import type { PackListItem, ToolkitRecommendation } from "@kit-skills/core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header } from "../components/Chrome.js";
import { ProgressBar, Spinner } from "../components/Motion.js";
import { ToolkitPicker } from "../components/ToolkitPicker.js";

export interface PacksProps {
  packs: PackListItem[];
  selectedIndex: number;
  frames: PixelFrame[];
  recommended: ToolkitRecommendation[];
  filter: string;
  appliedNames: Set<string>;
  busy?: boolean;
  statusMessage?: string;
  errorMessage?: string;
  packsError?: string;
  progress?: { current: number; total: number; skillName: string };
}

export function Packs({
  packs,
  selectedIndex,
  frames,
  recommended,
  filter,
  appliedNames,
  busy,
  statusMessage,
  errorMessage,
  packsError,
  progress,
}: PacksProps): React.ReactElement {
  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header screen="Packs" detail="toolkit picker" />

      <Box marginTop={1}>
        <MascotPlayer
          frames={frames}
          playing={!busy}
          size="compact"
          caption="Browse · filter · install · apply"
        />
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Toolkits</Text>
        {filter ? (
          <Text dimColor>
            filter: {filter} (type to filter · Esc clear)
          </Text>
        ) : (
          <Text dimColor>type to filter by tag, type, or name</Text>
        )}
        {packsError ? (
          <Text color="red">{packsError}</Text>
        ) : (
          <ToolkitPicker
            packs={packs}
            selectedIndex={selectedIndex}
            recommended={recommended}
            appliedNames={appliedNames}
            {...(filter ? { filter } : {})}
          />
        )}
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
          <Spinner label="Installing toolkit" active />
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

      <Footer keys="↑↓ select · type filter · i install · a apply · e explore · h home · q quit" />
    </Box>
  );
}
