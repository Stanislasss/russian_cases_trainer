import type { Item } from './gameReducer'

/** The word's meaning in the UI language (a definition for Russian), else English. */
export function translationFor(item: Item, language: string | undefined): string | undefined {
  const translations: Partial<Record<string, string | null>> = item.translations
  return (language && translations[language]) || item.translations.en || undefined
}
