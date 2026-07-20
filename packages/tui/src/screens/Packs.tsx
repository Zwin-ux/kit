import React from "react";
import { Box, Text } from "ink";
import type { PackListItem, ToolkitRecommendation } from "@kit-skills/core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header } from "../components/Chrome.js";
import {
  ErrorLine,
  ProgressBar,
  Spinner,
  SuccessLine,
} from "../components/Motion.js";
import { ActionFlash, BlinkCursor } from "../motion/index.js";
import { ToolkitPicker } from "../components/ToolkitPicker.js";

export interface PacksProps {
  packs: PackListItem[];
  selectedIndex: number;
  selectTick?: number;
  frames: PixelFrame[];
  recommended: ToolkitRecommendation[];
  filter: string;
  filtering?: boolean;
  appliedNames: Set<string>;
  busy?: boolean;
  statusMessage?: string;
  errorMessage?: string;
  packsError?: string;
  actionFlash?: string;
  actionNonce?: number;
  progress?: { current: number; total: number; skillName: string };
}

export function Packs({
  packs,
  selectedIndex,
  selectTick = 0,
  frames,
  recommended,
  filter,
  filtering,
  appliedNames,
  busy,
  statusMessage,
  errorMessage,
  packsError,
  actionFlash,
  actionNonce = 0,
  progress,
}: PacksProps): React.ReactElement {
  const selected = packs[selectedIndex];

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header screen="Packs" detail="toolkits" />

      <Box marginTop={1} flexDirection="row">
        <Box marginRight={2} flexShrink={0}>
          <MascotPlayer frames={frames} playing size="compact" />
        </Box>

        <Box flexDirection="column" flexGrow={1}>
          <Text bold>Browse</Text>
          {filter || filtering ? (
            <Text dimColor>
              /{filter}
              <BlinkCursor active={Boolean(filtering)} />
              {filter ? " · Esc clear" : ""}
            </Text>
          ) : (
            <Text dimColor>type to filter</Text>
          )}
          {packsError ? (
            <Text color="red">{packsError}</Text>
          ) : (
            <ToolkitPicker
              packs={packs}
              selectedIndex={selectedIndex}
              selectTick={selectTick}
              recommended={recommended}
              appliedNames={appliedNames}
              {...(filter ? { filter } : {})}
            />
          )}
          {selected && !busy ? (
            <Box marginTop={1}>
              <Text dimColor>
                ↵ install {selected.title}
                {selected.extends?.length
                  ? ` (+ ${selected.extends.join(", ")})`
                  : ""}{" "}
                · a apply · k link
              </Text>
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
          <Spinner label="Installing" active />
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

      <Footer keys="↑↓ · ↵ install · a apply · type filter · k paths · d doctor · h home · q quit" />
    </Box>
  );
}
