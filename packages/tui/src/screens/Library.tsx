import React from "react";
import { Box, Text } from "ink";
import type { CheckResult, InstalledSkill } from "@kit-skills/core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header } from "../components/Chrome.js";
import { ErrorLine, Pulse, SuccessLine } from "../components/Motion.js";
import { ActionFlash, SelectPulse } from "../motion/index.js";

export interface LibraryProps {
  skills: InstalledSkill[];
  selectedIndex: number;
  selectTick: number;
  frames: PixelFrame[];
  confirmRemove: boolean;
  statusMessage?: string;
  errorMessage?: string;
  libraryError?: string;
  actionFlash?: string;
  actionNonce?: number;
  /** Last validate/test checks for selected skill. */
  lastChecks?: CheckResult[];
}

export function Library({
  skills,
  selectedIndex,
  selectTick,
  frames,
  confirmRemove,
  statusMessage,
  errorMessage,
  libraryError,
  actionFlash,
  actionNonce = 0,
  lastChecks,
}: LibraryProps): React.ReactElement {
  const selected = skills[selectedIndex];
  const empty = skills.length === 0;

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header screen="Library" detail={`${skills.length} skill(s)`} />

      <Box marginTop={1} flexDirection="row">
        <Box marginRight={2} flexShrink={0}>
          <MascotPlayer frames={frames} playing size="compact" />
        </Box>

        <Box flexDirection="column" flexGrow={1}>
          {empty ? (
            <Box flexDirection="column">
              <Text bold>Empty</Text>
              <Text dimColor>p packs · ↵ install a starter</Text>
            </Box>
          ) : (
            <Box flexDirection="column">
              {skills.map((skill, index) => (
                <Text key={skill.name}>
                  <SelectPulse
                    selected={index === selectedIndex}
                    tick={selectTick}
                  />{" "}
                  <Text bold={index === selectedIndex}>{skill.name}</Text>
                  <Text dimColor>@{skill.version}</Text>
                </Text>
              ))}
            </Box>
          )}

          {selected ? (
            <Box marginTop={1} flexDirection="column">
              <Text dimColor>{selected.description}</Text>
              <Text dimColor>{selected.installPath}</Text>
              <Text dimColor>v validate · t test · r remove</Text>
            </Box>
          ) : null}

          {lastChecks && lastChecks.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              {lastChecks.slice(0, 6).map((c) => (
                <Text key={`${c.id}-${c.level}`} dimColor>
                  {c.level === "pass" ? "✓" : c.level === "fail" ? "✗" : "!"}{" "}
                  {c.message}
                </Text>
              ))}
            </Box>
          ) : null}

          {confirmRemove && selected ? (
            <Box marginTop={1}>
              <Pulse label={`Remove ${selected.name}? y / n`} />
            </Box>
          ) : null}

          <Box marginTop={1}>
            <ActionFlash message={actionFlash} nonce={actionNonce} />
          </Box>
        </Box>
      </Box>

      {statusMessage ? (
        <Box marginTop={1}>
          <SuccessLine message={statusMessage.replace(/^✓\s*/, "")} />
        </Box>
      ) : null}
      {errorMessage || libraryError ? (
        <Box marginTop={1}>
          <ErrorLine message={errorMessage ?? libraryError ?? ""} />
        </Box>
      ) : null}

      <Footer keys="↑↓ · v validate · t test · r remove · k paths · h home · q quit" />
    </Box>
  );
}
