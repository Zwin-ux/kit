import React, { useCallback, useEffect, useRef, useState } from "react";
import { useInput, useApp, Box, Text } from "ink";
import path from "node:path";
import {
  completeFirstRun,
  detectCodingRunners,
  detectSituation,
  doctorPlugin,
  describePaths,
  getFirstRunStatus,
  getLoggedInUser,
  getRegistryUrl,
  installPack,
  applyPack,
  linkSkills,
  listPacks,
  listPlugins,
  listSkills,
  loadSkill,
  readConfig,
  readProjectAppliedPacks,
  removeSkill,
  recommendToolkits,
  runDoctor,
  runCodingJob,
  runPluginTask,
  runReady,
  runStatus,
  runUnify,
  probeOllamaService,
  startOllamaServe,
  stopOllamaServe,
  pullOllamaModel,
  saveRun,
  listRuns,
  loadRun,
  testSkill,
  updateConfig,
  exploreListPacks,
  exploreSearch,
  type AppliedPackRecord,
  type CheckResult,
  type CodingJobMode,
  type CodingRunnerStatus,
  type DoctorReport,
  type HarnessId,
  type InstalledSkill,
  type LinkResult,
  type PackListItem,
  type PathReport,
  type PathScope,
  type SkillRecommendation,
  type ToolkitRecommendation,
  type RegistryPackSummary,
  type RegisteredPlugin,
  type OllamaServiceReport,
  type SavedRunSummary,
  type UserStory,
} from "@mzwin/kit-core";
import {
  formatReadyStatus,
  formatReadySteps,
  formatUnifyPreview,
  formatUnifyStatus,
} from "./product/formatReports.js";
import { loadAllMascotFrames } from "./mascot/loadFrames.js";
import type { MascotVariant, PixelFrame } from "./mascot/types.js";
import { useLayoutScale } from "./mascot/useLayoutScale.js";
import { useMouseClick } from "./mouse/useMouse.js";
import type { HitRegion } from "./mouse/HitMap.js";
import { shouldQuitWithQ } from "./input/quitShortcut.js";
import { Spinner } from "./components/Motion.js";
import { Splash } from "./screens/Splash.js";
import { mascotVisible, splashEnabled } from "./brand/mascotPolicy.js";
import { fillWorkbenchHits } from "./screens/workbenchHits.js";
import {
  windowSlice,
  workbenchGeometry,
} from "./screens/workbenchGeometry.js";
import { Home, type HomeConfirm } from "./screens/Home.js";
import { FirstRun } from "./screens/FirstRun.js";
import { Library } from "./screens/Library.js";
import { Packs } from "./screens/Packs.js";
import { Explore } from "./screens/Explore.js";
import { Doctor } from "./screens/Doctor.js";
import { Help } from "./screens/Help.js";
import {
  Workbench,
  TERMINAL_LANES,
  type TerminalLane,
  type WorkbenchRunStatus,
  type WorkbenchServiceTask,
} from "./screens/Workbench.js";
import {
  Setup,
  defaultSetupSteps,
  type SetupPhase,
  type SetupStep,
} from "./screens/Setup.js";
import {
  Paths,
  PATHS_LINKABLE_HARNESSES,
} from "./screens/Paths.js";

type Screen =
  | "loading"
  | "splash"
  | "first-run"
  | "setup"
  | "home"
  | "library"
  | "packs"
  | "explore"
  | "doctor"
  | "paths"
  | "workbench"
  | "help";

const FIRST_RUN_BY_KEY: Record<string, string> = {
  "1": "essentials",
  "2": "web-app",
  "3": "library",
  "4": "cli-tool",
  "5": "api-service",
  "6": "full-stack",
  "7": "data-ml",
};

export interface AppProps {
  /**
   * setup = default developer quickstart (recommended).
   * workbench = advanced multi-lane menu.
   * home = classic home.
   */
  initialScreen?: "setup" | "workbench" | "home";
}

