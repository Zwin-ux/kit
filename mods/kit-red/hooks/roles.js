// Kit-authored guidance, adapted from the MIT Kit starter kits at c35f608.
const base = 'Read project instructions and existing code first. Reuse existing patterns. Make the smallest correct change. Run relevant checks and report what actually passed, failed, or was not checked. Never claim a tool, MCP server, skill bundle or test is available merely because this role is selected.'
const frontend = 'Reuse design tokens and components. Check loading, empty and error states. Make controls keyboard reachable, labelled and visibly focused. When a browser is available, inspect the result at narrow and desktop widths and report them; otherwise say visual verification was not performed.'
export const roles = [
  { id: 'frontend-design', alias: 'frontend', title: 'Frontend Design', detail: 'Interfaces, accessibility, browser checks', example: 'Improve the accessibility of one existing component.', rules: frontend },
  { id: 'fullstack-design', alias: 'fullstack', title: 'Full-stack Design', detail: 'UI, APIs, data, end-to-end behavior', example: 'Trace the sign-up flow and fix its first reproducible problem.', rules: frontend + ' Define request, response and error contracts before changing either side. Validate server inputs. Ship schema changes as reversible migrations. Verify the complete user flow where tools allow.' },
  { id: 'backend-engineer', alias: 'backend', title: 'Backend Engineer', detail: 'APIs, databases, reliability', example: 'Add a test for an existing endpoint failure case.', rules: 'Validate and bound external input. Use parameterised queries. Test the success and main failure case of changed endpoints. Measure slow paths before optimising. Return useful errors without exposing internals. Keep secrets out of code, logs and fixtures.' },
  { id: 'llm-engineer', alias: 'llm', title: 'LLM Engineer', detail: 'Prompts, evaluations, model APIs', example: 'Create a small evaluation for an existing prompt.', rules: 'Establish evaluation examples before changing a prompt or model. Check current provider documentation for API parameters and model names. Keep prompts versioned. Measure token usage and latency where possible without logging sensitive content. Do not send private data to a provider without authorization.' },
]
export function parseRoles(text) {
  const names = text.split(/[\s,]+/).filter(Boolean)
  const found = names.map(name => roles.find(r => r.id === name || r.alias === name))
  return found.length && found.every(Boolean) ? [...new Set(found.map(r => r.id))] : null
}
export function contextFor(ids) {
  const chosen = roles.filter(r => ids.includes(r.id))
  return chosen.length
    ? 'Kit guidance selected by the user for this conversation: ' + chosen.map(r => r.title).join(', ') + '. This replaces previous Kit role preferences; follow current user requests and project instructions. This is working guidance, not an installation of the full Kit toolkits.\n' + base + '\n' + chosen.map(r => r.title + ': ' + r.rules).join('\n')
    : 'Kit role guidance is now off. Do not treat earlier Kit role preferences as active. Continue following the user and project instructions.'
}
