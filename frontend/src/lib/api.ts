import type { Business } from './schema'
import type { Application, Sources } from './types'

export class ApiError extends Error {
  code: string
  status: number

  constructor(code: string, message: string, status = 0) {
    super(message)
    this.code = code
    this.status = status
  }
}

const friendly: Record<string, string> = {
  unsupported_file: 'This file type is not supported',
  provider_unreachable: 'The document could not be read. Try again',
  provider_error: 'The document could not be read. Try again',
  provider_bad_answer: 'The document could not be read. Try again',
  already_submitted: 'This application has already been submitted',
  too_many_documents: 'Too many documents',
  incomplete: 'Some requirements are still open',
  internal_error: 'Something went wrong. Try again',
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  let response: Response
  try {
    response = await fetch(path, init)
  } catch {
    throw new ApiError('network', 'Connection lost. Try again')
  }
  if (!response.ok) {
    const body = await response.json().catch(() => ({}))
    const code = typeof body.error === 'string' ? body.error : 'unknown'
    throw new ApiError(code, friendly[code] ?? body.message ?? 'Something went wrong', response.status)
  }
  return response.json()
}

function json(method: string, body: unknown): RequestInit {
  return { method, headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) }
}

export function createApplication(business: Business, sources: Sources = {}) {
  return request<Application>('/api/applications', json('POST', { business, sources }))
}

export function getApplication(id: string) {
  return request<Application>(`/api/applications/${id}`)
}

export function saveBusiness(id: string, business: Business, sources: Sources) {
  return request<Application>(`/api/applications/${id}/business`, json('PUT', { business, sources }))
}

export function uploadDocument(id: string, file: File) {
  const body = new FormData()
  body.append('file', file)
  return request<Application>(`/api/applications/${id}/documents`, { method: 'POST', body })
}

export function deleteDocument(id: string, documentId: string) {
  return request<Application>(`/api/applications/${id}/documents/${documentId}`, { method: 'DELETE' })
}

export function submitApplication(id: string) {
  return request<Application>(`/api/applications/${id}/submit`, { method: 'POST' })
}
