import type { EngineInterface, Register, Timer } from 'claude-code'
import { agents, buckets, skills, catalogText, findAgent, invocation } from './catalog.ts'
import { initialState, reconcile, recordResult, resultDraft, resultStatus, statusLabel } from './state.ts'
import { foxFrameAt, foxRaster, reduceMotion, type FoxFrame } from './fox.ts'
// Keep API-calling helpers in this file: 2.1.287's static analyzer cannot follow $ across imports.
let state = initialState()
let filling = false
let generation = 0

async function refresh($: EngineInterface) {
  const current = generation
  try {
    const jobs = await $.agent.list()
    if (current === generation) state = reconcile(state, jobs)
  } catch {
    if (current === generation) state = { ...state, statusError: 'Native status is unavailable. No job state inferred.' }
  }
}

async function fill($: EngineInterface, text: string) {
  if (filling) return 'A draft action is already in progress.'
  filling = true
  try {
    const result = await $.prompt.fill({ text, mode: 'replace' })
    return result.isFilled
      ? 'Selected for next task. Review the visible draft and press Enter to invoke the native agent.'
      : 'Claude could not fill the draft here. Nothing started. Use this in an interactive session.'
  } finally {
    filling = false
  }
}


function select(bucket: string) { state = { ...state, selected: findAgent(bucket)!.id } }
function resetState() { generation++; state = initialState(); filling = false }
async function prepare($: EngineInterface) {
  return fill($, invocation(findAgent(state.selected)!, (await $.prompt.read()).text))
}
async function review($: EngineInterface, key: string) {
  const result = state.results.find(r => r.key === key)
  if (!result) return 'This result is no longer available.'
  if ((await $.prompt.read()).text.trim()) return 'Your draft contains text. Send or clear it before preparing a result handoff.'
  return fill($, resultDraft(result, findAgent('Security')!, 'review'))
}
const paneId = 'kit-specialists';
const colors = { active: '#FF3B46', text: '#F0F1F2', muted: '#788292', inactive: '#A6AFBF', divider: '#3C434E' };
function bucketWidths(columns: number): number[] {
    const width = Math.max(0, Math.floor(columns));
    const base = Math.floor(width / 4);
    return [base, base, base, width - base * 3];
}
async function openKit($: EngineInterface) {
    await $.ui.open({ id: paneId, title: 'Kit · Skills', columns: 30, rows: 22, focus: true, closeOnEscape: true });
}
// Native sites/control focus only. No keyboard, prompt, permission or auth interception.
function registerUi(on: Parameters<Register>[0]) {
    let expanded = true;
    let reduced = false;
    let frame: FoxFrame = 'idle';
    let notice = '';
    let timer: Timer | undefined;
    let epoch = 0;
    const reset = (continuing: boolean) => {
        epoch++;
        if (!continuing) { timer?.cancel(); timer = undefined; }
        expanded = true; frame = 'idle'; notice = '';
    };
    on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
        if (e.props.hasSurvey || e.props.bodyColumns < 12 || e.props.maxRows < 3)
            return next(e);
        const { Box, Text, Button } = $.ui.resolve(e);
        const selected = agents.find(a => a.id === state.selected)!;
        const padding = e.props.bodyColumns >= 44 ? 2 : 0;
        const widths = bucketWidths(e.props.bodyColumns - padding * 2);
        // At narrow widths labels form two rows; at normal widths exactly four equal segments.
        const narrow = widths[0]! < 10;
        const rows = narrow ? [buckets.slice(0, 2), buckets.slice(2)] : [buckets];
        return Box({
            flexDirection: 'column', paddingX: padding, paddingBottom: 1,
            children: [
                ...rows.map(row => Box({
                    flexDirection: 'row',
                    children: [
                        ...row.map(bucket => {
                            const index = buckets.indexOf(bucket);
                            const width = narrow ? Math.floor(e.props.bodyColumns / 2) : widths[index]!;
                            const active = selected.bucket === bucket;
                            return Box({
                                flexDirection: 'column', width, flexShrink: 0,
                                children: [
                                    Button({ key: `bucket-${bucket}`, plain: true,
                                        label: `${active ? '›' : ' '} ${bucket}`.slice(0, width), dimColor: !active,
                                        onPress: async () => { select(bucket); $.ui.invalidate('ui.render'); await refresh($); await openKit($); } }),
                                    Text({
                                        color: active ? colors.active : colors.divider, wrap: 'truncate',
                                        children: [
                                            (active ? '━' : '─').repeat(width)
                                        ]
                                    })
                                ]
                            });
                        })
                    ]
                }))
            ]
        });
    });
    on('ui.render', { component: 'Pane' }, async ($, e, next) => {
        if (e.requestId !== paneId)
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
        const showFox = expanded && state.jobs.length === 0 && e.props.bodyColumns >= 22 && e.props.scroll.bodyRows >= 20;
        const fox = showFox && e.surface === 'terminal'
            ? [$.ui.resolve({ component: 'Pane', surface: 'terminal' }).Raster({ key: 'kit-fox', columns: 22, rows: 8, cells: foxRaster(reduced ? 'idle' : frame) })] : [];
        const jobs = state.jobs;
        return Box({
            flexDirection: 'column', gap: 1,
            children: [
                ...fox,
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
                        }), Text({
                            color: colors.text, bold: true,
                            children: [
                                selected.title
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
                                `${selected.access} · Kit / MIT`
                            ]
                        })
                    ]
                }),
                Button({ key: 'prepare-draft', label: 'Prepare draft', variant: 'primary', onPress: () => run(() => prepare($)) }),
                Text({
                    color: colors.muted,
                    children: [
                        'Review in the native prompt; Enter invokes. Selecting a bucket starts nothing.'
                    ]
                }),
                ...(notice ? [Text({
                        color: colors.inactive,
                        children: [
                            notice
                        ]
                    })] : []),
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
                Box({
                    flexDirection: 'row', flexWrap: 'wrap', gap: 1,
                    children: [
                        Button({ key: 'toggle-fox', plain: true, label: expanded ? 'Hide fox' : 'Show fox', onPress: () => { expanded = !expanded; $.ui.invalidate('ui.render'); } }),
                        Button({ key: 'toggle-motion', plain: true, label: reduced ? 'Motion off' : 'Pause motion', onPress: () => { reduced = true; frame = 'idle'; timer?.cancel(); $.ui.invalidate('ui.render'); } })
                    ]
                })
            ]
        });
    });
    on('session.start', async ($, e, next) => {
        const env = { KIT_MOTION: await $.env.get('KIT_MOTION'), NO_COLOR: await $.env.get('NO_COLOR'), REDUCE_MOTION: await $.env.get('REDUCE_MOTION') };
        reduced = reduceMotion(env);
        // A short gesture (100ms clock resolution) followed by a long idle; only invalidate on a frame change.
        let elapsed = 0;
        timer?.cancel();
        if (!reduced)
            timer = $.clock.every(100, () => {
                elapsed += 100;
                const nextFrame = foxFrameAt(elapsed, reduced);
                if (nextFrame !== frame) {
                    frame = nextFrame;
                    if (expanded)
                        $.ui.invalidate('ui.render');
                }
            });
     await $.command.register({ name: 'kit', description: 'Browse Kit specialists and prepare visible task drafts',
      argumentHint: '[open | catalog | use <bucket> | jobs | review <result> | handoff <result> <bucket>]', immediate: true }); return next(e) });
    on('session.end', async ($, e, next) => { reset(e.reason === 'clear' || e.reason === 'resume'); resetState(); return next(e) });
}

