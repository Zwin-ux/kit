import type { AgentInfo, EngineInterface, Register, Timer } from 'claude-code'
import { agents, buckets, skills, catalogText, findAgent, invocation, type Specialist } from './catalog.ts'
import { initialState, reconcile, recordResult, resultDraft, resultStatus, statusLabel, type ResultSnapshot } from './state.ts'
import { bandLayout, bandRows } from './band.ts'
import { foxFrameAt, foxRaster, reduceMotion, type FoxFrame } from './fox.ts'
// Keep API-calling helpers in this file: 2.1.287's static analyzer cannot follow $ across imports.
let state = initialState()
let knownAttempts = new Map<string, AgentInfo>()
let pendingResults: Pick<ResultSnapshot, 'agentId' | 'turnId' | 'answer' | 'reason'>[] = []
let draftTransaction: number | undefined
let nextDraftTransaction = 0
let generation = 0
let latestRefresh = 0
let commandReady = false
let foxTimer: Timer | undefined
let frame: FoxFrame = 'idle'
let elapsed = 0
function stopFoxMotion() {
  foxTimer?.cancel(); foxTimer = undefined; frame = 'idle'; elapsed = 0
}

function capturePending(jobs: AgentInfo[]) {
  pendingResults = pendingResults.filter(result => {
    const observed = jobs.find(j => j.id === result.agentId)
    if (observed && !agents.some(a => a.id === observed.type)) return false
    const job = observed && observed.status !== 'not-attached' ? observed : knownAttempts.get(result.agentId)
    if (!job) return true
    if (!state.jobs.some(j => j.id === job.id)) {
      state = { ...reconcile(state, [...state.jobs, { ...job, status: 'not-attached' }]), statusError: state.statusError }
    }
    state = recordResult(state, { ...result, agentType: job.type, task: job.description })
    return false
  })
}

async function refresh($: EngineInterface) {
  const current = generation
  const request = ++latestRefresh
  try {
    const jobs = await $.agent.list()
    if (current === generation) {
      for (const job of jobs) {
        if (!agents.some(a => a.id === job.type)) knownAttempts.delete(job.id)
        else if (job.status !== 'not-attached') knownAttempts.set(job.id, job)
      }
      // Keep verified identity when native foreground attempts are pruned.
      capturePending(jobs)
      if (request === latestRefresh) {
        state = reconcile(state, jobs)
        if (state.jobs.length) stopFoxMotion()
      }
      return jobs
    }
  } catch {
    if (current === generation && request === latestRefresh) state = { ...state, statusError: 'Native status is unavailable. No job state inferred.' }
  }
}

type DraftOutcome = { text: string; filled: boolean }

// Lock Kit actions across read → append. Never replay or replace the user draft.
// A reset invalidates the generation,
// while the unique token prevents an old finally block from unlocking a newer draft.
async function prepareDraft($: EngineInterface, agent: Specialist, source?: ResultSnapshot, action: 'review' | 'handoff' = 'review'): Promise<DraftOutcome> {
  if (draftTransaction !== undefined) return { text: 'A draft action is already in progress.', filled: false }
  const transaction = ++nextDraftTransaction
  const current = generation
  draftTransaction = transaction
  const cancelled = 'Session changed. Draft preparation cancelled; no old task was restored.'
  try {
    const draft = await $.prompt.read()
    if (current !== generation || draftTransaction !== transaction) return { text: cancelled, filled: false }
    if (source && draft.text.trim()) return { text: 'Your draft contains text. Send or clear it before preparing a result handoff.', filled: false }
    if (!source && /@agent-[^\s]+|@"[^"]+\(agent\)"/.test(draft.text)) {
      return { text: 'Your draft already contains an agent mention. Edit it in the native prompt. Nothing changed.', filled: false }
    }
    const text = source ? `\n${resultDraft(source, agent, action)}` : invocation(agent)
    const result = await $.prompt.fill({ text, mode: 'append' })
    if (current !== generation || draftTransaction !== transaction) return { text: 'Session changed while the native fill was pending. Inspect the current prompt.', filled: false }
    return { filled: result.isFilled, text: result.isFilled
      ? 'Agent mention added. Review the visible draft and press Enter to invoke the native agent.'
      : 'Claude could not fill the draft here. Nothing started. Use this in an interactive session.' }
  } finally {
    if (draftTransaction === transaction) draftTransaction = undefined
  }
}

