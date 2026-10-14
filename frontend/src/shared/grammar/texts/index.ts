import { useTranslation } from 'react-i18next'
import { de } from './de'
import { en } from './en'
import { fr } from './fr'
import { ru } from './ru'
import type { GrammarTexts } from './types'

const TEXTS: Record<string, GrammarTexts> = { en, fr, de, ru }

/** The grammar texts in the current UI language (English if missing). */
export function useGrammarTexts(): GrammarTexts {
  const { i18n } = useTranslation()
  return TEXTS[i18n.resolvedLanguage ?? 'en'] ?? en
}

export type { GrammarTexts } from './types'
