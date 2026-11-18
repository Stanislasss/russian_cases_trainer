import { cleanup, fireEvent, screen, waitFor, within } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import i18n from '../../shared/i18n'
import { json, mockApi, mockAudio, renderAt, requestBodies } from '../../test/app'
import type { SentenceItem } from './api'

const kniga: SentenceItem = {
  template_id: 'prep.think',
  before: 'Я ду́маю о ',
  after: '.',
  case: 'prepositional',
  reason: 'about',
  translations: { en: 'I am thinking about ___.' },
  noun_phrases: { en: 'the book' },
  shown: 'кни́га',
  word: {
    lexeme_id: 1,
    stressed_lemma: 'кни́га',
    translations: { en: 'book; volume' },
    number: 'singular',
    gender: null,
    animacy: null,
    answer: 'е',
    accepted: ['е'],
    stressed_form: 'кни́ге',
    paradigm: 'noun_a',
  },
}
const stol: SentenceItem = {
  template_id: 'gen.none',
  before: 'У меня́ нет ',
  after: '.',
  case: 'genitive',
  reason: 'absence',
  translations: { en: 'I have no ___.' },
  noun_phrases: {},
  shown: 'стол',
  word: {
    lexeme_id: 2,
    stressed_lemma: 'стол',
    translations: { en: 'table' },
    number: 'singular',
    gender: null,
    animacy: null,
    answer: 'а',
    accepted: ['а'],
    stressed_form: 'стола́',
    paradigm: 'noun_masculine_hard',
  },
}
const batch = () =>
  json(200, {
    zones: [
      { case: 'genitive', endings: ['а', 'ы'] },
      { case: 'prepositional', endings: ['е', 'и'] },
    ],
    items: [kniga, stol],
  })

let spoken: string[]
beforeEach(async () => {
  localStorage.clear()
  spoken = mockAudio()
  await i18n.changeLanguage('en')
})
afterEach(() => vi.unstubAllGlobals())

/** Starts a round with the default settings, or `settings` as if saved last time. */
async function startRound(settings?: object) {
  if (settings) localStorage.setItem('sentence-game.settings', JSON.stringify(settings))
  const fetchMock = mockApi({ 'POST /api/games/sentences/batch': batch })
  renderAt('/play/sentences')
  fireEvent.click(screen.getByRole('button', { name: 'Start' }))
  expect(await screen.findByText('кни́га')).toBeInTheDocument()
  return fetchMock
}

test('only noun levels are offered, basic by default', async () => {
  const fetchMock = await startRound()
  const [body] = await requestBodies(fetchMock, '/api/games/sentences/batch')
  expect(body).toEqual({
    levels: ['basic_nouns'],
    cases: ['nominative', 'genitive', 'dative', 'accusative', 'instrumental', 'prepositional'],
    numbers: ['singular', 'plural'],
    count: 10,
  })
})

/** Starts a round of just `item`. */
async function startWith(item: SentenceItem) {
  mockApi({
    'POST /api/games/sentences/batch': () =>
      json(200, { zones: [{ case: item.case, endings: ['', 'ы'] }], items: [item] }),
  })
  renderAt('/play/sentences')
  fireEvent.click(screen.getByRole('button', { name: 'Start' }))
}

const komnaty: SentenceItem = {
  template_id: 'gen.afraid',
  before: 'Я бою́сь ',
  after: '.',
  case: 'genitive',
  reason: 'verbs',
  translations: { en: 'I am afraid of ___.' },
  noun_phrases: { en: 'the rooms' },
  shown: 'ко́мнаты',
  word: {
    lexeme_id: 3,
    stressed_lemma: 'ко́мната',
    translations: { en: 'room' },
    number: 'plural',
    gender: null,
    animacy: null,
    answer: '',
    accepted: [''],
    stressed_form: 'ко́мнат',
    paradigm: 'noun_a',
  },
}

test('a plural is shown in the plural', async () => {
  await startWith(komnaty)
  expect(await screen.findByText('ко́мнаты')).toBeInTheDocument()
  expect(screen.queryByText('ко́мната')).not.toBeInTheDocument()
  expect(screen.queryByText('in the plural')).not.toBeInTheDocument()
})

