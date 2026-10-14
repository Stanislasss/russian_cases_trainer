import type { Schemas } from '../api/client'

export type AnswerKind = Schemas['AnswerType']

/** Endings as the player sees them: "-ой", or "∅" for the zero ending. */
export function endingLabel(ending: string) {
  return ending === '' ? '∅' : `-${ending}`
}

/** A zone or answer: an ending ("-ой"), or a whole form as is ("мной"). */
export function answerLabel(answer: string, kind: AnswerKind) {
  return kind === 'ending' ? endingLabel(answer) : answer
}

/** Example notation the voice shouldn't read out: "дом → дома́" → "дом, дома́". */
export function speakable(text: string) {
  return text.replaceAll('→', ',').replaceAll('/', ',')
}