function select(bucket: string) { state = { ...state, selected: findAgent(bucket)!.id } }
function resetState() { generation++; state = initialState(); knownAttempts.clear(); pendingResults = []; draftTransaction = undefined }
async function prepare($: EngineInterface) {
  return (await prepareDraft($, findAgent(state.selected)!)).text
}
async function review($: EngineInterface, key: string) {
  const result = state.results.find(r => r.key === key)
  if (!result) return 'This result is no longer available.'
  return (await prepareDraft($, findAgent('Security')!, result, 'review')).text
}
const paneId = 'kit-specialists';
const colors = { active: 'claude', text: 'text', muted: 'subtle', inactive: 'inactive', divider: 'subtle' };
function bucketWidths(columns: number): number[] {
    const width = Math.max(0, Math.floor(columns));
    const base = Math.floor(width / 4);
    return [base, base, base, width - base * 3];
}
async function openKit($: EngineInterface) {
    const current = generation;
    await refresh($);
    if (!commandReady || current !== generation) return;
    await $.ui.open({ id: paneId, title: 'Kit · Skills', columns: 30, rows: 22, focus: true, closeOnEscape: true });
}
// Native sites/control focus only. No keyboard, prompt, permission or auth interception.
function registerUi(on: Parameters<Register>[0]) {
    let expanded = true;
    let reduced = false;
    let notice = '';
    let epoch = 0;
    const reset = () => {
        epoch++; stopFoxMotion();
        expanded = true; notice = '';
    };
    on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
        // The host props are read-only: call downstream unchanged and preserve its tree.
        // Source: official interface docs, "Band above the prompt".
        const downstream = await next(e);
        if (!commandReady || e.props.hasSurvey || e.props.bodyColumns < 12 || e.props.maxRows < 1)
            return downstream;
        const downstreamRows = bandRows(downstream, e.props.bodyColumns);
        if (downstreamRows === undefined || downstreamRows >= e.props.maxRows)
            return downstream;
        const { Box, Text, Button } = $.ui.resolve(e);
        const selected = agents.find(a => a.id === state.selected)!;
        const padding = e.props.bodyColumns >= 44 ? 2 : 0;
        const usable = e.props.bodyColumns - padding * 2;
        const layout = bandLayout(usable, e.props.maxRows - downstreamRows);
        const rows = layout.rows === 2 ? [buckets.slice(0, 2), buckets.slice(2)] : [buckets];
        const widths = bucketWidths(usable);
        // Each full role label needs ten cells including its selection marker.
        if ((layout.rows === 2 ? Math.floor(usable / 2) : widths[0]!) < 10)
            return Box({ key: 'kit-composed-band', flexDirection: 'column',
                children: [downstream, Box({ key: 'kit-opener-row', height: 1, flexShrink: 0,
                    children: [Button({ key: 'kit-open', plain: true, label: 'Kit', onPress: () => openKit($) })] })] });
        return Box({
            key: 'kit-composed-band', flexDirection: 'column',
            children: [downstream, Box({
                key: 'kit-bucket-strip', flexDirection: 'column', height: layout.height, paddingX: padding,
                paddingBottom: layout.padding, flexShrink: 0,
                children: rows.map(row => Box({
                    flexDirection: 'row', flexShrink: 0,
                    children: row.map((bucket, i) => {
                        const width = layout.rows === 2
                            ? (i === 0 ? Math.floor(usable / 2) : usable - Math.floor(usable / 2))
                            : widths[buckets.indexOf(bucket)]!;
                        const active = selected.bucket === bucket;
                        return Box({
                            flexDirection: 'column', width, flexShrink: 0,
                            children: [
                                Button({ key: `bucket-${bucket}`, plain: true,
                                    label: `${active ? '›' : ' '} ${bucket}`, dimColor: !active,
                                    onPress: async () => { select(bucket); $.ui.invalidate('ui.render'); await openKit($); } }),
                                ...(layout.rules ? [Text({
                                    color: active ? colors.active : colors.divider, wrap: 'truncate',
                                    children: [(active ? '━' : '─').repeat(width)],
                                })] : []),
                            ],
                        });
                    }),
                })),
            })],
        });
    });
    on('ui.render', { component: 'Pane' }, async ($, e, next) => {
        if (!commandReady || e.requestId !== paneId)
            return next(e);
        const { Box, Text, Button } = $.ui.resolve(e);
        const selected = agents.find(a => a.id === state.selected)!;
        const run = async (fn: () => Promise<string>) => {
            const before = epoch;
            try {
                const message = await fn();
                if (epoch === before)
                    notice = message;
            }
            catch {
                if (epoch === before)
                    notice = 'Action unavailable. Inspect the native prompt before trying again.';
            }
            $.ui.invalidate('ui.render');
        };
        const guidance = 'Review in the native prompt; Enter invokes. Selecting a bucket starts nothing.';
        const leadingRows = bandRows({ type: 'Text', children: [[selected.title, guidance, notice].filter(Boolean).join('\n')] }, e.props.bodyColumns);
        const foxApplicable = leadingRows !== undefined && e.props.scroll.bodyRows >= leadingRows + 11 && e.surface === 'terminal' && state.jobs.length === 0 && e.props.bodyColumns >= 22 && e.props.scroll.bodyRows >= 20;
        const showFox = expanded && foxApplicable;
        if (!showFox || reduced) stopFoxMotion();
        else if (!foxTimer) {
            // The native clock is active only while applicable art is drawn.
            foxTimer = $.clock.every(100, () => {
                if (!commandReady || !expanded || reduced || state.jobs.length) { stopFoxMotion(); return; }
                elapsed += 100;
                const nextFrame = foxFrameAt(elapsed, reduced);
                if (nextFrame !== frame) { frame = nextFrame; $.ui.invalidate('ui.render'); }
            });
        }
        const fox = showFox
            ? [$.ui.resolve({ component: 'Pane', surface: 'terminal' }).Raster({ key: 'kit-fox', columns: 22, rows: 8, cells: foxRaster(reduced ? 'idle' : frame) })] : [];
        const jobs = state.jobs;
        return Box({
            flexDirection: 'column',
            children: [
                Text({ color: colors.text, bold: true, children: [selected.title] }),
                Button({ key: 'prepare-draft', label: 'Prepare draft', variant: 'primary', onPress: () => run(() => prepare($)) }),
                Text({ color: colors.muted, children: [guidance] }),
                ...(notice ? [Text({ color: colors.inactive, children: [notice] })] : []),
                ...fox,
                ...(foxApplicable ? [Box({
                    flexDirection: 'row', flexWrap: 'wrap', gap: 1,
                    children: [
                        Button({ key: 'toggle-fox', plain: true, label: expanded ? 'Hide fox art' : 'Show fox art', onPress: () => { expanded = !expanded; stopFoxMotion(); $.ui.invalidate('ui.render'); } }),
                        ...(expanded ? [Button({ key: 'toggle-motion', plain: true, label: reduced ? 'Fox motion paused' : 'Pause fox motion', onPress: () => { reduced = true; stopFoxMotion(); $.ui.invalidate('ui.render'); } })] : [])
                    ]
                })] : []),
                Box({
                    flexDirection: 'column',
                    children: [
                        Text({
                            color: colors.active, bold: true,
                            children: [
                                'SKILLS'
                            ]
                        }),
                        Text({
                            color: colors.inactive,
                            children: [
                                selected.bucket
                            ]
                        }),
                        Text({
                            color: colors.inactive,
                            children: [
                                selected.purpose
                            ]
                        })
                    ]
                }),
                Box({
                    flexDirection: 'column',
                    children: [
                        ...selected.skills.map(id => Text({
                            color: colors.inactive,
                            children: [
                                skills.find(s => s.id === id)!.title
                            ]
                        })),
                        Text({
                            color: colors.muted,
                            children: [
                                `${selected.access} · ${selected.bucket === 'Security' ? 'CC-BY-SA-4.0' : 'MIT guidance'}`
                            ]
                        })
                    ]
                }),
                Text({
                    color: colors.active, bold: true,
                    children: [
                        'THIS SESSION'
                    ]
                }),
                ...(state.statusError ? [Text({
                        color: colors.inactive,
                        children: [
                            state.statusError
                        ]
                    })] : [
                    ...(jobs.length ? jobs.map(job => Box({
                        flexDirection: 'column',
                        children: [
                            Text({
                                color: colors.text,
                                children: [
                                    job.description
                                ]
                            }),
                            Text({
                                color: colors.inactive,
                                children: [
                                    `${state.tasks.find(t => t.attemptId === job.id)?.id ?? 'Task unknown'} · ${statusLabel(job.status)}`
                                ]
                            }),
                            Text({
                                color: colors.muted,
                                children: [
                                    `Attempt: ${job.id}`
                                ]
                            }),
                            Text({
                                color: colors.muted,
                                children: [
                                    `Owner: ${job.parentId ?? 'main session'}`
                                ]
                            }),
                            ...state.results.filter(r => r.agentId === job.id).map(result => Box({
                                flexDirection: 'column',
                                children: [
                                    Text({
                                        color: colors.inactive,
                                        children: [
                                            `${result.key} · ${resultStatus(result.reason)}`
                                        ]
                                    }),
                                    Button({ key: `review-${result.key}`, label: 'Prepare review', onPress: () => run(() => review($, result.key)) })
                                ]
                            }))
                        ]
                    })) : [Text({
                            color: colors.muted,
                            children: [
                                'No Kit attempts observed.'
                            ]
                        })]),
                ]),
                Button({ key: 'refresh-status', label: 'Refresh status', onPress: async () => { await refresh($); $.ui.invalidate('ui.render'); } }),
                Text({
                    color: colors.muted,
                    children: [
                        'Last observed native status. Completion still needs review.'
                    ]
                }),
            ]
        });
    });
    on('session.start', async ($, e, next) => {
        const current = generation;
        commandReady = false;
        reset();
        try {
            await $.command.register({ name: 'kit', description: 'Browse Kit specialists and prepare visible task drafts',
                argumentHint: '[open | catalog | use <bucket> | jobs | review <result> | handoff <result> <bucket>]', immediate: true });
        } catch {
            return next(e); // Refusal leaves the command and UI to their native owner.
        }
        if (current !== generation) return next(e);
        commandReady = true;
        reduced = true;
        try {
            const env = { KIT_MOTION: await $.env.get('KIT_MOTION'), NO_COLOR: await $.env.get('NO_COLOR'), REDUCE_MOTION: await $.env.get('REDUCE_MOTION') };
            if (current === generation) reduced = reduceMotion(env);
        } catch { /* Keep motion off when settings cannot be read. */ }
        if (current !== generation) return next(e);
        return next(e);
    });
    on('ui.close', { id: paneId }, async ($, e, next) => {
        const current = generation;
        const result = await next(e); // Op denial is a result, not necessarily a thrown error.
        if (current === generation && 'value' in result) stopFoxMotion();
        return result;
    });
    on('session.end', async ($, e, next) => {
        const continuing = e.reason === 'clear' || e.reason === 'resume';
        // /clear and resume retain process command registration; no start event follows.
        if (!continuing) commandReady = false;
        reset(); resetState(); return next(e);
    });
}