export function App({ initialScreen }: AppProps = {}): React.ReactElement {
  const { exit } = useApp();
  const scale = useLayoutScale();
  const [screen, setScreen] = useState<Screen>("loading");
  const [agentStatusLine, setAgentStatusLine] = useState<string | undefined>();
  const [frames, setFrames] = useState<PixelFrame[]>([]);
  const [scanFrames, setScanFrames] = useState<PixelFrame[]>([]);
  const [successFrames, setSuccessFrames] = useState<PixelFrame[]>([]);
  const [skills, setSkills] = useState<InstalledSkill[]>([]);
  const [packs, setPacks] = useState<PackListItem[]>([]);
  const [applied, setApplied] = useState<AppliedPackRecord[]>([]);
  const [recommended, setRecommended] = useState<ToolkitRecommendation[]>([]);
  const [skillRecs, setSkillRecs] = useState<SkillRecommendation[]>([]);
  const [topPick, setTopPick] = useState<string | null>(null);
  const [recommendSummary, setRecommendSummary] = useState<string | undefined>();
  /** Project Kit is pointed at for auto-recommend / apply / link. */
  const [targetProject, setTargetProject] = useState(process.cwd());
  const [pointingProject, setPointingProject] = useState(false);
  const [pointDraft, setPointDraft] = useState("");
  const [remotePacks, setRemotePacks] = useState<RegistryPackSummary[]>([]);
  const [exploreLoading, setExploreLoading] = useState(false);
  const [exploreQuery, setExploreQuery] = useState("");
  const [packFilter, setPackFilter] = useState("");
  const [filteringPacks, setFilteringPacks] = useState(false);
  const [selectedPackIndex, setSelectedPackIndex] = useState(0);
  const [selectedSkillIndex, setSelectedSkillIndex] = useState(0);
  const [selectedRemoteIndex, setSelectedRemoteIndex] = useState(0);
  const [selectedHarnessIndex, setSelectedHarnessIndex] = useState(0);
  const [selectTick, setSelectTick] = useState(0);
  const [selectDirection, setSelectDirection] = useState<
    "up" | "down" | "none"
  >("none");
  const [confirmRemove, setConfirmRemove] = useState(false);
  const [libraryError, setLibraryError] = useState<string | undefined>();
  const [packsError, setPacksError] = useState<string | undefined>();
  const [loadError, setLoadError] = useState<string | undefined>();
  const [statusMessage, setStatusMessage] = useState<string | undefined>();
  const [errorMessage, setErrorMessage] = useState<string | undefined>();
  const [celebrateCount, setCelebrateCount] = useState<number | undefined>();
  const [actionFlash, setActionFlash] = useState<string | undefined>();
  const [actionNonce, setActionNonce] = useState(0);
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<
    { current: number; total: number; skillName: string } | undefined
  >();
  const [offerFirstRun, setOfferFirstRun] = useState(false);
  const [userLogin, setUserLogin] = useState<string | undefined>();
  const [doctorReport, setDoctorReport] = useState<DoctorReport | undefined>();
  const [doctorLoading, setDoctorLoading] = useState(false);
  const [doctorSummary, setDoctorSummary] = useState<string | undefined>();
  const [pathReport, setPathReport] = useState<PathReport | undefined>();
  const [pathLoading, setPathLoading] = useState(false);
  const [linking, setLinking] = useState(false);
  const [linkResult, setLinkResult] = useState<LinkResult | undefined>();
  const [pathScope, setPathScope] = useState<PathScope>("project");
  const [confirmLinkWrite, setConfirmLinkWrite] = useState(false);
  const [lastChecks, setLastChecks] = useState<CheckResult[] | undefined>();
  const [codingRunners, setCodingRunners] = useState<CodingRunnerStatus[]>([]);
  const [workbenchPlugins, setWorkbenchPlugins] = useState<RegisteredPlugin[]>([]);
  const [workbenchPluginStatus, setWorkbenchPluginStatus] = useState<
    Record<string, "ready" | "review" | "missing">
  >({});
  const [workbenchLane, setWorkbenchLane] = useState<TerminalLane>("skills");
  const [selectedRunnerIndex, setSelectedRunnerIndex] = useState(0);
  const [selectedModelIndex, setSelectedModelIndex] = useState(0);
  const [selectedTaskIndex, setSelectedTaskIndex] = useState(0);
  const [selectedOpsIndex, setSelectedOpsIndex] = useState(0);
  const [jobMode, setJobMode] = useState<CodingJobMode>("inspect");
  const [jobPrompt, setJobPrompt] = useState("");
  const [editingJobPrompt, setEditingJobPrompt] = useState(false);
  const [terminalInputMode, setTerminalInputMode] = useState<
    "prompt" | "pull" | "point"
  >("prompt");
  const [confirmBuildJob, setConfirmBuildJob] = useState(false);
  const [workbenchOutput, setWorkbenchOutput] = useState<string | undefined>();
  const [workbenchError, setWorkbenchError] = useState<string | undefined>();
  const [workbenchRunStatus, setWorkbenchRunStatus] =
    useState<WorkbenchRunStatus>("idle");
  const [workbenchRunLabel, setWorkbenchRunLabel] =
    useState<string | undefined>();
  const [workbenchOutputScroll, setWorkbenchOutputScroll] = useState(0);
  const [ollamaService, setOllamaService] = useState<
    OllamaServiceReport | undefined
  >();
  const [story, setStory] = useState<UserStory | undefined>();
  const [homeConfirm, setHomeConfirm] = useState<HomeConfirm>("none");
  /** Ops write confirm stays inside the Action Terminal. */
  const [terminalOpsConfirm, setTerminalOpsConfirm] = useState<
    "none" | "ready-write" | "unify-write"
  >("none");
  const [planLines, setPlanLines] = useState<string[] | undefined>();
  const [helpFrom, setHelpFrom] = useState<Screen>("home");
  const [recentRuns, setRecentRuns] = useState<SavedRunSummary[]>([]);
  const [historyMode, setHistoryMode] = useState(false);
  const [selectedRunIndex, setSelectedRunIndex] = useState(0);
  const [pressedId, setPressedId] = useState<string | undefined>();
  const pressTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const workbenchAbort = useRef<AbortController | null>(null);
  /** Default Setup wizard state (simple path). */
  const [setupPhase, setSetupPhase] = useState<SetupPhase>("ready");
  const [setupSteps, setSetupSteps] = useState<SetupStep[]>(() =>
    defaultSetupSteps("Essentials"),
  );
  const [setupLog, setSetupLog] = useState<string[]>([]);
  const [setupPackName, setSetupPackName] = useState("essentials");
  const [setupCompleteNote, setSetupCompleteNote] = useState<
    string | undefined
  >();

  const appendSetupLog = useCallback((line: string) => {
    setSetupLog((prev) => [...prev, line].slice(-48));
  }, []);

  const appendTerminalLog = useCallback((line: string) => {
    setWorkbenchOutput((prev) => {
      const next = prev ? `${prev}\n${line}` : line;
      return next.slice(-64 * 1024);
    });
  }, []);

  const refreshRecentRuns = useCallback(async () => {
    const listed = await listRuns({ limit: 20 });
    if (listed.ok) {
      setRecentRuns(listed.value);
      setSelectedRunIndex((i) =>
        listed.value.length === 0
          ? 0
          : Math.min(i, listed.value.length - 1),
      );
    }
  }, []);

  const persistProof = useCallback(
    async (input: Parameters<typeof saveRun>[0]) => {
      const saved = await saveRun(input);
      if (saved.ok) {
        appendTerminalLog(`saved ${saved.value.logPath}`);
        await refreshRecentRuns();
      } else {
        appendTerminalLog(`! save proof: ${saved.error}`);
      }
    },
    [appendTerminalLog, refreshRecentRuns],
  );

  const clearFlash = useCallback(() => {
    setErrorMessage(undefined);
    setCelebrateCount(undefined);
  }, []);

  const flash = useCallback((message: string) => {
    setActionNonce((n) => n + 1);
    setActionFlash(message);
  }, []);

  /** Brief ★ press state on menu buttons (game-engine feel). */
  const markPress = useCallback((id: string) => {
    setPressedId(id);
    if (pressTimer.current) clearTimeout(pressTimer.current);
    pressTimer.current = setTimeout(() => setPressedId(undefined), 180);
  }, []);

  /**
   * Every control must leave a visible effect: flash + LOG line.
   * Developers use the log as the audit trail.
   */
  const feedback = useCallback(
    (message: string, options?: { pressId?: string; log?: boolean }) => {
      flash(message);
      if (options?.pressId) markPress(options.pressId);
      if (options?.log !== false) {
        appendTerminalLog(`▸ ${message}`);
      }
    },
    [flash, markPress, appendTerminalLog],
  );

  const openSavedRun = useCallback(
    async (id: string) => {
      const loaded = await loadRun(id);
      if (!loaded.ok) {
        appendTerminalLog(`! ${loaded.error}`);
        setWorkbenchError(loaded.error);
        return;
      }
      setWorkbenchOutput(loaded.value.transcript || "(empty log)");
      setWorkbenchRunLabel(loaded.value.label);
      setWorkbenchRunStatus(
        loaded.value.status === "succeeded"
          ? "succeeded"
          : loaded.value.status === "cancelled"
            ? "cancelled"
            : "failed",
      );
      setWorkbenchError(loaded.value.error);
      appendTerminalLog(`opened ${loaded.value.logPath}`);
      flash("opened run");
    },
    [appendTerminalLog, flash],
  );

  const bumpSelect = useCallback((direction: "up" | "down" | "none" = "none") => {
    setSelectDirection(direction);
    setSelectTick((t) => t + 1);
  }, []);

  /** Double-click tracking for game-menu activate. */
  const lastClickRef = useRef<{ id: string; at: number } | null>(null);

  /**
   * Late-bound actions for mouse (defined later in the component).
   * Avoids temporal-dead-zone on install/run handlers.
   */
  const mouseCtxRef = useRef<{
    screen: Screen;
    packs: PackListItem[];
    skillsLen: number;
    remoteLen: number;
    selectedPackIndex: number;
    selectedTaskIndex: number;
    selectedOpsIndex: number;
    runnersLen: number;
    tasksLen: number;
    bumpSelect: (d?: "up" | "down" | "none") => void;
    flash: (m: string) => void;
    feedback: (
      m: string,
      o?: { pressId?: string; log?: boolean },
    ) => void;
    installPack: (name: string, mode: "install" | "apply") => void;
    runJob: (confirmed: boolean) => void;
    runService: () => void;
    runOps: (id: string) => void;
    startOllama: () => void;
    openHelp: () => void;
    openHistory?: () => void;
    runSetup?: (write: boolean) => void;
    cycleSetupPack?: () => void;
  } | null>(null);

  /** Click buttons and list rows (mouse). Keyboard still works. */
  const onMouseClick = useCallback((region: HitRegion) => {
    const c = mouseCtxRef.current;
    if (!c) return;
    const action =
      typeof region.data?.action === "string" ? region.data.action : undefined;
    const idx =
      typeof region.data?.index === "number" ? region.data.index : undefined;

    if (c.screen === "setup") {
      if (action === "setup-plan") {
        c.feedback?.("setup plan", { pressId: "setup-primary" });
        c.runSetup?.(false);
        return;
      }
      if (action === "setup-write") {
        c.feedback?.("setup write", { pressId: "setup-primary" });
        c.runSetup?.(true);
        return;
      }
      if (action === "setup-pack") {
        c.cycleSetupPack?.();
        return;
      }
      if (action === "setup-done") {
        exit();
        return;
      }
      return;
    }

    if (c.screen === "workbench") {
      const fb = c.feedback ?? ((m: string) => c.flash(m));
      if (action?.startsWith("lane:")) {
        const lane = action.slice(5) as TerminalLane;
        if (TERMINAL_LANES.includes(lane)) {
          setWorkbenchLane(lane);
          fb(`lane ${lane}`, { pressId: `lane:${lane}` });
        }
        return;
      }
      if (region.id.startsWith("term-pack:") && idx !== undefined) {
        const already = idx === c.selectedPackIndex;
        setSelectedPackIndex(
          Math.max(0, Math.min(idx, Math.max(0, c.packs.length - 1))),
        );
        c.bumpSelect("none");
        const pack = c.packs[idx];
        fb(`focus ${pack?.title ?? "pack"}`, {
          pressId: pack ? `pack:${pack.name}` : "term-pack",
        });
        const now = Date.now();
        const prev = lastClickRef.current;
        const dbl = already && prev?.id === region.id && now - prev.at < 400;
        lastClickRef.current = { id: region.id, at: now };
        if (dbl && pack) {
          fb(`install ${pack.title}`, { pressId: "bar:install" });
          c.installPack(pack.name, "install");
        }
        return;
      }
      if (region.id.startsWith("term-runner:") && idx !== undefined) {
        setSelectedRunnerIndex(
          Math.max(0, Math.min(idx, Math.max(0, c.runnersLen - 1))),
        );
        setSelectedModelIndex(0);
        c.bumpSelect("none");
        fb("focus runner", { pressId: "term-runner" });
        return;
      }
      if (region.id.startsWith("term-task:") && idx !== undefined) {
        const already = idx === c.selectedTaskIndex;
        setSelectedTaskIndex(
          Math.max(0, Math.min(idx, Math.max(0, c.tasksLen - 1))),
        );
        c.bumpSelect("none");
        fb("focus service task", { pressId: "term-task" });
        const now = Date.now();
        const prev = lastClickRef.current;
        const dbl = already && prev?.id === region.id && now - prev.at < 400;
        lastClickRef.current = { id: region.id, at: now };
        if (dbl) {
          fb("run service", { pressId: "bar:run-service" });
          c.runService();
        }
        return;
      }
      if (region.id.startsWith("term-ops:") && idx !== undefined) {
        const already = idx === c.selectedOpsIndex;
        setSelectedOpsIndex(Math.max(0, Math.min(idx, 4)));
        c.bumpSelect("none");
        const ops = ["ready", "unify", "doctor", "paths", "refresh"] as const;
        const op = ops[idx] ?? "ready";
        fb(`focus ${op === "ready" ? "quickstart" : op}`, {
          pressId: `ops:${op}`,
        });
        const now = Date.now();
        const prev = lastClickRef.current;
        const dbl = already && prev?.id === region.id && now - prev.at < 400;
        lastClickRef.current = { id: region.id, at: now };
        if (dbl) {
          fb(`run ${op === "ready" ? "quickstart" : op}`, {
            pressId: "bar:run-ops",
          });
          c.runOps(op);
        }
        return;
      }
      if (action === "install") {
        const pack = c.packs[c.selectedPackIndex];
        if (pack) {
          fb(`install ${pack.title}`, { pressId: "bar:install" });
          c.installPack(pack.name, "install");
        }
        return;
      }
      if (action === "apply") {
        const pack = c.packs[c.selectedPackIndex];
        if (pack) {
          fb(`apply ${pack.title}`, { pressId: "bar:apply" });
          c.installPack(pack.name, "apply");
        }
        return;
      }
      if (action === "run") {
        fb("run agent job", { pressId: "bar:run" });
        c.runJob(false);
        return;
      }
      if (action === "run-service") {
        fb("run service", { pressId: "bar:run-service" });
        c.runService();
        return;
      }
      if (action === "run-ops") {
        const ops = ["ready", "unify", "doctor", "paths", "refresh"] as const;
        const op = ops[c.selectedOpsIndex] ?? "ready";
        fb(`run ${op === "ready" ? "quickstart" : op}`, {
          pressId: "bar:run-ops",
        });
        c.runOps(op);
        return;
      }
      if (action === "ollama-start") {
        fb("start ollama", { pressId: "bar:ollama" });
        c.startOllama();
        return;
      }
      if (action === "history") {
        fb("open history", { pressId: "bar:history" });
        c.openHistory?.();
        return;
      }
      if (action === "ollama-pull") {
        setTerminalInputMode("pull");
        setEditingJobPrompt(true);
        setJobPrompt("");
        fb("pull model — type name", { pressId: "bar:pull" });
        return;
      }
      if (action === "toggle-mode") {
        setJobMode((m) => (m === "inspect" ? "build" : "inspect"));
        fb("toggle mode", { pressId: "bar:mode" });
        return;
      }
      if (action === "help") {
        fb("open help", { pressId: "bar:help" });
        c.openHelp();
      }
      return;
    }

    if (typeof idx !== "number") return;
    if (region.id.startsWith("pack:")) {
      setSelectedPackIndex(
        Math.max(0, Math.min(idx, Math.max(0, c.packs.length - 1))),
      );
      c.bumpSelect("none");
      return;
    }
    if (region.id.startsWith("skill:")) {
      setSelectedSkillIndex(
        Math.max(0, Math.min(idx, Math.max(0, c.skillsLen - 1))),
      );
      c.bumpSelect("none");
      return;
    }
    if (region.id.startsWith("remote:")) {
      setSelectedRemoteIndex(
        Math.max(0, Math.min(idx, Math.max(0, c.remoteLen - 1))),
      );
      c.bumpSelect("none");
      return;
    }
    if (region.id.startsWith("harness:")) {
      setSelectedHarnessIndex(
        Math.max(
          0,
          Math.min(idx, Math.max(0, PATHS_LINKABLE_HARNESSES.length - 1)),
        ),
      );
      c.bumpSelect("none");
    }
  }, []);

  const hitMap = useMouseClick(onMouseClick, screen !== "loading");

  // Rebuild hit regions (lists + game-menu buttons)
  useEffect(() => {
    hitMap.clear();
    const start = scale.listStartRowHint;
    const col0 = 2;
    const col1 = Math.max(col0 + 20, scale.columns - 2);
    if (screen === "setup") {
      // Full-width primary button near bottom
      const row = Math.max(8, scale.rows - 4);
      hitMap.addButton({
        id: "setup-primary",
        row,
        col0: 2,
        col1: Math.max(20, scale.columns - 2),
        action:
          setupPhase === "confirm"
            ? "setup-write"
            : setupPhase === "ready" || setupPhase === "failed"
              ? "setup-plan"
              : "setup-done",
      });
      hitMap.addButton({
        id: "setup-pack",
        row: Math.max(6, scale.rows - 5),
        col0: 2,
        col1: Math.max(20, scale.columns - 2),
        action: "setup-pack",
      });
      return;
    }
    if (screen === "workbench") {
      const geo = workbenchGeometry(scale.columns, scale.rows);
      const taskCount = workbenchPlugins.reduce(
        (n, p) => n + (p.manifest.tasks?.length ?? 0),
        0,
      );
      let itemCount = packs.length;
      let selected = selectedPackIndex;
      if (workbenchLane === "agents") {
        itemCount = historyMode ? recentRuns.length : codingRunners.length;
        selected = historyMode ? selectedRunIndex : selectedRunnerIndex;
      } else if (workbenchLane === "services") {
        itemCount = taskCount;
        selected = selectedTaskIndex;
      } else if (workbenchLane === "ops") {
        itemCount = 5;
        selected = selectedOpsIndex;
      }
      const { offset, items } = windowSlice(
        Array.from({ length: itemCount }),
        selected,
        geo.listRows,
      );
      fillWorkbenchHits(hitMap, {
        geo,
        lane: workbenchLane,
        itemCount,
        listOffset: offset,
        visibleCount: items.length,
      });
      return;
    }
    if (screen === "home" || screen === "packs") {
      hitMap.addListRows({
        idPrefix: "pack",
        count: packs.length,
        startRow: start,
        col0,
        col1,
      });
    } else if (screen === "library") {
      hitMap.addListRows({
        idPrefix: "skill",
        count: skills.length,
        startRow: start,
        col0,
        col1,
      });
    } else if (screen === "explore") {
      hitMap.addListRows({
        idPrefix: "remote",
        count: remotePacks.length,
        startRow: start,
        col0,
        col1,
      });
    } else if (screen === "paths") {
      hitMap.addListRows({
        idPrefix: "harness",
        count: PATHS_LINKABLE_HARNESSES.length,
        startRow: start,
        col0,
        col1,
      });
    }
  }, [
    hitMap,
    screen,
    packs.length,
    skills.length,
    remotePacks.length,
    codingRunners.length,
    workbenchPlugins,
    workbenchLane,
    historyMode,
    recentRuns.length,
    selectedPackIndex,
    selectedRunnerIndex,
    selectedTaskIndex,
    selectedOpsIndex,
    selectedRunIndex,
    setupPhase,
    scale.columns,
    scale.rows,
  ]);

  const resolveTarget = useCallback(async (): Promise<string> => {
    const env = process.env.KIT_PROJECT_DIR?.trim();
    if (env) return path.resolve(env);
    try {
      const cfg = await readConfig();
      if (cfg.targetProjectDir?.trim()) {
        return path.resolve(cfg.targetProjectDir.trim());
      }
    } catch {
      // ignore
    }
    return process.cwd();
  }, []);

  const refreshData = useCallback(async () => {
    const projectDir = await resolveTarget();
    setTargetProject(projectDir);

    const listed = await listSkills();
    if (listed.ok) {
      setSkills(listed.value);
      setLibraryError(undefined);
      setSelectedSkillIndex((current) =>
        listed.value.length === 0
          ? 0
          : Math.min(current, listed.value.length - 1),
      );
    } else {
      setLibraryError(listed.error);
    }

    const packList = await listPacks();
    if (packList.ok) {
      setPacks(packList.value);
      setPacksError(undefined);
      setSelectedPackIndex((current) =>
        packList.value.length === 0
          ? 0
          : Math.min(current, packList.value.length - 1),
      );
    } else {
      setPacksError(packList.error);
    }

    const appliedFile = await readProjectAppliedPacks(projectDir);
    setApplied(Object.values(appliedFile.packs));

    // Auto-recommend packs + skills for the pointed project
    const rec = await recommendToolkits({ projectDir });
    if (rec.ok) {
      setRecommended(rec.value.recommendations);
      setSkillRecs(rec.value.skillRecommendations);
      setTopPick(rec.value.topPick);
      setRecommendSummary(rec.value.summary);
      if (packList.ok && rec.value.topPick) {
        const idx = packList.value.findIndex(
          (p) => p.name === rec.value.topPick,
        );
        if (idx >= 0) setSelectedPackIndex(idx);
      }
    }

    const who = await getLoggedInUser();
    if (who.ok) setUserLogin(who.value.user.login);
    else setUserLogin(undefined);

    try {
      const doc = await runDoctor({ projectDir });
      setDoctorReport(doc);
      if (doc.ok) {
        setDoctorSummary("doctor ok");
      } else {
        setDoctorSummary(
          `doctor ${doc.summary.failed} fail / ${doc.summary.warnings} warn`,
        );
      }
    } catch {
      setDoctorSummary(undefined);
    }

    try {
      const st = await runStatus({ projectDir });
      if (st.ok) {
        const bits = st.value.rows
          .filter((row) => row.scope === "project")
          .map((row) => {
            const name =
              row.harness === "claude-code"
                ? "claude"
                : row.harness === "grok-build"
                  ? "grok"
                  : "codex";
            const mark =
              row.state === "ok" ? "ok" : row.state === "partial" ? "~" : "x";
            return `${name}:${mark}`;
          });
        setAgentStatusLine(bits.join(" · "));
      }
    } catch {
      setAgentStatusLine(undefined);
    }

    try {
      const situation = await detectSituation({ projectDir });
      setStory(situation.story);
    } catch {
      setStory(undefined);
    }
  }, [resolveTarget]);

  const refreshWorkbench = useCallback(async () => {
    const [runners, plugins, ollama] = await Promise.all([
      detectCodingRunners(),
      listPlugins(),
      probeOllamaService(),
    ]);
    setCodingRunners(runners);
    setOllamaService(ollama);
    setSelectedModelIndex(0);
    if (plugins.ok) {
      setWorkbenchPlugins(plugins.value);
      const reports = await Promise.all(
        plugins.value.map(async (plugin) => ({
          name: plugin.manifest.name,
          report: await doctorPlugin(plugin.manifest.name),
        })),
      );
      setWorkbenchPluginStatus(
        Object.fromEntries(
          reports.map(({ name, report }) => [
            name,
            !report.ok || !report.value.executable
              ? "missing"
              : report.value.manifestChanged
                ? "review"
                : "ready",
          ]),
        ),
      );
    } else {
      setWorkbenchPlugins([]);
      setWorkbenchPluginStatus({});
      setWorkbenchError(plugins.error);
    }
  }, []);

  const startLocalOllama = useCallback(async () => {
    if (busy) return;
    setBusy(true);
    setWorkbenchError(undefined);
    setWorkbenchRunLabel("Ollama");
    setWorkbenchRunStatus("running");
    setWorkbenchOutputScroll(0);
    appendTerminalLog("→ start ollama serve");
    try {
      const result = await startOllamaServe({
        onProgress: (msg) => appendTerminalLog(msg),
      });
      if (!result.ok) {
        setWorkbenchError(result.error);
        setWorkbenchRunStatus("failed");
        appendTerminalLog(`! ${result.error}`);
        return;
      }
      setOllamaService(result.value);
      appendTerminalLog(`✓ ${result.value.detail}`);
      setWorkbenchRunStatus("succeeded");
      await refreshWorkbench();
      flash("ollama online");
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      setWorkbenchError(detail);
      setWorkbenchRunStatus("failed");
      appendTerminalLog(`! ${detail}`);
    } finally {
      setBusy(false);
    }
  }, [appendTerminalLog, busy, flash, refreshWorkbench]);

  const stopLocalOllama = useCallback(async () => {
    if (busy) return;
    setBusy(true);
    setWorkbenchError(undefined);
    setWorkbenchRunLabel("Ollama");
    setWorkbenchRunStatus("running");
    appendTerminalLog("→ stop kit-managed ollama");
    try {
      const result = await stopOllamaServe({
        onProgress: (msg) => appendTerminalLog(msg),
      });
      if (!result.ok) {
        setWorkbenchError(result.error);
        setWorkbenchRunStatus("failed");
        appendTerminalLog(`! ${result.error}`);
        const probe = await probeOllamaService();
        setOllamaService(probe);
        return;
      }
      setOllamaService(result.value);
      appendTerminalLog(`✓ ${result.value.detail}`);
      setWorkbenchRunStatus("succeeded");
      await refreshWorkbench();
      flash("ollama stopped");
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      setWorkbenchError(detail);
      setWorkbenchRunStatus("failed");
    } finally {
      setBusy(false);
    }
  }, [appendTerminalLog, busy, flash, refreshWorkbench]);

  const pullLocalModel = useCallback(
    async (modelName: string) => {
      if (busy) return;
      const name = modelName.trim();
      if (!name) {
        setWorkbenchError("Type a model name, then Enter.");
        return;
      }
      setBusy(true);
      setTerminalInputMode("prompt");
      setWorkbenchError(undefined);
      setWorkbenchRunLabel(`pull ${name}`);
      setWorkbenchRunStatus("running");
      setWorkbenchOutputScroll(0);
      appendTerminalLog(`→ ollama pull ${name}`);
      const controller = new AbortController();
      workbenchAbort.current = controller;
      try {
        const result = await pullOllamaModel(name, {
          signal: controller.signal,
          onProgress: (msg) => appendTerminalLog(msg),
          onOutput: (chunk) => {
            const line = chunk.replace(/\r/g, "\n").trim();
            if (line) appendTerminalLog(line.slice(0, 200));
          },
        });
        if (!result.ok) {
          setWorkbenchError(result.error);
          setWorkbenchRunStatus("failed");
          appendTerminalLog(`! ${result.error}`);
          return;
        }
        appendTerminalLog(
          `✓ pulled ${result.value.model} · ${result.value.models.length} models now`,
        );
        setWorkbenchRunStatus("succeeded");
        await refreshWorkbench();
        flash(`pulled ${name}`);
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        setWorkbenchError(detail);
        setWorkbenchRunStatus("failed");
      } finally {
        if (workbenchAbort.current === controller) {
          workbenchAbort.current = null;
        }
        setBusy(false);
      }
    },
    [appendTerminalLog, busy, flash, refreshWorkbench],
  );

  const workbenchTasks: WorkbenchServiceTask[] = workbenchPlugins.flatMap(
    (plugin) =>
      (plugin.manifest.tasks ?? []).map((task) => ({
        plugin: plugin.manifest.name,
        displayName: plugin.manifest.displayName,
        task: task.name,
        description: task.description,
        status: workbenchPluginStatus[plugin.manifest.name] ?? "missing",
      })),
  );

  const runWorkbenchJob = useCallback(
    async (confirmed: boolean) => {
      const runner = codingRunners[selectedRunnerIndex];
      if (!runner) return;
      const model = runner.models?.[
        Math.min(
          selectedModelIndex,
          Math.max(0, runner.models.length - 1),
        )
      ];
      if (!runner.available) {
        setWorkbenchOutput(undefined);
        setWorkbenchRunLabel(runner.label);
        setWorkbenchRunStatus("failed");
        setWorkbenchError(`${runner.label} is offline.`);
        return;
      }
      if (!jobPrompt.trim()) {
        setWorkbenchOutput(undefined);
        setWorkbenchRunLabel(runner.label);
        setWorkbenchRunStatus("failed");
        setWorkbenchError("Write a job first. Press e.");
        return;
      }
      if (runner.id === "ollama" && !model) {
        setWorkbenchOutput(undefined);
        setWorkbenchRunLabel(runner.label);
        setWorkbenchRunStatus("failed");
        setWorkbenchError("Install or select an Ollama model first.");
        return;
      }
      if (jobMode === "build" && !confirmed) {
        setConfirmBuildJob(true);
        setWorkbenchError(undefined);
        return;
      }
      setBusy(true);
      setConfirmBuildJob(false);
      setWorkbenchError(undefined);
      setWorkbenchOutput(undefined);
      setWorkbenchRunLabel(
        model ? `${runner.label} / ${model.name}` : runner.label,
      );
      setWorkbenchRunStatus("running");
      const controller = new AbortController();
      workbenchAbort.current = controller;
      let liveOutput = "";
      try {
        const result = await runCodingJob(
          {
            runner: runner.id,
            mode: jobMode,
            projectDir: targetProject,
            prompt: jobPrompt,
            ...(model ? { model: model.name } : {}),
            confirmBuild: confirmed,
          },
          {
            signal: controller.signal,
            onOutput: (chunk) => {
              liveOutput = `${liveOutput}${chunk}`.slice(-64 * 1024);
              setWorkbenchOutput(liveOutput);
            },
          },
        );
        if (!result.ok) {
          setWorkbenchError(result.error);
          setWorkbenchRunStatus("failed");
          return;
        }
        const report = result.value;
        const output = [
          report.stdout.trim(),
          report.stderr.trim(),
          report.truncated
            ? "[Kit stopped capturing output at the size limit.]"
            : "",
        ]
          .filter(Boolean)
          .join("\n");
        const finalOut =
          output ||
          `${runner.label} finished with exit code ${report.exitCode}.`;
        setWorkbenchOutput(finalOut);
        let status: "succeeded" | "failed" | "cancelled" | "timed_out" =
          "succeeded";
        if (report.timedOut) {
          setWorkbenchError(`${runner.label} timed out.`);
          setWorkbenchRunStatus("failed");
          status = "timed_out";
        } else if (report.cancelled) {
          setWorkbenchRunStatus("cancelled");
          status = "cancelled";
        } else if (report.exitCode !== 0) {
          setWorkbenchError(
            `${runner.label} exited with code ${report.exitCode}.`,
          );
          setWorkbenchRunStatus("failed");
          status = "failed";
        } else {
          setWorkbenchRunStatus("succeeded");
        }
        await persistProof({
          kind: "coding",
          label: model
            ? `${runner.label} / ${model.name}`
            : runner.label,
          projectDir: targetProject,
          status,
          transcript: finalOut,
          runner: runner.id,
          mode: jobMode,
          prompt: jobPrompt,
          ...(model ? { model: model.name } : {}),
          exitCode: report.exitCode,
          durationMs: report.durationMs,
        });
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        setWorkbenchError(detail);
        setWorkbenchRunStatus("failed");
      } finally {
        if (workbenchAbort.current === controller) {
          workbenchAbort.current = null;
        }
        setBusy(false);
      }
    },
    [
      codingRunners,
      jobMode,
      jobPrompt,
      selectedRunnerIndex,
      selectedModelIndex,
      targetProject,
      persistProof,
    ],
  );

  const runWorkbenchService = useCallback(async () => {
    const task = workbenchTasks[selectedTaskIndex];
    if (!task) return;
    if (task.status !== "ready") {
      setWorkbenchOutput(undefined);
      setWorkbenchRunLabel(`${task.displayName} / ${task.task}`);
      setWorkbenchRunStatus("failed");
      setWorkbenchError(
        task.status === "review"
          ? "Review the changed plugin before this task can run."
          : "This task is not ready.",
      );
      return;
    }
    setBusy(true);
    setWorkbenchError(undefined);
    setWorkbenchOutput(undefined);
    setWorkbenchRunLabel(`${task.displayName} / ${task.task}`);
    setWorkbenchRunStatus("running");
    const controller = new AbortController();
    workbenchAbort.current = controller;
    let liveOutput = "";
    try {
      const result = await runPluginTask(task.plugin, task.task, {
        stdio: "pipe",
        signal: controller.signal,
        onOutput: (chunk) => {
          liveOutput = `${liveOutput}${chunk}`.slice(-64 * 1024);
          setWorkbenchOutput(liveOutput);
        },
      });
      if (!result.ok) {
        setWorkbenchError(result.error);
        setWorkbenchRunStatus("failed");
        return;
      }
      const output = [
        result.value.stdout.trim(),
        result.value.stderr.trim(),
        result.value.truncated
          ? "[Kit stopped capturing output at the size limit.]"
          : "",
      ]
        .filter(Boolean)
        .join("\n");
      const finalOut =
        output ||
        `${task.displayName}/${task.task} finished with exit code ${result.value.exitCode}.`;
      setWorkbenchOutput(finalOut);
      let status: "succeeded" | "failed" | "cancelled" = "succeeded";
      if (result.value.cancelled) {
        setWorkbenchRunStatus("cancelled");
        status = "cancelled";
      } else if (result.value.exitCode !== 0) {
        setWorkbenchError(
          `${task.displayName}/${task.task} exited with code ${result.value.exitCode}.`,
        );
        setWorkbenchRunStatus("failed");
        status = "failed";
      } else {
        setWorkbenchRunStatus("succeeded");
      }
      await persistProof({
        kind: "service",
        label: `${task.displayName}/${task.task}`,
        projectDir: targetProject,
        status,
        transcript: finalOut,
        plugin: task.plugin,
        task: task.task,
        exitCode: result.value.exitCode,
      });
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      setWorkbenchError(detail);
      setWorkbenchRunStatus("failed");
    } finally {
      if (workbenchAbort.current === controller) {
        workbenchAbort.current = null;
      }
      setBusy(false);
    }
  }, [selectedTaskIndex, workbenchTasks, targetProject, persistProof]);

  const pointAtProject = useCallback(
    async (raw: string) => {
      const resolved = path.resolve(raw.trim() || process.cwd());
      try {
        await updateConfig({ targetProjectDir: resolved });
      } catch {
        // still use in-session even if config write fails
      }
      setTargetProject(resolved);
      setPointingProject(false);
      setPointDraft("");
      flash(`pointed · ${path.basename(resolved)}`);
      await refreshData();
    },
    [flash, refreshData],
  );

  const loadDoctor = useCallback(async () => {
    setDoctorLoading(true);
    setErrorMessage(undefined);
    try {
      const doc = await runDoctor({ projectDir: targetProject });
      setDoctorReport(doc);
      if (doc.ok) {
        setDoctorSummary("doctor ok");
        flash("checks passed");
      } else {
        setDoctorSummary(
          `doctor ${doc.summary.failed} fail / ${doc.summary.warnings} warn`,
        );
        flash("checks finished");
      }
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      setErrorMessage(detail);
    } finally {
      setDoctorLoading(false);
    }
  }, [flash, targetProject]);

  const loadPaths = useCallback(async () => {
    setPathLoading(true);
    setErrorMessage(undefined);
    setConfirmLinkWrite(false);
    try {
      const result = await describePaths({ projectDir: targetProject });
      if (!result.ok) {
        setErrorMessage(result.error);
        setPathReport(undefined);
        return;
      }
      setPathReport(result.value);
      flash("paths loaded");
    } finally {
      setPathLoading(false);
    }
  }, [flash, targetProject]);

  const runLink = useCallback(
    async (write: boolean) => {
      if (linking) return;
      const harness = PATHS_LINKABLE_HARNESSES[selectedHarnessIndex] as
        | HarnessId
        | undefined;
      if (!harness) return;
      setLinking(true);
      setErrorMessage(undefined);
      flash(write ? `linking ${harness}…` : `planning ${harness}…`);
      try {
        const result = await linkSkills({
          projectDir: targetProject,
          scope: pathScope,
          harnesses: [harness],
          write,
          force: write,
        });
        if (!result.ok) {
          setErrorMessage(result.error);
          setLinkResult(undefined);
          setConfirmLinkWrite(false);
          return;
        }
        setLinkResult(result.value);
        setConfirmLinkWrite(false);
        setStatusMessage(
          write
            ? `Linked ${result.value.linked} skill(s) → ${harness}`
            : `Plan: ${result.value.linked} would link → ${harness}`,
        );
        flash(write ? "linked" : "plan ready");
      } finally {
        setLinking(false);
      }
    },
    [linking, selectedHarnessIndex, pathScope, flash, targetProject],
  );

  const pathTargetRoot = (() => {
    const harness = PATHS_LINKABLE_HARNESSES[selectedHarnessIndex];
    if (!harness || !pathReport) return undefined;
    return pathReport.entries.find(
      (e) => e.harness === harness && e.scope === pathScope,
    )?.skillsRoot;
  })();

  const validateSelectedSkill = useCallback(async () => {
    const skill = skills[selectedSkillIndex];
    if (!skill) return;
    flash(`validate ${skill.name}`);
    const loaded = await loadSkill(skill.installPath);
    if (!loaded.ok) {
      setLastChecks(
        loaded.issues.map((i, idx) => ({
          id: `issue-${idx}`,
          level: "fail" as const,
          message: `${i.field}: ${i.message}`,
        })),
      );
      setErrorMessage(`Validate failed: ${skill.name}`);
      setStatusMessage(undefined);
      return;
    }
    setLastChecks([
      {
        id: "schema",
        level: "pass",
        message: `Schema OK (${loaded.skill.name}@${loaded.skill.version})`,
      },
    ]);
    setErrorMessage(undefined);
    setStatusMessage(`Validated ${skill.name}`);
    flash("validated");
  }, [skills, selectedSkillIndex, flash]);

  const testSelectedSkill = useCallback(async () => {
    const skill = skills[selectedSkillIndex];
    if (!skill) return;
    flash(`test ${skill.name}`);
    const result = await testSkill(skill.installPath);
    if (!result.ok) {
      setLastChecks(
        result.report?.checks ??
          ([
            {
              id: "error",
              level: "fail" as const,
              message: result.error,
            },
          ] satisfies CheckResult[]),
      );
      setErrorMessage(result.error);
      setStatusMessage(undefined);
      return;
    }
    setLastChecks(result.value.checks);
    setErrorMessage(undefined);
    setStatusMessage(`Tested ${skill.name}`);
    flash("tested");
  }, [skills, selectedSkillIndex, flash]);

  const loadExplore = useCallback(async (query?: string) => {
    setExploreLoading(true);
    setErrorMessage(undefined);
    try {
      if (query && query.trim()) {
        const result = await exploreSearch(query.trim());
        if (!result.ok) {
          setErrorMessage(result.error);
          setRemotePacks([]);
          return;
        }
        setRemotePacks(result.value.packs);
        setSelectedRemoteIndex(0);
      } else {
        const result = await exploreListPacks();
        if (!result.ok) {
          setErrorMessage(result.error);
          setRemotePacks([]);
          return;
        }
        setRemotePacks(result.value.packs);
        setSelectedRemoteIndex(0);
      }
    } finally {
      setExploreLoading(false);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;

    (async () => {
      const firstRun = await getFirstRunStatus();
      if (cancelled) return;
      setOfferFirstRun(firstRun.shouldOffer);

      await refreshData();
      if (cancelled) return;

      const projectDir = await resolveTarget();
      const sit = await detectSituation({ projectDir });
      if (cancelled) return;
      setStory(sit.story);

      const wantSplash = splashEnabled();
      const wantHome = initialScreen === "home";
      const wantAdvanced = initialScreen === "workbench";

      if (firstRun.shouldOffer && !wantHome && !wantAdvanced) {
        setScreen("first-run");
      } else if (wantSplash && !wantHome && !wantAdvanced) {
        setScreen("splash");
      } else if (wantHome) {
        setScreen("home");
      } else if (wantAdvanced) {
        await refreshWorkbench();
        if (cancelled) return;
        setScreen("workbench");
      } else {
        // DEFAULT: simple Setup wizard (not multi-lane hub)
        const packList = await listPacks();
        const recName =
          sit.snapshot.recommendedPack?.trim() || "essentials";
        const packItem = packList.ok
          ? packList.value.find((p) => p.name === recName) ??
            packList.value.find((p) => p.name === "essentials") ??
            packList.value[0]
          : undefined;
        const name = packItem?.name ?? recName;
        const title = packItem?.title ?? name;
        setSetupPackName(name);
        setSetupSteps(defaultSetupSteps(title));
        setSetupPhase("ready");
        setSetupCompleteNote(undefined);
        setSetupLog([
          `Project ${path.basename(projectDir)}`,
          `Pack ${title} (${name})`,
          sit.story.win,
          "Press Enter to plan. Press y to install and link.",
        ]);
        setScreen("setup");
      }

      if (mascotVisible() || wantSplash) {
        try {
          const loaded = await loadAllMascotFrames();
          if (cancelled) return;
          setFrames(loaded.idle);
          setScanFrames(loaded.scan);
          setSuccessFrames(loaded.success);
        } catch (error) {
          if (cancelled) return;
          const detail =
            error instanceof Error ? error.message : String(error);
          setLoadError(detail);
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [initialScreen, refreshData, refreshWorkbench, resolveTarget]);

  const leaveSplash = useCallback(() => {
    flash("open");
    if (offerFirstRun) setScreen("first-run");
    else setScreen("setup");
  }, [offerFirstRun, flash]);

  /** Setup wizard: plan or write via runReady. */
  const runSetup = useCallback(
    async (write: boolean) => {
      if (busy) return;
      setBusy(true);
      setSetupPhase("running");
      setErrorMessage(undefined);
      setSetupCompleteNote(undefined);
      appendSetupLog(write ? "→ Writing setup…" : "→ Planning setup…");
      flash(write ? "setup write" : "setup plan");
      try {
        const result = await runReady({
          projectDir: targetProject,
          pack: setupPackName,
          write,
          onProgress: (msg) => appendSetupLog(msg),
        });
        if (!result.ok) {
          setErrorMessage(result.error);
          appendSetupLog(`! ${result.error}`);
          setSetupPhase("failed");
          flash("setup failed");
          return;
        }
        const report = result.value;
        setSetupSteps(
          report.steps.map((s) => ({
            id: s.id,
            label: s.detail,
            status:
              s.status === "done"
                ? "done"
                : s.status === "failed"
                  ? "failed"
                  : s.status === "skipped"
                    ? "skipped"
                    : s.status === "planned"
                      ? "pending"
                      : "pending",
          })),
        );
        for (const s of report.steps) {
          appendSetupLog(`${s.status} · ${s.detail}`);
        }
        if (report.dryRun) {
          appendSetupLog("Plan ready. Press y to install and link.");
          setSetupPhase("confirm");
          flash("press y to write");
        } else if (report.complete) {
          appendSetupLog("Setup complete. Agents can use these skills.");
          setSetupCompleteNote(
            "Done. Open Claude, Codex, or Grok in this project.",
          );
          setSetupPhase("done");
          flash("setup complete");
          await completeFirstRun("installed", {
            preferredPack: setupPackName,
          });
          setOfferFirstRun(false);
          await refreshData();
        } else {
          appendSetupLog("Setup partial. Press d for doctor or retry.");
          setSetupPhase("failed");
          flash("setup partial");
          await refreshData();
        }
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        setErrorMessage(detail);
        appendSetupLog(`! ${detail}`);
        setSetupPhase("failed");
      } finally {
        setBusy(false);
      }
    },
    [
      busy,
      targetProject,
      setupPackName,
      appendSetupLog,
      flash,
      refreshData,
    ],
  );

  const cycleSetupPack = useCallback(() => {
    if (packs.length === 0) return;
    const idx = packs.findIndex((p) => p.name === setupPackName);
    const next = packs[(idx + 1 + packs.length) % packs.length] ?? packs[0]!;
    setSetupPackName(next.name);
    setSetupSteps(defaultSetupSteps(next.title));
    setSetupPhase("ready");
    appendSetupLog(`Pack → ${next.title}`);
    flash(`pack ${next.title}`);
  }, [packs, setupPackName, appendSetupLog, flash]);

  const installSelectedPack = useCallback(
    async (packName: string, mode: "install" | "apply") => {
      if (busy) return;
      setBusy(true);
      clearFlash();
      setProgress(undefined);
      flash(mode === "apply" ? `apply ${packName}` : `install ${packName}`);
      setStatusMessage(
        mode === "apply"
          ? `Applying ${packName}…`
          : `Installing ${packName}…`,
      );

      const onProgress = (info: {
        current: number;
        total: number;
        skillName: string;
      }) => {
        setProgress(info);
        setStatusMessage(
          `${mode === "apply" ? "Applying" : "Installing"} ${info.current}/${info.total}: ${info.skillName}`,
        );
      };

      try {
        if (mode === "apply") {
          const result = await applyPack(packName, {
            force: true,
            onProgress,
            projectDir: targetProject,
          });
          if (!result.ok) {
            setErrorMessage(result.error);
            setStatusMessage(undefined);
            setCelebrateCount(undefined);
            return;
          }
          setCelebrateCount(result.value.installed.length);
          setStatusMessage(
            `${result.value.reapplied ? "Reapplied" : "Applied"} ${result.value.pack.name}@${result.value.pack.version} (${result.value.installed.length} skills)`,
          );
          flash("applied · press k to link");
        } else {
          const result = await installPack(packName, {
            force: true,
            onProgress,
          });
          if (!result.ok) {
            setErrorMessage(result.error);
            setStatusMessage(undefined);
            setCelebrateCount(undefined);
            return;
          }
          setCelebrateCount(result.value.installed.length);
          setStatusMessage(
            `Installed ${result.value.pack.name}@${result.value.pack.version} (${result.value.installed.length} skills)`,
          );
          flash("installed · a apply · k link");
        }

        if (offerFirstRun || screen === "first-run") {
          await completeFirstRun("installed", { preferredPack: packName });
          setOfferFirstRun(false);
        }

        await refreshData();
        // Land in Action Terminal (not splash/home theater)
        if (screen === "first-run" || screen === "workbench") {
          await refreshWorkbench();
          appendTerminalLog(`✓ pack ${packName} ready`);
          setScreen("workbench");
        } else {
          setScreen("home");
        }
      } finally {
        setBusy(false);
        setProgress(undefined);
      }
    },
    [
      busy,
      clearFlash,
      offerFirstRun,
      refreshData,
      refreshWorkbench,
      appendTerminalLog,
      screen,
      flash,
      targetProject,
    ],
  );

  const removeSelectedSkill = useCallback(async () => {
    const skill = skills[selectedSkillIndex];
    if (!skill || busy) return;
    setBusy(true);
    clearFlash();
    flash(`remove ${skill.name}`);
    try {
      const result = await removeSkill(skill.name);
      if (!result.ok) {
        setErrorMessage(result.error);
        return;
      }
      setStatusMessage(`Removed ${result.value.name}`);
      setConfirmRemove(false);
      setLastChecks(undefined);
      await refreshData();
    } finally {
      setBusy(false);
    }
  }, [busy, clearFlash, refreshData, selectedSkillIndex, skills, flash]);

  /** Dry-run or write: recommend → install → apply → link → doctor. */
  const executeReady = useCallback(
    async (write: boolean) => {
      if (busy) return;
      setBusy(true);
      clearFlash();
      setHomeConfirm("none");
      setErrorMessage(undefined);
      flash(write ? "ready write…" : "ready plan…");
      setStatusMessage(
        write
          ? "Running Ready (write)…"
          : "Planning Ready (dry-run)…",
      );
      try {
        const result = await runReady({
          projectDir: targetProject,
          write,
          onProgress: (msg) => setStatusMessage(msg),
        });
        if (!result.ok) {
          setErrorMessage(result.error);
          setStatusMessage(undefined);
          setPlanLines(undefined);
          if (result.value) {
            setPlanLines(formatReadySteps(result.value));
          }
          return;
        }
        const report = result.value;
        setPlanLines(formatReadySteps(report));
        setStatusMessage(formatReadyStatus(report));
        if (report.dryRun) {
          setHomeConfirm("ready-write");
          flash("ready plan · y write");
        } else if (report.complete) {
          setCelebrateCount(
            report.steps.filter((s) => s.status === "done").length,
          );
          flash("ready complete");
          await refreshData();
        } else {
          flash("ready partial");
          await refreshData();
        }
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        setErrorMessage(detail);
        setStatusMessage(undefined);
      } finally {
        setBusy(false);
      }
    },
    [busy, clearFlash, flash, refreshData, targetProject],
  );

  /** Dry-run or write: scan agent skill dumps → rank keepers → adopt. */
  const executeUnify = useCallback(
    async (write: boolean) => {
      if (busy) return;
      setBusy(true);
      clearFlash();
      setHomeConfirm("none");
      setErrorMessage(undefined);
      flash(write ? "unify write…" : "unify plan…");
      setStatusMessage(
        write ? "Running Unify (write)…" : "Planning Unify (dry-run)…",
      );
      try {
        const result = await runUnify({
          projectDir: targetProject,
          write,
          link: write,
          onProgress: (msg) => setStatusMessage(msg),
        });
        if (!result.ok) {
          setErrorMessage(result.error);
          setStatusMessage(undefined);
          setPlanLines(undefined);
          return;
        }
        const report = result.value;
        setPlanLines(formatUnifyPreview(report));
        setStatusMessage(formatUnifyStatus(report));
        if (report.dryRun) {
          setHomeConfirm("unify-write");
          flash("unify plan · y write");
        } else {
          if (report.adopted > 0) {
            setCelebrateCount(report.adopted);
          }
          flash(report.adopted > 0 ? "unify adopted" : "unify done");
          await refreshData();
        }
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        setErrorMessage(detail);
        setStatusMessage(undefined);
      } finally {
        setBusy(false);
      }
    },
    [busy, clearFlash, flash, refreshData, targetProject],
  );

  const openHelp = useCallback(() => {
    if (screen === "help" || screen === "loading" || screen === "splash") {
      return;
    }
    setHelpFrom(screen);
    flash("help");
    setScreen("help");
  }, [flash, screen]);

  /** Ops lane: plan → confirm y/n in this terminal → write. */
  const runTerminalOps = useCallback(
    async (opId: string, write = false) => {
      if (busy) return;
      setWorkbenchOutputScroll(0);
      setWorkbenchError(undefined);
      setWorkbenchRunLabel(opId);
      if (opId === "ready") {
        appendTerminalLog(
          write ? "→ ready write" : "→ ready plan (dry-run)",
        );
        setWorkbenchRunStatus("running");
        setBusy(true);
        setTerminalOpsConfirm("none");
        try {
          const result = await runReady({
            projectDir: targetProject,
            write,
            onProgress: (msg) => appendTerminalLog(msg),
          });
          if (!result.ok) {
            setWorkbenchError(result.error);
            appendTerminalLog(`! ${result.error}`);
            setWorkbenchRunStatus("failed");
            return;
          }
          for (const line of formatReadySteps(result.value)) {
            appendTerminalLog(line);
          }
          appendTerminalLog(formatReadyStatus(result.value));
          setPlanLines(formatReadySteps(result.value));
          setStatusMessage(formatReadyStatus(result.value));
          if (result.value.dryRun) {
            appendTerminalLog("Press y to write. Press n to cancel.");
            setTerminalOpsConfirm("ready-write");
            setWorkbenchRunStatus("succeeded");
            flash("ready plan · y write");
          } else {
            setWorkbenchRunStatus(
              result.value.complete ? "succeeded" : "failed",
            );
            flash(
              result.value.complete ? "ready complete" : "ready partial",
            );
            await refreshData();
            await persistProof({
              kind: "ops",
              label: "ready write",
              projectDir: targetProject,
              status: result.value.complete ? "succeeded" : "failed",
              transcript: formatReadySteps(result.value).join("\n"),
            });
          }
        } finally {
          setBusy(false);
        }
        return;
      }
      if (opId === "unify") {
        appendTerminalLog(
          write ? "→ unify write" : "→ unify plan (dry-run)",
        );
        setWorkbenchRunStatus("running");
        setBusy(true);
        setTerminalOpsConfirm("none");
        try {
          const result = await runUnify({
            projectDir: targetProject,
            write,
            link: write,
            onProgress: (msg) => appendTerminalLog(msg),
          });
          if (!result.ok) {
            setWorkbenchError(result.error);
            appendTerminalLog(`! ${result.error}`);
            setWorkbenchRunStatus("failed");
            return;
          }
          for (const line of formatUnifyPreview(result.value)) {
            appendTerminalLog(line);
          }
          appendTerminalLog(formatUnifyStatus(result.value));
          setPlanLines(formatUnifyPreview(result.value));
          setStatusMessage(formatUnifyStatus(result.value));
          if (result.value.dryRun) {
            appendTerminalLog("Press y to write. Press n to cancel.");
            setTerminalOpsConfirm("unify-write");
            setWorkbenchRunStatus("succeeded");
            flash("unify plan · y write");
          } else {
            setWorkbenchRunStatus("succeeded");
            flash("unify wrote");
            await refreshData();
            await persistProof({
              kind: "ops",
              label: "unify write",
              projectDir: targetProject,
              status: "succeeded",
              transcript: formatUnifyPreview(result.value).join("\n"),
            });
          }
        } finally {
          setBusy(false);
        }
        return;
      }
      if (opId === "doctor") {
        appendTerminalLog("→ doctor");
        setWorkbenchRunStatus("running");
        setBusy(true);
        try {
          const doc = await runDoctor({ projectDir: targetProject });
          setDoctorReport(doc);
          appendTerminalLog(
            doc.ok
              ? `✓ doctor ok · ${doc.summary.passed} pass`
              : `! doctor ${doc.summary.failed} fail / ${doc.summary.warnings} warn`,
          );
          for (const check of doc.checks.slice(0, 14)) {
            appendTerminalLog(
              `  ${check.level === "pass" ? "+" : check.level === "fail" ? "!" : "~"} ${check.message}`,
            );
          }
          setWorkbenchRunStatus(doc.ok ? "succeeded" : "failed");
          if (!doc.ok) {
            setWorkbenchError(
              `${doc.summary.failed} failed · ${doc.summary.warnings} warnings`,
            );
          }
        } finally {
          setBusy(false);
        }
        return;
      }
      if (opId === "paths") {
        flash("paths");
        setScreen("paths");
        void loadPaths();
        return;
      }
      if (opId === "refresh") {
        appendTerminalLog("→ refresh discovery");
        setBusy(true);
        setWorkbenchRunStatus("running");
        try {
          await refreshData();
          await refreshWorkbench();
          appendTerminalLog("✓ runners · plugins · packs · ollama");
          setWorkbenchRunStatus("succeeded");
          flash("refreshed");
        } finally {
          setBusy(false);
        }
      }
    },
    [
      busy,
      appendTerminalLog,
      flash,
      targetProject,
      refreshData,
      refreshWorkbench,
      loadPaths,
      persistProof,
    ],
  );

  // Keep mouse handler in sync with latest actions (game-menu clicks).
  mouseCtxRef.current = {
    screen,
    packs,
    skillsLen: skills.length,
    remoteLen: remotePacks.length,
    selectedPackIndex,
    selectedTaskIndex,
    selectedOpsIndex,
    runnersLen: codingRunners.length,
    tasksLen: workbenchTasks.length,
    bumpSelect,
    flash,
    feedback,
    installPack: (name, mode) => {
      void installSelectedPack(name, mode);
    },
    runJob: (confirmed) => {
      void runWorkbenchJob(confirmed);
    },
    runService: () => {
      void runWorkbenchService();
    },
    runOps: (id) => {
      void runTerminalOps(id, false);
    },
    startOllama: () => {
      void startLocalOllama();
    },
    openHelp,
    openHistory: () => {
      void refreshRecentRuns().then(() => {
        setHistoryMode(true);
        feedback("open history", { pressId: "bar:history" });
      });
    },
    runSetup: (write) => {
      void runSetup(write);
    },
    cycleSetupPack,
  };

  useInput((input, key) => {
    if (key.ctrl && input === "c") {
      workbenchAbort.current?.abort();
      exit();
      return;
    }

    const enteringText =
      pointingProject ||
      filteringPacks ||
      (screen === "explore" && exploreQuery.startsWith("/")) ||
      (screen === "workbench" &&
        (editingJobPrompt || terminalInputMode === "pull"));
    const awaitingChoice =
      confirmRemove ||
      confirmLinkWrite ||
      confirmBuildJob ||
      homeConfirm !== "none" ||
      terminalOpsConfirm !== "none";
    if (
      shouldQuitWithQ({
        input,
        busy,
        enteringText,
        awaitingChoice,
      })
    ) {
      exit();
      return;
    }

    if (screen === "workbench" && busy) {
      if (key.escape || input === "x") {
        if (workbenchAbort.current) {
          setWorkbenchRunStatus("stopping");
          workbenchAbort.current.abort();
          flash("stopping run");
        }
        return;
      }
      // Allow scroll while a job is running
      if (key.pageUp) {
        setWorkbenchOutputScroll((n) => n + 8);
        return;
      }
      if (key.pageDown) {
        setWorkbenchOutputScroll((n) => Math.max(0, n - 8));
        return;
      }
      return;
    }

    if (screen === "loading" || busy) return;

    if (screen === "help") {
      if (key.escape || input === "h" || input === "?") {
        setScreen(helpFrom === "help" ? "home" : helpFrom);
        flash("back");
      }
      return;
    }

    if (screen === "splash") {
      leaveSplash();
      return;
    }

    if (screen === "first-run") {
      if (input === "s") {
        void (async () => {
          await completeFirstRun("skipped");
          setOfferFirstRun(false);
          flash("skipped · open setup");
          setScreen("setup");
        })();
        return;
      }
      const packName = FIRST_RUN_BY_KEY[input];
      if (packName) {
        setSetupPackName(packName);
        const title =
          packs.find((p) => p.name === packName)?.title ?? packName;
        setSetupSteps(defaultSetupSteps(title));
        flash(`pack ${title}`);
        setScreen("setup");
        void runSetup(false);
      }
      return;
    }

    // DEFAULT: simple setup wizard
    if (screen === "setup") {
      if (input === "a") {
        feedback("advanced menu");
        void refreshWorkbench().then(() => setScreen("workbench"));
        return;
      }
      if (input === "p") {
        cycleSetupPack();
        return;
      }
      if (setupPhase === "confirm") {
        if (input === "y") {
          void runSetup(true);
          return;
        }
        if (input === "n" || key.escape) {
          setSetupPhase("ready");
          appendSetupLog("Write cancelled.");
          flash("cancelled");
          return;
        }
        return;
      }
      if (setupPhase === "done") {
        if (key.return || input === "q") {
          exit();
          return;
        }
        return;
      }
      if (setupPhase === "failed") {
        if (key.return) {
          void runSetup(false);
          return;
        }
        return;
      }
      if (key.return) {
        void runSetup(false);
        return;
      }
      return;
    }

    // Point Kit at a project path (auto-recommend re-runs)
    if (pointingProject) {
      if (key.escape) {
        setPointingProject(false);
        setPointDraft("");
        flash("point cancelled");
        return;
      }
      if (key.return) {
        void pointAtProject(pointDraft || process.cwd());
        return;
      }
      if (key.backspace || key.delete) {
        setPointDraft((d) => d.slice(0, -1));
        return;
      }
      if (input && input.length === 1 && !key.ctrl) {
        setPointDraft((d) => d + input);
        return;
      }
      return;
    }

    if (
      screen === "workbench" &&
      (editingJobPrompt || terminalInputMode === "pull")
    ) {
      if (key.escape) {
        setEditingJobPrompt(false);
        setTerminalInputMode("prompt");
        flash("cancelled");
        return;
      }
      if (key.return) {
        if (terminalInputMode === "pull") {
          const name = jobPrompt.trim();
          setEditingJobPrompt(false);
          void pullLocalModel(name);
          return;
        }
        setEditingJobPrompt(false);
        flash("job ready");
        return;
      }
      if (key.backspace || key.delete) {
        setJobPrompt((value) => value.slice(0, -1));
        return;
      }
      if (input && !key.ctrl) {
        const clean =
          terminalInputMode === "pull"
            ? input.replace(/\s+/g, "")
            : input.replace(/\s+/g, " ");
        setJobPrompt((value) => `${value}${clean}`.slice(0, 8_000));
        return;
      }
      return;
    }

    if (screen === "packs" && filteringPacks) {
      if (key.escape) {
        setFilteringPacks(false);
        setPackFilter("");
        flash("filter cleared");
        return;
      }
      if (key.return) {
        setFilteringPacks(false);
        flash("filter set");
        return;
      }
      if (key.backspace || key.delete) {
        setPackFilter((f) => f.slice(0, -1));
        return;
      }
      if (input && input.length === 1 && !key.ctrl) {
        setPackFilter((f) => f + input);
        return;
      }
      return;
    }

    if (screen === "explore" && exploreQuery.startsWith("/")) {
      if (key.escape) {
        setExploreQuery("");
        void loadExplore();
        return;
      }
      if (key.return) {
        const q = exploreQuery.slice(1);
        setExploreQuery(q);
        flash("search");
        void loadExplore(q);
        return;
      }
      if (key.backspace || key.delete) {
        setExploreQuery((q) => (q.length <= 1 ? "" : q.slice(0, -1)));
        return;
      }
      if (input && input.length === 1 && !key.ctrl) {
        setExploreQuery((q) => q + input);
        return;
      }
      return;
    }

    const mainScreens: Screen[] = [
      "home",
      "library",
      "packs",
      "explore",
      "doctor",
      "paths",
      "workbench",
    ];
    if (mainScreens.includes(screen)) {
      if (input === "?") {
        openHelp();
        return;
      }
      if (input === "s" && splashEnabled()) {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setHomeConfirm("none");
        flash("splash");
        setScreen("splash");
        return;
      }
      if (input === "h") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setHomeConfirm("none");
        flash("home");
        setScreen("home");
        return;
      }
      if (input === "l") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setHomeConfirm("none");
        flash("library");
        setScreen("library");
        return;
      }
      if (input === "p" && screen !== "paths") {
        // on paths screen, p = plan; elsewhere p = packs
        if (screen !== "doctor") {
          setConfirmRemove(false);
          setFilteringPacks(false);
          setHomeConfirm("none");
          flash("packs");
          setScreen("packs");
          return;
        }
      }
      if (input === "e" && screen !== "workbench") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setHomeConfirm("none");
        flash("explore");
        setScreen("explore");
        void loadExplore(exploreQuery || undefined);
        return;
      }
      if (input === "d") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setHomeConfirm("none");
        flash("doctor");
        setScreen("doctor");
        void loadDoctor();
        return;
      }
      if (input === "k") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setHomeConfirm("none");
        flash("paths");
        setScreen("paths");
        void loadPaths();
        return;
      }
      if (input === "w") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setHomeConfirm("none");
        setConfirmBuildJob(false);
        setWorkbenchError(undefined);
        setWorkbenchOutputScroll(0);
        flash("workbench");
        setScreen("workbench");
        void refreshWorkbench();
        return;
      }
    }

    if (screen === "home") {
      if (homeConfirm !== "none") {
        if (input === "y") {
          if (homeConfirm === "ready-write") {
            void executeReady(true);
          } else {
            void executeUnify(true);
          }
          return;
        }
        if (input === "n" || key.escape) {
          setHomeConfirm("none");
          flash("write cancelled");
          return;
        }
        return;
      }
      if (input === "r") {
        void executeReady(false);
        return;
      }
      if (input === "u") {
        void executeUnify(false);
        return;
      }
      if (input === "o") {
        setPointingProject(true);
        setPointDraft(targetProject);
        flash("point at project");
        return;
      }
      if (key.upArrow) {
        setSelectedPackIndex((i) =>
          packs.length === 0 ? 0 : (i - 1 + packs.length) % packs.length,
        );
        bumpSelect("up");
        return;
      }
      if (key.downArrow) {
        setSelectedPackIndex((i) =>
          packs.length === 0 ? 0 : (i + 1) % packs.length,
        );
        bumpSelect("down");
        return;
      }
      if (key.return || input === "i") {
        const pack = packs[selectedPackIndex];
        if (pack) void installSelectedPack(pack.name, "install");
        return;
      }
      if (input === "a") {
        const pack = packs[selectedPackIndex];
        if (pack) void installSelectedPack(pack.name, "apply");
        return;
      }
      if (/^[1-7]$/.test(input)) {
        const index = Number(input) - 1;
        const pack = packs[index];
        if (pack) {
          setSelectedPackIndex(index);
          bumpSelect("none");
          void installSelectedPack(pack.name, "install");
        }
      }
      return;
    }

    if (screen === "packs") {
      if (input === "/" || (input && /^[a-zA-Z]$/.test(input) && !key.ctrl)) {
        setFilteringPacks(true);
        if (input !== "/") setPackFilter((f) => f + input);
        flash("filter");
        return;
      }
      if (key.escape) {
        setPackFilter("");
        return;
      }
      if (key.upArrow) {
        setSelectedPackIndex((i) =>
          packs.length === 0 ? 0 : (i - 1 + packs.length) % packs.length,
        );
        bumpSelect("up");
        return;
      }
      if (key.downArrow) {
        setSelectedPackIndex((i) =>
          packs.length === 0 ? 0 : (i + 1) % packs.length,
        );
        bumpSelect("down");
        return;
      }
      if (key.return || input === "i") {
        const pack = packs[selectedPackIndex];
        if (pack) void installSelectedPack(pack.name, "install");
        return;
      }
      if (input === "a") {
        const pack = packs[selectedPackIndex];
        if (pack) void installSelectedPack(pack.name, "apply");
      }
      return;
    }

    if (screen === "explore") {
      if (input === "/") {
        setExploreQuery("/");
        return;
      }
      if (input === "r") {
        flash("refresh");
        void loadExplore(exploreQuery || undefined);
        return;
      }
      if (key.upArrow) {
        setSelectedRemoteIndex((i) =>
          remotePacks.length === 0
            ? 0
            : (i - 1 + remotePacks.length) % remotePacks.length,
        );
        bumpSelect("up");
        return;
      }
      if (key.downArrow) {
        setSelectedRemoteIndex((i) =>
          remotePacks.length === 0
            ? 0
            : (i + 1) % remotePacks.length,
        );
        bumpSelect("down");
        return;
      }
      if (key.return || input === "i") {
        const pack = remotePacks[selectedRemoteIndex];
        if (pack) void installSelectedPack(pack.name, "install");
      }
      return;
    }

    if (screen === "library") {
      if (confirmRemove) {
        if (input === "y") {
          void removeSelectedSkill();
          return;
        }
        if (input === "n" || key.escape) {
          setConfirmRemove(false);
          flash("cancelled");
          return;
        }
        return;
      }
      if (key.upArrow) {
        setSelectedSkillIndex((i) =>
          skills.length === 0 ? 0 : (i - 1 + skills.length) % skills.length,
        );
        bumpSelect("up");
        setLastChecks(undefined);
        return;
      }
      if (key.downArrow) {
        setSelectedSkillIndex((i) =>
          skills.length === 0 ? 0 : (i + 1) % skills.length,
        );
        bumpSelect("down");
        setLastChecks(undefined);
        return;
      }
      if (input === "v" && skills[selectedSkillIndex]) {
        void validateSelectedSkill();
        return;
      }
      if (input === "t" && skills[selectedSkillIndex]) {
        void testSelectedSkill();
        return;
      }
      if (input === "r" && skills[selectedSkillIndex]) {
        setConfirmRemove(true);
        flash("confirm remove");
      }
      return;
    }

    if (screen === "doctor") {
      if (input === "r") {
        void loadDoctor();
        return;
      }
      if (input === "p") {
        flash("packs");
        setScreen("packs");
      }
      return;
    }

    if (screen === "paths") {
      if (confirmLinkWrite) {
        if (input === "y") {
          void runLink(true);
          return;
        }
        if (input === "n" || key.escape) {
          setConfirmLinkWrite(false);
          flash("write cancelled");
          return;
        }
        return;
      }
      if (key.upArrow) {
        setSelectedHarnessIndex((i) =>
          (i - 1 + PATHS_LINKABLE_HARNESSES.length) %
          PATHS_LINKABLE_HARNESSES.length,
        );
        bumpSelect("up");
        setConfirmLinkWrite(false);
        return;
      }
      if (key.downArrow) {
        setSelectedHarnessIndex((i) =>
          (i + 1) % PATHS_LINKABLE_HARNESSES.length,
        );
        bumpSelect("down");
        setConfirmLinkWrite(false);
        return;
      }
      if (key.tab) {
        setPathScope((s) => (s === "project" ? "personal" : "project"));
        setConfirmLinkWrite(false);
        flash(
          pathScope === "project" ? "scope: personal" : "scope: project",
        );
        return;
      }
      // Enter proposes the target folder — must y to write
      if (key.return) {
        setConfirmLinkWrite(true);
        flash("approve folder?");
        return;
      }
      if (input === "p") {
        void runLink(false);
        return;
      }
      if (input === "r") {
        void loadPaths();
      }
      return;
    }

    if (screen === "workbench") {
      if (terminalOpsConfirm !== "none") {
        if (input === "y") {
          const op =
            terminalOpsConfirm === "ready-write" ? "ready" : "unify";
          void runTerminalOps(op, true);
          return;
        }
        if (input === "n" || key.escape) {
          setTerminalOpsConfirm("none");
          appendTerminalLog("write cancelled");
          flash("cancelled");
          return;
        }
        return;
      }
      if (confirmBuildJob) {
        if (input === "y") {
          setWorkbenchOutputScroll(0);
          void runWorkbenchJob(true);
          return;
        }
        if (input === "n" || key.escape) {
          setConfirmBuildJob(false);
          flash("build cancelled");
          return;
        }
        return;
      }
      if (key.pageUp) {
        setWorkbenchOutputScroll((n) => n + 8);
        return;
      }
      if (key.pageDown) {
        setWorkbenchOutputScroll((n) => Math.max(0, n - 8));
        return;
      }
      if (key.escape) {
        if (historyMode) {
          setHistoryMode(false);
          flash("runners");
          return;
        }
        setScreen("home");
        flash("home");
        return;
      }
      // Lane jump 1–4 — always flash + log
      if (input === "1") {
        setWorkbenchLane("skills");
        feedback("lane skills", { pressId: "lane:skills" });
        return;
      }
      if (input === "2") {
        setWorkbenchLane("agents");
        feedback("lane agents", { pressId: "lane:agents" });
        return;
      }
      if (input === "3") {
        setWorkbenchLane("services");
        feedback("lane services", { pressId: "lane:services" });
        return;
      }
      if (input === "4") {
        setWorkbenchLane("ops");
        feedback("lane ops · quickstart", { pressId: "lane:ops" });
        return;
      }
      if (key.tab) {
        setWorkbenchLane((lane) => {
          const i = TERMINAL_LANES.indexOf(lane);
          const next = TERMINAL_LANES[(i + 1) % TERMINAL_LANES.length]!;
          feedback(`lane ${next}`, { pressId: `lane:${next}` });
          return next;
        });
        return;
      }
      if (key.upArrow) {
        if (workbenchLane === "skills") {
          setSelectedPackIndex((i) =>
            packs.length === 0 ? 0 : (i - 1 + packs.length) % packs.length,
          );
        } else if (workbenchLane === "agents") {
          setSelectedRunnerIndex((index) =>
            codingRunners.length === 0
              ? 0
              : (index - 1 + codingRunners.length) % codingRunners.length,
          );
          setSelectedModelIndex(0);
        } else if (workbenchLane === "services") {
          setSelectedTaskIndex((index) =>
            workbenchTasks.length === 0
              ? 0
              : (index - 1 + workbenchTasks.length) % workbenchTasks.length,
          );
        } else {
          setSelectedOpsIndex((index) => (index - 1 + 5) % 5);
        }
        bumpSelect("up");
        return;
      }
      if (key.downArrow) {
        if (workbenchLane === "skills") {
          setSelectedPackIndex((i) =>
            packs.length === 0 ? 0 : (i + 1) % packs.length,
          );
        } else if (workbenchLane === "agents") {
          setSelectedRunnerIndex((index) =>
            codingRunners.length === 0
              ? 0
              : (index + 1) % codingRunners.length,
          );
          setSelectedModelIndex(0);
        } else if (workbenchLane === "services") {
          setSelectedTaskIndex((index) =>
            workbenchTasks.length === 0
              ? 0
              : (index + 1) % workbenchTasks.length,
          );
        } else {
          setSelectedOpsIndex((index) => (index + 1) % 5);
        }
        bumpSelect("down");
        return;
      }

      // Agents: Ollama lifecycle + job controls + proof vault
      if (workbenchLane === "agents") {
        if (historyMode) {
          if (key.upArrow) {
            setSelectedRunIndex((i) =>
              recentRuns.length === 0
                ? 0
                : (i - 1 + recentRuns.length) % recentRuns.length,
            );
            bumpSelect("up");
            return;
          }
          if (key.downArrow) {
            setSelectedRunIndex((i) =>
              recentRuns.length === 0
                ? 0
                : (i + 1) % recentRuns.length,
            );
            bumpSelect("down");
            return;
          }
          if (key.return) {
            const run = recentRuns[selectedRunIndex];
            if (run) void openSavedRun(run.id);
            return;
          }
          if (input === "H") {
            setHistoryMode(false);
            flash("runners");
            return;
          }
          return;
        }
        if (input === "H") {
          void refreshRecentRuns().then(() => {
            setHistoryMode(true);
            feedback("open history", { pressId: "bar:history" });
          });
          return;
        }
        if (input === "o") {
          feedback("start ollama", { pressId: "bar:ollama" });
          void startLocalOllama();
          return;
        }
        if (input === "O") {
          feedback("stop ollama", { pressId: "bar:ollama" });
          void stopLocalOllama();
          return;
        }
        if (input === "p") {
          setTerminalInputMode("pull");
          setEditingJobPrompt(true);
          setJobPrompt("");
          setWorkbenchError(undefined);
          feedback("pull model — type name", { pressId: "bar:pull" });
          return;
        }
        if (input === "e") {
          setTerminalInputMode("prompt");
          setEditingJobPrompt(true);
          setWorkbenchError(undefined);
          feedback("edit job prompt", { pressId: "bar:edit" });
          return;
        }
        if (input === "m") {
          setJobMode((mode) => (mode === "inspect" ? "build" : "inspect"));
          setConfirmBuildJob(false);
          feedback(
            jobMode === "inspect" ? "mode build" : "mode inspect",
            { pressId: "bar:mode" },
          );
          return;
        }
        if (key.leftArrow || key.rightArrow) {
          const models =
            codingRunners[selectedRunnerIndex]?.models ??
            ollamaService?.models ??
            [];
          if (models.length > 0) {
            setSelectedModelIndex((index) =>
              key.leftArrow
                ? (index - 1 + models.length) % models.length
                : (index + 1) % models.length,
            );
            feedback("select model", { pressId: "bar:model" });
          }
          return;
        }
        if (key.return) {
          setWorkbenchOutputScroll(0);
          feedback("run agent job", { pressId: "bar:run" });
          void runWorkbenchJob(false);
          return;
        }
        return;
      }

      if (workbenchLane === "skills") {
        if (key.return || input === "i") {
          const pack = packs[selectedPackIndex];
          if (!pack) return;
          setWorkbenchOutputScroll(0);
          setWorkbenchRunLabel(`install ${pack.name}`);
          feedback(`install ${pack.title}`, { pressId: "bar:install" });
          void installSelectedPack(pack.name, "install").then(() => {
            appendTerminalLog("✓ install finished");
            feedback("install done", { log: false });
          });
          return;
        }
        if (input === "a") {
          const pack = packs[selectedPackIndex];
          if (!pack) return;
          setWorkbenchOutputScroll(0);
          setWorkbenchRunLabel(`apply ${pack.name}`);
          feedback(`apply ${pack.title}`, { pressId: "bar:apply" });
          void installSelectedPack(pack.name, "apply").then(() => {
            appendTerminalLog("✓ apply finished");
          });
          return;
        }
        return;
      }

      if (workbenchLane === "services") {
        if (key.return) {
          setWorkbenchOutputScroll(0);
          feedback("run service", { pressId: "bar:run-service" });
          void runWorkbenchService();
        }
        return;
      }

      if (workbenchLane === "ops") {
        if (key.return) {
          const ops = ["ready", "unify", "doctor", "paths", "refresh"] as const;
          const op = ops[selectedOpsIndex] ?? "ready";
          feedback(
            `run ${op === "ready" ? "quickstart" : op}`,
            { pressId: "bar:run-ops" },
          );
          void runTerminalOps(op);
        }
      }
    }
  });

  /** Pick mascot loop + frames for current UI state. */
  const pickMascot = (
    mode: "idle" | "scan" | "success" | "auto",
    auto?: { busy?: boolean; ok?: boolean },
  ): { frames: PixelFrame[]; variant: MascotVariant } => {
    let variant: MascotVariant = "idle";
    if (mode === "scan" || (mode === "auto" && auto?.busy)) {
      variant = "scan";
    } else if (mode === "success" || (mode === "auto" && auto?.ok)) {
      variant = "success";
    }
    const set =
      variant === "scan"
        ? scanFrames.length
          ? scanFrames
          : frames
        : variant === "success"
          ? successFrames.length
            ? successFrames
            : frames
          : frames;
    return { frames: set, variant };
  };

  if (screen === "loading") {
    return (
      <Box paddingX={1} paddingY={0} flexDirection="column">
        <Text bold inverse>
          {" "}
          KIT{" "}
        </Text>
        <Text dimColor>Starting…</Text>
        {loadError ? <Text color="red">{loadError}</Text> : null}
      </Box>
    );
  }

  if (screen === "splash") {
    return <Splash frames={frames.length > 0 ? frames : []} />;
  }

  if (screen === "first-run") {
    const m = pickMascot("auto", {
      busy,
      ok: Boolean(statusMessage && !errorMessage),
    });
    return (
      <FirstRun
        frames={m.frames}
        mascotVariant={m.variant}
        busy={busy}
        {...(statusMessage !== undefined ? { statusMessage } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
      />
    );
  }

  if (screen === "setup") {
    const packItem =
      packs.find((p) => p.name === setupPackName) ?? packs[0];
    const packTitle = packItem?.title ?? setupPackName;
    const packReason =
      recommended.find((r) => r.packName === setupPackName)?.reasons[0] ??
      story?.win ??
      "Best match for this project.";
    return (
      <Setup
        projectDir={targetProject}
        projectName={path.basename(targetProject)}
        packName={setupPackName}
        packTitle={packTitle}
        packReason={packReason}
        skillCount={skills.length}
        phase={setupPhase}
        steps={setupSteps}
        logLines={setupLog}
        actionNonce={actionNonce}
        {...(agentStatusLine !== undefined
          ? { agentLine: agentStatusLine }
          : {})}
        {...(actionFlash !== undefined ? { actionFlash } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
        {...(setupCompleteNote !== undefined
          ? { completeNote: setupCompleteNote }
          : {})}
      />
    );
  }

  if (screen === "doctor") {
    const m = pickMascot(
      doctorLoading ? "scan" : doctorReport?.ok ? "success" : "idle",
    );
    return (
      <Doctor
        frames={m.frames}
        mascotVariant={m.variant}
        loading={doctorLoading}
        {...(doctorReport !== undefined ? { report: doctorReport } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
        {...(actionFlash !== undefined ? { actionFlash } : {})}
        actionNonce={actionNonce}
      />
    );
  }

  if (screen === "paths") {
    const m = pickMascot("auto", {
      busy: pathLoading || linking,
      ok: Boolean(linkResult && !linkResult.dryRun),
    });
    return (
      <Paths
        frames={m.frames}
        mascotVariant={m.variant}
        selectedHarnessIndex={selectedHarnessIndex}
        selectTick={selectTick}
        selectDirection={selectDirection}
        scope={pathScope}
        confirmWrite={confirmLinkWrite}
        loading={pathLoading}
        linking={linking}
        actionNonce={actionNonce}
        {...(pathTargetRoot !== undefined ? { targetRoot: pathTargetRoot } : {})}
        {...(pathReport !== undefined ? { report: pathReport } : {})}
        {...(linkResult !== undefined ? { linkResult } : {})}
        {...(statusMessage !== undefined ? { statusMessage } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
        {...(actionFlash !== undefined ? { actionFlash } : {})}
      />
    );
  }

  if (screen === "help") {
    const m = pickMascot("idle");
    return (
      <Help
        frames={m.frames}
        mascotVariant={m.variant}
        fromScreen={helpFrom}
      />
    );
  }

  if (screen === "workbench") {
    return (
      <Workbench
        projectDir={targetProject}
        lane={workbenchLane}
        packs={packs}
        selectedPackIndex={selectedPackIndex}
        recommended={recommended}
        appliedNames={new Set(applied.map((a) => a.name))}
        runners={codingRunners}
        serviceTasks={workbenchTasks}
        selectedRunnerIndex={selectedRunnerIndex}
        selectedTaskIndex={selectedTaskIndex}
        selectedModelIndex={selectedModelIndex}
        selectedOpsIndex={selectedOpsIndex}
        mode={jobMode}
        prompt={jobPrompt}
        editingPrompt={editingJobPrompt}
        inputMode={terminalInputMode}
        confirmBuild={confirmBuildJob}
        busy={busy}
        runStatus={workbenchRunStatus}
        outputScroll={workbenchOutputScroll}
        opsConfirm={terminalOpsConfirm}
        historyMode={historyMode}
        recentRuns={recentRuns}
        selectedRunIndex={selectedRunIndex}
        actionNonce={actionNonce}
        skillCount={skills.length}
        {...(actionFlash !== undefined ? { actionFlash } : {})}
        {...(errorMessage
          ? { actionIsError: true }
          : {})}
        {...(pressedId !== undefined ? { pressedId } : {})}
        {...(agentStatusLine || doctorSummary
          ? {
              setupLine: [
                path.basename(targetProject),
                agentStatusLine ? `agents ${agentStatusLine}` : null,
                doctorSummary ?? null,
                `${skills.length} skills`,
              ]
                .filter(Boolean)
                .join(" · "),
            }
          : {
              setupLine: `${path.basename(targetProject)} · ${skills.length} skills`,
            })}
        {...(story
          ? {
              nextLine:
                story.id === "chaos-cleanup"
                  ? "Unify — clean skill dumps"
                  : story.id === "multi-agent-sync"
                    ? "Link agents — same skills in every agent"
                    : story.id === "already-solid"
                      ? "Doctor or run an agent job"
                      : "Quickstart — install · apply · link",
              storyTitle: story.title,
            }
          : {})}
        {...(workbenchRunLabel !== undefined
          ? { runLabel: workbenchRunLabel }
          : {})}
        {...(workbenchOutput !== undefined
          ? { output: workbenchOutput }
          : {})}
        {...(workbenchError !== undefined
          ? { errorMessage: workbenchError }
          : {})}
      />
    );
  }

  if (screen === "library") {
    const m = pickMascot(
      skills.length === 0
        ? "idle"
        : statusMessage && !errorMessage
          ? "success"
          : "idle",
    );
    return (
      <Library
        skills={skills}
        selectedIndex={selectedSkillIndex}
        selectTick={selectTick}
        selectDirection={selectDirection}
        frames={m.frames}
        mascotVariant={m.variant}
        confirmRemove={confirmRemove}
        actionNonce={actionNonce}
        {...(statusMessage !== undefined ? { statusMessage } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
        {...(libraryError !== undefined ? { libraryError } : {})}
        {...(actionFlash !== undefined ? { actionFlash } : {})}
        {...(lastChecks !== undefined ? { lastChecks } : {})}
      />
    );
  }

  if (screen === "packs") {
    const m = pickMascot("auto", {
      busy,
      ok: Boolean(statusMessage && !errorMessage && !busy),
    });
    return (
      <Packs
        packs={packs}
        selectedIndex={selectedPackIndex}
        selectTick={selectTick}
        selectDirection={selectDirection}
        frames={m.frames}
        mascotVariant={m.variant}
        recommended={recommended}
        filter={packFilter}
        filtering={filteringPacks}
        appliedNames={new Set(applied.map((a) => a.name))}
        busy={busy}
        actionNonce={actionNonce}
        {...(progress !== undefined ? { progress } : {})}
        {...(statusMessage !== undefined ? { statusMessage } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
        {...(packsError !== undefined ? { packsError } : {})}
        {...(actionFlash !== undefined ? { actionFlash } : {})}
      />
    );
  }

  if (screen === "explore") {
    const m = pickMascot(exploreLoading ? "scan" : "idle");
    return (
      <Explore
        frames={m.frames}
        mascotVariant={m.variant}
        packs={remotePacks}
        selectedIndex={selectedRemoteIndex}
        selectTick={selectTick}
        selectDirection={selectDirection}
        loading={exploreLoading}
        registryUrl={getRegistryUrl()}
        query={exploreQuery}
        {...(statusMessage !== undefined ? { statusMessage } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
      />
    );
  }

  const homeMascot = pickMascot("auto", {
    busy,
    ok: Boolean(
      celebrateCount !== undefined || (statusMessage && !errorMessage && !busy),
    ),
  });

  return (
    <Home
      frames={homeMascot.frames}
      mascotVariant={homeMascot.variant}
      skills={skills}
      packs={packs}
      applied={applied}
      selectedPackIndex={selectedPackIndex}
      selectTick={selectTick}
      selectDirection={selectDirection}
      recommended={recommended}
      skillRecs={skillRecs}
      topPick={topPick}
      targetProject={targetProject}
      pointingProject={pointingProject}
      pointDraft={pointDraft}
      busy={busy}
      statusIsError={Boolean(errorMessage)}
      confirmAction={homeConfirm}
      {...(recommendSummary !== undefined ? { recommendSummary } : {})}
      {...(userLogin !== undefined ? { userLogin } : {})}
      {...(doctorSummary !== undefined ? { doctorSummary } : {})}
      {...(progress !== undefined ? { progress } : {})}
      {...(libraryError !== undefined ? { libraryError } : {})}
      {...(packsError !== undefined ? { packsError } : {})}
      {...(celebrateCount !== undefined ? { celebrateCount } : {})}
      {...(actionFlash !== undefined ? { actionFlash } : {})}
      actionNonce={actionNonce}
      {...(statusMessage !== undefined
        ? { statusMessage: errorMessage ?? statusMessage }
        : errorMessage !== undefined
          ? { statusMessage: errorMessage }
          : {})}
      {...(agentStatusLine !== undefined ? { agentStatusLine } : {})}
      {...(story !== undefined ? { story } : {})}
      {...(planLines !== undefined ? { planLines } : {})}
    />
  );
}
