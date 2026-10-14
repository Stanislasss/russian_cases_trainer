import { fireEvent, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import i18n from '../../shared/i18n'
import { json, mockApi, renderAt, requestBodies } from '../../test/app'
import type { Item } from './gameReducer'

const kniga: Item = {
  lexeme_id: 1,
  stressed_lemma: 'кни́га',
  translation: 'book',
  number: 'singular',
  answer: 'ой',
  accepted: ['ой', 'ою'],
  stressed_form: 'кни́гой',
}
const stol: Item = {
  lexeme_id: 2,
  stressed_lemma: 'стол',
  translation: 'table',
  number: 'singular',
  answer: 'ом',
  accepted: ['ом'],
  stressed_form: 'столо́м',
}
const batch = () => json(200, { zones: ['ом', 'ой'], items: [kniga, stol] })

beforeEach(async () => {
  localStorage.clear()
  await i18n.changeLanguage('en')
})
afterEach(() => vi.unstubAllGlobals())

async function playPracticeRound() {
  renderAt('/play/endings')
  fireEvent.click(screen.getByLabelText('Instrumental'))
  fireEvent.click(screen.getByRole('button', { name: 'Start' }))

  // Tap-to-answer: tapping a zone answers the word on the board.
  expect(await screen.findByText('кни́га')).toBeInTheDocument()
  fireEvent.click(screen.getByRole('button', { name: '-ой' }))
  expect(await screen.findByText('✓ кни́гой')).toBeInTheDocument()

  expect(screen.getByText('стол')).toBeInTheDocument()
  fireEvent.click(screen.getByRole('button', { name: '-ой' }))

  // The pool had two words, so the round ends early.
  expect(await screen.findByRole('heading', { name: 'Results' })).toBeInTheDocument()
}

test('a practice round from setup to summary', async () => {
  const fetchMock = mockApi({ 'POST /api/games/endings/batch': batch })
  await playPracticeRound()

  expect(screen.getByText('1 of 2 correct')).toBeInTheDocument()
  expect(screen.getByText('столо́м')).toBeInTheDocument() // listed under mistakes
  // Besides the header's link, the summary offers one that returns to the game.
  const loginLinks = screen.getAllByRole('link', { name: 'Log in' })
  expect(loginLinks.map((a) => a.getAttribute('href'))).toContain('/login?next=%2Fplay%2Fendings')

  const [request] = await requestBodies(fetchMock, '/api/games/endings/batch')
  expect(request).toEqual({
    levels: ['basic_nouns'],
    case: 'instrumental',
    count: 10,
    locale: 'en',
  })
  // Logged out: nothing is recorded.
  expect(await requestBodies(fetchMock, '/api/games/endings/attempts')).toEqual([])
})

test('logged-in players get their answers recorded', async () => {
  const fetchMock = mockApi({
    'GET /api/me': () =>
      json(200, { id: '1', email: 'anna@example.com', display_name: 'Anna', ui_locale: 'en' }),
    'POST /api/games/endings/batch': batch,
    'POST /api/games/endings/attempts': () => new Response(null, { status: 204 }),
  })
  await playPracticeRound()

  expect(await screen.findByText('Your results are saved.')).toBeInTheDocument()
  await waitFor(async () =>
    expect(await requestBodies(fetchMock, '/api/games/endings/attempts')).toEqual([
      {
        attempts: [
          { lexeme_id: 1, case: 'instrumental', number: 'singular', answer: 'ой', correct: true },
          { lexeme_id: 2, case: 'instrumental', number: 'singular', answer: 'ой', correct: false },
        ],
      },
    ]),
  )
})

test('explains when no words match', async () => {
  mockApi({ 'POST /api/games/endings/batch': () => json(200, { zones: [], items: [] }) })
  renderAt('/play/endings')
  fireEvent.click(screen.getByLabelText('Nominative'))
  fireEvent.click(screen.getByRole('button', { name: 'Start' }))

  expect(await screen.findByRole('alert')).toHaveTextContent('only plurals are asked')
})

test('remembers the last settings', async () => {
  mockApi({ 'POST /api/games/endings/batch': batch })
  renderAt('/play/endings')
  fireEvent.click(screen.getByLabelText(/Intermediate nouns/))
  fireEvent.click(screen.getByLabelText('Dative'))
  fireEvent.click(screen.getByRole('button', { name: 'Start' }))
  await screen.findByText('кни́га')

  expect(JSON.parse(localStorage.getItem('endings-game.settings')!)).toEqual({
    levels: ['basic_nouns', 'intermediate_nouns'],
    case: 'dative',
    mode: 'practice',
  })
})
