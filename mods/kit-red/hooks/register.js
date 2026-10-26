import { roles, parseRoles, contextFor } from './roles.js'
import { sprites } from './art.js'

const PANE = 'kit-red-picker'
let selected = []
// ponytail: module reload clears guidance; use declared $.state if live reload must retain it.
let interactive = false
let commandAvailable = false
let mono = true
let message = ''

// Session-only on purpose: opening another repo never silently applies a role.
function choose(ids) {
  selected = ids
  message = ids.length ? 'Applies to your next message. No tools installed.' : 'Kit guidance is off.'
}

async function draft($, text) {
  try {
    const current = await $.prompt.read()
    if (current.text.trim()) {
      message = 'Your prompt already has text. Send or clear it before adding an example.'
    } else {
      // Appending also protects text typed between read() and fill().
      const result = await $.prompt.fill({ text: '\n' + text, mode: 'append' })
      if (result.isFilled) {
        try {
          await $.ui.close({ id: PANE })
          message = ''
        } catch {
          message = 'Added to your prompt. Close Kit to review it. Nothing was sent.'
        }
      } else message = 'Could not fill the prompt. Nothing was sent.'
    }
  } catch {
    message = 'Could not fill the prompt. Nothing was sent.'
  }
  $.ui.invalidate('ui.render')
}

async function openPicker($) {
  try {
    await $.ui.open({ id: PANE, title: 'Kit', focus: true, closeOnEscape: true, rows: 20, columns: 64 })
  } catch {
    message = 'Kit could not open. Choose a role with /kit frontend, fullstack, backend or llm.'
    $.ui.invalidate('ui.render')
    return false
  }
  message = ''
  try {
    if ((await $.ui.panes()).some(pane => pane.id === PANE && pane.isPlaced)) await $.store.set('intro-seen', true)
  } catch {
    message = 'First-launch preference could not be saved. /kit remains available.'
  }
  $.ui.invalidate('ui.render')
  return true
}

