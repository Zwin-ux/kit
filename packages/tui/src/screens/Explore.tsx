import React, { useEffect, useState } from "react";
import { Box, Text } from "ink";
import type { RegistryPackSummary } from "@mzwin/kit-core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header } from "../components/Chrome.js";
import { ErrorLine, Spinner, SuccessLine } from "../components/Motion.js";

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
 * Remote registry browser. Install still uses the local pack by name.
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
  const [slowLoad, setSlowLoad] = useState(false);

  useEffect(() => {
    if (!loading) {
      setSlowLoad(false);
      return;
    }
    setSlowLoad(false);
    const t = setTimeout(() => setSlowLoad(true), 2000);
    return () => clearTimeout(t);
  }, [loading]);

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header screen="Explore" detail="registry" />

      <Box marginTop={1} flexDirection="row">
        <Box marginRight={2} flexShrink={0}>
          <MascotPlayer frames={frames} playing size="compact" />
        </Box>

        <Box flexDirection="column" flexGrow={1}>
          <Text dimColor>{registryUrl}</Text>
          {query ? <Text dimColor>/{query}</Text> : null}

          <Box marginTop={1} flexDirection="column">
            <Text bold>Remote</Text>
            {loading ? (
              <Spinner
                label={slowLoad ? "Still loading…" : "Loading"}
                active
              />
            ) : packs.length === 0 ? (
              <Text dimColor>No packs · check KIT_REGISTRY_URL</Text>
            ) : (
              packs.map((pack, index) => {
                const mark = index === selectedIndex ? "›" : " ";
                return (
                  <Text key={pack.name}>
                    {mark}{" "}
                    <Text bold={index === selectedIndex}>{pack.title}</Text>
                    <Text dimColor>
                      {" "}
                      · {pack.skillCount} skills
                    </Text>
                  </Text>
                );
              })
            )}
          </Box>

          {selected && !loading ? (
            <Box marginTop={1} flexDirection="column">
              <Text dimColor>{selected.description}</Text>
              <Text dimColor>↵ install local match · / search · r refresh</Text>
            </Box>
          ) : null}
        </Box>
      </Box>

      {statusMessage ? (
        <Box marginTop={1}>
          <SuccessLine message={statusMessage.replace(/^✓\s*/, "")} />
        </Box>
      ) : null}
      {errorMessage ? (
        <Box marginTop={1}>
          <ErrorLine message={errorMessage} />
        </Box>
      ) : null}

      <Footer keys="↑↓ · ↵ install · / search · r refresh · h home · p packs · q quit" />
    </Box>
  );
}
