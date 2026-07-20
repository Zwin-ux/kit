import React, { useEffect, useState } from "react";
import { useInput, useApp, Box, Text } from "ink";
import { listSkills, type InstalledSkill } from "@kit-skills/core";
import { loadMascotFrames } from "./mascot/loadFrames.js";
import { FRAME_DELAY_MS, type PixelFrame } from "./mascot/types.js";
import { Splash } from "./screens/Splash.js";
import { Home } from "./screens/Home.js";

type Screen = "loading" | "splash" | "home";

export function App(): React.ReactElement {
  const { exit } = useApp();
  const [screen, setScreen] = useState<Screen>("loading");
  const [frames, setFrames] = useState<PixelFrame[]>([]);
  const [frameIndex, setFrameIndex] = useState(0);
  const [usingFiles, setUsingFiles] = useState(false);
  const [skills, setSkills] = useState<InstalledSkill[]>([]);
  const [libraryError, setLibraryError] = useState<string | undefined>();
  const [loadError, setLoadError] = useState<string | undefined>();

  useEffect(() => {
    let cancelled = false;

    (async () => {
      try {
        const loaded = await loadMascotFrames();
        if (cancelled) return;
        setFrames(loaded.frames);
        setUsingFiles(loaded.usedFiles);
        setScreen("splash");
      } catch (error) {
        if (cancelled) return;
        const detail = error instanceof Error ? error.message : String(error);
        setLoadError(detail);
        setScreen("splash");
      }

      const listed = await listSkills();
      if (cancelled) return;
      if (listed.ok) {
        setSkills(listed.value);
      } else {
        setLibraryError(listed.error);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (frames.length === 0) return;
    const timer = setInterval(() => {
      setFrameIndex((current) => (current + 1) % frames.length);
    }, FRAME_DELAY_MS);
    return () => clearInterval(timer);
  }, [frames.length]);

  useInput((input, key) => {
    if (input === "q" || (key.ctrl && input === "c")) {
      exit();
      return;
    }

    if (screen === "loading") return;

    if (screen === "splash") {
      setScreen("home");
      return;
    }

    if (screen === "home" && input === "s") {
      setScreen("splash");
    }
  });

  const frame = frames[frameIndex] ?? frames[0];

  if (screen === "loading" || !frame) {
    return (
      <Box padding={1}>
        <Text>Loading Kit…</Text>
        {loadError ? <Text color="red"> {loadError}</Text> : null}
      </Box>
    );
  }

  if (screen === "splash") {
    return (
      <Splash
        frame={frame}
        frameIndex={frameIndex}
        frameCount={frames.length}
        usingFiles={usingFiles}
      />
    );
  }

  return (
    <Home
      frame={frame}
      skills={skills}
      {...(libraryError !== undefined ? { libraryError } : {})}
    />
  );
}
