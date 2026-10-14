import type { GrammaticalCase, Level } from './api'
import type { Mode } from './gameReducer'

export interface Settings {
  levels: Level[]
  case: GrammaticalCase
  mode: Mode
}

/** Shown as checkboxes; unavailable ones are greyed out until their words are imported. */
export const LEVEL_OPTIONS: { id: string; level?: Level }[] = [
  { id: 'basic_nouns', level: 'basic_nouns' },
  { id: 'intermediate_nouns', level: 'intermediate_nouns' },
  { id: 'advanced_nouns', level: 'advanced_nouns' },
  { id: 'basic_adjectives' },
  { id: 'intermediate_adjectives' },
]

export const CASES: GrammaticalCase[] = [
  'nominative',
  'genitive',
  'dative',
  'accusative',
  'instrumental',
  'prepositional',
]

const KEY = 'endings-game.settings'
const defaults: Settings = { levels: ['basic_nouns'], case: 'genitive', mode: 'practice' }
const knownLevels = new Set(LEVEL_OPTIONS.flatMap((o) => (o.level ? [o.level] : [])))

/** The last used settings, ignoring anything stale or tampered with in localStorage. */
export function loadSettings(): Settings {
  try {
    const saved = JSON.parse(localStorage.getItem(KEY) ?? 'null') as Partial<Settings> | null
    if (!saved) return defaults
    const levels = (saved.levels ?? []).filter((l) => knownLevels.has(l))
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
