import { act, fireEvent, screen, waitFor, within } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import i18n from '../../shared/i18n'
import { json, mockApi, renderAt, requestBodies } from '../../test/app'
import type { Item } from './gameReducer'

const kniga: Item = {
  lexeme_id: 1,
  stressed_lemma: 'кни́га',
  translations: { en: 'book', de: 'Buch', ru: 'произведение печати' },
  number: 'singular',
  gender: null,
  animacy: null,
  answer: 'ой',
  accepted: ['ой', 'ою'],
  stressed_form: 'кни́гой',
}
const stol: Item = {
  lexeme_id: 2,
  stressed_lemma: 'стол',
  translations: { en: 'table' },
  number: 'singular',
  gender: null,
  animacy: null,
  answer: 'ом',
  accepted: ['ом'],
  stressed_form: 'столо́м',
}
const batch = () => json(200, { answer_kind: 'ending', zones: ['ом', 'ой'], items: [kniga, stol] })

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
  expect(screen.getByText('Instrumental', { selector: 'strong' })).toBeInTheDocument()
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
          {
            lexeme_id: 1,
            case: 'instrumental',
            number: 'singular',
            gender: null,
            animacy: null,
            answer: 'ой',
            correct: true,
          },
          {
            lexeme_id: 2,
            case: 'instrumental',
            number: 'singular',
            gender: null,
            animacy: null,
            answer: 'ой',
            correct: false,
          },
        ],
      },
    ]),
  )
})

test('explains when no words match', async () => {
  mockApi({
    'POST /api/games/endings/batch': () =>
      json(200, { answer_kind: 'ending', zones: [], items: [] }),
  })
  renderAt('/play/endings')
  fireEvent.click(screen.getByLabelText('Nominative'))
  fireEvent.click(screen.getByRole('button', { name: 'Start' }))

  expect(await screen.findByRole('alert')).toHaveTextContent('never asked')
})

test('remembers the last settings', async () => {
  mockApi({ 'POST /api/games/endings/batch': batch })
  renderAt('/play/endings')
  const nouns = screen.getByRole('group', { name: 'Nouns' })
  fireEvent.click(within(nouns).getByLabelText(/Intermediate/))
  fireEvent.click(screen.getByLabelText('Dative'))
  fireEvent.click(screen.getByRole('button', { name: 'Start' }))
  await screen.findByText('кни́га')

  expect(JSON.parse(localStorage.getItem('endings-game.settings')!)).toEqual({
    levels: ['basic_nouns', 'intermediate_nouns'],
    case: 'dative',
    mode: 'practice',
  })
})

test('personal pronouns are played alone, on whole forms', async () => {
  const on: Item = {
    lexeme_id: 3,
    stressed_lemma: 'он',
    translations: { en: 'he, it' },
    number: 'singular',
    gender: 'masculine',
    animacy: null,
    answer: 'им',
    accepted: ['им', 'ним'],
    stressed_form: 'им',
  }
  const fetchMock = mockApi({
    'POST /api/games/endings/batch': () =>
      json(200, { answer_kind: 'whole_form', zones: ['мной', 'им'], items: [on] }),
  })
  renderAt('/play/endings')
  const pronouns = screen.getByRole('group', { name: 'Pronouns' })
  fireEvent.click(within(pronouns).getByLabelText(/Personal/))
  // Checking it unchecks the others: it can't share a board with endings.
  expect(screen.getAllByRole('checkbox', { checked: true })).toHaveLength(1)
  fireEvent.click(screen.getByLabelText('Instrumental'))
  fireEvent.click(screen.getByRole('button', { name: 'Start' }))

  // Zones show whole forms, without the ending dash; the card shows the gender.
  expect(await screen.findByRole('button', { name: 'мной' })).toBeInTheDocument()
  expect(screen.getByText('m.')).toBeInTheDocument()
  fireEvent.click(screen.getByRole('button', { name: 'им' }))
  expect(await screen.findByRole('heading', { name: 'Results' })).toBeInTheDocument()

  const [request] = await requestBodies(fetchMock, '/api/games/endings/batch')
  expect(request.levels).toEqual(['personal_pronouns'])
})

test('the game speaks German too', async () => {
  await i18n.changeLanguage('de')
  mockApi({ 'POST /api/games/endings/batch': batch })
  renderAt('/play/endings')
  fireEvent.click(screen.getByLabelText('Instrumental'))
  fireEvent.click(screen.getByRole('button', { name: 'Los geht’s' }))

  expect(await screen.findByText('Buch')).toBeInTheDocument()
})

test('translations follow the language, even mid-game', async () => {
  mockApi({ 'POST /api/games/endings/batch': batch })
  renderAt('/play/endings')
  fireEvent.click(screen.getByLabelText('Instrumental'))
  fireEvent.click(screen.getByRole('button', { name: 'Start' }))
  expect(await screen.findByText('book')).toBeInTheDocument()

  await act(() => i18n.changeLanguage('ru'))
  expect(await screen.findByText('произведение печати')).toBeInTheDocument() // a definition
  await act(() => i18n.changeLanguage('fr'))
  expect(await screen.findByText('book')).toBeInTheDocument() // no French: English
})
