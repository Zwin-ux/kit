import React from "react";
import { Box, Text } from "ink";
import type { PixelFrame, MascotVariant } from "../mascot/types.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { Footer, Header } from "../components/Chrome.js";
import { ScreenShell } from "../components/ScreenShell.js";

export interface HelpProps {
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
  /** Where the user opened help from (for context footer). */
  fromScreen?: string;
}

interface HelpSection {
  title: string;
  rows: Array<{ key: string; action: string }>;
}

const SECTIONS: HelpSection[] = [
  {
    title: "Everywhere",
    rows: [
      { key: "?", action: "This help" },
      { key: "h", action: "Home" },
      { key: "w", action: "Workbench (runners + services)" },
      { key: "q", action: "Quit (not while typing)" },
      { key: "Ctrl+C", action: "Force quit" },
    ],
  },
  {
    title: "Home · product",
    rows: [
      { key: "r", action: "Ready plan (install → apply → link → doctor)" },
      { key: "y", action: "Confirm write after Ready/Unify plan" },
      { key: "u", action: "Unify plan (scan · rank · keepers)" },
      { key: "o", action: "Point at a project path" },
      { key: "↑↓", action: "Move toolkit focus" },
      { key: "Enter/i", action: "Install focused toolkit" },
      { key: "a", action: "Apply toolkit to project" },
      { key: "1–7", action: "Jump-install pack by number" },
    ],
  },
  {
    title: "Screens",
    rows: [
      { key: "p", action: "Packs (filter by typing)" },
      { key: "l", action: "Library (v validate · t test · r remove)" },
      { key: "e", action: "Explore remote catalog" },
      { key: "d", action: "Doctor health checks" },
      { key: "k", action: "Paths / link agents" },
    ],
  },
  {
    title: "Action Terminal (w)",
    rows: [
      { key: "1–4 / Tab", action: "Skills · Agents · Services · Ops" },
      { key: "Enter", action: "Run focused action" },
      { key: "o / O", action: "Start / stop local Ollama (kit-managed)" },
      { key: "p", action: "Pull Ollama model" },
      { key: "e / m", action: "Edit job · toggle inspect/build" },
      { key: "PgUp/PgDn", action: "Scroll shared action log" },
      { key: "Esc", action: "Stop run · or back home" },
    ],
  },
];

/**
 * Keyboard map — product surface, not a debug dump.
 * Opened with `?` from any main screen.
 */
export function Help({
  frames,
  mascotVariant = "idle",
  fromScreen,
}: HelpProps): React.ReactElement {
  const scale = useLayoutScale();
  const compact = scale.mode === "stack" || scale.rows < 26;
  const sections = compact ? SECTIONS.slice(0, 3) : SECTIONS;

  return (
    <Box
      flexDirection="column"
      paddingX={scale.padX}
      paddingY={scale.padY}
      width="100%"
    >
      <Header screen="Help" detail="keys" />

      <Box marginTop={1} width="100%">
        <ScreenShell
          frames={frames}
          mascotVariant={mascotVariant}
          hideMascot={frames.length === 0}
        >
          <Text bold>What can you do</Text>
          <Text dimColor wrap="truncate">
            Kit is a workbench for packs, skills, agents, and local runners.
          </Text>

          {sections.map((section) => (
            <Box key={section.title} marginTop={1} flexDirection="column">
              <Text bold inverse>
                {" "}
                {section.title.toUpperCase()}{" "}
              </Text>
              {section.rows.map((row) => (
                <Text key={`${section.title}-${row.key}`} wrap="truncate">
                  <Text bold>{row.key.padEnd(10)}</Text>
                  <Text dimColor>{row.action}</Text>
                </Text>
              ))}
            </Box>
          ))}
        </ScreenShell>
      </Box>

      <Footer
        keys={
          fromScreen
            ? `Esc back · ${fromScreen} · q quit`
            : "Esc or h home · q quit"
        }
      />
    </Box>
  );
}