test('in nominative frames, where the plural is the answer, it is only labelled', async () => {
  await startWith({
    ...komnaty,
    template_id: 'nom.where',
    before: 'Где ',
    after: '?',
    case: 'nominative',
    reason: 'subject',
    shown: 'ко́мната',
    word: { ...komnaty.word, answer: 'ы', accepted: ['ы'], stressed_form: 'ко́мнаты' },
  })
  expect(await screen.findByText('ко́мната')).toBeInTheDocument()
  expect(screen.getByText('in the plural')).toBeInTheDocument()
})

test('the sentence shows the noun and the translation with its meaning', async () => {
  await startRound()
  expect(screen.getByText('Я ду́маю о', { exact: false })).toBeInTheDocument()
  expect(screen.getByText('the book')).toBeInTheDocument() // with its article
  expect(screen.getByText('Which case?')).toBeInTheDocument()
})

test('after a right answer, the case’s rules open over the game', async () => {
  await startRound()
  fireEvent.click(screen.getByRole('button', { name: /^Prepositional/ }))
  fireEvent.click(await screen.findByRole('button', { name: '-е' }))

  fireEvent.click(await screen.findByRole('button', { name: /All the rules for this case/ }))
  const rules = screen.getByRole('dialog', { name: 'Prepositional' })
  expect(within(rules).getByRole('heading', { name: 'Prepositional' })).toBeInTheDocument()
  // Enter belongs to the dialog, not to the game behind it.
  fireEvent.keyDown(within(rules).getByRole('button', { name: 'Close' }), { key: 'Enter' })
  expect(screen.getByText('кни́ге')).toBeInTheDocument()

  fireEvent.click(within(rules).getByRole('button', { name: 'Close' }))
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  fireEvent.click(screen.getByRole('button', { name: /Next/ }))
  expect(await screen.findByText('стол')).toBeInTheDocument()
})

test('the rules of the right case open from a wrong case’s explanation', async () => {
  await startRound()
  fireEvent.click(screen.getByRole('button', { name: /^Genitive/ }))
  const why = await screen.findByRole('complementary')
  fireEvent.click(within(why).getByRole('button', { name: /All the rules for this case/ }))
  expect(screen.getByRole('dialog', { name: 'Prepositional' })).toBeInTheDocument()
})

test('a wrong case is explained, then the ending is asked', async () => {
  await startRound()
  fireEvent.click(screen.getByRole('button', { name: /^Genitive/ }))

  const why = await screen.findByRole('complementary')
  expect(within(why).getByText('Here it’s the Prepositional')).toBeInTheDocument()
  expect(within(why).getByText('About: о (об, обо)')).toBeInTheDocument()
  expect(within(why).getByText('The Genitive is used for:')).toBeInTheDocument()
  expect(within(why).getByText('Absence: нет, не́ было, не бу́дет')).toBeInTheDocument()

  fireEvent.click(screen.getByRole('button', { name: 'Continue' }))
  expect(await screen.findByText('Which ending?')).toBeInTheDocument()
  expect(screen.getByRole('button', { name: '-е' })).toBeEnabled()
})

test('a wrong ending shows the rule, a right one completes the sentence', async () => {
  await startRound()
  fireEvent.click(screen.getByRole('button', { name: /^Prepositional/ }))
  fireEvent.click(await screen.findByRole('button', { name: '-и' }))
  const rule = await screen.findByRole('complementary')
  expect(within(rule).getByText(/Nouns in -а\/-я/)).toBeInTheDocument()
  fireEvent.click(screen.getByRole('button', { name: 'Continue' }))

  // The next sentence: right case, right ending.
  expect(await screen.findByText('стол')).toBeInTheDocument()
  fireEvent.keyDown(window, { key: '2' }) // Genitive
  fireEvent.keyDown(window, { key: '1' }) // -а
  expect(await screen.findByText('стола́')).toBeInTheDocument()
  await waitFor(() => expect(spoken).toContain('У меня́ нет стола́.'))
})

test('the summary lists the mistakes with their explanation', async () => {
  await startRound()
  fireEvent.click(screen.getByRole('button', { name: /^Accusative/ })) // wrong case
  fireEvent.click(screen.getByRole('button', { name: 'Continue' }))
  fireEvent.click(await screen.findByRole('button', { name: '-е' }))
  fireEvent.click(await screen.findByRole('button', { name: 'Next →' }))
  await screen.findByText('стол')
  fireEvent.click(screen.getByRole('button', { name: 'Quit' }))

  expect(await screen.findByRole('heading', { name: 'Results' })).toBeInTheDocument()
  expect(screen.getByText('0 of 1 correct')).toBeInTheDocument()
  fireEvent.click(screen.getByText('кни́ге'))
  expect(await screen.findByText('Here it’s the Prepositional')).toBeInTheDocument()
})

