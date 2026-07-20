import React from "react";
import { Box, Text } from "ink";
import type { PackListItem } from "@kit-skills/core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header } from "../components/Chrome.js";

export interface PacksProps {
  packs: PackListItem[];
  selectedIndex: number;
  frames: PixelFrame[];
  busy?: boolean;
  statusMessage?: string;
  errorMessage?: string;
  packsError?: string;
}

export function Packs({
  packs,
  selectedIndex,
  frames,
  busy,
  statusMessage,
  errorMessage,
  packsError,
}: PacksProps): React.ReactElement {
  const selected = packs[selectedIndex];

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header screen="Packs" detail="offline starter packs" />

      <Box marginTop={1}>
        <MascotPlayer frames={frames} playing={!busy} size="compact" />
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Available packs</Text>
        {packsError ? (
          <Text color="red">{packsError}</Text>
        ) : packs.length === 0 ? (
          <Text dimColor>
            No packs found. Run from the Kit repo or set KIT_PACKS.
          </Text>
        ) : (
          packs.map((pack, index) => {
            const mark = index === selectedIndex ? "›" : " ";
            return (
              <Text key={pack.name}>
                {mark} {pack.title}{" "}
                <Text dimColor>
                  ({pack.name} · {pack.skillCount} skills · v{pack.version})
                </Text>
              </Text>
            );
          })
        )}
      </Box>

      {selected ? (
        <Box marginTop={1} flexDirection="column">
          <Text bold>{selected.title}</Text>
          <Text dimColor>{selected.description}</Text>
          {selected.projectTypes.length > 0 ? (
            <Text dimColor>
              project types: {selected.projectTypes.join(", ")}
            </Text>
          ) : null}
          {selected.tags.length > 0 ? (
            <Text dimColor>tags: {selected.tags.join(", ")}</Text>
          ) : null}
        </Box>
      ) : null}

      {busy ? (
        <Box marginTop={1}>
          <Text>Working…</Text>
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

      <Footer keys="↑↓ select · i install · a apply · l library · h home · s splash · q quit" />
    </Box>
  );
}
