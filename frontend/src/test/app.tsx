// Helpers for tests that render the real app routes against a fake backend.
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render } from '@testing-library/react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { vi } from 'vitest'
import { routes } from '../app/routes'

export function json(status: number, body: unknown) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  })
}

export function problem(status: number, code: string) {
  return json(status, { type: 'about:blank', title: code, status, code })
}

/** Fake backend: `handlers` maps "METHOD /path" to a response; logged out by default. */
export function mockApi(handlers: Record<string, () => Response>) {
  const defaults: Record<string, () => Response> = {
    'GET /api/me': () => problem(401, 'unauthenticated'),
    'GET /api/health': () => json(200, { status: 'ok' }),
  }
  const fetchMock = vi.fn(async (request: Request) => {
    const key = `${request.method} ${new URL(request.url).pathname}`
    const handler = handlers[key] ?? defaults[key]
    if (!handler) throw new Error(`unexpected request ${key}`)
    return handler()
  })
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

/** The bodies of all requests made to `path`. */
export async function requestBodies(fetchMock: ReturnType<typeof mockApi>, path: string) {
  const requests = fetchMock.mock.calls.map(([r]) => r).filter((r) => r.url.endsWith(path))
  return Promise.all(requests.map((r) => r.clone().json()))
}

export function renderAt(path: string) {
  const router = createMemoryRouter(routes, { initialEntries: [path] })
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
  return router
}
