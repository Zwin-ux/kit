import React from "react";
import { Box, Text } from "ink";
import type {
  CodingJobMode,
  CodingRunnerStatus,
} from "@mzwin/kit-core";
import type { MascotVariant, PixelFrame } from "../mascot/types.js";
import { Footer, Header } from "../components/Chrome.js";
import { ScreenShell } from "../components/ScreenShell.js";
import { ErrorLine, Spinner, SuccessLine } from "../components/Motion.js";
import { BlinkCursor, selectCursorGlyph } from "../motion/index.js";

export interface WorkbenchServiceTask {
  plugin: string;
  displayName: string;
  task: string;
  description: string;
  status: "ready" | "review" | "missing";
}

export interface WorkbenchProps {
  frames: PixelFrame[];
  mascotVariant?: MascotVariant;
  projectDir: string;
  runners: CodingRunnerStatus[];
  serviceTasks: WorkbenchServiceTask[];
  lane: "runner" | "service";
  selectedRunnerIndex: number;
  selectedTaskIndex: number;
  mode: CodingJobMode;
  prompt: string;
  editingPrompt?: boolean;
  confirmBuild?: boolean;
  busy?: boolean;
  output?: string;
  errorMessage?: string;
}

function shortPath(value: string): string {
  const home = process.env.USERPROFILE ?? process.env.HOME ?? "";
  const normalized = value.replace(/\\/g, "/");
  return home && value.startsWith(home)
    ? `~${value.slice(home.length).replace(/\\/g, "/")}`
    : normalized;
}

function outputLines(value: string): string[] {
  return value
    .split(/\r?\n/)
    .map((line) => line.trimEnd())
    .filter(Boolean)
    .slice(-5);
}

export function Workbench({
  frames,
  mascotVariant = "idle",
  projectDir,
  runners,
  serviceTasks,
  lane,
  selectedRunnerIndex,
  selectedTaskIndex,
  mode,
  prompt,
  editingPrompt,
  confirmBuild,
  busy,
  output,
  errorMessage,
}: WorkbenchProps): React.ReactElement {
  const activeRunner = runners[selectedRunnerIndex];
  const activeTask = serviceTasks[selectedTaskIndex];
  const lines = output ? outputLines(output) : [];

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1} width="100%">
      <Header
        screen="Workbench"
        detail={busy ? "running..." : `${mode} · local`}
      />

      <Box marginTop={1} width="100%">
        <ScreenShell frames={frames} mascotVariant={mascotVariant}>
          <Text bold>Project</Text>
          <Text dimColor wrap="truncate">
            {shortPath(projectDir)}
          </Text>

          <Box marginTop={1} flexDirection="column">
            <Text bold inverse={lane === "runner"}>
              {" "}
              RUNNERS{" "}
            </Text>
            {runners.map((runner, index) => {
              const selected =
                lane === "runner" && index === selectedRunnerIndex;
              return (
                <Text key={runner.id} bold={selected} wrap="truncate">
                  {selectCursorGlyph(selected)}
                  {runner.label.padEnd(12)}{" "}
                  {runner.available ? "ready" : "missing"}
                </Text>
              );
            })}
          </Box>

          <Box marginTop={1} flexDirection="column">
            <Text bold>Job · {mode}</Text>
            <Text dimColor wrap="truncate">
              {editingPrompt
                ? prompt || "Describe one bounded job"
                : prompt || "e to write a job"}
              {editingPrompt ? <BlinkCursor active /> : null}
            </Text>
            {confirmBuild ? (
              <Text color="yellow">
                Build can edit this project. Press y to run or n to cancel.
              </Text>
            ) : null}
          </Box>

          <Box marginTop={1} flexDirection="column">
            <Text bold inverse={lane === "service"}>
              {" "}
              SERVICES{" "}
            </Text>
            {serviceTasks.length === 0 ? (
              <Text dimColor>No fixed service tasks.</Text>
            ) : (
              serviceTasks.map((item, index) => {
                const selected =
                  lane === "service" && index === selectedTaskIndex;
                return (
                  <Text
                    key={`${item.plugin}:${item.task}`}
                    bold={selected}
                    wrap="truncate"
                  >
                    {selectCursorGlyph(selected)}
                    {item.displayName}/{item.task} [{item.status}] ·{" "}
                    {item.description}
                  </Text>
                );
              })
            )}
          </Box>

          {busy ? (
            <Box marginTop={1}>
              <Spinner
                active
                style="icon"
                label={
                  lane === "runner"
                    ? `Running ${activeRunner?.label ?? "job"}`
                    : `Running ${activeTask?.displayName ?? "service"}`
                }
              />
            </Box>
          ) : null}

          {lines.length > 0 ? (
            <Box marginTop={1} flexDirection="column">
              <Text bold>Last output</Text>
              {lines.map((line, index) => (
                <Text key={`${index}-${line.slice(0, 20)}`} dimColor wrap="truncate">
                  {line}
                </Text>
              ))}
            </Box>
          ) : null}
        </ScreenShell>
      </Box>

      {errorMessage ? (
        <Box marginTop={1}>
          <ErrorLine message={errorMessage} />
        </Box>
      ) : output && !busy ? (
        <Box marginTop={1}>
          <SuccessLine message="Run finished" />
        </Box>
      ) : null}

      <Footer
        keys={
          lane === "runner"
            ? "tab services · up/down runner · e job · m mode · enter run · h home · q quit"
            : "tab runners · up/down task · enter run · h home · q quit"
        }
      />
    </Box>
  );
}
