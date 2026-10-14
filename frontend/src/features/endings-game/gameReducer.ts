import type { Schemas } from '../../shared/api/client'

export type Item = Schemas['EndingsItem']
export type Mode = 'practice' | 'arcade'
/** Whether zones are endings (-ой) or whole forms (мной). */
export type AnswerKind = Schemas['AnswerType']
export type GrammaticalCase = Schemas['GrammaticalCase']

export const PRACTICE_ROUND = 10
export const ARCADE_LIVES = 3
/** At most this many words on screen at once in arcade mode. */
const MAX_FALLING = 4

/** A word on the board. Practice shows one at a time; arcade lets several fall. */
export interface Word {
  id: number
  item: Item
  spawnedAt: number
  /** Time (ms) to reach the bottom; Infinity in practice mode. */
  duration: number
}

export interface Result {
  item: Item
  /** The chosen ending, or null when the word fell off the board. */
  chosen: string | null
  correct: boolean
}

export interface Playing {
  phase: 'playing'
  mode: Mode
  answerKind: AnswerKind
  /** The case every word is put into. */
  gameCase: GrammaticalCase
  zones: string[]
  queue: Item[]
  words: Word[]
  results: Result[]
  score: number
  streak: number
  lives: number
  nextId: number
  nextSpawnAt: number
}

export interface Finished {
  phase: 'summary'
  mode: Mode
  answerKind: AnswerKind
  /** The case every word is put into. */
  gameCase: GrammaticalCase
  results: Result[]
  score: number
}

export type GameState = { phase: 'setup' } | Playing | Finished

export type GameAction =
  | {
      type: 'start'
      mode: Mode
      answerKind: AnswerKind
      /** The case every word is put into. */
      gameCase: GrammaticalCase
      zones: string[]
      items: Item[]
      now: number
    }
  | { type: 'refill'; items: Item[] }
  | { type: 'answer'; wordId: number; ending: string; now: number }
  | { type: 'tick'; now: number }
  | { type: 'quit' }
  | { type: 'reset' }

export const initialState: GameState = { phase: 'setup' }

export function isCorrect(item: Item, ending: string): boolean {
  return item.accepted.includes(ending)
}

/** Arcade gets faster with every correct answer. */
export function arcadePace(correctSoFar: number) {
  return {
    spawnEvery: Math.max(900, 2600 - 100 * correctSoFar),
    fallDuration: Math.max(4000, 9000 - 150 * correctSoFar),
  }
}

export function gameReducer(state: GameState, action: GameAction): GameState {
  switch (action.type) {
    case 'start': {
      const playing: Playing = {
        phase: 'playing',
        mode: action.mode,
        answerKind: action.answerKind,
        gameCase: action.gameCase,
        zones: action.zones,
        queue: action.mode === 'practice' ? action.items.slice(0, PRACTICE_ROUND) : action.items,
        words: [],
        results: [],
        score: 0,
        streak: 0,
        lives: ARCADE_LIVES,
        nextId: 1,
        nextSpawnAt: action.now,
      }
      return spawn(playing, action.now)
    }
    case 'refill':
      return state.phase === 'playing' && state.mode === 'arcade'
        ? { ...state, queue: [...state.queue, ...action.items] }
        : state
    case 'answer': {
      if (state.phase !== 'playing') return state
      const word = state.words.find((w) => w.id === action.wordId)
      if (!word) return state
      const correct = isCorrect(word.item, action.ending)
      const next = record(state, word, action.ending, correct)
      return next.phase === 'playing' ? spawn(next, action.now) : next
    }
    case 'tick': {
      if (state.phase !== 'playing' || state.mode !== 'arcade') return state
      let next: GameState = state
      for (const word of state.words) {
        if (action.now - word.spawnedAt >= word.duration && next.phase === 'playing') {
          next = record(next, word, null, false)
        }
      }
      return next.phase === 'playing' ? spawn(next, action.now) : next
    }
    case 'quit':
      return state.phase === 'playing' ? finish(state) : state
    case 'reset':
      return initialState
  }
}

/** Removes `word` from the board and books the result. */
function record(state: Playing, word: Word, chosen: string | null, correct: boolean): GameState {
  const results = [...state.results, { item: word.item, chosen, correct }]
  const arcade = state.mode === 'arcade'
  const streak = correct ? state.streak + 1 : 0
  const next: Playing = {
    ...state,
    words: state.words.filter((w) => w.id !== word.id),
    results,
    streak,
    // Arcade rewards streaks; practice simply counts correct answers.
    score: state.score + (correct ? (arcade ? 10 + 2 * Math.min(state.streak, 10) : 1) : 0),
    lives: arcade && !correct ? state.lives - 1 : state.lives,
  }
  if (arcade && next.lives <= 0) return finish(next)
  if (!arcade && results.length >= PRACTICE_ROUND) return finish(next)
  if (!arcade && next.queue.length === 0 && next.words.length === 0) return finish(next)
  return next
}

/** Puts the next word(s) on the board when it's time. */
function spawn(state: Playing, now: number): Playing {
  if (state.mode === 'practice') {
    if (state.words.length > 0 || state.queue.length === 0) return state
    const [item, ...queue] = state.queue
    return {
      ...state,
      queue,
      words: [{ id: state.nextId, item, spawnedAt: now, duration: Infinity }],
      nextId: state.nextId + 1,
    }
  }

  // An empty board gets a word right away instead of waiting for the next slot.
  const due = state.words.length === 0 || now >= state.nextSpawnAt
  if (!due || state.words.length >= MAX_FALLING || state.queue.length === 0) {
    return state
  }
  const [item, ...queue] = state.queue
  const pace = arcadePace(state.results.filter((r) => r.correct).length)
  return {
    ...state,
    queue,
    words: [
      ...state.words,
      { id: state.nextId, item, spawnedAt: now, duration: pace.fallDuration },
    ],
    nextId: state.nextId + 1,
    nextSpawnAt: now + pace.spawnEvery,
  }
}

function finish(state: Playing): Finished {
  return {
    phase: 'summary',
    mode: state.mode,
    answerKind: state.answerKind,
    gameCase: state.gameCase,
    results: state.results,
    score: state.score,
  }
}

/** Arcade asks for more words before the queue runs dry. */
export function needsRefill(state: GameState): boolean {
  return state.phase === 'playing' && state.mode === 'arcade' && state.queue.length < 5
}
