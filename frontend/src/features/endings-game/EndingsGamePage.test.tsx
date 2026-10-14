import { act, fireEvent, screen, waitFor, within } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest'
import i18n from '../../shared/i18n'
import { json, mockApi, mockAudio, renderAt, requestBodies } from '../../test/app'
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
  paradigm: 'noun_a',
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
  paradigm: 'noun_masculine_hard',
}
const batch = () => json(200, { answer_kind: 'ending', zones: ['ом', 'ой'], items: [kniga, stol] })

/** Practice settings with this pause after a correct answer (0: tests don't wait). */
function useDelay(ms: number) {
  localStorage.setItem('endings-game.settings', JSON.stringify({ nextWordDelay: ms }))
}

beforeEach(async () => {
  localStorage.clear()
  useDelay(0)
  mockAudio() // jsdom has no audio playback
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

  expect(await screen.findByText('стол')).toBeInTheDocument() // after the form was heard
  fireEvent.click(screen.getByRole('button', { name: '-ой' }))

  // The pool had two words, so the round ends early.
  expect(await screen.findByRole('heading', { name: 'Results' })).toBeInTheDocument()
}

test('a practice round from setup to summary', async () => {
  const fetchMock = mockApi({ 'POST /api/games/endings/batch': batch })
  await playPracticeRound()

  expect(screen.getByText('1 of 2 correct')).toBeInTheDocument()
  expect(screen.getByText('столо́м', { selector: 'summary strong' })).toBeInTheDocument() // listed under mistakes
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
    nextWordDelay: 0,
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
    paradigm: 'personal_pronoun',
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

describe('pronunciation', () => {
  async function startPractice() {
    mockApi({ 'POST /api/games/endings/batch': batch })
    renderAt('/play/endings')
    fireEvent.click(screen.getByLabelText('Instrumental'))
    fireEvent.click(screen.getByRole('button', { name: 'Start' }))
    await screen.findByText('кни́га')
  }

  test('practice says each word, and the form after a correct drop only', async () => {
    const spoken = mockAudio()
    await startPractice()
    await waitFor(() => expect(spoken).toEqual(['кни́га']))

    fireEvent.click(screen.getByRole('button', { name: '-ой' })) // correct
    // The form first, then the next word, which waited for it.
    await waitFor(() => expect(spoken).toEqual(['кни́га', 'кни́гой', 'стол']))

    fireEvent.click(screen.getByRole('button', { name: '-ой' })) // wrong: столо́м
    await screen.findByRole('heading', { name: 'Results' })
    expect(spoken).toEqual(['кни́га', 'кни́гой', 'стол'])
  })

  test('the card button says the word again', async () => {
    const spoken = mockAudio()
    await startPractice()
    await waitFor(() => expect(spoken).toHaveLength(1))

    fireEvent.click(screen.getByRole('button', { name: 'Listen' }))
    await waitFor(() => expect(spoken).toEqual(['кни́га', 'кни́га']))
    // Listening doesn't answer or select the word.
    expect(screen.getByText('0 / 10')).toBeInTheDocument()
  })

  test('muting silences everything and is remembered', async () => {
    const spoken = mockAudio()
    await startPractice()
    await waitFor(() => expect(spoken).toHaveLength(1))

    fireEvent.click(screen.getByRole('button', { name: 'Mute pronunciation' }))
    expect(screen.queryByRole('button', { name: 'Listen' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: '-ой' })) // correct, but muted
    await screen.findByText('стол')
    expect(spoken).toEqual(['кни́га'])
    expect(localStorage.getItem('speech.muted')).toBe('true')
  })
})

describe('explanations', () => {
  const okno: Item = {
    lexeme_id: 3,
    stressed_lemma: 'окно́',
    translations: { en: 'window' },
    number: 'singular',
    gender: null,
    animacy: null,
    answer: 'ом',
    accepted: ['ом'],
    stressed_form: 'окно́м',
    paradigm: 'noun_neuter_o',
  }

  test('a wrong drop in practice shows the rule and waits for Continue', async () => {
    mockApi({
      'POST /api/games/endings/batch': () =>
        json(200, { answer_kind: 'ending', zones: ['ом', 'ой'], items: [kniga, okno, stol] }),
    })
    renderAt('/play/endings')
    fireEvent.click(screen.getByLabelText('Instrumental'))
    fireEvent.click(screen.getByRole('button', { name: 'Start' }))
    await screen.findByText('кни́га')

    fireEvent.click(screen.getByRole('button', { name: '-ом' })) // wrong: кни́гой
    const panel = await screen.findByRole('complementary')
    expect(within(panel).getByText(/Nouns in -а\/-я/)).toBeInTheDocument()
    // Why -ом is wrong: where it belongs instead.
    expect(
      within(panel).getByText(/-ом is used for: Masculine nouns ending in a consonant/),
    ).toBeInTheDocument()
    expect(screen.queryByText('окно́')).not.toBeInTheDocument() // the next word waits
    expect(screen.getByText('кни́га')).toBeInTheDocument() // the word answered stays in view
    expect(screen.getByRole('button', { name: '-ой' })).toBeDisabled()

    fireEvent.click(screen.getByRole('button', { name: 'Continue' }))
    expect(await screen.findByText('окно́')).toBeInTheDocument()
    expect(screen.queryByRole('complementary')).not.toBeInTheDocument()
  })

  test('arcade mistakes are explained in the summary', async () => {
    mockApi({ 'POST /api/games/endings/batch': batch })
    renderAt('/play/endings')
    fireEvent.click(screen.getByLabelText(/Arcade/))
    fireEvent.click(screen.getByLabelText('Instrumental'))
    fireEvent.click(screen.getByRole('button', { name: 'Start' }))
    await screen.findByText('кни́га')

    fireEvent.click(screen.getByRole('button', { name: '-ом' })) // wrong, no pause in arcade
    expect(screen.queryByRole('complementary')).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Quit' }))

    await screen.findByRole('heading', { name: 'Results' })
    fireEvent.click(screen.getByText('кни́гой', { selector: 'summary strong' })) // open the mistake
    expect(
      await screen.findByRole('link', { name: /All the rules for this case/ }),
    ).toHaveAttribute('href', '/rules/instrumental#noun_a')
  })

  test('a rules page can open the game on its case', async () => {
    mockApi({})
    renderAt('/play/endings?case=dative')
    expect(screen.getByLabelText('Dative')).toBeChecked()
  })
})

describe('pause after a correct answer', () => {
  async function answerFirstCorrectly() {
    mockApi({ 'POST /api/games/endings/batch': batch })
    renderAt('/play/endings')
    fireEvent.click(screen.getByLabelText('Instrumental'))
    fireEvent.click(screen.getByRole('button', { name: 'Start' }))
    await screen.findByText('кни́га')
    fireEvent.click(screen.getByRole('button', { name: '-ой' })) // correct
  }

  test('the next word comes after the form has been heard, plus the delay', async () => {
    useDelay(1000)
    const spoken = mockAudio()
    await answerFirstCorrectly()

    await waitFor(() => expect(spoken).toEqual(['кни́га', 'кни́гой']))
    await new Promise((resolve) => setTimeout(resolve, 300))
    expect(screen.queryByText('стол')).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: '-ом' })).toBeDisabled()

    expect(await screen.findByText('стол', {}, { timeout: 2000 })).toBeInTheDocument()
    await waitFor(() => expect(spoken).toEqual(['кни́га', 'кни́гой', 'стол']))
  })

  test('Next skips the wait', async () => {
    useDelay(3000)
    await answerFirstCorrectly()
    fireEvent.click(await screen.findByRole('button', { name: 'Next →' }))
    expect(screen.getByText('стол')).toBeInTheDocument()
  })

  test('without sound there is no pause', async () => {
    useDelay(3000)
    localStorage.setItem('speech.muted', 'true')
    await answerFirstCorrectly()
    expect(await screen.findByText('стол')).toBeInTheDocument()
  })

  test('the delay is a practice setting', () => {
    mockApi({})
    renderAt('/play/endings')
    const select = screen.getByLabelText('Pause after a correct answer')
    expect(select).toHaveValue('0')
    fireEvent.change(select, { target: { value: '2000' } })
    expect(select).toHaveValue('2000')

    fireEvent.click(screen.getByLabelText(/Arcade/))
    expect(screen.queryByLabelText('Pause after a correct answer')).not.toBeInTheDocument()
  })
})
