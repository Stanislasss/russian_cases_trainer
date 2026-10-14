import { useCallback, useEffect, useReducer, useRef, useState } from 'react'
import { useSession } from '../../shared/session/session'
import { type AttemptPayload, fetchBatch, postAttempts } from './api'
import { Board } from './Board'
import { gameReducer, initialState, needsRefill, PRACTICE_ROUND, type Result } from './gameReducer'
import { loadSettings, saveSettings, type Settings } from './settings'
import { SetupScreen } from './SetupScreen'
import { SummaryScreen } from './SummaryScreen'

const ARCADE_BATCH = 20
/** Logged-in players' answers are sent in groups of this size (and at the end). */
const SEND_EVERY = 10

export function EndingsGamePage() {
  const loggedIn = Boolean(useSession().data)
  const [settings, setSettings] = useState(loadSettings)
  const [state, dispatch] = useReducer(gameReducer, initialState)
  const [starting, setStarting] = useState(false)
  const [setupError, setSetupError] = useState<'empty' | 'failed' | null>(null)
  // The settings the current game was started with (the form may change meanwhile).
  const game = useRef(settings)
  const sent = useRef(0)
  const refilling = useRef(false)

  const start = async () => {
    setStarting(true)
    setSetupError(null)
    saveSettings(settings)
    try {
      const count = settings.mode === 'practice' ? PRACTICE_ROUND : ARCADE_BATCH
      const batch = await fetchBatch({
        levels: settings.levels,
        case: settings.case,
        count,
      })
      if (batch.items.length === 0) {
        setSetupError('empty')
        return
      }
      game.current = settings
      sent.current = 0
      dispatch({
        type: 'start',
        mode: settings.mode,
        answerKind: batch.answer_kind,
        gameCase: settings.case,
        zones: batch.zones,
        items: batch.items,
        now: performance.now(),
      })
    } catch {
      setSetupError('failed')
    } finally {
      setStarting(false)
    }
  }

  const onAnswer = useCallback(
    (wordId: number, ending: string) =>
      dispatch({ type: 'answer', wordId, ending, now: performance.now() }),
    [],
  )

  // Arcade clock: spawns words and notices the ones that reach the bottom.
  const arcadePlaying = state.phase === 'playing' && state.mode === 'arcade'
  useEffect(() => {
    if (!arcadePlaying) return
    const timer = setInterval(() => dispatch({ type: 'tick', now: performance.now() }), 100)
    return () => clearInterval(timer)
  }, [arcadePlaying])

  // Arcade never runs out of words.
  const refill = needsRefill(state)
  useEffect(() => {
    if (!refill || refilling.current) return
    refilling.current = true
    const { levels, case: gameCase } = game.current
    fetchBatch({ levels, case: gameCase, count: ARCADE_BATCH })
      .then((batch) => dispatch({ type: 'refill', items: batch.items }))
      .catch(() => {}) // the game goes on with what's left
      .finally(() => (refilling.current = false))
  }, [refill])

  // Save answers for logged-in players. Words that fell off the board aren't
  // answers, so they aren't recorded.
  useEffect(() => {
    if (!loggedIn || state.phase === 'setup') return
    const unsent = state.results.length - sent.current
    if (unsent === 0 || (state.phase === 'playing' && unsent < SEND_EVERY)) return
    const batch = state.results.slice(sent.current)
    sent.current = state.results.length
    const attempts = batch.flatMap((r) => toAttempt(r, game.current.case))
    if (attempts.length > 0) postAttempts(attempts).catch(() => {}) // stats only; never block play
  }, [state, loggedIn])

  switch (state.phase) {
    case 'setup':
      return (
        <SetupScreen
          settings={settings}
          onChange={setSettings}
          onStart={start}
          starting={starting}
          error={setupError}
        />
      )
    case 'playing':
      return <Board state={state} onAnswer={onAnswer} onQuit={() => dispatch({ type: 'quit' })} />
    case 'summary':
      return (
        <SummaryScreen
          state={state}
          loggedIn={loggedIn}
          onAgain={start}
          onSettings={() => dispatch({ type: 'reset' })}
        />
      )
  }
}

function toAttempt(result: Result, gameCase: Settings['case']): AttemptPayload[] {
  if (result.chosen === null) return []
  return [
    {
      lexeme_id: result.item.lexeme_id,
      case: gameCase,
      number: result.item.number,
      gender: result.item.gender,
      animacy: result.item.animacy,
      answer: result.chosen,
      correct: result.correct,
    },
  ]
}
