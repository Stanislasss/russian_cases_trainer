import { useCallback, useEffect, useReducer, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { Listen } from '../../shared/grammar/Listen'
import { RuleCard } from '../../shared/grammar/RuleCard'
import { useSession } from '../../shared/session/session'
import { useSpeech } from '../../shared/speech/useSpeech'
import { Button } from '../../shared/ui/Button'
import {
  type AttemptPayload,
  fetchSentences,
  type GrammaticalCase,
  postSentenceAttempts,
} from './api'
import { CaseExplanation } from './CaseExplanation'
import { fullSentence, gameReducer, initialState, type Result } from './gameReducer'
import { SentenceBoard } from './SentenceBoard'
import { asksCase, loadSettings, playableCases, saveSettings } from './settings'
import { SetupForm } from './SetupForm'

/** After a right answer the sentence is spoken, then this pause before the next one. */
const NEXT_DELAY = 1200

export function SentenceGamePage() {
  const loggedIn = Boolean(useSession().data)
  const [settings, setSettings] = useState(loadSettings)
  const [state, dispatch] = useReducer(gameReducer, initialState)
  const [starting, setStarting] = useState(false)
  const [error, setError] = useState<'empty' | 'failed' | null>(null)
  const speech = useSpeech()

  const start = async () => {
    setStarting(true)
    setError(null)
    saveSettings(settings)
    try {
      const batch = await fetchSentences(settings)
      if (batch.items.length === 0) setError('empty')
      else
        dispatch({
          type: 'start',
          batch,
          cases: playableCases(settings),
          askCase: asksCase(settings),
        })
    } catch {
      setError('failed')
    } finally {
      setStarting(false)
    }
  }

  const onContinue = useCallback(() => dispatch({ type: 'continue' }), [])
  const onCase = useCallback((c: GrammaticalCase) => dispatch({ type: 'chooseCase', case: c }), [])
  const onEnding = useCallback((ending: string) => dispatch({ type: 'chooseEnding', ending }), [])

  // A right answer: hear the whole sentence, then move on (at once if unheard).
  const solved =
    state.phase === 'playing' && state.step === 'correct' ? state.items[state.index] : null
  const { speak } = speech
  const speakRef = useRef(speak)
  useEffect(() => {
    speakRef.current = speak
  })
  // A wrong ending: hear how the sentence should sound; the player moves on.
  const corrected =
    state.phase === 'playing' && state.step === 'endingMistake' ? state.items[state.index] : null
  useEffect(() => {
    if (corrected) void speakRef.current(fullSentence(corrected))
  }, [corrected])
  useEffect(() => {
    if (!solved) return
    let cancelled = false
    let timer: ReturnType<typeof setTimeout> | undefined
    void speakRef.current(fullSentence(solved)).then((outcome) => {
      if (cancelled) return
      timer = setTimeout(onContinue, outcome === 'played' ? NEXT_DELAY : NEXT_DELAY * 1.5)
    })
    return () => {
      cancelled = true
      clearTimeout(timer)
    }
  }, [solved, onContinue])

  // Results are saved once the round is over.
  const results = state.phase === 'summary' ? state.results : null
  useEffect(() => {
    if (!results || results.length === 0 || !loggedIn) return
    postSentenceAttempts(results.map(toAttempt)).catch(() => {}) // stats only
  }, [results, loggedIn])

  switch (state.phase) {
    case 'setup':
      return (
        <SetupForm
          settings={settings}
          onChange={setSettings}
          onStart={() => void start()}
          starting={starting}
          error={error}
        />
      )
    case 'playing':
      return (
        <SentenceBoard
          state={state}
          onCase={onCase}
          onEnding={onEnding}
          onContinue={onContinue}
          onQuit={() => dispatch({ type: 'quit' })}
          speech={speech}
          hideTranslation={settings.hideTranslation}
        />
      )
    case 'summary':
      return (
        <Summary
          results={state.results}
          loggedIn={loggedIn}
          onAgain={() => void start()}
          onSettings={() => dispatch({ type: 'reset' })}
          speak={speech.listen}
        />
      )
  }
}

function Summary({
  results,
  loggedIn,
  onAgain,
  onSettings,
  speak,
}: {
  results: Result[]
  loggedIn: boolean
  onAgain: () => void
  onSettings: () => void
  speak?: (text: string) => void
}) {
  const { t } = useTranslation()
  const correct = results.filter((r) => r.correct).length
  const mistakes = results.filter((r) => !r.correct)

  return (
    <section className="mx-auto max-w-2xl space-y-6">
      <header className="space-y-1 text-center">
        <h1 className="text-2xl font-bold">{t('game.summary.title')}</h1>
        <p className="text-slate-600">
          {t('game.summary.correct', { correct, total: results.length })}
        </p>
      </header>

      {mistakes.length > 0 ? (
        <div>
          <h2 className="mb-2 font-semibold">{t('game.summary.mistakes')}</h2>
          <ul className="divide-y divide-slate-200 rounded border border-slate-200 bg-white">
            {mistakes.map((r, i) => (
              <li key={i} className="flex items-start gap-2 px-3 py-2">
                <details className="flex-1">
                  <summary className="cursor-pointer" lang="ru">
                    {r.item.before}
                    <strong>{r.item.word.stressed_form}</strong>
                    {r.item.after}
                  </summary>
                  <div className="mt-2 space-y-2">
                    {r.wrongCase && <CaseExplanation item={r.item} chosen={r.wrongCase} />}
                    {r.wrongEnding !== null && (
                      <RuleCard
                        item={r.item.word}
                        chosen={r.wrongEnding}
                        gameCase={r.item.case}
                        answerKind="ending"
                        speak={speak}
                      />
                    )}
                  </div>
                </details>
                <Listen text={fullSentence(r.item)} speak={speak} />
              </li>
            ))}
          </ul>
        </div>
      ) : (
        <p className="text-center text-green-700">{t('game.summary.perfect')}</p>
      )}

      {results.length > 0 && (
        <p className="text-center text-sm text-slate-500">
          {loggedIn ? (
            t('game.summary.saved')
          ) : (
            <>
              <Link to="/login?next=%2Fplay%2Fsentences" className="text-sky-700 underline">
                {t('auth.login.title')}
              </Link>{' '}
              {t('game.summary.loginToSave')}
            </>
          )}
        </p>
      )}

      <div className="flex gap-3">
        <Button onClick={onAgain} className="flex-1">
          {t('game.summary.again')}
        </Button>
        <Button variant="ghost" onClick={onSettings} className="flex-1">
          {t('game.summary.settings')}
        </Button>
      </div>
    </section>
  )
}

function toAttempt(r: Result): AttemptPayload {
  return {
    lexeme_id: r.item.word.lexeme_id,
    case: r.item.case,
    number: r.item.word.number,
    gender: null,
    animacy: null,
    answer: r.wrongEnding ?? r.item.word.answer,
    correct: r.correct,
  }
}
