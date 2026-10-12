import type { Details } from './schema'
import type { Application, SourceField, Sources } from './types'

const fields: SourceField[] = ['legalName', 'registrationNumber', 'address']

export function sourcesFor(details: Details, application: Application): Sources {
  const sources: Sources = {}
  for (const field of fields) {
    const suggested = application.suggestions[field]?.value
    if (suggested !== undefined && suggested === details[field]) {
      sources[field] = 'document'
    } else if (application.sources[field] && application.business?.[field] === details[field]) {
      sources[field] = application.sources[field]
    }
  }
  return sources
}
