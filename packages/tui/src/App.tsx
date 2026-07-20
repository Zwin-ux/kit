import React, { useCallback, useEffect, useState } from "react";
import { useInput, useApp, Box, Text } from "ink";
import path from "node:path";
import {
  completeFirstRun,
  describePaths,
  getFirstRunStatus,
  getLoggedInUser,
  getRegistryUrl,
  installPack,
  applyPack,
  linkSkills,
  listPacks,
  listSkills,
  loadSkill,
  readConfig,
  readProjectAppliedPacks,
  removeSkill,
  recommendToolkits,
  runDoctor,
  testSkill,
  updateConfig,
  exploreListPacks,
  exploreSearch,
  type AppliedPackRecord,
  type CheckResult,
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
} from "@mzwin/kit-core";
import { loadAllMascotFrames } from "./mascot/loadFrames.js";
import type { MascotVariant, PixelFrame } from "./mascot/types.js";
import { Spinner } from "./components/Motion.js";
import { Splash } from "./screens/Splash.js";
import { Home } from "./screens/Home.js";
import { FirstRun } from "./screens/FirstRun.js";
import { Library } from "./screens/Library.js";
import { Packs } from "./screens/Packs.js";
import { Explore } from "./screens/Explore.js";
import { Doctor } from "./screens/Doctor.js";
import {
  Paths,
  PATHS_LINKABLE_HARNESSES,
} from "./screens/Paths.js";

type Screen =
  | "loading"
  | "splash"
  | "first-run"
  | "home"
  | "library"
  | "packs"
  | "explore"
  | "doctor"
  | "paths";

const FIRST_RUN_BY_KEY: Record<string, string> = {
  "1": "essentials",
  "2": "web-app",
  "3": "library",
  "4": "cli-tool",
  "5": "api-service",
  "6": "full-stack",
  "7": "data-ml",
};

export function App(): React.ReactElement {
  const { exit } = useApp();
  const [screen, setScreen] = useState<Screen>("loading");
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

  const clearFlash = useCallback(() => {
    setErrorMessage(undefined);
    setCelebrateCount(undefined);
  }, []);

  const flash = useCallback((message: string) => {
    setActionNonce((n) => n + 1);
    setActionFlash(message);
  }, []);

  const bumpSelect = useCallback((direction: "up" | "down" | "none" = "none") => {
    setSelectDirection(direction);
    setSelectTick((t) => t + 1);
  }, []);

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
  }, [resolveTarget]);

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
      try {
        const loaded = await loadAllMascotFrames();
        if (cancelled) return;
        setFrames(loaded.idle);
        setScanFrames(loaded.scan);
        setSuccessFrames(loaded.success);
      } catch (error) {
        if (cancelled) return;
        const detail = error instanceof Error ? error.message : String(error);
        setLoadError(detail);
      }

      const firstRun = await getFirstRunStatus();
      if (cancelled) return;
      setOfferFirstRun(firstRun.shouldOffer);

      await refreshData();
      if (cancelled) return;

      setScreen("splash");
    })();

    return () => {
      cancelled = true;
    };
  }, [refreshData]);

  const leaveSplash = useCallback(() => {
    flash("welcome");
    if (offerFirstRun) setScreen("first-run");
    else setScreen("home");
  }, [offerFirstRun, flash]);

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
        setScreen("home");
      } finally {
        setBusy(false);
        setProgress(undefined);
      }
    },
    [busy, clearFlash, offerFirstRun, refreshData, screen, flash, targetProject],
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

  useInput((input, key) => {
    if (input === "q" || (key.ctrl && input === "c")) {
      exit();
      return;
    }

    if (screen === "loading" || busy) return;

    if (screen === "splash") {
      leaveSplash();
      return;
    }

    if (screen === "first-run") {
      if (input === "s") {
        void (async () => {
          await completeFirstRun("skipped");
          setOfferFirstRun(false);
          setStatusMessage("Skipped first-run. Pick a toolkit anytime.");
          flash("skipped");
          setScreen("home");
        })();
        return;
      }
      const packName = FIRST_RUN_BY_KEY[input];
      if (packName) {
        flash(`install ${packName}`);
        void installSelectedPack(packName, "install");
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
    ];
    if (mainScreens.includes(screen)) {
      if (input === "s") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        flash("splash");
        setScreen("splash");
        return;
      }
      if (input === "h") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        flash("home");
        setScreen("home");
        return;
      }
      if (input === "l") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        flash("library");
        setScreen("library");
        return;
      }
      if (input === "p" && screen !== "paths") {
        // on paths screen, p = plan; elsewhere p = packs
        if (screen !== "doctor") {
          setConfirmRemove(false);
          setFilteringPacks(false);
          flash("packs");
          setScreen("packs");
          return;
        }
      }
      if (input === "e") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        flash("explore");
        setScreen("explore");
        void loadExplore(exploreQuery || undefined);
        return;
      }
      if (input === "d") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        flash("doctor");
        setScreen("doctor");
        void loadDoctor();
        return;
      }
      if (input === "k") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        flash("paths");
        setScreen("paths");
        void loadPaths();
        return;
      }
    }

    if (screen === "home") {
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

  if (screen === "loading" || frames.length === 0) {
    return (
      <Box padding={1} flexDirection="column">
        <Spinner label="Loading Kit" active style="icon" />
        {loadError ? <Text color="red">{loadError}</Text> : null}
      </Box>
    );
  }

  if (screen === "splash") {
    return <Splash frames={frames} />;
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
    />
  );
}
