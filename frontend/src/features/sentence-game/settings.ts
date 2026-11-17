import { CASES } from '../endings-game/settings'
import type { GrammaticalCase, Level } from './api'

/** Sentences are filled with nouns only. */
export const NOUN_LEVELS: Level[] = ['basic_nouns', 'intermediate_nouns', 'advanced_nouns']

/** Which forms are asked, whatever the levels say (basic nouns are singular only). */
export type NumberChoice = 'singular' | 'plural' | 'both'
export const NUMBER_CHOICES: NumberChoice[] = ['singular', 'plural', 'both']

export const ROUND_LENGTHS = [10, 20, 30] as const
export type RoundLength = (typeof ROUND_LENGTHS)[number]

export interface Settings {
  levels: Level[]
  cases: GrammaticalCase[]
  number: NumberChoice
  roundLength: RoundLength
  /** Skip "Which case?": the case is shown, only the ending is asked. */
  endingsOnly: boolean
  /** The sentence's translation shows only once it's answered. */
  hideTranslation: boolean
}

const KEY = 'sentence-game.settings'
const defaults: Settings = {
  levels: ['basic_nouns'],
  cases: CASES,
  number: 'both',
  roundLength: 10,
  endingsOnly: false,
  hideTranslation: false,
}

/** The last used settings, ignoring anything stale in localStorage. */
export function loadSettings(): Settings {
  try {
    const saved = JSON.parse(localStorage.getItem(KEY) ?? 'null') as Partial<Settings> | null
    if (!saved) return defaults
    const levels = (saved.levels ?? []).filter((l) => NOUN_LEVELS.includes(l))
    // Kept in the usual order, whatever order they were picked in.
    const cases = CASES.filter((c) => saved.cases?.includes(c))
    return {
      levels: levels.length > 0 ? levels : defaults.levels,
      cases: cases.length > 0 ? cases : defaults.cases,
      number: NUMBER_CHOICES.find((n) => n === saved.number) ?? defaults.number,
      roundLength: ROUND_LENGTHS.find((n) => n === saved.roundLength) ?? defaults.roundLength,
      endingsOnly: saved.endingsOnly === true,
      hideTranslation: saved.hideTranslation === true,
    }
  } catch {
    return defaults
  }
}

export function saveSettings(settings: Settings) {
  localStorage.setItem(KEY, JSON.stringify(settings))
}

/** Whether each sentence asks for its case first: not when the case is a given. */
export function asksCase(settings: Settings): boolean {
  return !settings.endingsOnly && playableCases(settings).length > 1
}

/** The nominative is only asked in the plural (the singular is the word shown). */
export function playableCases(settings: Settings): GrammaticalCase[] {
  return settings.number === 'singular'
    ? settings.cases.filter((c) => c !== 'nominative')
    : settings.cases
}

/** Toggles `c`, keeping the usual order. */
export function toggleCase(cases: GrammaticalCase[], c: GrammaticalCase): GrammaticalCase[] {
  return CASES.filter((x) => (x === c ? !cases.includes(c) : cases.includes(x)))
}
