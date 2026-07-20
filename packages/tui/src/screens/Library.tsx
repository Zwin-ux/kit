import React from "react";
import { Box, Text } from "ink";
import type { CheckResult, InstalledSkill } from "@mzwin/kit-core";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { Footer, Header } from "../components/Chrome.js";
import { ScreenShell } from "../components/ScreenShell.js";
import { ErrorLine, Pulse, SuccessLine } from "../components/Motion.js";
import {
  ActionFlash,
  fixedLine,
  fixedLines,
  SelectPulse,
  StaggerLines,
  type SelectDirection,
} from "../motion/index.js";

export interface LibraryProps {
  skills: InstalledSkill[];
  selectedIndex: number;
  selectTick: number;
  selectDirection?: SelectDirection;
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
  selectDirection = "none",
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
  const scale = useLayoutScale();
  const selected = skills[selectedIndex];
  const empty = skills.length === 0;
  const focusLabel =
    selected && skills.length > 0
      ? `${selectedIndex + 1}/${skills.length} ${selected.name}`
      : undefined;

  return (
    <Box
      flexDirection="column"
      paddingX={scale.padX}
      paddingY={scale.padY}
      width="100%"
    >
      <Header screen="Library" detail={`${skills.length} skill(s)`} />

      <Box marginTop={1} width="100%">
        <ScreenShell
          frames={frames}
          mascotVariant={mascotVariant}
          {...(focusLabel !== undefined ? { focusLabel } : {})}
        >
          {empty ? (
            <Box flexDirection="column">
              <Box>
                <Text bold>Empty library</Text>
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
                    direction={selectDirection}
                  />
                  <Text dimColor>s </Text>
                  <Text bold={index === selectedIndex}>{skill.name}</Text>
                  <Text dimColor>@{skill.version}</Text>
                </Text>
              ))}
            </Box>
          )}

          {/* Always 3 lines so ↑↓ never reflows (pad when empty) */}
          <Box marginTop={1} flexDirection="column" flexShrink={0}>
            {fixedLines(selected?.description ?? "", 2, 56).map((line, i) => (
              <Text key={`d${i}`} dimColor>
                {line}
              </Text>
            ))}
            <Text dimColor>
              {fixedLine(
                selected
                  ? `${selected.installPath}  ·  v validate · t test · r remove`
                  : " ",
                56,
              )}
            </Text>
          </Box>

          {lastChecks && lastChecks.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              {lastChecks.slice(0, 3).map((c) => (
                <Text key={`${c.id}-${c.level}`} dimColor>
                  {fixedLine(
                    `${c.level === "pass" ? "ok" : c.level === "fail" ? "x" : "!"} ${c.message}`,
                    56,
                  )}
                </Text>
              ))}
            </Box>
          ) : null}

          {confirmRemove && selected ? (
            <Box marginTop={1}>
              <Pulse label={`Remove ${selected.name}? y / n · Esc cancel`} />
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
