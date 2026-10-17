import type { GrammaticalCase, SentenceItem, SentencesBatch } from './api'

export const ROUND = 10

/**
 * Where the player is with the current sentence: picking the case, reading why
 * the case was wrong, picking the ending, reading why the ending was wrong, or
 * hearing the finished sentence.
 */
export type Step = 'case' | 'caseMistake' | 'ending' | 'endingMistake' | 'correct'

export interface Result {
  item: SentenceItem
  /** The wrong case chosen first, if any. */
  wrongCase: GrammaticalCase | null
  /** The wrong ending chosen first, if any. */
  wrongEnding: string | null
  correct: boolean
}

export interface Playing {
  phase: 'playing'
  items: SentenceItem[]
  zones: Partial<Record<GrammaticalCase, string[]>>
  index: number
  step: Step
  wrongCase: GrammaticalCase | null
  wrongEnding: string | null
  results: Result[]
}

export interface Finished {
  phase: 'summary'
  results: Result[]
}

export type GameState = { phase: 'setup' } | Playing | Finished

export type GameAction =
  | { type: 'start'; batch: SentencesBatch }
  | { type: 'chooseCase'; case: GrammaticalCase }
  | { type: 'chooseEnding'; ending: string }
  | { type: 'continue' }
  | { type: 'quit' }
  | { type: 'reset' }

export const initialState: GameState = { phase: 'setup' }

export function current(state: Playing): SentenceItem {
  return state.items[state.index]
}

export function gameReducer(state: GameState, action: GameAction): GameState {
  switch (action.type) {
    case 'start':
      return {
        phase: 'playing',
        items: action.batch.items.slice(0, ROUND),
        zones: Object.fromEntries(action.batch.zones.map((z) => [z.case, z.endings])),
        index: 0,
        step: 'case',
        wrongCase: null,
        wrongEnding: null,
        results: [],
      }
    case 'chooseCase': {
      if (state.phase !== 'playing' || state.step !== 'case') return state
      return action.case === current(state).case
        ? { ...state, step: 'ending' }
        : { ...state, step: 'caseMistake', wrongCase: action.case }
    }
    case 'chooseEnding': {
      if (state.phase !== 'playing' || state.step !== 'ending') return state
      return current(state).word.accepted.includes(action.ending)
        ? { ...state, step: 'correct' }
        : { ...state, step: 'endingMistake', wrongEnding: action.ending }
    }
    case 'continue': {
      if (state.phase !== 'playing') return state
      if (state.step === 'caseMistake') return { ...state, step: 'ending' }
      if (state.step !== 'endingMistake' && state.step !== 'correct') return state
      const results = [
        ...state.results,
        {
          item: current(state),
          wrongCase: state.wrongCase,
          wrongEnding: state.wrongEnding,
          correct: state.wrongCase === null && state.wrongEnding === null,
        },
      ]
      if (state.index + 1 >= state.items.length) return { phase: 'summary', results }
      return {
        ...state,
        index: state.index + 1,
        step: 'case',
        wrongCase: null,
        wrongEnding: null,
        results,
      }
    }
    case 'quit':
      return state.phase === 'playing' ? { phase: 'summary', results: state.results } : state
    case 'reset':
      return initialState
  }
}

/** The finished sentence, as spoken and shown after a right answer. */
export function fullSentence(item: SentenceItem): string {
  return `${item.before}${item.word.stressed_form}${item.after}`
}
