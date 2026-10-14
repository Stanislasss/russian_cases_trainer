import { expect, test } from 'vitest'
import { toggleLevel } from './settings'

test('solo levels replace the others and are replaced by them', () => {
  expect(toggleLevel(['basic_nouns'], 'basic_adjectives')).toEqual([
    'basic_nouns',
    'basic_adjectives',
  ])
  expect(toggleLevel(['basic_nouns', 'surnames'], 'personal_pronouns')).toEqual([
    'personal_pronouns',
  ])
  expect(toggleLevel(['personal_pronouns'], 'pronouns')).toEqual(['pronouns'])
  expect(toggleLevel(['pronouns', 'surnames'], 'pronouns')).toEqual(['surnames'])
})
