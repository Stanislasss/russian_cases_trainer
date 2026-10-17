import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { QUESTIONS } from '../../shared/grammar/cases'
import { answerLabel } from '../../shared/grammar/format'
import { RuleCard } from '../../shared/grammar/RuleCard'
import { useGrammarTexts } from '../../shared/grammar/texts'
import type { Speech } from '../../shared/speech/useSpeech'
import { AnswerButton } from '../../shared/ui/AnswerButton'
import { Button } from '../../shared/ui/Button'
import { translationFor } from '../endings-game/format'
import { CASES } from '../endings-game/settings'
import type { GrammaticalCase, SentenceItem } from './api'
import { CaseExplanation } from './CaseExplanation'
import { current, type Playing, ROUND } from './gameReducer'

interface Props {
  state: Playing
  onCase: (c: GrammaticalCase) => void
  onEnding: (ending: string) => void
  onContinue: () => void
  onQuit: () => void
  speech: Speech
}

const NO_ZONES: string[] = []

/** One sentence: pick the case (keys 1–6), then the ending (keys 1–9). */
export function SentenceBoard({ state, onCase, onEnding, onContinue, onQuit, speech }: Props) {
  const { t } = useTranslation()
  const g = useGrammarTexts()
  const item = current(state)
  const zones = state.zones[item.case] ?? NO_ZONES
  const { step } = state
  const caseStep = step === 'case' || step === 'caseMistake'
  // A side panel (explanation, Next) only after an answer; centred until then.
  const panel = step !== 'case' && step !== 'ending'

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (step === 'caseMistake' || step === 'endingMistake' || step === 'correct') {
        if (e.key === 'Enter') onContinue()
        return
      }
      const index = Number(e.key) - 1
      if (!Number.isInteger(index) || index < 0) return
      if (step === 'case' && index < CASES.length) onCase(CASES[index])
      if (step === 'ending' && index < zones.length) onEnding(zones[index])
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [step, zones, onCase, onEnding, onContinue])

  return (
    <div
      className={`mx-auto grid gap-6 ${panel ? 'max-w-6xl lg:grid-cols-[minmax(0,1fr)_22rem]' : 'max-w-4xl'}`}
    >
      <section className="flex flex-col gap-5">
        <div className="flex items-center justify-between text-sm">
          <span className="font-medium">
            {t('game.progress', {
              done: state.results.length,
              total: Math.min(ROUND, state.items.length),
            })}
          </span>
          <span>
            {speech.available && (
              <Button
                variant="ghost"
                onClick={speech.toggleMuted}
                aria-pressed={speech.muted}
                aria-label={t(speech.muted ? 'game.unmute' : 'game.mute')}
                title={t(speech.muted ? 'game.unmute' : 'game.mute')}
              >
                {speech.muted ? '🔇' : '🔊'}
              </Button>
            )}
            <Button variant="ghost" onClick={onQuit}>
              {t('game.quit')}
            </Button>
          </span>
        </div>

        <Sentence item={item} done={step === 'correct'} speak={speech.listen} />

        {caseStep ? (
          <div className="space-y-2">
            <p className="text-center font-medium">{t('sentences.pickCase')}</p>
            <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
              {CASES.map((c, i) => (
                <AnswerButton
                  key={c}
                  shortcut={i + 1}
                  onClick={() => onCase(c)}
                  disabled={step !== 'case'}
                  className={
                    step === 'caseMistake' && c === item.case
                      ? 'border-green-600! bg-green-50! opacity-100!'
                      : ''
                  }
                >
                  <span className="block">{t(`cases.${c}`)}</span>
                  <span className="block text-xs font-normal text-slate-500">{QUESTIONS[c]}</span>
                </AnswerButton>
              ))}
            </div>
          </div>
        ) : (
          <div className="space-y-2">
            <p className="text-center font-medium">
              <span className="mr-2 rounded bg-green-100 px-2 py-0.5 text-green-800">
                {t(`cases.${item.case}`)}
              </span>
              {t('sentences.pickEnding')}
            </p>
            <div className="grid grid-cols-[repeat(auto-fill,minmax(5.5rem,1fr))] gap-2">
              {zones.map((zone, i) => (
                <AnswerButton
                  key={zone}
                  shortcut={i + 1}
                  onClick={() => onEnding(zone)}
                  disabled={step !== 'ending'}
                  aria-label={zone === '' ? t('game.zeroEnding') : answerLabel(zone, 'ending')}
                >
                  {answerLabel(zone, 'ending')}
                </AnswerButton>
              ))}
            </div>
          </div>
        )}
        <p className="text-center text-xs text-slate-500">{t('sentences.howTo')}</p>
      </section>

      {/* Beside the sentence on wide windows, below it on narrow ones. */}
      <div className="space-y-3">
        {step === 'caseMistake' && state.wrongCase && (
          <CaseExplanation item={item} chosen={state.wrongCase} />
        )}
        {step === 'endingMistake' && state.wrongEnding !== null && (
          <RuleCard
            item={item.word}
            chosen={state.wrongEnding}
            gameCase={item.case}
            answerKind="ending"
            speak={speech.listen}
          />
        )}
        {(step === 'caseMistake' || step === 'endingMistake') && (
          // Focused: reading, then Enter, is the only thing to do now.
          <Button onClick={onContinue} autoFocus className="w-full">
            {g.explanation.continue}
          </Button>
        )}
        {step === 'correct' && (
          <Button variant="ghost" onClick={onContinue} className="w-full">
            {t('game.next')} →
          </Button>
        )}
      </div>
    </div>
  )
}

/** The sentence with its noun: the dictionary form until it's solved. */
function Sentence({
  item,
  done,
  speak,
}: {
  item: SentenceItem
  done: boolean
  speak?: (text: string) => void
}) {
  const { t, i18n } = useTranslation()
  // The first meaning only: the dictionaries list several ("position, location, …").
  const meaning = translationFor(item.word, i18n.resolvedLanguage)?.split(/[;,]/)[0].trim()
  const translations: Partial<Record<string, string | null>> = item.translations
  const sentenceTranslation = i18n.resolvedLanguage ? translations[i18n.resolvedLanguage] : null

  return (
    <div className="rounded-lg border border-slate-200 bg-white px-4 py-6 text-center sm:px-8 sm:py-10">
      <p lang="ru" className="text-2xl leading-relaxed md:text-3xl lg:text-4xl">
        {item.before}
        {done ? (
          <strong className="text-green-700">{item.word.stressed_form}</strong>
        ) : (
          <span className="mx-1 inline-flex items-baseline gap-1 rounded-md border-2 border-dashed border-sky-400 bg-sky-50 px-2">
            <span className="font-semibold">{item.word.stressed_lemma}</span>
            {item.word.number === 'plural' && (
              <span className="rounded bg-slate-200 px-1 align-middle text-xs font-medium text-slate-700">
                {t('game.badges.plural')}
              </span>
            )}
          </span>
        )}
        {item.after}
        {done && speak && (
          <button
            type="button"
            onClick={() => speak(`${item.before}${item.word.stressed_form}${item.after}`)}
            aria-label={t('game.listen')}
            className="ml-2 rounded px-1 align-middle text-lg hover:bg-slate-100"
          >
            🔊
          </button>
        )}
      </p>
      {sentenceTranslation && (
        <p className="mt-3 text-slate-500">
          {sentenceTranslation.split('___').map((part, i) => (
            <span key={i}>
              {i > 0 && <em className="text-slate-700">{meaning ?? '…'}</em>}
              {part}
            </span>
          ))}
        </p>
      )}
    </div>
  )
}
