import type { Level } from './api'

/** Sentences are filled with nouns only. */
export const NOUN_LEVELS: Level[] = ['basic_nouns', 'intermediate_nouns', 'advanced_nouns']

export interface Settings {
  levels: Level[]
}

const KEY = 'sentence-game.settings'
const defaults: Settings = { levels: ['basic_nouns'] }

/** The last used levels, ignoring anything stale in localStorage. */
export function loadSettings(): Settings {
  try {
    const saved = JSON.parse(localStorage.getItem(KEY) ?? 'null') as Partial<Settings> | null
    const levels = (saved?.levels ?? []).filter((l) => NOUN_LEVELS.includes(l))
    return { levels: levels.length > 0 ? levels : defaults.levels }
  } catch {
    return defaults
  }
}

export function saveSettings(settings: Settings) {
  localStorage.setItem(KEY, JSON.stringify(settings))
}
