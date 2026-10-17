import { screen } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import i18n from '../../shared/i18n'
import { mockApi, problem, renderAt } from '../../test/app'

beforeEach(async () => {
  await i18n.changeLanguage('en')
})
afterEach(() => vi.unstubAllGlobals())

test('the home page leads to the rules, both games and each case', async () => {
  mockApi({})
  renderAt('/')
  const hrefs = (await screen.findAllByRole('link')).map((l) => l.getAttribute('href'))
  expect(hrefs).toEqual(
    expect.arrayContaining(['/rules', '/play/endings', '/play/sentences', '/rules/genitive']),
  )
  expect(screen.getByRole('heading', { name: 'Sentences' })).toBeInTheDocument()
  expect(screen.getByText('кого́? чего́?')).toBeInTheDocument()
  expect(screen.queryByRole('alert')).not.toBeInTheDocument()
})

test('a warning shows only when the server is unreachable', async () => {
  mockApi({ 'GET /api/health': () => problem(500, 'internal') })
  renderAt('/')
  expect(await screen.findByRole('alert')).toHaveTextContent('isn’t responding')
})

test('the version is shown in the footer', async () => {
  mockApi({})
  renderAt('/')
  expect(await screen.findByText('v1.2.3')).toBeInTheDocument()
})
