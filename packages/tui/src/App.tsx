import React, { useCallback, useEffect, useState } from "react";
import { useInput, useApp, Box, Text } from "ink";
import {
  completeFirstRun,
  getFirstRunStatus,
  installPack,
  applyPack,
  listPacks,
  listSkills,
  readProjectAppliedPacks,
  removeSkill,
  type AppliedPackRecord,
  type InstalledSkill,
  type PackListItem,
} from "@kit-skills/core";
import { loadMascotFrames } from "./mascot/loadFrames.js";
import type { PixelFrame } from "./mascot/types.js";
import { Splash } from "./screens/Splash.js";
import { Home } from "./screens/Home.js";
import { FirstRun } from "./screens/FirstRun.js";
import { Library } from "./screens/Library.js";
import { Packs } from "./screens/Packs.js";

type Screen =
  | "loading"
  | "splash"
  | "first-run"
  | "home"
  | "library"
  | "packs";

export function App(): React.ReactElement {
  const { exit } = useApp();
  const [screen, setScreen] = useState<Screen>("loading");
  const [frames, setFrames] = useState<PixelFrame[]>([]);
  const [usingFiles, setUsingFiles] = useState(false);
  const [skills, setSkills] = useState<InstalledSkill[]>([]);
  const [packs, setPacks] = useState<PackListItem[]>([]);
  const [applied, setApplied] = useState<AppliedPackRecord[]>([]);
  const [selectedPackIndex, setSelectedPackIndex] = useState(0);
  const [selectedSkillIndex, setSelectedSkillIndex] = useState(0);
  const [confirmRemove, setConfirmRemove] = useState(false);
  const [libraryError, setLibraryError] = useState<string | undefined>();
  const [packsError, setPacksError] = useState<string | undefined>();
  const [loadError, setLoadError] = useState<string | undefined>();
  const [statusMessage, setStatusMessage] = useState<string | undefined>();
  const [errorMessage, setErrorMessage] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const [offerFirstRun, setOfferFirstRun] = useState(false);

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
    if (offerFirstRun) {
      setScreen("first-run");
    } else {
      setScreen("home");
    }
  }, [offerFirstRun]);

  const installSelectedPack = useCallback(
    async (packName: string, mode: "install" | "apply") => {
      if (busy) return;
      setBusy(true);
      clearFlash();
      setStatusMessage(
        mode === "apply"
          ? `Applying ${packName} to this project…`
          : `Installing ${packName}…`,
      );

      try {
        if (mode === "apply") {
          const result = await applyPack(packName, { force: true });
          if (!result.ok) {
            setErrorMessage(result.error);
            setStatusMessage(undefined);
            return;
          }
          setStatusMessage(
            `${result.value.reapplied ? "Reapplied" : "Applied"} ${result.value.pack.name}@${result.value.pack.version} (${result.value.installed.length} skills)`,
          );
        } else {
          const result = await installPack(packName, { force: true });
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
          setStatusMessage(
            "First-run skipped. Install a pack anytime from Home or Packs.",
          );
          setScreen("home");
        })();
        return;
      }
      if (input === "1") void installSelectedPack("essentials", "install");
      if (input === "2") void installSelectedPack("web-app", "install");
      if (input === "3") void installSelectedPack("library", "install");
      return;
    }

    // Global navigation from main screens
    if (
      screen === "home" ||
      screen === "library" ||
      screen === "packs"
    ) {
      if (input === "s") {
        setConfirmRemove(false);
        setScreen("splash");
        return;
      }
      if (input === "h") {
        setConfirmRemove(false);
        setScreen("home");
        return;
      }
      if (input === "l") {
        setConfirmRemove(false);
        setScreen("library");
        return;
      }
      if (input === "p") {
        setConfirmRemove(false);
        setScreen("packs");
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
      if (input === "r") {
        if (skills[selectedSkillIndex]) {
          setConfirmRemove(true);
        }
      }
    }
  });

  if (screen === "loading" || frames.length === 0) {
    return (
      <Box padding={1} flexDirection="column">
        <Text>Loading Kit…</Text>
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
        busy={busy}
        {...(statusMessage !== undefined ? { statusMessage } : {})}
        {...(errorMessage !== undefined ? { errorMessage } : {})}
        {...(packsError !== undefined ? { packsError } : {})}
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
      busy={busy}
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
