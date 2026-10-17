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
    // The web app: accounts, and a speech service.
    'GET /api/config': () =>
      json(200, { version: '1.2.3', accounts: true, speech: { state: 'service' } }),
    'GET /api/speech': () => new Response('RIFF', { headers: { 'content-type': 'audio/wav' } }),
  }
  // The API client passes a Request; speech prefetching passes a URL.
  const fetchMock = vi.fn(async (input: Request | string) => {
    const request = typeof input === 'string' ? new Request(input) : input
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
  const requests = fetchMock.mock.calls
    .map(([r]) => r)
    .filter((r): r is Request => typeof r !== 'string' && r.url.endsWith(path))
  return Promise.all(requests.map((r) => r.clone().json()))
}

/** Replaces `Audio`; returns the words "played", in order (`text` of their URL). */
export function mockAudio() {
  const spoken: string[] = []
  class FakeAudio extends EventTarget {
    readonly src: string
    constructor(src: string) {
      super()
      this.src = src
    }
    paused = true
    play() {
      this.paused = false
      spoken.push(new URL(this.src).searchParams.get('text') ?? '')
      // Finish at once, so queued words play next.
      queueMicrotask(() => {
        this.paused = true
        this.dispatchEvent(new Event('ended'))
      })
      return Promise.resolve()
    }
    pause() {}
  }
  vi.stubGlobal('Audio', FakeAudio)
  return spoken
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
