import { requirements, type Requirement } from './schema'
import type { RequirementResult } from './types'
import type { DocumentEntry } from './draft'

export function isBlocking(result: RequirementResult) {
  return !result.comparison.kindAccepted || result.comparison.issues.some((i) => i.severity === 'blocking')
}

export function counted(document: DocumentEntry) {
  return (document.results ?? []).filter((r) => r.comparison.kindAccepted)
}

export function satisfiedBy(documents: DocumentEntry[], requirement: Requirement) {
  return documents.find(
    (d) =>
      d.status === 'ready' && d.results?.some((r) => r.requirement === requirement && !isBlocking(r)),
  )
}

export function missing(documents: DocumentEntry[]) {
  return requirements.filter((r) => !satisfiedBy(documents, r))
}
