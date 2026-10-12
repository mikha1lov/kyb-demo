import { requirements, type Business } from './schema'
import type { Extracted, RequirementResult } from './types'

export class ApiError extends Error {
  code: string

  constructor(code: string, message: string) {
    super(message)
    this.code = code
  }
}

const friendly: Record<string, string> = {
  unsupported_file: 'This file type is not supported',
  provider_unreachable: 'The document could not be read. Try again',
  provider_error: 'The document could not be read. Try again',
  provider_bad_answer: 'The document could not be read. Try again',
}

async function request<T>(path: string, init: RequestInit): Promise<T> {
  let response: Response
  try {
    response = await fetch(path, init)
  } catch {
    throw new ApiError('network', 'Connection lost. Try again')
  }
  if (!response.ok) {
    const body = await response.json().catch(() => ({}))
    const code = typeof body.error === 'string' ? body.error : 'unknown'
    throw new ApiError(code, friendly[code] ?? body.message ?? 'Something went wrong')
  }
  return response.json()
}

function typed(business: Business) {
  return {
    legalName: business.legalName,
    registrationNumber: business.registrationNumber,
    address: business.address,
  }
}

export type Analysis = { extracted: Extracted; results: RequirementResult[] }

export function analyzeDocument(file: File, business: Business): Promise<Analysis> {
  const body = new FormData()
  body.append('file', file)
  for (const requirement of requirements) body.append('requirement', requirement)
  body.append('typed', JSON.stringify(typed(business)))
  return request('/api/documents/analyze', { method: 'POST', body })
}

export async function compareDocument(extracted: Extracted, business: Business): Promise<RequirementResult[]> {
  const { results } = await request<{ results: RequirementResult[] }>('/api/documents/compare', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ requirements, typed: typed(business), extracted }),
  })
  return results
}
