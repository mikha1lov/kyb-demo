import type { Business } from './schema'
import type { SourceField, Sources } from './types'

const fields: SourceField[] = ['legalName', 'registrationNumber', 'address']

export function keepSources(previous: Business | null, next: Business, sources: Sources): Sources {
  const kept: Sources = {}
  for (const field of fields) {
    if (sources[field] && previous && previous[field] === next[field]) kept[field] = sources[field]
  }
  return kept
}
