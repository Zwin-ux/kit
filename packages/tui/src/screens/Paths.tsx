import React from "react";
import { Box, Text } from "ink";
import type {
  HarnessId,
  LinkResult,
  PathReport,
  PathScope,
} from "@mzwin/kit-core";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { StatusIcon } from "../mascot/StatusIcon.js";
import { harnessToStatusIcon } from "../mascot/statusIcons.js";
import { Footer, Header } from "../components/Chrome.js";
import { ScreenShell } from "../components/ScreenShell.js";
import { ErrorLine, Pulse, Spinner, SuccessLine } from "../components/Motion.js";
import { ActionFlash, SelectPulse } from "../motion/index.js";

const LINKABLE: HarnessId[] = ["claude-code", "codex", "grok-build"];

export interface PathsProps {
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
  report?: PathReport;
  loading?: boolean;
  linking?: boolean;
  selectedHarnessIndex: number;
  selectTick: number;
  scope: PathScope;
  targetRoot?: string;
  confirmWrite?: boolean;
  linkResult?: LinkResult;
  statusMessage?: string;
  errorMessage?: string;
  actionFlash?: string;
  actionNonce?: number;
}

/**
 * Wire Kit skills into agent harness folders — only after you approve the path.
 */
export function Paths({
  frames,
  mascotVariant = "idle",
  report,
  loading,
  linking,
  selectedHarnessIndex,
  selectTick,
  scope,
  targetRoot,
  confirmWrite,
  linkResult,
  statusMessage,
  errorMessage,
  actionFlash,
  actionNonce = 0,
}: PathsProps): React.ReactElement {
  const selected = LINKABLE[selectedHarnessIndex] ?? "claude-code";
  const variant =
    mascotVariant ??
    (linking || loading
      ? "scan"
      : linkResult && !linkResult.dryRun
        ? "success"
        : "idle");

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1} width="100%">
      <Header screen="Paths" detail={`${scope} · link`} />

      <Box marginTop={1} width="100%">
        <ScreenShell frames={frames} mascotVariant={variant}>
          <Box>
            <StatusIcon id="link" size="mini" />
            <Text bold> Where should skills land?</Text>
          </Box>
          <Text dimColor wrap="wrap">
            Pick a harness · scope · approve the folder before any write
          </Text>

          {report ? (
            <Box marginTop={1} flexDirection="column">
              <Text dimColor>
                <StatusIcon id="skill" size="mini" dimColor /> library ·{" "}
                {report.installedSkillNames.length} skills ready
              </Text>
            </Box>
          ) : null}

          <Box marginTop={1} flexDirection="column">
            <Text bold>Harness</Text>
            {LINKABLE.map((id, index) => {
              const entry = report?.entries.find(
                (e) => e.harness === id && e.scope === scope,
              );
              return (
                <Text key={id} wrap="truncate">
                  <SelectPulse
                    selected={index === selectedHarnessIndex}
                    tick={selectTick}
                  />{" "}
                  <StatusIcon id={harnessToStatusIcon(id)} size="mini" />{" "}
                  <Text bold={index === selectedHarnessIndex}>{id}</Text>
                  <Text dimColor>
                    {entry
                      ? entry.exists
                        ? " · exists"
                        : " · new folder"
                      : ""}
                  </Text>
                </Text>
              );
            })}
          </Box>

          <Box marginTop={1} flexDirection="column">
            <Text bold>Scope</Text>
            <Text dimColor>
              {scope === "project" ? "› project" : "  project"} ·{" "}
              {scope === "personal" ? "› personal" : "  personal"}
              {"  "}
              (tab to switch)
            </Text>
          </Box>

          {targetRoot ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>
                <StatusIcon id="folder" size="mini" /> Target folder
              </Text>
              <Text wrap="truncate">{targetRoot}</Text>
            </Box>
          ) : null}

          {confirmWrite && targetRoot ? (
            <Box marginTop={1}>
              <Pulse
                label={`Write into this folder? y yes · n cancel`}
              />
            </Box>
          ) : (
            <Box marginTop={1}>
              <Text dimColor wrap="truncate">
                <StatusIcon id="arrow" size="mini" dimColor /> ↵ propose{" "}
                {selected} · p plan only · tab scope · r refresh
              </Text>
            </Box>
          )}

          {loading || linking ? (
            <Box marginTop={1}>
              <Spinner
                label={linking ? "Linking" : "Loading paths"}
                active
                style="icon"
              />
            </Box>
          ) : null}

          {linkResult ? (
            <Box marginTop={1} flexDirection="column">
              <Text dimColor wrap="truncate">
                <StatusIcon
                  id={linkResult.dryRun ? "info" : "ok"}
                  size="mini"
                />{" "}
                {linkResult.dryRun ? "plan" : "wrote"} · {linkResult.linked}{" "}
                linked · {linkResult.skipped} skipped
                {linkResult.failed.length
                  ? ` · ${linkResult.failed.length} failed`
                  : ""}
              </Text>
              {!linkResult.dryRun && linkResult.items[0] ? (
                <Text dimColor wrap="truncate">
                  → {linkResult.items[0].targetDir.replace(/[/\\][^/\\]+$/, "")}
                </Text>
              ) : null}
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
      {errorMessage ? (
        <Box marginTop={1}>
          <ErrorLine message={errorMessage} />
        </Box>
      ) : null}

      <Footer keys="↑↓ harness · tab scope · ↵ propose · y approve · p plan · r refresh · h home · q quit" />
    </Box>
  );
}

export { LINKABLE as PATHS_LINKABLE_HARNESSES };
