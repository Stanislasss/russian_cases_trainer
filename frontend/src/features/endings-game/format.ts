import type { AnswerKind, Item } from './gameReducer'

/** The word's meaning in the UI language (a definition for Russian), else English. */
export function translationFor(item: Item, language: string | undefined): string | undefined {
  const translations: Partial<Record<string, string | null>> = item.translations
  return (language && translations[language]) || item.translations.en || undefined
}

/** Endings as the player sees them: "-ой", or "∅" for the zero ending. */
export function endingLabel(ending: string) {
  return ending === '' ? '∅' : `-${ending}`
}

/** A zone or answer: an ending ("-ой"), or a whole form as is ("мной"). */
export function answerLabel(answer: string, kind: AnswerKind) {
  return kind === 'ending' ? endingLabel(answer) : answer
}