test('the noun can be heard, and a wrong ending plays the corrected sentence', async () => {
  await startRound()
  fireEvent.click(screen.getByRole('button', { name: 'Listen: кни́га' }))
  await waitFor(() => expect(spoken).toContain('кни́га'))

  fireEvent.click(screen.getByRole('button', { name: /^Prepositional/ }))
  fireEvent.click(await screen.findByRole('button', { name: '-и' })) // wrong
  // The right form is shown and heard, while the rule explains the mistake.
  expect(await screen.findByText('кни́ге', { selector: 'p[lang=ru] strong' })).toBeInTheDocument()
  await waitFor(() => expect(spoken).toContain('Я ду́маю о кни́ге.'))
  expect(screen.getByRole('button', { name: 'Continue' })).toBeInTheDocument()
})

test('the options go with the request', async () => {
  const fetchMock = mockApi({ 'POST /api/games/sentences/batch': batch })
  renderAt('/play/sentences')
  for (const c of ['Nominative', 'Dative', 'Accusative', 'Instrumental']) {
    fireEvent.click(screen.getByRole('checkbox', { name: c }))
  }
  fireEvent.click(screen.getByRole('radio', { name: 'Plural' }))
  fireEvent.click(screen.getByRole('radio', { name: '20 sentences' }))
  fireEvent.click(screen.getByRole('button', { name: 'Start' }))
  await screen.findByText('кни́га')

  const [body] = await requestBodies(fetchMock, '/api/games/sentences/batch')
  expect(body).toEqual({
    levels: ['basic_nouns'],
    cases: ['genitive', 'prepositional'],
    numbers: ['plural'],
    count: 20,
  })
})

test('only the chosen cases are offered', async () => {
  await startRound({ levels: ['basic_nouns'], cases: ['genitive', 'prepositional'] })
  expect(screen.getByText('Which case?')).toBeInTheDocument()
  expect(screen.queryByRole('button', { name: /^Dative/ })).not.toBeInTheDocument()
  fireEvent.keyDown(window, { key: '2' }) // the second offered: Prepositional
  expect(await screen.findByText('Which ending?')).toBeInTheDocument()
})

test('with one case, or endings only, the case is not asked', async () => {
  await startRound({ levels: ['basic_nouns'], cases: ['prepositional'] })
  expect(screen.queryByText('Which case?')).not.toBeInTheDocument()
  expect(screen.getByText('Which ending?')).toBeInTheDocument()
  vi.unstubAllGlobals()
  cleanup()

  await startRound({ levels: ['basic_nouns'], endingsOnly: true })
  expect(screen.queryByText('Which case?')).not.toBeInTheDocument()
  fireEvent.click(screen.getByRole('button', { name: '-е' }))
  // The next sentence starts at its ending too.
  fireEvent.click(await screen.findByRole('button', { name: 'Next →' }))
  expect(await screen.findByText('стол')).toBeInTheDocument()
  expect(screen.getByText('Which ending?')).toBeInTheDocument()
})

test('a hidden translation shows once the sentence is answered', async () => {
  await startRound({ levels: ['basic_nouns'], hideTranslation: true })
  expect(screen.queryByText('the book')).not.toBeInTheDocument()
  fireEvent.click(screen.getByRole('button', { name: /^Prepositional/ }))
  fireEvent.click(await screen.findByRole('button', { name: '-е' }))
  expect(await screen.findByText('the book')).toBeInTheDocument()
})

test('the nominative needs the plural', async () => {
  mockApi({})
  renderAt('/play/sentences')
  fireEvent.click(screen.getByRole('radio', { name: 'Singular' }))
  expect(screen.getByRole('checkbox', { name: 'Nominative' })).toBeDisabled()
  expect(screen.getByText('The nominative is only asked in the plural.')).toBeInTheDocument()

  for (const c of ['Genitive', 'Dative', 'Accusative', 'Instrumental', 'Prepositional']) {
    fireEvent.click(screen.getByRole('checkbox', { name: c }))
  }
  expect(screen.getByRole('alert')).toHaveTextContent('Choose at least one case.')
  expect(screen.getByRole('button', { name: 'Start' })).toBeDisabled()
})