export function register(on) {
  on('session.start', async ($, e, next) => {
    interactive = e.isInteractive === true
    try { mono = (await $.env.get('NO_COLOR')) !== undefined } catch { mono = true }
    commandAvailable = false
    try {
      await $.command.register({ name: 'kit', description: 'Choose what you want help with', immediate: true, argumentHint: '[frontend | fullstack | backend | llm | off]' })
      commandAvailable = true
    } catch {
      $.ui.status('Kit could not register /kit. Check for a command conflict.')
      $.ui.invalidate('ui.render')
      return next(e)
    }
    $.ui.status(undefined)
    if (interactive) {
      let seen = false
      try { seen = (await $.store.get('intro-seen')) === true } catch { /* Unreadable preference still allows arrival. */ }
      if (!seen) await openPicker($)
    }
    return next(e)
  })

  on('classic.SessionStart', { source: ['clear', 'resume', 'fork'] }, async ($, e, next) => {
    // Clear/resume/fork must not silently inherit an unrelated active choice.
    selected = []
    message = ''
    $.ui.invalidate('ui.render')
    return next(e)
  })

  on('command.run', { command: 'kit' }, async ($, e, next) => {
    if (!commandAvailable) return next(e)
    const args = e.args.trim().toLowerCase()
    if (args === 'off') {
      choose([])
      $.ui.invalidate('ui.render')
      return { text: 'Kit guidance is off.', context: [contextFor([])] }
    }
    if (args) {
      const ids = parseRoles(args)
      if (!ids) return { text: 'Choose frontend, fullstack, backend or llm. Combine them with commas. /kit off clears the choice.' }
      choose(ids)
      $.ui.invalidate('ui.render')
      return { text: 'Kit: ' + roles.filter(r => ids.includes(r.id)).map(r => r.title).join(', ') + '. Guidance for this conversation; no tools installed.', context: [contextFor(ids)] }
    }
    if (!interactive) return { text: 'Kit: /kit frontend, /kit fullstack, /kit backend, /kit llm, or /kit off. These add working guidance; they do not install full toolkits.' }
    return await openPicker($) ? {} : { text: message }
  })

  on('prompt.submit', async ($, e, next) => {
    return next({ ...e, context: [...(e.context ?? []), contextFor(selected)] })
  })

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const downstream = await next(e)
    if (e.props?.hasSurvey) return downstream
    const { Box, Text } = $.ui.resolve(e)
    if (!commandAvailable) return Box({ flexDirection: 'column', children: [downstream, Text({ children: ['Kit unavailable'] })] })
    const names = roles.filter(r => selected.includes(r.id)).map(r => r.title)
    const full = names.length ? 'Kit · ' + names.join(' + ') + ' · /kit' : 'Kit · /kit'
    const width = e.props?.bodyColumns ?? 80
    const label = full.length <= width ? full : width >= 24 ? 'Kit · ' + names.length + ' roles · /kit' : 'Kit · /kit'
    return Box({ flexDirection: 'column', children: [downstream, Text({ ...(mono ? {} : { color: '#E45F59' }), children: [label] })] })
  })

  on('ui.render', { component: 'Pane' }, async ($, e, next) => {
    if (e.requestId !== PANE) return next(e)
    const { Box, Text, Button } = $.ui.resolve(e)
    const width = e.props.bodyColumns ?? 60
    const rich = e.surface === 'terminal' && !mono && width >= 58 && e.props.placement === 'dock' && (e.props.scroll?.bodyRows ?? 0) >= 24
    const role = selected.length === 1 ? roles.find(role => role.id === selected[0]) : undefined
    const art = rich ? $.ui.resolve(e).Raster({ key: 'kit-emblem', ...(role ? sprites[role.alias] : sprites.fox) }) : Text({ bold: true, children: ['Kit'] })
    return Box({ flexDirection: 'column', children: [
      Box({ flexDirection: 'row', columnGap: 2, children: [art, Box({ flexDirection: 'column', children: [Text({ bold: true, children: ['What are you making?'] }), Text({ children: ['Choose one or more. Keep chatting normally.'] })] })] }),
      ...roles.map(role => Box({ key: 'row-' + role.id, flexDirection: 'column', children: [
        Button({ key: role.id, label: (selected.includes(role.id) ? '[x] ' : '[ ] ') + role.title, plain: true, ...(role === roles[0] ? { autoFocus: true } : {}), onPress: () => {
          choose(selected.includes(role.id) ? selected.filter(id => id !== role.id) : [...selected, role.id])
          $.ui.invalidate('ui.render')
        } }),
        Text({ dimColor: true, children: [role.detail] }),
      ] })),
      Text({ children: [message || 'Adds Kit working guidance. Full toolkits are separate.'] }),
      Button({ key: 'continue', label: 'Back to chat', onPress: async () => { await $.ui.close({ id: PANE }) } }),
      ...(selected.length ? [Button({ key: 'example', label: 'Try an example', onPress: async () => {
        const role = roles.find(r => selected.includes(r.id))
        if (role) await draft($, role.example)
      } }), Button({ key: 'toolkit', label: 'Set up the full toolkit', onPress: async () => {
        const ids = roles.filter(r => selected.includes(r.id)).map(r => r.id).join(' ')
        await draft($, 'Help me install the full Kit toolkit for ' + ids + ' in this repository for Claude Code. First check whether the official Kit CLI (Zwin-ux/kit) is installed and inspect its help. If available, show the no-write plan using kit add ' + ids + ' --agent claude --print. If missing, explain the official installation first. Show exact files, scope, sources and code-running MCP/hooks, then wait for my approval before installation. Do not change global configuration or run checks implicitly.')
      } })] : []),
      Button({ key: 'off', label: 'Turn Kit guidance off', plain: true, onPress: () => { choose([]); $.ui.invalidate('ui.render') } }),
      Text({ dimColor: true, children: ['Tab moves · Enter chooses · Esc closes'] }),
    ] })
  })
}
