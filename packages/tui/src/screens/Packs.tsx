import React from "react";
import { Box, Text } from "ink";
import type { PackListItem, ToolkitRecommendation } from "@mzwin/kit-core";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { StatusIcon } from "../mascot/StatusIcon.js";
import { Footer, Header } from "../components/Chrome.js";
import { ScreenShell } from "../components/ScreenShell.js";
import {
  ErrorLine,
  ProgressBar,
  Spinner,
  SuccessLine,
} from "../components/Motion.js";
import {
  ActionFlash,
  BlinkCursor,
  fixedLine,
  type SelectDirection,
} from "../motion/index.js";
import { ToolkitPicker } from "../components/ToolkitPicker.js";

export interface PacksProps {
  packs: PackListItem[];
  selectedIndex: number;
  selectTick?: number;
  selectDirection?: SelectDirection;
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
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
  selectDirection = "none",
  frames,
  mascotVariant = "idle",
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
  const variant =
    mascotVariant ??
    (busy ? "scan" : statusMessage ? "success" : "idle");

  const scale = useLayoutScale();
  return (
    <Box
      flexDirection="column"
      paddingX={scale.padX}
      paddingY={scale.padY}
      width="100%"
    >
      <Header screen="Packs" detail="toolkits" />

      <Box marginTop={1} width="100%">
        <ScreenShell frames={frames} mascotVariant={variant}>
          <Text bold>
            <StatusIcon id="pack" size="mini" /> Browse
          </Text>
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
              selectDirection={selectDirection}
              recommended={recommended}
              appliedNames={appliedNames}
              {...(filter ? { filter } : {})}
            />
          )}
          <Box marginTop={1} flexShrink={0}>
            <Text dimColor>
              {fixedLine(
                selected && !busy
                  ? `enter install ${selected.title}${selected.extends?.length ? ` +${selected.extends.join("+")}` : ""} · a apply · k link`
                  : " ",
                64,
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

      <Footer keys="↑↓ · ↵ install · a apply · type filter · k paths · d doctor · h home · q quit" />
    </Box>
  );
}
