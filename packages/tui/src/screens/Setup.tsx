import React from "react";
import { Box, Text } from "ink";
import { KIT_PACKAGE_VERSION } from "@mzwin/kit-shared";
import { Spinner } from "../components/Motion.js";
import { ActionFlash } from "../motion/index.js";
import { useLayoutScale } from "../mascot/useLayoutScale.js";
import { theme, mark, stepMark, stepTone } from "../theme.js";

export type SetupPhase =
  | "ready"
  | "running"
  | "confirm"
  | "done"
  | "failed";

export interface SetupStep {
  id: string;
  label: string;
  status: "pending" | "running" | "done" | "failed" | "skipped";
}

export interface SetupProps {
  projectDir: string;
  projectName: string;
  packName: string;
  packTitle: string;
  packReason: string;
  skillCount: number;
  agentLine?: string;
  phase: SetupPhase;
  steps: SetupStep[];
  logLines: string[];
  actionFlash?: string;
  actionNonce?: number;
  errorMessage?: string;
  completeNote?: string;
}

function shortPath(value: string): string {
  const home = process.env.USERPROFILE ?? process.env.HOME ?? "";
  if (home && value.startsWith(home)) {
    return `~${value.slice(home.length).replace(/\\/g, "/")}`;
  }
  return value.replace(/\\/g, "/");
}

/** Left label column like marketing ads (OK PACK INSTALL · ESSENTIALS). */
function Field(props: {
  label: string;
  value: string;
  accent?: boolean;
}): React.ReactElement {
  const pad = props.label.padEnd(9, " ");
  return (
    <Text wrap="truncate">
      <Text dimColor>{pad}</Text>
      <Text
        bold={Boolean(props.accent)}
        {...(props.accent ? { color: theme.accent } : {})}
      >
        {props.value}
      </Text>
    </Text>
  );
}

/**
 * Default Kit surface — ink-console setup (matches docs/assets ads).
 *
 * One job: install skills + link agents for this repo.
 * Visual DNA: pixel marketing, fox-orange, command STE — not Clack cyan.
 */
