import React from "react";
import { Box, Text } from "ink";
import type {
  CodingJobMode,
  CodingRunnerStatus,
  PackListItem,
  ToolkitRecommendation,
} from "@mzwin/kit-core";
import { KIT_PACKAGE_VERSION } from "@mzwin/kit-shared";
import { Spinner } from "../components/Motion.js";
import { MenuButton } from "../components/MenuButton.js";
import {
  laneMenuIcon,
  opsMenuIcon,
  runnerMenuIcon,
} from "../mascot/menuIcons.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { ActionFlash, BlinkCursor } from "../motion/index.js";
import { theme } from "../theme.js";
import {
  windowSlice,
  workbenchGeometry,
} from "./workbenchGeometry.js";

/** Product lanes */
export type TerminalLane = "skills" | "agents" | "services" | "ops";

export const TERMINAL_LANES: TerminalLane[] = [
  "skills",
  "agents",
  "services",
  "ops",
];

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

export interface OpsAction {
  id: string;
  label: string;
  detail: string;
  hint: string;
}

export const DEFAULT_OPS_ACTIONS: OpsAction[] = [
  {
    id: "ready",
    label: "Quickstart",
    detail: "Install pack. Apply to this repo. Link agents.",
    hint: "setup for agentic dev",
  },
  {
    id: "unify",
    label: "Unify",
    detail: "Scan skill folders. Keep the best skills.",
    hint: "clean the skill pile",
  },
  {
    id: "doctor",
    label: "Doctor",
    detail: "Check the library and the agent links.",
    hint: "check health",
  },
  {
    id: "paths",
    label: "Link agents",
    detail: "Write skill links for this project.",
    hint: "open paths",
  },
  {
    id: "refresh",
    label: "Refresh",
    detail: "Reload runners, packs, and status.",
    hint: "reload status",
  },
];

export interface WorkbenchProps {
  projectDir: string;
  lane: TerminalLane;
  packs: PackListItem[];
  selectedPackIndex: number;
  recommended?: ToolkitRecommendation[];
  appliedNames?: Set<string>;
  runners: CodingRunnerStatus[];
  selectedRunnerIndex: number;
  selectedModelIndex: number;
  mode: CodingJobMode;
  serviceTasks: WorkbenchServiceTask[];
  selectedTaskIndex: number;
  opsActions?: OpsAction[];
  selectedOpsIndex: number;
  prompt: string;
  editingPrompt?: boolean;
  inputMode?: "prompt" | "pull" | "point";
  confirmBuild?: boolean;
  busy?: boolean;
  runStatus?: WorkbenchRunStatus;
  runLabel?: string;
  output?: string;
  errorMessage?: string;
  outputScroll?: number;
  storyTitle?: string;
  opsConfirm?: "none" | "ready-write" | "unify-write";
  historyMode?: boolean;
  recentRuns?: Array<{
    id: string;
    label: string;
    status: string;
    createdAt: string;
  }>;
  selectedRunIndex?: number;
  actionFlash?: string;
  actionNonce?: number;
  actionIsError?: boolean;
  pressedId?: string;
  setupLine?: string;
  nextLine?: string;
  skillCount?: number;
}

function shortPath(value: string): string {
  const home = process.env.USERPROFILE ?? process.env.HOME ?? "";
  const normalized = value.replace(/\\/g, "/");
  return home && value.startsWith(home)
    ? `~${value.slice(home.length).replace(/\\/g, "/")}`
    : normalized;
}

