import { fireEvent, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import i18n from '../../shared/i18n'
import { json, mockApi, renderAt } from '../../test/app'

// The desktop app: one local user, and a speech pack to download.
const me = {
  id: '00000000-0000-0000-0000-000000000000',
  email: 'me@localhost.local',
  display_name: 'Me',
  ui_locale: 'en',
}
let speech: object

function mockDesktop(handlers: Record<string, () => Response> = {}) {
  return mockApi({
    'GET /api/me': () => json(200, me),
    'GET /api/config': () => json(200, { accounts: false, speech }),
    ...handlers,
  })
}

beforeEach(async () => {
  localStorage.clear()
  speech = { state: 'not_installed' }
  await i18n.changeLanguage('en')
})
afterEach(() => vi.unstubAllGlobals())

test('there are settings instead of accounts', async () => {
  mockDesktop()
  renderAt('/')
  expect(await screen.findByRole('link', { name: 'Settings' })).toBeInTheDocument()
  expect(screen.queryByRole('link', { name: 'Log in' })).not.toBeInTheDocument()
  expect(screen.queryByRole('button', { name: 'Log out' })).not.toBeInTheDocument()
})

test('account pages lead home', async () => {
  mockDesktop()
  const router = renderAt('/login')
  await waitFor(() => expect(router.state.location.pathname).toBe('/'))
})

test('without the speech pack there are no 🔊 buttons', async () => {
  mockDesktop()
  renderAt('/rules/dative')
  await screen.findByRole('link', { name: 'Settings' }) // config loaded
  expect(screen.queryByRole('button', { name: /^Listen/ })).not.toBeInTheDocument()
})

test('the speech pack is downloaded from the settings', async () => {
  const fetchMock = mockDesktop({
    'POST /api/speech/pack': () => {
      speech = { state: 'installing', progress: 0.25 }
      return new Response(null, { status: 202 })
    },
  })
  renderAt('/settings')

  fireEvent.click(await screen.findByRole('button', { name: 'Download the speech pack' }))
  expect((await screen.findAllByText('25 %')).length).toBeGreaterThan(0)
  expect(fetchMock).toHaveBeenCalledWith(
    expect.objectContaining({ method: 'POST', url: expect.stringContaining('/api/speech/pack') }),
  )

  // Progress is polled until the pack is in place.
  speech = { state: 'ready' }
  expect(
    await screen.findByText(/The speech pack is installed/, {}, { timeout: 2000 }),
  ).toBeInTheDocument()
})

test('a failed download can be retried', async () => {
  speech = { state: 'failed', error: 'no network' }
  mockDesktop()
  renderAt('/settings')
  expect(await screen.findByRole('alert')).toHaveTextContent('The download failed: no network')
  expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument()
})

test('pronunciation is offered from the header on every page', async () => {
  const fetchMock = mockDesktop({
    'POST /api/speech/pack': () => {
      speech = { state: 'installing', progress: 0.4 }
      return new Response(null, { status: 202 })
    },
  })
  renderAt('/rules/dative')
  const offer = await screen.findByRole('button', { name: /Enable pronunciation/ })
  fireEvent.click(offer)

  expect(await screen.findByText('40 %')).toBeInTheDocument()
  expect(fetchMock).toHaveBeenCalledWith(
    expect.objectContaining({ method: 'POST', url: expect.stringContaining('/api/speech/pack') }),
  )
  speech = { state: 'ready' }
  await waitFor(() => expect(screen.queryByText('40 %')).not.toBeInTheDocument(), { timeout: 2000 })
  expect(screen.queryByRole('button', { name: /Enable pronunciation/ })).not.toBeInTheDocument()
})

test('a failed download can be retried from the header', async () => {
  speech = { state: 'failed', error: 'no network' }
  mockDesktop()
  renderAt('/')
  const retry = await screen.findByRole('button', { name: /Try again/ })
  expect(retry).toHaveAttribute('title', 'The download failed: no network')
})
