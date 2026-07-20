import React from "react";
import { Box, Text } from "ink";
import type { CheckResult, InstalledSkill } from "@mzwin/kit-core";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { StatusIcon } from "../mascot/StatusIcon.js";
import { levelToStatusIcon } from "../mascot/statusIcons.js";
import { Footer, Header } from "../components/Chrome.js";
import { ScreenShell } from "../components/ScreenShell.js";
import { ErrorLine, Pulse, SuccessLine } from "../components/Motion.js";
import { ActionFlash, SelectPulse, StaggerLines } from "../motion/index.js";

export interface LibraryProps {
  skills: InstalledSkill[];
  selectedIndex: number;
  selectTick: number;
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
  confirmRemove: boolean;
  statusMessage?: string;
  errorMessage?: string;
  libraryError?: string;
  actionFlash?: string;
  actionNonce?: number;
  lastChecks?: CheckResult[];
}

export function Library({
  skills,
  selectedIndex,
  selectTick,
  frames,
  mascotVariant = "idle",
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
    <Box flexDirection="column" paddingX={2} paddingY={1} width="100%">
      <Header screen="Library" detail={`${skills.length} skill(s)`} />

      <Box marginTop={1} width="100%">
        <ScreenShell frames={frames} mascotVariant={mascotVariant}>
          {empty ? (
            <Box flexDirection="column">
              <Box>
                <StatusIcon id="folder" size="mini" dimColor />
                <Text bold> Empty library</Text>
              </Box>
              <StaggerLines>
                {[
                  <Text key="a" dimColor>
                    p packs · install a starter
                  </Text>,
                  <Text key="b" dimColor>
                    kit ready --write · one-shot setup
                  </Text>,
                ]}
              </StaggerLines>
            </Box>
          ) : (
            <Box flexDirection="column">
              {skills.map((skill, index) => (
                <Text key={skill.name} wrap="truncate">
                  <SelectPulse
                    selected={index === selectedIndex}
                    tick={selectTick}
                  />{" "}
                  <StatusIcon id="skill" size="mini" dimColor />{" "}
                  <Text bold={index === selectedIndex}>{skill.name}</Text>
                  <Text dimColor>@{skill.version}</Text>
                </Text>
              ))}
            </Box>
          )}

          {selected ? (
            <Box marginTop={1} flexDirection="column">
              <Text dimColor wrap="wrap">
                {selected.description}
              </Text>
              <Text dimColor wrap="truncate">
                {selected.installPath}
              </Text>
              <Text dimColor>
                <StatusIcon id="arrow" size="mini" dimColor /> v validate · t
                test · r remove
              </Text>
            </Box>
          ) : null}

          {lastChecks && lastChecks.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              {lastChecks.slice(0, 6).map((c) => (
                <Text key={`${c.id}-${c.level}`} dimColor wrap="truncate">
                  <StatusIcon id={levelToStatusIcon(c.level)} size="mini" />{" "}
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
        </ScreenShell>
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
