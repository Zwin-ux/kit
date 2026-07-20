import React, { useCallback, useEffect, useState } from "react";
import { useInput, useApp, Box, Text } from "ink";
import {
  completeFirstRun,
  getFirstRunStatus,
  getLoggedInUser,
  getRegistryUrl,
  installPack,
  applyPack,
  listPacks,
  listSkills,
  readProjectAppliedPacks,
  removeSkill,
  recommendToolkits,
  exploreListPacks,
  exploreSearch,
  type AppliedPackRecord,
  type InstalledSkill,
  type PackListItem,
  type ToolkitRecommendation,
  type RegistryPackSummary,
} from "@kit-skills/core";
import { loadMascotFrames } from "./mascot/loadFrames.js";
import type { PixelFrame } from "./mascot/types.js";
import { Spinner } from "./components/Motion.js";
import { Splash } from "./screens/Splash.js";
import { Home } from "./screens/Home.js";
import { FirstRun } from "./screens/FirstRun.js";
import { Library } from "./screens/Library.js";
import { Packs } from "./screens/Packs.js";
import { Explore } from "./screens/Explore.js";

type Screen =
  | "loading"
  | "splash"
  | "first-run"
  | "home"
  | "library"
  | "packs"
  | "explore";

export function App(): React.ReactElement {
  const { exit } = useApp();
  const [screen, setScreen] = useState<Screen>("loading");
  const [frames, setFrames] = useState<PixelFrame[]>([]);
  const [usingFiles, setUsingFiles] = useState(false);
  const [skills, setSkills] = useState<InstalledSkill[]>([]);
  const [packs, setPacks] = useState<PackListItem[]>([]);
  const [applied, setApplied] = useState<AppliedPackRecord[]>([]);
  const [recommended, setRecommended] = useState<ToolkitRecommendation[]>([]);
  const [topPick, setTopPick] = useState<string | null>(null);
  const [remotePacks, setRemotePacks] = useState<RegistryPackSummary[]>([]);
  const [exploreLoading, setExploreLoading] = useState(false);
  const [exploreQuery, setExploreQuery] = useState("");
  const [packFilter, setPackFilter] = useState("");
  const [filteringPacks, setFilteringPacks] = useState(false);
  const [selectedPackIndex, setSelectedPackIndex] = useState(0);
  const [selectedSkillIndex, setSelectedSkillIndex] = useState(0);
  const [selectedRemoteIndex, setSelectedRemoteIndex] = useState(0);
  const [confirmRemove, setConfirmRemove] = useState(false);
  const [libraryError, setLibraryError] = useState<string | undefined>();
  const [packsError, setPacksError] = useState<string | undefined>();
  const [loadError, setLoadError] = useState<string | undefined>();
  const [statusMessage, setStatusMessage] = useState<string | undefined>();
  const [errorMessage, setErrorMessage] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<
    { current: number; total: number; skillName: string } | undefined
  >();
  const [offerFirstRun, setOfferFirstRun] = useState(false);
  const [userLogin, setUserLogin] = useState<string | undefined>();

  const clearFlash = useCallback(() => {
    setErrorMessage(undefined);
  }, []);

  const refreshData = useCallback(async () => {
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

    const appliedFile = await readProjectAppliedPacks(process.cwd());
    setApplied(Object.values(appliedFile.packs));

    const rec = await recommendToolkits({ projectDir: process.cwd() });
    if (rec.ok) {
      setRecommended(rec.value.recommendations);
      setTopPick(rec.value.topPick);
      // Prefer top pick selection when library empty
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
  }, []);

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
        const loaded = await loadMascotFrames();
        if (cancelled) return;
        setFrames(loaded.frames);
        setUsingFiles(loaded.usedFiles);
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
    if (offerFirstRun) setScreen("first-run");
    else setScreen("home");
  }, [offerFirstRun]);

  const installSelectedPack = useCallback(
    async (packName: string, mode: "install" | "apply") => {
      if (busy) return;
      setBusy(true);
      clearFlash();
      setProgress(undefined);
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
          });
          if (!result.ok) {
            setErrorMessage(result.error);
            setStatusMessage(undefined);
            return;
          }
          setStatusMessage(
            `${result.value.reapplied ? "Reapplied" : "Applied"} ${result.value.pack.name}@${result.value.pack.version} (${result.value.installed.length} skills)`,
          );
        } else {
          const result = await installPack(packName, {
            force: true,
            onProgress,
          });
          if (!result.ok) {
            setErrorMessage(result.error);
            setStatusMessage(undefined);
            return;
          }
          setStatusMessage(
            `Installed ${result.value.pack.name}@${result.value.pack.version} (${result.value.installed.length} skills)`,
          );
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
    [busy, clearFlash, offerFirstRun, refreshData, screen],
  );

  const removeSelectedSkill = useCallback(async () => {
    const skill = skills[selectedSkillIndex];
    if (!skill || busy) return;
    setBusy(true);
    clearFlash();
    try {
      const result = await removeSkill(skill.name);
      if (!result.ok) {
        setErrorMessage(result.error);
        return;
      }
      setStatusMessage(`Removed ${result.value.name}`);
      setConfirmRemove(false);
      await refreshData();
    } finally {
      setBusy(false);
    }
  }, [busy, clearFlash, refreshData, selectedSkillIndex, skills]);

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
          setScreen("home");
        })();
        return;
      }
      if (input === "1") void installSelectedPack("essentials", "install");
      if (input === "2") void installSelectedPack("web-app", "install");
      if (input === "3") void installSelectedPack("library", "install");
      return;
    }

    // Pack filter typing mode
    if (screen === "packs" && filteringPacks) {
      if (key.escape) {
        setFilteringPacks(false);
        setPackFilter("");
        return;
      }
      if (key.return) {
        setFilteringPacks(false);
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

    // Explore search typing (prefix with /)
    if (screen === "explore" && exploreQuery.startsWith("/")) {
      if (key.escape) {
        setExploreQuery("");
        void loadExplore();
        return;
      }
      if (key.return) {
        const q = exploreQuery.slice(1);
        setExploreQuery(q);
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
    ];
    if (mainScreens.includes(screen)) {
      if (input === "s") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setScreen("splash");
        return;
      }
      if (input === "h") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setScreen("home");
        return;
      }
      if (input === "l") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setScreen("library");
        return;
      }
      if (input === "p") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setScreen("packs");
        return;
      }
      if (input === "e") {
        setConfirmRemove(false);
        setFilteringPacks(false);
        setScreen("explore");
        void loadExplore(exploreQuery || undefined);
        return;
      }
    }

    if (screen === "home") {
      if (key.upArrow) {
        setSelectedPackIndex((i) =>
          packs.length === 0 ? 0 : (i - 1 + packs.length) % packs.length,
        );
        return;
      }
      if (key.downArrow) {
        setSelectedPackIndex((i) =>
          packs.length === 0 ? 0 : (i + 1) % packs.length,
        );
        return;
      }
      if (input === "i") {
        const pack = packs[selectedPackIndex];
        if (pack) void installSelectedPack(pack.name, "install");
        return;
      }
      if (input === "a") {
        const pack = packs[selectedPackIndex];
        if (pack) void installSelectedPack(pack.name, "apply");
        return;
      }
      if (input === "1" || input === "2" || input === "3") {
        const index = Number(input) - 1;
        const pack = packs[index];
        if (pack) {
          setSelectedPackIndex(index);
          void installSelectedPack(pack.name, "install");
        }
      }
      return;
    }

    if (screen === "packs") {
      if (input === "/" || (input && /^[a-zA-Z]$/.test(input) && !key.ctrl)) {
        setFilteringPacks(true);
        if (input !== "/") setPackFilter((f) => f + input);
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
        return;
      }
      if (key.downArrow) {
        setSelectedPackIndex((i) =>
          packs.length === 0 ? 0 : (i + 1) % packs.length,
        );
        return;
      }
      if (input === "i") {
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
        void loadExplore(exploreQuery || undefined);
        return;
      }
      if (key.upArrow) {
        setSelectedRemoteIndex((i) =>
          remotePacks.length === 0
            ? 0
            : (i - 1 + remotePacks.length) % remotePacks.length,
        );
        return;
      }
      if (key.downArrow) {
        setSelectedRemoteIndex((i) =>
          remotePacks.length === 0
            ? 0
            : (i + 1) % remotePacks.length,
        );
        return;
      }
      if (input === "i") {
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
          return;
        }
        return;
      }
      if (key.upArrow) {
        setSelectedSkillIndex((i) =>
          skills.length === 0 ? 0 : (i - 1 + skills.length) % skills.length,
        );
        return;
      }
      if (key.downArrow) {
        setSelectedSkillIndex((i) =>
          skills.length === 0 ? 0 : (i + 1) % skills.length,
        );
        return;
      }
      if (input === "r" && skills[selectedSkillIndex]) {
        setConfirmRemove(true);
      }
    }
  });

  if (screen === "loading" || frames.length === 0) {
    return (
      <Box padding={1} flexDirection="column">
        <Spinner label="Loading Kit" active />
        {loadError ? <Text color="red">{loadError}</Text> : null}
      </Box>
    );
  }

  if (screen === "splash") {
    return <Splash frames={frames} usingFiles={usingFiles} />;
  }

  if (screen === "first-run") {
    return (
      <FirstRun
        frames={frames}
        busy={busy}
        {...(statusMessage !== undefined ? { statusMessage } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
      />
    );
  }

  if (screen === "library") {
    return (
      <Library
        skills={skills}
        selectedIndex={selectedSkillIndex}
        frames={frames}
        confirmRemove={confirmRemove}
        {...(statusMessage !== undefined ? { statusMessage } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
        {...(libraryError !== undefined ? { libraryError } : {})}
      />
    );
  }

  if (screen === "packs") {
    return (
      <Packs
        packs={packs}
        selectedIndex={selectedPackIndex}
        frames={frames}
        recommended={recommended}
        filter={packFilter}
        appliedNames={new Set(applied.map((a) => a.name))}
        busy={busy}
        {...(progress !== undefined ? { progress } : {})}
        {...(statusMessage !== undefined ? { statusMessage } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
        {...(packsError !== undefined ? { packsError } : {})}
      />
    );
  }

  if (screen === "explore") {
    return (
      <Explore
        frames={frames}
        packs={remotePacks}
        selectedIndex={selectedRemoteIndex}
        loading={exploreLoading}
        registryUrl={getRegistryUrl()}
        query={exploreQuery}
        {...(statusMessage !== undefined ? { statusMessage } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
      />
    );
  }

  return (
    <Home
      frames={frames}
      skills={skills}
      packs={packs}
      applied={applied}
      selectedPackIndex={selectedPackIndex}
      recommended={recommended}
      topPick={topPick}
      busy={busy}
      {...(userLogin !== undefined ? { userLogin } : {})}
      {...(progress !== undefined ? { progress } : {})}
      {...(libraryError !== undefined ? { libraryError } : {})}
      {...(packsError !== undefined ? { packsError } : {})}
      {...(statusMessage !== undefined
        ? { statusMessage: errorMessage ?? statusMessage }
        : errorMessage !== undefined
          ? { statusMessage: errorMessage }
          : {})}
    />
  );
}
