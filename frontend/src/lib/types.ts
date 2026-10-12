import type { Business, Requirement } from './schema'

export type DocKind = 'registration' | 'bank_statement' | 'utility_bill' | 'contract' | 'invoice' | 'other'

export type ExtractedField = { value: string | null; confidence: number; page: number | null }

export type Extracted = {
  kind: DocKind
  legalName: ExtractedField
  registrationNumber: ExtractedField
  address: ExtractedField
  periodStart: ExtractedField
  periodEnd: ExtractedField
  issueDate: ExtractedField
  notes: string | null
}

export type FieldCheck =
  | { status: 'match' }
  | { status: 'partial'; missingInTyped: string[]; missingInDocument: string[] }
  | { status: 'mismatch' }
  | { status: 'not_found' }
  | { status: 'not_checked' }

export type Freshness =
  | { status: 'fresh'; ageDays: number; maxDays: number }
  | { status: 'stale'; ageDays: number; maxDays: number }
  | { status: 'unknown' }
  | { status: 'not_required' }

export type Issue = {
  code: string
  field: string
  severity: 'blocking' | 'warning'
  message: string
}

export type Comparison = {
  kindAccepted: boolean
  legalName: FieldCheck
  registrationNumber: FieldCheck
  address: FieldCheck
  freshness: Freshness
  issues: Issue[]
}

export type RequirementResult = { requirement: Requirement; comparison: Comparison }

export const docKindLabels: Record<DocKind, string> = {
  registration: 'Registration certificate',
  bank_statement: 'Bank statement',
  utility_bill: 'Utility bill',
  contract: 'Contract',
  invoice: 'Invoice',
  other: 'Document',
}

export type SourceField = 'legalName' | 'registrationNumber' | 'address'

export type Sources = Partial<Record<SourceField, 'document'>>

export type DocumentView = {
  id: string
  name: string
  size: number
  extracted: Extracted
  results: RequirementResult[]
  createdAt: string
}

export type ChecklistItem = { requirement: Requirement; documentId: string | null }

export type Application = {
  id: string
  status: 'draft' | 'submitted'
  business: Business | null
  sources: Sources
  documents: DocumentView[]
  checklist: ChecklistItem[]
  complete: boolean
  submittedAt: string | null
  createdAt: string
  updatedAt: string
}
