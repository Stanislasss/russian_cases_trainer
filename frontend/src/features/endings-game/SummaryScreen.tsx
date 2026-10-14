import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { Button } from '../../shared/ui/Button'
import { answerLabel } from '../../shared/grammar/format'
import { RuleCard } from '../../shared/grammar/RuleCard'
import type { Finished } from './gameReducer'

interface Props {
  state: Finished
  loggedIn: boolean
  onAgain: () => void
  onSettings: () => void
  /** Plays a Russian text, for the 🔊 buttons in the rule explanations. */
  speak?: (text: string) => void
}

export function SummaryScreen({ state, loggedIn, onAgain, onSettings, speak }: Props) {
  const { t } = useTranslation()
  const correct = state.results.filter((r) => r.correct).length
  const mistakes = state.results.filter((r) => !r.correct)

  return (
    <section className="mx-auto max-w-xl space-y-6">
      <header className="space-y-1 text-center">
        <h1 className="text-2xl font-bold">{t('game.summary.title')}</h1>
        <p className="text-slate-500">
          {t('game.case')}: {t(`cases.${state.gameCase}`)}
        </p>
        {state.mode === 'arcade' && (
          <p className="text-4xl font-bold text-sky-700">
            {t('game.score')}: {state.score}
          </p>
        )}
        <p className="text-slate-600">
          {t('game.summary.correct', { correct, total: state.results.length })}
        </p>
      </header>

      {mistakes.length > 0 ? (
        <div>
          <h2 className="mb-2 font-semibold">{t('game.summary.mistakes')}</h2>
          <ul className="divide-y divide-slate-200 rounded border border-slate-200 bg-white">
            {mistakes.map((r, i) => (
              <li key={i} className="px-3 py-2">
                {/* Open a mistake to see the rule behind it. */}
                <details>
                  <summary className="flex cursor-pointer items-baseline justify-between gap-2">
                    <span>
                      {r.item.stressed_lemma} → <strong>{r.item.stressed_form}</strong>
                    </span>
                    <span className="text-sm text-red-600">
                      {r.chosen === null
                        ? t('game.summary.missed')
                        : answerLabel(r.chosen, state.answerKind)}
                    </span>
                  </summary>
                  {r.chosen !== null && (
                    <div className="mt-2">
                      <RuleCard
                        item={r.item}
                        chosen={r.chosen}
                        gameCase={state.gameCase}
                        answerKind={state.answerKind}
                        speak={speak}
                      />
                    </div>
                  )}
                </details>
              </li>
            ))}
          </ul>
        </div>
      ) : (
        <p className="text-center text-green-700">{t('game.summary.perfect')}</p>
      )}

      <p className="text-center text-sm text-slate-500">
        {loggedIn ? (
          t('game.summary.saved')
        ) : (
          <>
            <Link to="/login?next=%2Fplay%2Fendings" className="text-sky-700 underline">
              {t('auth.login.title')}
            </Link>{' '}
            {t('game.summary.loginToSave')}
          </>
        )}
      </p>

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
