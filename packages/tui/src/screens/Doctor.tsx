import React from "react";
import { Box, Text } from "ink";
import type { DoctorReport } from "@mzwin/kit-core";
import type { PixelFrame } from "../mascot/types.js";
import { MascotPlayer } from "../mascot/MascotPlayer.js";
import { Footer, Header } from "../components/Chrome.js";
import { ErrorLine, Spinner } from "../components/Motion.js";
import { ActionFlash } from "../motion/index.js";

export interface DoctorProps {
  frames: PixelFrame[];
  report?: DoctorReport;
  loading?: boolean;
  errorMessage?: string;
  actionFlash?: string;
  actionNonce?: number;
}

function mark(level: string): string {
  if (level === "pass") return "✓";
  if (level === "fail") return "✗";
  if (level === "warn") return "!";
  return "·";
}

/**
 * Health console — same checks as `kit doctor`.
 */
export function Doctor({
  frames,
  report,
  loading,
  errorMessage,
  actionFlash,
  actionNonce = 0,
}: DoctorProps): React.ReactElement {
  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Header
        screen="Doctor"
        {...(report
          ? { detail: report.ok ? "healthy" : "issues" }
          : loading
            ? { detail: "running…" }
            : {})}
      />

      <Box marginTop={1} flexDirection="row">
        <Box marginRight={2} flexShrink={0}>
          <MascotPlayer frames={frames} playing size="compact" />
        </Box>

        <Box flexDirection="column" flexGrow={1}>
          <Text bold>Install health</Text>
          <Text dimColor>Same checks as kit doctor · r re-run</Text>

          {loading ? (
            <Box marginTop={1}>
              <Spinner label="Running checks" active />
            </Box>
          ) : null}

          {report ? (
            <Box marginTop={1} flexDirection="column">
              <Text dimColor>
                {report.summary.passed} pass · {report.summary.warnings} warn ·{" "}
                {report.summary.failed} fail
              </Text>
              {report.checks.map((c) => (
                <Text key={`${c.id}-${c.message.slice(0, 24)}`}>
                  {mark(c.level)} {c.message}
                </Text>
              ))}
            </Box>
          ) : null}

          <Box marginTop={1}>
            <ActionFlash message={actionFlash} nonce={actionNonce} />
          </Box>
        </Box>
      </Box>

      {errorMessage ? (
        <Box marginTop={1}>
          <ErrorLine message={errorMessage} />
        </Box>
      ) : null}

      <Footer keys="r re-run · k paths · h home · p packs · q quit" />
    </Box>
  );
}
