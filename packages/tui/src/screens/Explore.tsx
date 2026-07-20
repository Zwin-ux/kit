import React from "react";
import { Box, Text } from "ink";
import type { RegistryPackSummary } from "@kit-skills/core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header } from "../components/Chrome.js";
import { Spinner } from "../components/Motion.js";

export interface ExploreProps {
  frames: PixelFrame[];
  packs: RegistryPackSummary[];
  selectedIndex: number;
  loading: boolean;
  registryUrl: string;
  query: string;
  statusMessage?: string;
  errorMessage?: string;
}

/**
 * Remote registry browser (Railway catalog).
 * Install still happens locally via packs after user chooses.
 */
export function Explore({
  frames,
  packs,
  selectedIndex,
  loading,
  registryUrl,
  query,
  statusMessage,
  errorMessage,
}: ExploreProps): React.ReactElement {
  const selected = packs[selectedIndex];

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header screen="Explore" detail="registry" />

      <Box marginTop={1}>
        <MascotPlayer
          frames={frames}
          playing={!loading}
          size="compact"
          caption="Browse public toolkits · install offline after pick"
        />
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text dimColor>{registryUrl}</Text>
        {query ? <Text dimColor>filter: {query}</Text> : null}
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Remote packs</Text>
        {loading ? (
          <Spinner label="Loading catalog" active />
        ) : packs.length === 0 ? (
          <Text dimColor>No packs from registry. Check network or KIT_REGISTRY_URL.</Text>
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

      {selected && !loading ? (
        <Box marginTop={1} flexDirection="column">
          <Text bold>{selected.title}</Text>
          <Text dimColor>{selected.description}</Text>
          <Text dimColor>
            skills: {selected.skills.join(", ")}
          </Text>
          <Text dimColor>
            Press i to install this pack from local catalog (same name)
          </Text>
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

      <Footer keys="↑↓ select · / search · i install local · r refresh · h home · p packs · q quit" />
    </Box>
  );
}
