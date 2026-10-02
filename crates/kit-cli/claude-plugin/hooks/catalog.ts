export const buckets = ['Frontend', 'Backend', 'Security', 'Product'] as const
export type Bucket = (typeof buckets)[number]

export const skills = [
  { id: 'animate-ui', title: 'Animate UI', bucket: 'Frontend' },
  { id: 'vercel-react-best-practices', title: 'React best practices', bucket: 'Frontend' },
  { id: 'vercel-composition-patterns', title: 'Composition patterns', bucket: 'Frontend' },
  { id: 'accessibility', title: 'Accessibility', bucket: 'Frontend' },
  { id: 'api-and-interface-design', title: 'API design', bucket: 'Backend' },
  { id: 'test-driven-development', title: 'Test-driven development', bucket: 'Backend' },
  { id: 'systematic-debugging', title: 'Systematic debugging', bucket: 'Backend' },
  { id: 'supabase-postgres-best-practices', title: 'Postgres best practices', bucket: 'Backend' },
  { id: 'audit-context-building', title: 'Audit context', bucket: 'Security' },
  { id: 'insecure-defaults', title: 'Insecure defaults', bucket: 'Security' },
  { id: 'sharp-edges', title: 'Sharp edges', bucket: 'Security' },
  { id: 'variant-analysis', title: 'Variant analysis', bucket: 'Security' },
  { id: 'create-prd', title: 'Create PRD', bucket: 'Product' },
  { id: 'product-strategy', title: 'Product strategy', bucket: 'Product' },
  { id: 'opportunity-solution-tree', title: 'Opportunity solution tree', bucket: 'Product' },
  { id: 'prioritization-frameworks', title: 'Prioritization frameworks', bucket: 'Product' },
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
// Four pinned skills per specialist; source and license details live in provenance.json.
export const agents: readonly Specialist[] = [
  { id: 'kit:frontend-ui-builder', title: 'UI builder', bucket: 'Frontend',
    purpose: 'Build accessible interfaces against a concrete design and acceptance criteria.',
    skills: ['animate-ui', 'vercel-react-best-practices', 'vercel-composition-patterns', 'accessibility'], access: 'worktree writer' },
  { id: 'kit:backend-api-builder', title: 'API builder', bucket: 'Backend',
    purpose: 'Implement API contracts and verify authorization, validation and failure paths.',
    skills: ['api-and-interface-design', 'test-driven-development', 'systematic-debugging', 'supabase-postgres-best-practices'], access: 'worktree writer' },
  { id: 'kit:security-reviewer', title: 'Security reviewer', bucket: 'Security',
    purpose: 'Review a fixed change for exploitable paths and dependency risk, with evidence.',
    skills: ['audit-context-building', 'insecure-defaults', 'sharp-edges', 'variant-analysis'], access: 'read only' },
  { id: 'kit:product-spec-writer', title: 'Spec writer', bucket: 'Product',
    purpose: 'Turn a user outcome into a scoped specification and checkable acceptance criteria.',
    skills: ['create-prd', 'product-strategy', 'opportunity-solution-tree', 'prioritization-frameworks'], access: 'worktree writer' },
]

export function findAgent(value: string): Specialist | undefined {
  const query = value.trim().toLowerCase()
  return agents.find(a => a.id === query || a.bucket.toLowerCase() === query)
}

export function invocation(agent: Specialist): string {
  return `\n@agent-${agent.id}\n`
}

export function catalogText(): string {
  return buckets.map(bucket => [bucket, ...agents.filter(a => a.bucket === bucket).map(a =>
    `  ${a.title} (${a.id})\n    ${a.skills.map(id => skills.find(s => s.id === id)!.title).join(' · ')}\n    ${a.access}; ${a.bucket === 'Security' ? 'Trail of Bits / CC-BY-SA-4.0' : 'MIT skill guidance'}; source pins: provenance.json`,
  )].join('\n')).join('\n\n')
}
