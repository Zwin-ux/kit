import React from "react";
import { Box, Text } from "ink";
import type {
  CodingJobMode,
  CodingRunnerStatus,
} from "@mzwin/kit-core";
import { KIT_PACKAGE_VERSION } from "@mzwin/kit-shared";
import { Spinner } from "../components/Motion.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { BlinkCursor, selectCursorGlyph } from "../motion/index.js";
import { workbenchLayoutFromTerminal } from "./workbenchLayout.js";

export interface WorkbenchServiceTask {
  plugin: string;
  displayName: string;
  task: string;
  description: string;
  status: "ready" | "review" | "missing";
}

export type WorkbenchRunStatus =
  | "idle"
  | "running"
  | "stopping"
  | "succeeded"
  | "failed"
  | "cancelled";

export interface WorkbenchProps {
  projectDir: string;
  runners: CodingRunnerStatus[];
  serviceTasks: WorkbenchServiceTask[];
  lane: "runner" | "service";
  selectedRunnerIndex: number;
  selectedTaskIndex: number;
  selectedModelIndex: number;
  mode: CodingJobMode;
  prompt: string;
  editingPrompt?: boolean;
  confirmBuild?: boolean;
  busy?: boolean;
  runStatus?: WorkbenchRunStatus;
  runLabel?: string;
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

function cleanOutput(value: string, maximum: number): string[] {
  return value
    .replace(/\u001b\[[0-?]*[ -/]*[@-~]/g, "")
    .split(/\r?\n/)
    .map((line) => line.replace(/\t/g, "  ").trimEnd())
    .filter(Boolean)
    .slice(-maximum);
}

function windowed<T>(items: T[], selected: number, maximum: number): T[] {
  if (items.length <= maximum) return items;
  const start = Math.max(
    0,
    Math.min(selected - Math.floor(maximum / 2), items.length - maximum),
  );
  return items.slice(start, start + maximum);
}

function modelSize(bytes: number): string {
  return `${(bytes / 1_000_000_000).toFixed(1)} GB`;
}

function runStatusLabel(status: WorkbenchRunStatus): string {
  switch (status) {
    case "running":
      return "RUNNING";
    case "stopping":
      return "STOPPING";
    case "succeeded":
      return "DONE";
    case "failed":
      return "FAILED";
    case "cancelled":
      return "STOPPED";
    default:
      return "IDLE";
  }
}

function controlLines(props: {
  compact: boolean;
  lane: "runner" | "service";
  ollama: boolean;
  editing: boolean;
  confirmBuild: boolean;
  runStatus: WorkbenchRunStatus;
}): string[] {
  if (props.editing) {
    return props.compact
      ? ["EDIT PROMPT", "Enter save   Esc cancel"]
      : ["EDIT PROMPT   Enter save   Esc cancel"];
  }
  if (props.confirmBuild) {
    return props.compact
      ? ["BUILD MAY EDIT FILES", "Y run   N cancel"]
      : ["BUILD MAY EDIT FILES   Y run   N cancel"];
  }
  if (props.runStatus === "running" || props.runStatus === "stopping") {
    return props.compact
      ? [runStatusLabel(props.runStatus), "Esc or X stop   Ctrl+C quit"]
      : [`${runStatusLabel(props.runStatus)}   Esc or X stop   Ctrl+C quit`];
  }
  if (props.lane === "service") {
    return props.compact
      ? ["Up/Down task   Tab runners", "Enter run   Esc home   Q quit"]
      : ["Up/Down task   Tab runners   Enter run   Esc home   Q quit"];
  }
  if (props.ollama) {
    return props.compact
      ? [
          "Up/Down runner   Left/Right model   Tab tools",
          "E prompt   M mode   Enter run   Q quit",
        ]
      : [
          "U/D runner  L/R model  Tab tools  E job  M mode  Enter run  Q quit",
        ];
  }
  return props.compact
    ? ["Up/Down runner   Tab services", "E prompt   M mode   Enter run   Q quit"]
    : ["Up/Down runner   Tab services   E job   M mode   Enter run   Esc home   Q quit"];
}

function RunnerRows(props: {
  runners: CodingRunnerStatus[];
  selectedIndex: number;
  active: boolean;
  maximum?: number;
}): React.ReactElement {
  const visible =
    props.maximum === undefined
      ? props.runners
      : windowed(props.runners, props.selectedIndex, props.maximum);
  const offset = visible.length > 0 ? props.runners.indexOf(visible[0]!) : 0;
  return (
    <Box flexDirection="column">
      {visible.map((runner, visibleIndex) => {
        const index = offset + visibleIndex;
        const selected = props.active && index === props.selectedIndex;
        const state = runner.available ? "ready" : runner.detail ?? "missing";
        return (
          <Text key={runner.id} bold={selected} wrap="truncate">
            {selectCursorGlyph(selected)}
            {runner.label.padEnd(15)} {state}
          </Text>
        );
      })}
      {props.runners.length > visible.length ? (
        <Text dimColor>
          {props.selectedIndex + 1}/{props.runners.length} runners
        </Text>
      ) : null}
    </Box>
  );
}

function ServiceRows(props: {
  tasks: WorkbenchServiceTask[];
  selectedIndex: number;
  active: boolean;
  maximum: number;
}): React.ReactElement {
  if (props.tasks.length === 0) {
    return <Text dimColor>No attached tasks.</Text>;
  }
  const visible = windowed(props.tasks, props.selectedIndex, props.maximum);
  const offset = props.tasks.indexOf(visible[0]!);
  return (
    <Box flexDirection="column">
      {visible.map((item, visibleIndex) => {
        const index = offset + visibleIndex;
        const selected = props.active && index === props.selectedIndex;
        return (
          <Text
            key={`${item.plugin}:${item.task}`}
            bold={selected}
            wrap="truncate"
          >
            {selectCursorGlyph(selected)}
            {item.displayName}/{item.task} [{item.status}]
          </Text>
        );
      })}
      {props.tasks.length > visible.length ? (
        <Text dimColor>
          {visible.length}/{props.tasks.length} tasks
        </Text>
      ) : null}
    </Box>
  );
}

export function Workbench({
  projectDir,
  runners,
  serviceTasks,
  lane,
  selectedRunnerIndex,
  selectedTaskIndex,
  selectedModelIndex,
  mode,
  prompt,
  editingPrompt,
  confirmBuild,
  busy,
  runStatus = busy ? "running" : "idle",
  runLabel,
  output,
  errorMessage,
}: WorkbenchProps): React.ReactElement {
  const terminal = useLayoutScale();
  const layout = workbenchLayoutFromTerminal(
    terminal.columns,
    terminal.rows,
  );
  const activeRunner = runners[selectedRunnerIndex];
  const activeTask = serviceTasks[selectedTaskIndex];
  const models = activeRunner?.models ?? [];
  const activeModel = models[
    Math.min(selectedModelIndex, Math.max(0, models.length - 1))
  ];
  const lines = output ? cleanOutput(output, layout.outputRows) : [];
  const compactLines = lines.slice(-Math.min(4, layout.outputRows));
  const promptText = prompt || (editingPrompt
    ? "Describe one bounded job"
    : "Press e to write a job");
  const controls = controlLines({
    compact: layout.mode === "compact",
    lane,
    ollama: activeRunner?.id === "ollama",
    editing: Boolean(editingPrompt),
    confirmBuild: Boolean(confirmBuild),
    runStatus,
  });

  const statusColor =
    runStatus === "failed"
      ? "red"
      : runStatus === "stopping"
        ? "yellow"
        : runStatus === "succeeded"
          ? "green"
          : undefined;
  const emptyOutput =
    runStatus === "running"
      ? "Waiting for output..."
      : runStatus === "stopping"
        ? "Stopping the process..."
        : "No run yet.";

  const jobPanel = (
    <Box flexDirection="column" flexGrow={1} overflow="hidden">
      {lane === "runner" ? (
        <>
          <Box justifyContent="space-between">
            <Text bold>JOB / {mode.toUpperCase()}</Text>
            <Text dimColor>
              {activeRunner?.available ? "READY" : "OFFLINE"}
            </Text>
          </Box>
          {activeRunner?.id === "ollama" ? (
            <Text wrap="truncate">
              Local model:{" "}
              <Text bold>{activeModel?.name ?? "none installed"}</Text>
              {activeModel
                ? ` · ${activeModel.parameterSize ?? modelSize(activeModel.size)}`
                : ""}
              {models.length > 1
                ? ` · ${selectedModelIndex + 1}/${models.length}`
                : ""}
            </Text>
          ) : (
            <Text dimColor wrap="truncate">
              Runner: {activeRunner?.label ?? "none"}
            </Text>
          )}
          <Box marginTop={1} flexDirection="column">
            <Text bold>PROMPT{editingPrompt ? " / EDITING" : ""}</Text>
            <Text inverse={Boolean(editingPrompt)} wrap="truncate">
              {" "}
              {promptText}
              {editingPrompt ? <BlinkCursor active /> : null}{" "}
            </Text>
            {confirmBuild ? (
              <Text color="yellow">
                Build can edit this project. Press y to run or n to cancel.
              </Text>
            ) : null}
          </Box>
        </>
      ) : (
        <>
          <Box justifyContent="space-between">
            <Text bold>TASK / READ-ONLY</Text>
            <Text dimColor>{activeTask?.status.toUpperCase() ?? "NONE"}</Text>
          </Box>
          <Text bold wrap="truncate">
            {activeTask
              ? `${activeTask.displayName} / ${activeTask.task}`
              : "No attached task"}
          </Text>
          <Text dimColor wrap="truncate">
            {activeTask?.description ?? "Attach a safe CLI task to run it here."}
          </Text>
        </>
      )}

      <Box marginTop={1} flexDirection="column" flexGrow={1} overflow="hidden">
        <Box justifyContent="space-between">
          <Text bold wrap="truncate">
            RUN / {runLabel ?? "NO RUN YET"}
          </Text>
          {runStatus === "running" || runStatus === "stopping" ? (
            <Spinner
              active={runStatus === "running"}
              style="icon"
              label={runStatus === "stopping" ? "stopping" : "working"}
            />
          ) : (
            <Text
              bold
              {...(statusColor !== undefined ? { color: statusColor } : {})}
            >
              {runStatusLabel(runStatus)}
            </Text>
          )}
        </Box>
        {lines.length === 0 ? (
          <Text dimColor>{emptyOutput}</Text>
        ) : (
          lines.map((line, index) => (
            <Text key={`${index}-${line.slice(0, 24)}`} wrap="truncate">
              {line}
            </Text>
          ))
        )}
      </Box>

      {errorMessage ? (
        <Text bold color="red" wrap="truncate">
          ERROR / {errorMessage}
        </Text>
      ) : runStatus === "cancelled" ? (
        <Text bold>STOPPED / Run ended by user.</Text>
      ) : runStatus === "succeeded" ? (
        <Text bold>DONE / Run finished.</Text>
      ) : null}
    </Box>
  );

  const compactJobPanel = (
    <Box flexDirection="column" flexGrow={1} overflow="hidden">
      {lane === "runner" ? (
        <>
          <Text bold wrap="truncate">
            JOB / {mode.toUpperCase()} /{" "}
            {activeRunner?.id === "ollama"
              ? activeModel?.name ?? "NO LOCAL MODEL"
              : activeRunner?.label ?? "NO RUNNER"}{" "}
            / {activeRunner?.available ? "READY" : "OFFLINE"}
          </Text>
          <Text inverse={Boolean(editingPrompt)} wrap="truncate">
            {" "}
            {promptText}
            {editingPrompt ? <BlinkCursor active /> : null}{" "}
          </Text>
          {confirmBuild ? (
            <Text color="yellow" wrap="truncate">
              Build can edit this project. Press y to run or n to cancel.
            </Text>
          ) : null}
        </>
      ) : (
        <>
          <Text bold wrap="truncate">
            TASK / READ-ONLY /{" "}
            {activeTask
              ? `${activeTask.displayName}/${activeTask.task}`
              : "NONE"}{" "}
            / {activeTask?.status.toUpperCase() ?? "MISSING"}
          </Text>
          <Text dimColor wrap="truncate">
            {activeTask?.description ?? "No attached task."}
          </Text>
        </>
      )}
      <Text
        bold
        wrap="truncate"
        {...(statusColor !== undefined ? { color: statusColor } : {})}
      >
        RUN / {runLabel ?? "NO RUN YET"} / {runStatusLabel(runStatus)}
      </Text>
      {compactLines.length === 0 ? (
        <Text dimColor>{emptyOutput}</Text>
      ) : (
        compactLines.map((line, index) => (
          <Text key={`${index}-${line.slice(0, 24)}`} wrap="truncate">
            {line}
          </Text>
        ))
      )}
      {errorMessage ? (
        <Text bold color="red" wrap="truncate">
          ERROR / {errorMessage}
        </Text>
      ) : runStatus === "cancelled" ? (
        <Text bold>STOPPED / Run ended by user.</Text>
      ) : runStatus === "succeeded" ? (
        <Text bold>DONE / Run finished.</Text>
      ) : null}
    </Box>
  );

  return (
    <Box
      flexDirection="column"
      width="100%"
      height={terminal.rows}
      paddingX={layout.paddingX}
      overflow="hidden"
    >
      <Box justifyContent="space-between" height={1}>
        <Text bold inverse>
          {" "}
          KIT / WORKBENCH{" "}
        </Text>
        <Text dimColor>
          v{KIT_PACKAGE_VERSION} · {terminal.columns}x{terminal.rows} ·{" "}
          {layout.mode}
        </Text>
      </Box>

      {layout.showProjectPath ? (
        <Text dimColor wrap="truncate">
          PROJECT / {shortPath(projectDir)}
        </Text>
      ) : null}

      {layout.mode === "compact" ? (
        <Box flexDirection="column" flexGrow={1} overflow="hidden">
          <Box marginTop={1} flexDirection="column">
            <Text bold inverse={lane === "runner"}>
              {" "}
              {lane === "runner" ? "RUNNERS" : "SERVICES"}{" "}
            </Text>
            {lane === "runner" ? (
              <RunnerRows
                runners={runners}
                selectedIndex={selectedRunnerIndex}
                active
                maximum={layout.runnerRows}
              />
            ) : (
              <ServiceRows
                tasks={serviceTasks}
                selectedIndex={selectedTaskIndex}
                active
                maximum={layout.serviceRows}
              />
            )}
          </Box>
          <Box marginTop={1} flexGrow={1} overflow="hidden">
            {compactJobPanel}
          </Box>
        </Box>
      ) : (
        <Box
          flexDirection="row"
          flexGrow={1}
          marginTop={1}
          overflow="hidden"
        >
          <Box
            width={layout.sidebarWidth}
            flexShrink={0}
            flexDirection="column"
            borderStyle="single"
            paddingX={1}
            overflow="hidden"
          >
            <Text bold inverse={lane === "runner"}>
              {" "}
              RUNNERS{" "}
            </Text>
            <RunnerRows
              runners={runners}
              selectedIndex={selectedRunnerIndex}
              active={lane === "runner"}
            />
            <Box marginTop={1} flexDirection="column">
              <Text bold inverse={lane === "service"}>
                {" "}
                SERVICES{" "}
              </Text>
              <ServiceRows
                tasks={serviceTasks}
                selectedIndex={selectedTaskIndex}
                active={lane === "service"}
                maximum={layout.serviceRows}
              />
            </Box>
          </Box>
          <Box
            marginLeft={1}
            flexGrow={1}
            flexDirection="column"
            borderStyle="single"
            paddingX={1}
            overflow="hidden"
          >
            {jobPanel}
          </Box>
        </Box>
      )}

      <Box flexDirection="column">
        {controls.map((line) => (
          <Text key={line} bold inverse wrap="truncate">
            {" "}
            {line}{" "}
          </Text>
        ))}
      </Box>
    </Box>
  );
}
