import { describe, expect, test } from 'vitest'
import {
  ARCADE_LIVES,
  arcadePace,
  gameReducer,
  type GameAction,
  type GameState,
  initialState,
  type Item,
  needsRefill,
  type Playing,
  PRACTICE_ROUND,
} from './gameReducer'

function item(n: number, answer = 'ой', accepted = [answer]): Item {
  return {
    lexeme_id: n,
    stressed_lemma: `слово${n}`,
    translations: {},
    number: 'singular',
    gender: null,
    animacy: null,
    answer,
    accepted,
    stressed_form: `слово${n}${answer}`,
  }
}

const items = (count: number) => Array.from({ length: count }, (_, i) => item(i + 1))

function run(actions: GameAction[], state: GameState = initialState): GameState {
  return actions.reduce(gameReducer, state)
}

function playing(state: GameState): Playing {
  if (state.phase !== 'playing') throw new Error(`expected playing, got ${state.phase}`)
  return state
}

const start = (mode: 'practice' | 'arcade', list: Item[]): GameAction => ({
  type: 'start',
  mode,
  answerKind: 'ending',
  gameCase: 'instrumental',
  zones: ['ом', 'ой'],
  items: list,
  now: 0,
})

/** Answers the first word on the board. */
const answerFirst = (state: GameState, ending: string, now = 0): GameState =>
  gameReducer(state, { type: 'answer', wordId: playing(state).words[0].id, ending, now })

describe('practice', () => {
  test('shows one word at a time and ends after a round', () => {
    let state = run([start('practice', items(15))])
    expect(playing(state).words).toHaveLength(1)

    for (let i = 0; i < PRACTICE_ROUND; i++) {
      state = answerFirst(state, i % 2 === 0 ? 'ой' : 'ом')
      // A mistake pauses the round until the player moves on.
      if (state.phase === 'playing' && state.paused) {
        state = gameReducer(state, { type: 'continue', now: 0 })
      }
    }
    expect(state).toMatchObject({ phase: 'summary', score: PRACTICE_ROUND / 2 })
    if (state.phase === 'summary') {
      expect(state.results.map((r) => r.correct)).toEqual(
        Array.from({ length: PRACTICE_ROUND }, (_, i) => i % 2 === 0),
      )
    }
  })

  test('accepts the endings of alternative forms', () => {
    const state = answerFirst(
      run([start('practice', [item(1, 'ой', ['ой', 'ою']), item(2)])]),
      'ою',
    )
    expect(playing(state).results[0]).toMatchObject({ chosen: 'ою', correct: true })
  })

  test('ends early when the pool is smaller than a round', () => {
    const state = answerFirst(run([start('practice', items(1))]), 'ой')
    expect(state.phase).toBe('summary')
  })

  test('mistakes cost no lives and time does not matter', () => {
    let state = answerFirst(run([start('practice', items(5))]), 'ом')
    state = gameReducer(state, { type: 'tick', now: 1_000_000 })
    expect(playing(state)).toMatchObject({ lives: ARCADE_LIVES, streak: 0 })
  })

  test('a mistake pauses until the player continues', () => {
    let state = answerFirst(run([start('practice', items(5))]), 'ом')
    expect(playing(state)).toMatchObject({ paused: true, words: [] })
    // Answers while paused go nowhere: there's no word on the board.
    state = gameReducer(state, { type: 'continue', now: 5 })
    expect(playing(state).paused).toBe(false)
    expect(playing(state).words).toHaveLength(1)
  })

  test('a correct answer does not pause', () => {
    const state = answerFirst(run([start('practice', items(5))]), 'ой')
    expect(playing(state)).toMatchObject({ paused: false })
    expect(playing(state).words).toHaveLength(1)
  })
})

describe('arcade', () => {
  test('mistakes never pause', () => {
    const state = answerFirst(run([start('arcade', items(20))]), 'ом', 1)
    expect(playing(state)).toMatchObject({ paused: false })
    expect(playing(state).words).toHaveLength(1)
  })

  test('spawns words over time, up to a limit', () => {
    const pace = arcadePace(0)
    let state = run([start('arcade', items(20))])
    expect(playing(state).words).toHaveLength(1)

    state = gameReducer(state, { type: 'tick', now: pace.spawnEvery - 1 })
    expect(playing(state).words).toHaveLength(1)
    state = gameReducer(state, { type: 'tick', now: pace.spawnEvery })
    expect(playing(state).words).toHaveLength(2)
  })

  test('an empty board is refilled at once', () => {
    const state = answerFirst(run([start('arcade', items(20))]), 'ой', 1)
    expect(playing(state).words).toHaveLength(1)
  })

  test('a word that reaches the bottom costs a life', () => {
    const pace = arcadePace(0)
    const state = run([start('arcade', items(20)), { type: 'tick', now: pace.fallDuration }])
    expect(playing(state).lives).toBe(ARCADE_LIVES - 1)
    expect(playing(state).results[0]).toMatchObject({ chosen: null, correct: false })
  })

  test('streaks raise the score and mistakes reset them', () => {
    let state = run([start('arcade', items(20))])
    state = answerFirst(state, 'ой', 1)
    state = answerFirst(state, 'ой', 2)
    expect(playing(state)).toMatchObject({ score: 10 + 12, streak: 2 })

    state = answerFirst(state, 'ом', 3)
    expect(playing(state)).toMatchObject({ score: 22, streak: 0, lives: ARCADE_LIVES - 1 })
  })

  test('game over when the lives run out', () => {
    let state = run([start('arcade', items(20))])
    for (let i = 0; i < ARCADE_LIVES; i++) state = answerFirst(state, 'ом', i)
    expect(state).toMatchObject({ phase: 'summary', mode: 'arcade' })
  })

  test('asks for a refill before running dry', () => {
    let state = run([start('arcade', items(6))])
    expect(needsRefill(state)).toBe(false)
    state = answerFirst(state, 'ой', 1)
    expect(needsRefill(state)).toBe(true)

    state = gameReducer(state, { type: 'refill', items: items(10) })
    expect(needsRefill(state)).toBe(false)
  })

  test('gets faster with correct answers', () => {
    expect(arcadePace(10).spawnEvery).toBeLessThan(arcadePace(0).spawnEvery)
    expect(arcadePace(10).fallDuration).toBeLessThan(arcadePace(0).fallDuration)
    expect(arcadePace(1000).fallDuration).toBe(4000)
  })
})

test('answers for words no longer on the board are ignored', () => {
  const state = run([start('practice', items(5))])
  expect(gameReducer(state, { type: 'answer', wordId: 999, ending: 'ой', now: 0 })).toBe(state)
})

test('quitting shows the summary', () => {
  const state = run([start('arcade', items(5)), { type: 'quit' }])
  expect(state.phase).toBe('summary')
})
