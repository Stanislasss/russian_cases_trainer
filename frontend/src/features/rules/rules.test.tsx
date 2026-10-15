import { fireEvent, screen, within } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import i18n from '../../shared/i18n'
import { mockApi, mockAudio, renderAt } from '../../test/app'

beforeEach(async () => {
  mockApi({})
  mockAudio()
  await i18n.changeLanguage('en')
})
afterEach(() => vi.unstubAllGlobals())

test('the overview links to every case', () => {
  renderAt('/rules')
  expect(screen.getByRole('heading', { name: 'Grammar rules' })).toBeInTheDocument()
  const links = screen.getAllByRole('link', {
    name: /Nominative|Genitive|Dative|Accusative|Instrumental|Prepositional/,
  })
  expect(links.map((a) => a.getAttribute('href'))).toEqual(
    expect.arrayContaining(['/rules/genitive', '/rules/prepositional']),
  )
})

test('a case page has uses, ending tables and exceptions', () => {
  renderAt('/rules/genitive')
  expect(screen.getByRole('heading', { level: 1, name: 'Genitive' })).toBeInTheDocument()
  expect(screen.getByText('Absence: нет, не́ было, не бу́дет')).toBeInTheDocument()
  // The row a wrong answer links to (#noun_a), with its genitive endings.
  const row = document.getElementById('noun_a')!
  expect(within(row).getByText('-ы / -и')).toBeInTheDocument()
  expect(screen.getByText('Fleeting vowel')).toBeInTheDocument()
  expect(screen.getByRole('link', { name: 'Practise this case' })).toHaveAttribute(
    'href',
    '/play/endings?case=genitive',
  )
})

test('examples can be listened to', async () => {
  const spoken = mockAudio()
  renderAt('/rules/dative')
  // once the server has said it can speak
  fireEvent.click(await screen.findByRole('button', { name: 'Listen: Мне хо́лодно.' }))
  expect(spoken).toEqual(['Мне хо́лодно.'])
})

test.each([
  ['fr', 'Génitif', 'Absence : нет, не́ было, не бу́дет'],
  ['de', 'Genitiv', 'Fehlen: нет, не́ было, не бу́дет'],
  ['ru', 'Родительный', 'Отсутствие: нет, не́ было, не бу́дет'],
])('the rules speak %s', async (language, caseName, use) => {
  await i18n.changeLanguage(language)
  renderAt('/rules/genitive')
  expect(screen.getByRole('heading', { level: 1, name: caseName })).toBeInTheDocument()
  expect(screen.getByText(use)).toBeInTheDocument()
})

test('an unknown case goes back to the overview', () => {
  renderAt('/rules/vocative')
  expect(screen.getByRole('heading', { name: 'Grammar rules' })).toBeInTheDocument()
})
