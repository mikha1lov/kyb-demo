import type { DocumentView, RequirementResult } from './types'

export function isBlocking(result: RequirementResult) {
  return !result.comparison.kindAccepted || result.comparison.issues.some((i) => i.severity === 'blocking')
}

export function counted(document: DocumentView) {
  return document.results.filter((r) => r.comparison.kindAccepted)
}
