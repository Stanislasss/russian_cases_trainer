import type { GrammaticalCase, Level } from './api'
import type { Mode } from './gameReducer'

export interface Settings {
  levels: Level[]
  case: GrammaticalCase
  mode: Mode
}

/** The level checkboxes, grouped under headings. */
export const LEVEL_GROUPS: { id: string; levels: Level[] }[] = [
  { id: 'nouns', levels: ['basic_nouns', 'intermediate_nouns', 'advanced_nouns'] },
  { id: 'adjectives', levels: ['basic_adjectives', 'intermediate_adjectives'] },
  { id: 'pronouns', levels: ['pronouns', 'personal_pronouns'] },
  { id: 'names', levels: ['surnames'] },
]

/** Levels played on their own: their zones are whole forms (мной), not endings. */
export const SOLO_LEVELS: Level[] = ['personal_pronouns']

export const CASES: GrammaticalCase[] = [
  'nominative',
  'genitive',
  'dative',
  'accusative',
  'instrumental',
  'prepositional',
]

/** Checks or unchecks `level`, keeping solo levels apart from the others. */
export function toggleLevel(levels: Level[], level: Level): Level[] {
  if (levels.includes(level)) return levels.filter((l) => l !== level)
  if (SOLO_LEVELS.includes(level)) return [level]
  return [...levels.filter((l) => !SOLO_LEVELS.includes(l)), level]
}

const KEY = 'endings-game.settings'
const defaults: Settings = { levels: ['basic_nouns'], case: 'genitive', mode: 'practice' }
const knownLevels = new Set<string>(LEVEL_GROUPS.flatMap((g) => g.levels))

/** The last used settings, ignoring anything stale or tampered with in localStorage. */
export function loadSettings(): Settings {
  try {
    const saved = JSON.parse(localStorage.getItem(KEY) ?? 'null') as Partial<Settings> | null
    if (!saved) return defaults
    let levels = (saved.levels ?? []).filter((l) => knownLevels.has(l))
    if (levels.some((l) => SOLO_LEVELS.includes(l)) && levels.length > 1) levels = [levels[0]]
    return {
      levels: levels.length > 0 ? levels : defaults.levels,
      case: saved.case && CASES.includes(saved.case) ? saved.case : defaults.case,
      mode: saved.mode === 'arcade' ? 'arcade' : 'practice',
    }
  } catch {
    return defaults
  }
}

export function saveSettings(settings: Settings) {
  localStorage.setItem(KEY, JSON.stringify(settings))
}