// Source: official Mods create/test docs and declarations generated by 2.1.287.
// No model/agent spawn, prompt submission, permission, tool or auth interception.
export const register: Register = on => {
  registerUi(on)
  on('command.run', { command: 'kit' }, async ($, e) => {
    const [action = 'catalog', target = '', bucket = ''] = e.args.trim().split(/\s+/).filter(Boolean)
    if (action === 'open') { await refresh($); await openKit($); return {} }
    if (action === 'catalog') {
      return { text: `${catalogText()}\n\nSelected for next task: ${state.selected}\n/kit open shows the native pane. /kit use <bucket> prepares a visible draft. Nothing starts until you send it.` }
    }
    if (action === 'use') {
      const agent = findAgent(target)
      if (!agent) return { text: 'Choose Frontend, Backend, Security or Product.' }
      const draft = await $.prompt.read()
      const text = await fill($, invocation(agent, draft.text))
      // Selection describes intent only, never a running task.
      select(agent.id)
      $.ui.invalidate('ui.render')
      return { text }
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
      const draft = await $.prompt.read()
      if (draft.text.trim()) return { text: 'Your draft contains text. Send or clear it before preparing a result handoff.' }
      return { text: await fill($, resultDraft(result, agent, action)) }
    }
    return { text: 'Use /kit open, /kit catalog, /kit use <bucket>, /kit jobs, /kit review <result>, or /kit handoff <result> <bucket>.' }
  })

  on('turn.complete', async ($, e, next) => {
    const current = generation
    const result = await next(e)
    if (current !== generation) return result
    if (e.agentId) {
      await refresh($)
      if (current !== generation) return result
      const job = !state.statusError && state.jobs.find(j => j.id === e.agentId && j.status !== 'not-attached')
      if (job && agents.some(a => a.id === job.type)) {
        state = recordResult(state, {
          agentId: e.agentId, agentType: job.type, task: job.description,
          turnId: e.turnId, answer: e.answer, reason: e.reason,
        })
      }
    }
    $.ui.invalidate('ui.render')
    return result
  })
}