function cleanLines(value: string, max: number, scroll = 0): string[] {
  const all = value
    .replace(/\u001b\[[0-?]*[ -/]*[@-~]/g, "")
    .split(/\r?\n/)
    .map((l) => l.replace(/\t/g, "  ").trimEnd())
    .filter(Boolean);
  const maxScroll = Math.max(0, all.length - max);
  const offset = Math.min(Math.max(0, scroll), maxScroll);
  const end = all.length - offset;
  return all.slice(Math.max(0, end - max), end);
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

function laneLabel(lane: TerminalLane): string {
  switch (lane) {
    case "skills":
      return "PACKS";
    case "agents":
      return "AGENTS";
    case "services":
      return "TOOLS";
    case "ops":
      return "SETUP";
  }
}

/**
 * Kit main UI — simple full-width list.
 * Geometry is shared with mouse hit-testing (workbenchGeometry).
 * No Ollama status chrome. Clicks map to absolute list indices.
 */
export function Workbench({
  projectDir,
  lane,
  packs,
  selectedPackIndex,
  recommended = [],
  appliedNames,
  runners,
  selectedRunnerIndex,
  selectedModelIndex,
  mode,
  serviceTasks,
  selectedTaskIndex,
  opsActions = DEFAULT_OPS_ACTIONS,
  selectedOpsIndex,
  prompt,
  editingPrompt,
  inputMode = "prompt",
  confirmBuild,
  busy,
  runStatus = busy ? "running" : "idle",
  runLabel,
  output,
  errorMessage,
  outputScroll = 0,
  opsConfirm = "none",
  historyMode = false,
  recentRuns = [],
  selectedRunIndex = 0,
  actionFlash,
  actionNonce = 0,
  actionIsError = false,
  pressedId,
  setupLine,
  nextLine,
  skillCount,
}: WorkbenchProps): React.ReactElement {
  const terminal = useLayoutScale();
  const geo = workbenchGeometry(terminal.columns, terminal.rows);
  const topPack = recommended[0]?.packName;
  const activeRunner = runners[selectedRunnerIndex];
  const models = activeRunner?.models ?? [];
  const activeModel = models[
    Math.min(selectedModelIndex, Math.max(0, models.length - 1))
  ];
  const activePack = packs[selectedPackIndex];
  const activeOps = opsActions[selectedOpsIndex];

  // --- build absolute-indexed list ---
  type Row = {
    absIndex: number;
    icon: string;
    label: string;
    meta: string;
    selected: boolean;
    pressed: boolean;
  };

  let listRows: Row[] = [];
  let listTitle = "ITEMS";
  let listEmpty = "Nothing here.";

  if (lane === "skills") {
    listTitle = "PACKS";
    listEmpty = "No packs found.";
    const { items, offset } = windowSlice(
      packs,
      selectedPackIndex,
      geo.listRows,
    );
    listRows = items.map((pack, v) => {
      const abs = offset + v;
      const selected = abs === selectedPackIndex;
      const star = pack.name === topPack ? "*" : "";
      const on = appliedNames?.has(pack.name) ? "on" : "";
      return {
        absIndex: abs,
        icon: "pack",
        label: pack.title,
        meta: `${pack.skillCount} skills${star ? " *" : ""}${on ? " on" : ""}`,
        selected,
        pressed:
          pressedId === `pack:${pack.name}` ||
          pressedId === `term-pack:${abs}`,
      };
    });
  } else if (lane === "agents") {
    if (historyMode) {
      listTitle = "HISTORY";
      listEmpty = "No saved proofs yet.";
      const { items, offset } = windowSlice(
        recentRuns,
        selectedRunIndex,
        geo.listRows,
      );
      listRows = items.map((run, v) => {
        const abs = offset + v;
        return {
          absIndex: abs,
          icon: "kit",
          label: run.label,
          meta: `${run.status} · ${run.createdAt.slice(0, 16).replace("T", " ")}`,
          selected: abs === selectedRunIndex,
          pressed: pressedId === `run:${run.id}`,
        };
      });
    } else {
      listTitle = "AGENTS";
      listEmpty = "No agents detected.";
      const { items, offset } = windowSlice(
        runners,
        selectedRunnerIndex,
        geo.listRows,
      );
      listRows = items.map((runner, v) => {
        const abs = offset + v;
        return {
          absIndex: abs,
          icon: runnerMenuIcon(runner.id),
          label: runner.label,
          meta: runner.available
            ? "ready"
            : (runner.detail ?? "missing").slice(0, 18),
          selected: abs === selectedRunnerIndex,
          pressed: pressedId === `runner:${runner.id}`,
        };
      });
    }
  } else if (lane === "services") {
    listTitle = "TOOLS";
    listEmpty = "No tools. Add a plugin.";
    const { items, offset } = windowSlice(
      serviceTasks,
      selectedTaskIndex,
      geo.listRows,
    );
    listRows = items.map((task, v) => {
      const abs = offset + v;
      return {
        absIndex: abs,
        icon: "plugin",
        label: `${task.displayName}/${task.task}`,
        meta: task.status,
        selected: abs === selectedTaskIndex,
        pressed: pressedId === `task:${task.plugin}:${task.task}`,
      };
    });
  } else {
    listTitle = "SETUP";
    listEmpty = "No actions.";
    const { items, offset } = windowSlice(
      opsActions,
      selectedOpsIndex,
      geo.listRows,
    );
    listRows = items.map((op, v) => {
      const abs = offset + v;
      return {
        absIndex: abs,
        icon: opsMenuIcon(op.id),
        label: op.label,
        meta: op.hint,
        selected: abs === selectedOpsIndex,
        pressed:
          pressedId === `ops:${op.id}` || pressedId === `bar:${op.id}`,
      };
    });
  }

  const logLines = output
    ? cleanLines(output, geo.logRows, outputScroll)
    : [];

  const promptText =
    prompt ||
    (inputMode === "pull"
      ? "model name e.g. llama3.2"
      : editingPrompt
        ? "Describe one job"
        : lane === "agents"
          ? "Press e to write a job"
          : lane === "skills"
            ? "Enter installs the focused pack"
            : lane === "ops"
              ? "Enter runs Quickstart plan"
              : "Enter runs the focused item");

  const context =
    lane === "skills" && activePack
      ? `${activePack.title} · Enter install`
      : lane === "agents" && !historyMode
        ? `${activeRunner?.label ?? "—"} · ${mode}${
            activeModel ? ` · ${activeModel.name}` : ""
          }`
        : lane === "ops" && activeOps
          ? `${activeOps.label} · ${activeOps.detail}`
          : shortPath(projectDir);

  const statusColor =
    runStatus === "failed"
      ? theme.error
      : runStatus === "succeeded"
        ? theme.success
        : runStatus === "stopping"
          ? theme.warning
          : runStatus === "running"
            ? theme.accent
            : undefined;

  // Fixed row skeleton matching geo (no dual columns)
  return (
    <Box
      flexDirection="column"
      width="100%"
      height={terminal.rows}
      paddingX={geo.paddingX}
      overflow="hidden"
    >
      {/* row 1 — brand title (ink console, no size debug) */}
      <Box justifyContent="space-between" height={1}>
        <Text>
          <Text bold inverse>
            {" "}
            KIT{" "}
          </Text>
          <Text bold color={theme.accent}>
            {" "}
            {laneLabel(lane)}
          </Text>
        </Text>
        <Text dimColor>v{KIT_PACKAGE_VERSION}</Text>
      </Box>

      {/* row 2 — tabs */}
      <Box height={1}>
        {TERMINAL_LANES.map((id, i) => (
          <MenuButton
            key={id}
            icon={laneMenuIcon(id)}
            label={laneLabel(id)}
            hotkey={String(i + 1)}
            selected={id === lane}
            pressed={pressedId === `lane:${id}`}
            variant="chip"
          />
        ))}
      </Box>

      {/* row 3 — status / next */}
      <Text wrap="truncate">
        {nextLine ? (
          <>
            <Text bold color={theme.accent}>
              →{" "}
            </Text>
            <Text bold>{nextLine}</Text>
          </>
        ) : (
          <Text dimColor>{shortPath(projectDir)}</Text>
        )}
      </Text>

      {/* row 4 — flash */}
      <ActionFlash
        message={actionFlash}
        nonce={actionNonce}
        isError={actionIsError}
      />

      {/* list title — one row only (geometry/hits depend on fixed stack) */}
      <Text bold color={theme.accent} wrap="truncate">
        {listTitle}
        {skillCount !== undefined ? ` · ${skillCount} skills` : ""}
        {setupLine ? ` · ${setupLine}` : ""}
      </Text>

      {/* list — one row per item, full width */}
      <Box flexDirection="column" height={geo.listRows} overflow="hidden">
        {listRows.length === 0 ? (
          <Text dimColor>{listEmpty}</Text>
        ) : (
          listRows.map((row) => (
            <MenuButton
              key={`${row.absIndex}-${row.label}`}
              icon={row.icon}
              label={row.label}
              meta={row.meta}
              selected={row.selected}
              pressed={row.pressed}
              variant="list"
            />
          ))
        )}
      </Box>

      {/* log */}
      <Box flexDirection="column" height={geo.logRows + 1} overflow="hidden">
        <Box justifyContent="space-between">
          <Text bold>
            LOG{runLabel ? ` · ${runLabel}` : ""}
          </Text>
          {runStatus === "running" || runStatus === "stopping" ? (
            <Spinner
              active={runStatus === "running"}
              style="icon"
              label={runStatus === "stopping" ? "stop" : "run"}
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
        {logLines.length === 0 ? (
          <Text dimColor>
            {lane === "ops"
              ? "Select Quickstart. Press Enter. Press y to write."
              : lane === "skills"
                ? "Click a pack. Press Enter to install."
                : "Output appears here."}
          </Text>
        ) : (
          logLines.map((line, i) => (
            <Text key={`${i}-${line.slice(0, 16)}`} wrap="truncate">
              {line}
            </Text>
          ))
        )}
        {errorMessage ? (
          <Text bold color="red" wrap="truncate">
            ! {errorMessage}
          </Text>
        ) : null}
      </Box>

      {/* context + prompt */}
      <Text dimColor wrap="truncate">
        {context}
      </Text>
      {confirmBuild ? (
        <Text color="yellow">Build can edit files. y run · n cancel</Text>
      ) : null}
      {opsConfirm !== "none" ? (
        <Text color="yellow">
          Write plan to disk? y write · n cancel
        </Text>
      ) : null}
      {lane === "agents" && !historyMode ? (
        <Text inverse={Boolean(editingPrompt) || inputMode === "pull"} wrap="truncate">
          {" "}
          {inputMode === "pull" ? "pull> " : "> "}
          {promptText}
          {editingPrompt || inputMode === "pull" ? (
            <BlinkCursor active />
          ) : null}{" "}
        </Text>
      ) : null}

      {/* action bar — full width buttons */}
      <Box flexDirection="row" height={1}>
        {opsConfirm !== "none" ? (
          <>
            <MenuButton
              icon="ready"
              label="Write"
              hotkey="y"
              variant="bar"
              primary
              pressed={pressedId === "bar:write"}
            />
            <MenuButton
              icon="stop"
              label="Cancel"
              hotkey="n"
              variant="bar"
              pressed={pressedId === "bar:cancel"}
            />
          </>
        ) : lane === "skills" ? (
          <>
            <MenuButton
              icon="install"
              label="Install"
              hotkey="Enter"
              variant="bar"
              primary
              pressed={pressedId === "bar:install"}
            />
            <MenuButton
              icon="apply"
              label="Apply"
              hotkey="a"
              variant="bar"
              pressed={pressedId === "bar:apply"}
            />
          </>
        ) : lane === "agents" && !historyMode ? (
          <>
            <MenuButton
              icon="run"
              label="Run"
              hotkey="Enter"
              variant="bar"
              primary
              pressed={pressedId === "bar:run"}
            />
            <MenuButton
              icon="model"
              label="Pull model"
              hotkey="p"
              variant="bar"
              pressed={pressedId === "bar:pull"}
            />
            <MenuButton
              icon="kit"
              label="History"
              hotkey="H"
              variant="bar"
              pressed={pressedId === "bar:history"}
            />
          </>
        ) : lane === "agents" && historyMode ? (
          <MenuButton
            icon="kit"
            label="Open proof"
            hotkey="Enter"
            variant="bar"
            primary
            pressed={pressedId === "bar:open-run"}
          />
        ) : lane === "services" ? (
          <MenuButton
            icon="run"
            label="Run tool"
            hotkey="Enter"
            variant="bar"
            primary
            pressed={pressedId === "bar:run-service"}
          />
        ) : (
          <MenuButton
            icon="run"
            label="Run setup"
            hotkey="Enter"
            variant="bar"
            primary
            pressed={pressedId === "bar:run-ops"}
          />
        )}
      </Box>

      {/* footer keys */}
      <Text bold inverse wrap="truncate">
        {" "}
        {opsConfirm !== "none"
          ? "y write · n cancel"
          : editingPrompt || inputMode === "pull"
            ? "EDIT · Enter save · Esc cancel"
            : lane === "skills"
              ? "Click pack · Enter install · a apply · 1-4 lane · q quit"
              : lane === "ops"
                ? "Click action · Enter plan · y write · 1-4 lane · q quit"
                : "Click row · Enter run · 1-4 lane · ? help · q quit"}{" "}
      </Text>
    </Box>
  );
}

/** Exported for hit-map builders in App. */
export { workbenchGeometry, windowSlice } from "./workbenchGeometry.js";
