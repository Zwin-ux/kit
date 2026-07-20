import React from "react";
import { Box, Text } from "ink";
import type { InstalledSkill, PackListItem } from "@kit-skills/core";
import type { AppliedPackRecord } from "@kit-skills/core";
import { renderFrame } from "../mascot/renderBitmap.js";
import type { PixelFrame } from "../mascot/types.js";
import { Footer, Header, StatusLine } from "../components/Chrome.js";

export interface HomeProps {
  frame: PixelFrame;
  skills: InstalledSkill[];
  packs: PackListItem[];
  applied: AppliedPackRecord[];
  selectedPackIndex: number;
  libraryError?: string;
  packsError?: string;
  statusMessage?: string;
  busy?: boolean;
  /** Animate mascot only when true (keep Home calm by default). */
  showMascot?: boolean;
}

export function Home({
  frame,
  skills,
  packs,
  applied,
  selectedPackIndex,
  libraryError,
  packsError,
  statusMessage,
  busy,
  showMascot = false,
}: HomeProps): React.ReactElement {
  const selected = packs[selectedPackIndex];

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header
        screen="Home"
        detail={busy ? "working…" : "offline"}
      />

      {showMascot ? (
        <Box marginTop={1}>
          <Text>{renderFrame(frame)}</Text>
        </Box>
      ) : null}

      <Box marginTop={1} flexDirection="column">
        <Text bold>Installed skills</Text>
        {libraryError ? (
          <Text color="red">{libraryError}</Text>
        ) : skills.length === 0 ? (
          <Text dimColor>
            None yet. Press 1–3 to install a pack, or open First run from splash.
          </Text>
        ) : (
          skills.slice(0, 8).map((skill) => (
            <Text key={skill.name}>
              · {skill.name}@{skill.version}
              <Text dimColor> — {skill.description}</Text>
            </Text>
          ))
        )}
        {skills.length > 8 ? (
          <Text dimColor>  …and {skills.length - 8} more</Text>
        ) : null}
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Starter packs</Text>
        {packsError ? (
          <Text color="red">{packsError}</Text>
        ) : packs.length === 0 ? (
          <Text dimColor>
            No packs found. Run Kit from the repo or set KIT_PACKS.
          </Text>
        ) : (
          packs.map((pack, index) => {
            const mark = index === selectedPackIndex ? "›" : " ";
            return (
              <Text key={pack.name}>
                {mark} {index + 1}. {pack.title}{" "}
                <Text dimColor>
                  ({pack.name} · {pack.skillCount} skills)
                </Text>
              </Text>
            );
          })
        )}
        {selected ? (
          <Text dimColor>  {selected.description}</Text>
        ) : null}
      </Box>

      <Box marginTop={1} flexDirection="column">
        <Text bold>Applied in this project</Text>
        {applied.length === 0 ? (
          <Text dimColor>
            None. After install: kit pack apply {"<pack>"} --dir .
          </Text>
        ) : (
          applied.map((pack) => (
            <Text key={pack.name}>
              · {pack.title}@{pack.version}{" "}
              <Text dimColor>({pack.skills.length} skills)</Text>
            </Text>
          ))
        )}
      </Box>

      <StatusLine
        skillCount={skills.length}
        packCount={packs.length}
        {...(statusMessage !== undefined ? { message: statusMessage } : {})}
      />

      <Footer keys="↑↓ select pack · i install · a apply to cwd · 1–3 quick · s splash · q quit" />
    </Box>
  );
}