export function Setup({
  projectDir,
  projectName,
  packName,
  packTitle,
  packReason,
  skillCount,
  agentLine,
  phase,
  steps,
  logLines,
  actionFlash,
  actionNonce = 0,
  errorMessage,
  completeNote,
}: SetupProps): React.ReactElement {
  const scale = useLayoutScale();
  const path = shortPath(projectDir);
  const logMax = Math.max(3, Math.min(7, scale.rows - 17));
  const visibleLog = logLines.slice(-logMax);
  const rule = "─".repeat(Math.max(12, Math.min(scale.columns - 4, 52)));

  return (
    <Box
      flexDirection="column"
      width="100%"
      height={scale.rows}
      paddingX={2}
      paddingY={1}
      overflow="hidden"
    >
      {/* Title bar — KIT wordmark + command */}
      <Box justifyContent="space-between" width="100%">
        <Text>
          <Text bold inverse>
            {" "}
            KIT{" "}
          </Text>
          <Text bold color={theme.accent}>
            {" "}
            SETUP
          </Text>
          <Text dimColor> --DIR {path}</Text>
        </Text>
        <Text dimColor>v{KIT_PACKAGE_VERSION}</Text>
      </Box>

      {/* Orange rule — brand accent from marketing banners */}
      <Text color={theme.accent}>{rule}</Text>

      {/* Project block — tabular fields */}
      <Box flexDirection="column" marginTop={1}>
        <Field label="PROJECT" value={projectName} />
        <Field label="DIR" value={path} />
        <Field
          label="AGENTS"
          value={agentLine?.toUpperCase() ?? "UNCHECKED"}
        />
        <Field label="LIBRARY" value={`${skillCount} SKILLS`} />
      </Box>

      {/* Recommended pack — arrow mark like kit recommend */}
      <Box flexDirection="column" marginTop={1}>
        <Text>
          <Text bold color={theme.accent}>
            {mark.arrow}{" "}
          </Text>
          <Text bold>{packTitle.toUpperCase()}</Text>
          <Text dimColor>  {packName}</Text>
        </Text>
        <Text dimColor wrap="truncate">
          {"  "}
          {packReason || "Best match for this project."}
        </Text>
      </Box>

      {/* Plan — OK rows when done, · when pending (ad-ready language) */}
      <Box flexDirection="column" marginTop={1}>
        <Text bold dimColor>
          PLAN
        </Text>
        {steps.map((s) => {
          const m = stepMark(s.status);
          const tone = stepTone(s.status);
          const soft = s.status === "pending" || s.status === "skipped";
          return (
            <Text key={s.id} wrap="truncate">
              <Text
                bold={s.status === "done" || s.status === "running"}
                dimColor={soft}
                {...(tone ? { color: tone } : {})}
              >
                {m.padEnd(3, " ")}
              </Text>
              <Text dimColor={soft}>{s.label}</Text>
            </Text>
          );
        })}
      </Box>

      <ActionFlash message={actionFlash} nonce={actionNonce} />

      {/* Log — grows to push CTA to bottom */}
      <Box flexDirection="column" flexGrow={1} overflow="hidden" marginTop={0}>
        <Text bold dimColor>
          LOG
          {phase === "running"
            ? " · WORKING"
            : phase === "done"
              ? " · DONE"
              : phase === "failed"
                ? " · FAILED"
                : phase === "confirm"
                  ? " · CONFIRM"
                  : ""}
        </Text>
        {phase === "running" ? (
          <Spinner active label="Setup" style="icon" />
        ) : null}
        {visibleLog.length === 0 ? (
          <Text dimColor>
            Press Enter to plan. Then press y to install and link.
          </Text>
        ) : (
          visibleLog.map((line, i) => (
            <Text key={`${i}-${line.slice(0, 24)}`} dimColor wrap="truncate">
              {line}
            </Text>
          ))
        )}
        {errorMessage ? (
          <Text bold color={theme.error} wrap="truncate">
            {mark.fail} {errorMessage}
          </Text>
        ) : null}
        {completeNote ? (
          <Text bold color={theme.accent} wrap="truncate">
            {completeNote.toUpperCase()}
          </Text>
        ) : null}
      </Box>

      {/* Primary CTA — inverse + orange (one action) */}
      <Box flexDirection="column" marginTop={1}>
        {phase === "ready" ? (
          <Text bold inverse color={theme.accent}>
            {" "}
            {mark.cta} ENTER · SETUP THIS PROJECT{" "}
          </Text>
        ) : null}
        {phase === "confirm" ? (
          <Text bold inverse color={theme.warning}>
            {" "}
            {mark.cta} Y · WRITE (INSTALL + LINK) · N CANCEL{" "}
          </Text>
        ) : null}
        {phase === "running" ? (
          <Text bold color={theme.accent}>
            {" "}
            WORKING… ESC OR CTRL+C TO STOP{" "}
          </Text>
        ) : null}
        {phase === "done" ? (
          <Text bold inverse color={theme.success}>
            {" "}
            {mark.cta} ENTER · FINISH · A ADVANCED · Q QUIT{" "}
          </Text>
        ) : null}
        {phase === "failed" ? (
          <Text bold inverse color={theme.error}>
            {" "}
            {mark.cta} ENTER · RETRY · A ADVANCED · Q QUIT{" "}
          </Text>
        ) : null}
        <Text dimColor wrap="truncate">
          p pack {mark.sep} a advanced {mark.sep} q quit
        </Text>
      </Box>
    </Box>
  );
}

/** Default checklist — labels mirror kit ready CLI / ads. */
export function defaultSetupSteps(packTitle: string): SetupStep[] {
  return [
    {
      id: "pack-install",
      label: `PACK INSTALL   ${packTitle}`,
      status: "pending",
    },
    {
      id: "pack-apply",
      label: "PACK APPLY     this project",
      status: "pending",
    },
    {
      id: "link",
      label: "LINK           Claude · Codex · Grok",
      status: "pending",
    },
    {
      id: "doctor",
      label: "DOCTOR         health",
      status: "pending",
    },
  ];
}
