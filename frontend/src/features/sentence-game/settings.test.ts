import { beforeEach, expect, test } from 'vitest'
import { asksCase, loadSettings, saveSettings } from './settings'

beforeEach(() => localStorage.clear())

test('settings are kept, and stale values replaced by the defaults', () => {
  const chosen = {
    levels: ['advanced_nouns'],
    cases: ['dative', 'genitive'],
    number: 'plural',
    roundLength: 30,
    endingsOnly: true,
    hideTranslation: true,
  } as const
  saveSettings({ ...chosen, levels: [...chosen.levels], cases: [...chosen.cases] })
  // Cases come back in the usual order.
  expect(loadSettings()).toEqual({ ...chosen, cases: ['genitive', 'dative'] })

  localStorage.setItem(
    'sentence-game.settings',
    JSON.stringify({
      levels: ['basic_adjectives'],
      cases: ['vocative'],
      number: 'dual',
      roundLength: 7,
    }),
  )
  const settings = loadSettings()
  expect(settings).toMatchObject({ levels: ['basic_nouns'], number: 'both', roundLength: 10 })
  expect(settings.cases).toHaveLength(6)
})

test('the case is asked only when there is a choice', () => {
  const settings = loadSettings()
  expect(asksCase(settings)).toBe(true)
  expect(asksCase({ ...settings, endingsOnly: true })).toBe(false)
  expect(asksCase({ ...settings, cases: ['genitive'] })).toBe(false)
  // Singular drops the nominative, leaving one case.
  expect(asksCase({ ...settings, number: 'singular', cases: ['nominative', 'genitive'] })).toBe(
    false,
  )
})
