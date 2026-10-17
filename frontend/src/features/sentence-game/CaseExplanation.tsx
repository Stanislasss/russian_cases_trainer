import { useTranslation } from 'react-i18next'
import { QUESTIONS } from '../../shared/grammar/cases'
import { useGrammarTexts } from '../../shared/grammar/texts'
import type { UseEntry } from '../../shared/grammar/texts/types'
import type { GrammaticalCase, SentenceItem } from './api'

interface Props {
  item: SentenceItem
  /** The wrong case the player chose. */
  chosen: GrammaticalCase
}

/** Why the sentence needs its case, and what the chosen case is for instead. */
export function CaseExplanation({ item, chosen }: Props) {
  const { t } = useTranslation()
  const g = useGrammarTexts()
  // The use ids are typed per case; the frame's reason is one of its case's.
  const uses = (c: GrammaticalCase) => g.cases[c].uses as Record<string, UseEntry>
  const reason = uses(item.case)[item.reason]
  const caseName = (c: GrammaticalCase) => t(`cases.${c}`)

  return (
    <aside className="space-y-3 rounded-lg border border-amber-300 bg-amber-50 p-4 text-sm">
      <h2 className="font-semibold">{t('sentences.whyCase', { case: caseName(item.case) })}</h2>
      <p>
        <span className="text-red-700 line-through">{caseName(chosen)}</span>
        <span className="mx-1 text-slate-400">→</span>
        <strong className="text-base text-green-800">{caseName(item.case)}</strong>
        <span className="ml-2 text-slate-500">({QUESTIONS[item.case]})</span>
      </p>
      {reason && (
        <div className="rounded bg-white/70 p-2">
          <p className="font-medium">{reason.title}</p>
          {reason.detail && <p className="text-slate-600">{reason.detail}</p>}
        </div>
      )}
      <div>
        <p className="font-medium">{t('sentences.chosenUsedFor', { case: caseName(chosen) })}</p>
        <ul className="mt-1 list-disc space-y-0.5 pl-5 text-slate-700">
          {Object.values(uses(chosen)).map((use) => (
            <li key={use.title}>{use.title}</li>
          ))}
        </ul>
      </div>
    </aside>
  )
}
