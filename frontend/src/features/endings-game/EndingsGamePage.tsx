import { useCallback, useEffect, useReducer, useRef, useState } from 'react'
import { useSearchParams } from 'react-router'
import { useSession } from '../../shared/session/session'
import { useSpeech } from '../../shared/speech/useSpeech'
import { type AttemptPayload, fetchBatch, postAttempts } from './api'
import { Board } from './Board'
import {
  gameReducer,
  initialState,
  needsRefill,
  PRACTICE_ROUND,
  type Result,
  type Word,
} from './gameReducer'
import { CASES, loadSettings, saveSettings, type Settings } from './settings'
import { SetupScreen } from './SetupScreen'
import { SummaryScreen } from './SummaryScreen'

const ARCADE_BATCH = 20
const NO_RESULTS: Result[] = []
const NO_WORDS: Word[] = []
/** Logged-in players' answers are sent in groups of this size (and at the end). */
const SEND_EVERY = 10

export function EndingsGamePage() {
  const loggedIn = Boolean(useSession().data)
  // A rules page can link here with ?case=… to practise that case.
  const [params] = useSearchParams()
  const [settings, setSettings] = useState(() => {
    const saved = loadSettings()
    const linked = CASES.find((c) => c === params.get('case'))
    return linked ? { ...saved, case: linked } : saved
  })
  const [state, dispatch] = useReducer(gameReducer, initialState)
  const speech = useSpeech()
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

  const onContinue = useCallback(() => dispatch({ type: 'continue', now: performance.now() }), [])

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
  const refills = state.phase === 'playing' ? state.refills : 0
  useEffect(() => {
    if (!refill || refilling.current) return
    refilling.current = true
    const { levels, case: gameCase } = game.current
    fetchBatch({ levels, case: gameCase, count: ARCADE_BATCH })
      .then((batch) => dispatch({ type: 'refill', items: batch.items }))
      .catch(() => {}) // the game goes on with what's left
      .finally(() => (refilling.current = false))
  }, [refill, refills])

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

  // Speech. A correct answer's form is spoken; in practice the next word then
  // follows after the chosen delay, or at once if the form wasn't heard (muted,
  // blocked). The latest `speak` is read through a ref so that muting doesn't
  // cancel the wait below.
  const { speak, enqueue, prefetch } = speech
  const speakRef = useRef(speak)
  useEffect(() => {
    speakRef.current = speak
  })
  const results = state.phase === 'setup' ? NO_RESULTS : state.results
  const waitingForNext = state.phase === 'playing' && state.pause === 'next'
  const spokenResults = useRef(0)
  useEffect(() => {
    if (results.length < spokenResults.current) spokenResults.current = 0 // a new game
    if (results.length === spokenResults.current) return
    spokenResults.current = results.length
    const last = results[results.length - 1]
    if (!last.correct) return

    // Cleared when the player moves on first (Next, Enter) or leaves the game.
    let cancelled = false
    let timer: ReturnType<typeof setTimeout> | undefined
    void speakRef.current(last.item.stressed_form).then((outcome) => {
      if (cancelled || !waitingForNext) return
      if (outcome === 'played') timer = setTimeout(onContinue, game.current.nextWordDelay)
      else onContinue()
    })
    return () => {
      cancelled = true
      clearTimeout(timer)
    }
  }, [results, waitingForNext, onContinue])

  // Practice (the slow mode) says each new word; arcade would be a cacophony.
  const practiceWord =
    state.phase === 'playing' && state.mode === 'practice' ? state.words[0] : undefined
  // A word keeps the same object while it's on the board, so this runs once per word.
  useEffect(() => {
    if (practiceWord) enqueue(practiceWord.item.stressed_lemma)
  }, [practiceWord, enqueue])

  // Warm up the answer's audio, so it plays right after the drop.
  const words = state.phase === 'playing' ? state.words : NO_WORDS
  useEffect(() => {
    for (const word of words) prefetch(word.item.stressed_form)
  }, [words, prefetch])

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
      return (
        <Board
          state={state}
          onAnswer={onAnswer}
          onQuit={() => dispatch({ type: 'quit' })}
          onContinue={onContinue}
          speech={speech}
        />
      )
    case 'summary':
      return (
        <SummaryScreen
          state={state}
          loggedIn={loggedIn}
          onAgain={start}
          onSettings={() => dispatch({ type: 'reset' })}
          speak={speech.muted ? undefined : speech.speak}
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
