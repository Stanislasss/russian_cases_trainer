import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { explain, type Item } from './explain'
import { answerLabel, type AnswerKind, endingLabel, speakable } from './format'
import { type Case, CASES } from './paradigms'
import { useGrammarTexts } from './texts'

interface Props {
  item: Item
  /** The wrong answer the player chose. */
  chosen: string
  gameCase: Case
  answerKind: AnswerKind
  /** Plays a Russian text; 🔊 buttons appear when given. */
  speak?: (text: string) => void
}

/** Why an answer is wrong: the word's rule, the right ending, and where the choice belongs. */
export function RuleCard({ item, chosen, gameCase, answerKind, speak }: Props) {
  const { t } = useTranslation()
  const g = useGrammarTexts()
  const e = explain(item, chosen, gameCase, answerKind)
  const chosenLabel = answerLabel(chosen, answerKind)
  const caseName = (c: Case) => t(`cases.${c}`)

  return (
    <aside className="space-y-3 rounded-lg border border-amber-300 bg-amber-50 p-4 text-sm">
      <h2 className="font-semibold">{g.explanation.title}</h2>
      <p>
        <span className="text-red-700 line-through">{chosenLabel}</span>{' '}
        <span className="mx-1 text-slate-400">→</span>
        <strong className="text-base text-green-800">{item.stressed_form}</strong>
        {speak && <Listen text={item.stressed_form} speak={speak} />}
      </p>

      {e.pronoun && (
        <div>
          <p className="font-medium">
            {g.paradigms.personal_pronoun}: {e.pronoun.lemma}
          </p>
          <ul className="mt-1 grid grid-cols-2 gap-x-3">
            {CASES.map((c) => (
              <li key={c} className={c === gameCase ? 'font-semibold text-green-800' : ''}>
                {caseName(c)}: {e.pronoun?.forms[c]}
              </li>
            ))}
          </ul>
        </div>
      )}

      {e.paradigm && e.paradigm !== 'personal_pronoun' && e.cell && e.column && (
        <div>
          <p className="font-medium">
            {g.paradigms[e.paradigm]} · {caseName(gameCase)} · {g.columns[e.column]}
          </p>
          <p className="mt-1">
            {e.cell.endings.map((ending, i) => (
              <span key={ending}>
                {i > 0 && ' / '}
                <span className={ending === item.answer ? 'font-bold text-green-800' : ''}>
                  {endingLabel(ending)}
                </span>
              </span>
            ))}
            <span className="ml-2 text-slate-600">({e.cell.example})</span>
            {speak && <Listen text={e.cell.example} speak={speak} />}
          </p>
          {e.cell.animacy && <p className="mt-1 text-slate-600">{g.animacy}</p>}
        </div>
      )}

      {e.spelling.map((rule) => (
        <p key={rule} className="rounded bg-white/70 p-2">
          <span className="font-medium">{g.spelling[rule].title}.</span> {g.spelling[rule].detail}
        </p>
      ))}

      {e.sameCell && e.spelling.length > 0 && <p>{g.explanation.sameCell(chosenLabel)}</p>}
      {e.usedFor.length > 0 && (
        <p>
          {g.explanation.usedFor(
            chosenLabel,
            e.usedFor.map((o) => `${g.paradigms[o.paradigm]} (${o.example})`).join('; '),
          )}
        </p>
      )}
      {e.chosenPronoun.length > 0 && (
        <p>
          {g.explanation.usedFor(
            chosenLabel,
            e.chosenPronoun.map((p) => `${p.lemma} — ${caseName(p.case)}`).join('; '),
          )}
        </p>
      )}
      {!e.paradigm && <p>{g.explanation.noTable}</p>}

      <Link
        to={`/rules/${gameCase}${e.paradigm ? `#${e.paradigm}` : ''}`}
        className="inline-block text-sky-700 underline"
      >
        {g.readMore} →
      </Link>
    </aside>
  )
}

/** A small 🔊 button; also used by the rules pages. */
export function Listen({ text, speak }: { text: string; speak: (text: string) => void }) {
  const { t } = useTranslation()
  return (
    <button
      type="button"
      onClick={() => speak(speakable(text))}
      aria-label={`${t('game.listen')}: ${text}`}
      title={t('game.listen')}
      className="ml-1 rounded px-1 align-middle hover:bg-slate-200"
    >
      🔊
    </button>
  )
}
