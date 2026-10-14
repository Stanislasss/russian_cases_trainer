import createClient from 'openapi-fetch'
import type { components, paths } from './schema'

export type Schemas = components['schemas']
export type ErrorCode = Schemas['ErrorCode']

/** Typed client for the backend; paths and bodies come from the generated `schema.d.ts`. */
export const api = createClient<paths>({
  baseUrl: window.location.origin,
  // Resolve fetch per call (not once at import) so tests can stub it.
  fetch: (request) => globalThis.fetch(request),
})

/** A failed API call, carrying the backend's stable error `code`. */
export class ApiError extends Error {
  readonly code: ErrorCode
  readonly status: number

  constructor(code: ErrorCode, status: number, message: string) {
    super(message)
    this.name = 'ApiError'
    this.code = code
    this.status = status
  }
}

function isProblem(value: unknown): value is Schemas['Problem'] {
  return typeof value === 'object' && value !== null && 'code' in value && 'status' in value
}

/** Returns the response data, or throws an `ApiError` for error responses. */
export async function unwrap<T>(
  call: Promise<{ data?: T; error?: unknown; response: Response }>,
): Promise<T> {
  const { data, error, response } = await call
  if (!response.ok) {
    if (isProblem(error)) throw new ApiError(error.code, error.status, error.title)
    throw new ApiError('internal', response.status, response.statusText)
  }
  return data as T
}