// Source: official Mods create/test docs and declarations generated by 2.1.287.
// No model/agent spawn, prompt submission, permission, tool or auth interception.
export const register: Register = on => {
  registerUi(on)
  on('command.run', { command: 'kit' }, async ($, e, next) => {
    if (!commandReady) return next(e)
    const [action = 'catalog', target = '', bucket = ''] = e.args.trim().split(/\s+/).filter(Boolean)
    if (action === 'open') { await openKit($); return {} }
    if (action === 'catalog') {
      return { text: `${catalogText()}\n\nSelected for next task: ${state.selected}\n/kit open shows the native pane. /kit use <bucket> prepares a visible draft. Nothing starts until you send it.` }
    }
    if (action === 'use') {
      const agent = findAgent(target)
      if (!agent) return { text: 'Choose Frontend, Backend, Security or Product.' }
      const current = generation
      const draft = await prepareDraft($, agent)
      // Selection describes intent only and never carries into another session.
      if (current === generation && draft.filled) select(agent.id)
      $.ui.invalidate('ui.render')
      return { text: draft.text }
    }
    if (action === 'jobs') {
      await refresh($)
      if (state.statusError) return { text: state.statusError }
      const rows = state.jobs.map(j => `${state.tasks.find(t => t.attemptId === j.id)?.id} · Attempt ${j.id} · ${j.type} · ${statusLabel(j.status)}\n  ${j.description}`)
      const results = state.results.filter(r => state.jobs.some(j => j.id === r.agentId))
        .map(r => `${r.key} · ${resultStatus(r.reason)}`)
      return { text: ['This Claude session only.', ...rows,
        ...(rows.length ? [] : ['No Kit jobs observed.']),
        ...results.map(r => `Result: ${r}`)].join('\n') }
    }
    if (action === 'review' || action === 'handoff') {
      const result = state.results.find(r => r.key === target)
      if (!result) return { text: 'No captured result with that exact key. Run /kit jobs; results from before this mod loaded are unavailable.' }
      const agent = action === 'review' ? findAgent('Security') : findAgent(bucket)
      if (!agent) return { text: 'Choose the receiving bucket after the result key.' }
      return { text: (await prepareDraft($, agent, result, action)).text }
    }
    return { text: 'Use /kit open, /kit catalog, /kit use <bucket>, /kit jobs, /kit review <result>, or /kit handoff <result> <bucket>.' }
  })

  on('agent.spawn', async ($, e, next) => {
    const current = generation
    const result = await next(e)
    if (!commandReady || current !== generation || !result.agentId) return result
    const started = next.trace.find(entry => entry.tier === 'core' && entry.returned?.agentId === result.agentId)
    if (!started || !agents.some(a => a.id === started.received.subagentType)) return result
    const input = started.received
    knownAttempts.set(result.agentId, {
      id: result.agentId, type: input.subagentType, description: input.description,
      status: 'running', parentId: input.parentAgentId,
    })
    capturePending([])
    $.ui.invalidate('ui.render')
    return result
  })

  on('turn.complete', async ($, e, next) => {
    const current = generation
    const result = await next(e)
    if (!commandReady || current !== generation) return result
    if (e.agentId) {
      if (!state.results.some(r => r.agentId === e.agentId && r.turnId === e.turnId) &&
          !pendingResults.some(r => r.agentId === e.agentId && r.turnId === e.turnId)) {
        pendingResults = [...pendingResults, {
          agentId: e.agentId, turnId: e.turnId, answer: e.answer, reason: e.reason,
        }].slice(-50)
      }
      const jobs = await refresh($)
      if (current !== generation) return result
      // Verified same-session metadata can retain an answer without inventing status.
      if (!jobs) capturePending(state.jobs)
    }
    $.ui.invalidate('ui.render')
    return result
  })
}
