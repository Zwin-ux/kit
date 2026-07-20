import React, { useEffect, useState } from "react";
import { Box, Text } from "ink";
import type { RegistryPackSummary } from "@mzwin/kit-core";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { PackIcon } from "../mascot/PackIcon.js";
import { StatusIcon } from "../mascot/StatusIcon.js";
import { Footer, Header } from "../components/Chrome.js";
import { ScreenShell } from "../components/ScreenShell.js";
import { ErrorLine, Spinner, SuccessLine } from "../components/Motion.js";
import { SelectPulse, type SelectDirection } from "../motion/index.js";

export interface ExploreProps {
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
  packs: RegistryPackSummary[];
  selectedIndex: number;
  selectTick?: number;
  selectDirection?: SelectDirection;
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
  mascotVariant = "idle",
  packs,
  selectedIndex,
  selectTick = 0,
  selectDirection = "none",
  loading,
  registryUrl,
  query,
  statusMessage,
  errorMessage,
}: ExploreProps): React.ReactElement {
  const selected = packs[selectedIndex];
  const [slowLoad, setSlowLoad] = useState(false);
  const variant = mascotVariant ?? (loading ? "scan" : "idle");

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
    <Box flexDirection="column" paddingX={2} paddingY={1} width="100%">
      <Header screen="Explore" detail="registry" />

      <Box marginTop={1} width="100%">
        <ScreenShell frames={frames} mascotVariant={variant}>
          <Text dimColor wrap="truncate">
            {registryUrl}
          </Text>
          {query ? <Text dimColor>/{query}</Text> : null}

          <Box marginTop={1} flexDirection="column">
            <Text bold>
              <StatusIcon id="pack" size="mini" /> Remote
            </Text>
            {loading ? (
              <Spinner
                label={slowLoad ? "Still loading…" : "Loading"}
                active
                style="icon"
              />
            ) : packs.length === 0 ? (
              <Text dimColor>
                <StatusIcon id="folder" size="mini" dimColor /> No packs ·
                check KIT_REGISTRY_URL
              </Text>
            ) : (
              packs.map((pack, index) => {
                return (
                  <Text key={pack.name} wrap="truncate">
                    <SelectPulse
                      selected={index === selectedIndex}
                      tick={selectTick}
                      direction={selectDirection}
                    />
                    <PackIcon packName={pack.name} size="mini" animate />{" "}
                    <Text bold={index === selectedIndex}>{pack.title}</Text>
                    <Text dimColor> · {pack.skillCount} skills</Text>
                  </Text>
                );
              })
            )}
          </Box>

          {selected && !loading ? (
            <Box marginTop={1} flexDirection="column">
              <Text dimColor wrap="wrap">
                {selected.description}
              </Text>
              <Text dimColor>
                <StatusIcon id="arrow" size="mini" dimColor /> ↵ install local
                match · / search · r refresh
              </Text>
            </Box>
          ) : null}
        </ScreenShell>
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
