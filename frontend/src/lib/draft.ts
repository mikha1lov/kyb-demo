import { createStore } from '@tanstack/react-store'
import { analyzeDocument, compareDocument } from './api'
import type { Business } from './schema'
import type { Extracted, RequirementResult } from './types'

export type DocumentEntry = {
  id: string
  name: string
  size: number
  status: 'analyzing' | 'ready' | 'failed'
  error?: string
  extracted?: Extracted
  results?: RequirementResult[]
}

export type FromDocument = 'legalName' | 'registrationNumber' | 'address'

export type Draft = {
  business: Business | null
  sources: Partial<Record<FromDocument, 'document'>>
  documents: DocumentEntry[]
  submittedAt: string | null
}

const storageKey = 'kyb-proto:draft'
const fromDocumentFields: FromDocument[] = ['legalName', 'registrationNumber', 'address']
const empty: Draft = { business: null, sources: {}, documents: [], submittedAt: null }

function load(): Draft {
  if (typeof window === 'undefined') return empty
  try {
    const raw = window.localStorage.getItem(storageKey)
    if (!raw) return empty
    const saved = JSON.parse(raw) as Draft
    return { ...empty, ...saved, documents: (saved.documents ?? []).filter((d) => d.status === 'ready') }
  } catch {
    return empty
  }
}

export const draft = createStore<Draft>(load())

if (typeof window !== 'undefined') {
  draft.subscribe((state) => {
    try {
      const documents = state.documents.filter((d) => d.status === 'ready')
      window.localStorage.setItem(storageKey, JSON.stringify({ ...state, documents }))
    } catch {
      return
    }
  })
}

let refreshToken = 0

async function refresh() {
  const { business, documents } = draft.get()
  if (!business) return
  const token = ++refreshToken
  const entries = await Promise.all(
    documents.map(async (d): Promise<[string, RequirementResult[] | undefined]> => {
      if (d.status !== 'ready' || !d.extracted) return [d.id, d.results]
      try {
        return [d.id, await compareDocument(d.extracted, business)]
      } catch {
        return [d.id, d.results]
      }
    }),
  )
  if (token !== refreshToken) return
  const results = new Map(entries)
  draft.setState((s) => ({
    ...s,
    documents: s.documents.map((d) => ({ ...d, results: results.get(d.id) ?? d.results })),
  }))
}

function patchDocument(id: string, patch: Partial<DocumentEntry>) {
  draft.setState((s) => ({
    ...s,
    documents: s.documents.map((d) => (d.id === id ? { ...d, ...patch } : d)),
  }))
}

export function saveBusiness(next: Business) {
  draft.setState((s) => {
    const sources = { ...s.sources }
    for (const field of fromDocumentFields) {
      if (s.business && s.business[field] !== next[field]) delete sources[field]
    }
    return { ...s, business: next, sources }
  })
  void refresh()
}

export function applyFromDocument(field: FromDocument, value: string) {
  draft.setState((s) =>
    s.business
      ? { ...s, business: { ...s.business, [field]: value }, sources: { ...s.sources, [field]: 'document' } }
      : s,
  )
  void refresh()
}

export async function addDocument(file: File) {
  const { business } = draft.get()
  if (!business) return
  const id = crypto.randomUUID()
  draft.setState((s) => ({
    ...s,
    documents: [...s.documents, { id, name: file.name, size: file.size, status: 'analyzing' }],
  }))
  try {
    const { extracted, results } = await analyzeDocument(file, business)
    patchDocument(id, { status: 'ready', extracted, results })
  } catch (error) {
    patchDocument(id, { status: 'failed', error: error instanceof Error ? error.message : 'Something went wrong' })
  }
}

export function removeDocument(id: string) {
  draft.setState((s) => ({ ...s, documents: s.documents.filter((d) => d.id !== id) }))
}

export function submit() {
  draft.setState((s) => ({ ...s, submittedAt: new Date().toISOString() }))
}

export function reset() {
  refreshToken++
  draft.setState(() => empty)
}
