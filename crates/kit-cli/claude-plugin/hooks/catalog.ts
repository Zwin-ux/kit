export const buckets = ['Frontend', 'Backend', 'Security', 'Product'] as const
export type Bucket = (typeof buckets)[number]

export const skills = [
  { id: 'ui-craft', title: 'Interface craft', bucket: 'Frontend' },
  { id: 'accessibility', title: 'Accessibility', bucket: 'Frontend' },
  { id: 'api-design', title: 'API design', bucket: 'Backend' },
  { id: 'backend-verification', title: 'Backend verification', bucket: 'Backend' },
  { id: 'security-review', title: 'Security review', bucket: 'Security' },
  { id: 'dependency-review', title: 'Dependency review', bucket: 'Security' },
  { id: 'product-spec', title: 'Product specification', bucket: 'Product' },
  { id: 'acceptance-review', title: 'Acceptance review', bucket: 'Product' },
] as const

export type SkillId = (typeof skills)[number]['id']
export type Specialist = {
  id: string
  title: string
  bucket: Bucket
  purpose: string
  skills: readonly SkillId[]
  access: 'worktree writer' | 'read only'
}

// Flat files explicitly declared in plugin.json work with Claude Code 2.1.287.
// Product categories are catalog data, independent of native file discovery.
// Knowledge is first-party Kit content; no upstream skill is installed here.
export const agents: readonly Specialist[] = [
  { id: 'kit:frontend-ui-builder', title: 'UI builder', bucket: 'Frontend',
    purpose: 'Build accessible interfaces against a concrete design and acceptance criteria.',
    skills: ['ui-craft', 'accessibility'], access: 'worktree writer' },
  { id: 'kit:backend-api-builder', title: 'API builder', bucket: 'Backend',
    purpose: 'Implement API contracts and verify authorization, validation and failure paths.',
    skills: ['api-design', 'backend-verification'], access: 'worktree writer' },
  { id: 'kit:security-reviewer', title: 'Security reviewer', bucket: 'Security',
    purpose: 'Review a fixed change for exploitable paths and dependency risk, with evidence.',
    skills: ['security-review', 'dependency-review'], access: 'read only' },
  { id: 'kit:product-spec-writer', title: 'Spec writer', bucket: 'Product',
    purpose: 'Turn a user outcome into a scoped specification and checkable acceptance criteria.',
    skills: ['product-spec', 'acceptance-review'], access: 'worktree writer' },
]

export function findAgent(value: string): Specialist | undefined {
  const query = value.trim().toLowerCase()
  return agents.find(a => a.id === query || a.bucket.toLowerCase() === query)
}

export function invocation(agent: Specialist, draft: string): string {
  // Only replace one Kit invocation at the beginning; keep the user's task verbatim.
  const task = draft.replace(/^@agent-kit:(?:frontend-ui-builder|backend-api-builder|security-reviewer|product-spec-writer)(?:\r?\n|[ \t]+|$)/, '')
  return `@agent-${agent.id}\n${task}`
}

export function catalogText(): string {
  return buckets.map(bucket => [bucket, ...agents.filter(a => a.bucket === bucket).map(a =>
    `  ${a.title} (${a.id})\n    ${a.skills.map(id => skills.find(s => s.id === id)!.title).join(' · ')}\n    ${a.access}; skills: Kit 2.0.0 / MIT`,
  )].join('\n')).join('\n\n')
}
