import { useTranslation } from 'react-i18next'
import { endingLabel } from '../../shared/grammar/format'
import {
  type Case,
  CASES,
  type Column,
  type Group,
  PERSONAL_PRONOUNS,
  TABLES,
  tablesOf,
} from '../../shared/grammar/paradigms'
import { Listen } from '../../shared/grammar/RuleCard'
import { useGrammarTexts } from '../../shared/grammar/texts'
import { useSpeech } from '../../shared/speech/useSpeech'

/** The endings of one case for every paradigm of a group; rows are linkable (#paradigm). */
export function EndingsTable({ group, gameCase }: { group: Group; gameCase: Case }) {
  const g = useGrammarTexts()
  const { speak } = useSpeech()
  const paradigms = tablesOf(group)
  // Columns in the order they first appear (nouns: sg/pl; adjectives: m/n/f/pl).
  const columns = [...new Set(paradigms.flatMap((p) => TABLES[p].columns))] as Column[]

  return (
    <div className="overflow-x-auto">
      <table className="w-full border-collapse text-sm">
        <thead>
          <tr className="border-b border-slate-300 text-left">
            <th className="py-2 pr-3 font-medium">{g.groups[group]}</th>
            {columns.map((c) => (
              <th key={c} className="px-2 py-2 font-medium">
                {g.columns[c]}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {paradigms.map((paradigm) => {
            const table = TABLES[paradigm]
            return (
              <tr
                key={paradigm}
                id={paradigm}
                className="border-b border-slate-200 target:bg-amber-50"
              >
                <th className="py-2 pr-3 text-left align-top font-normal">
                  <span className="block font-medium">{g.paradigms[paradigm]}</span>
                  <span className="text-slate-500">{table.words}</span>
                </th>
                {columns.map((column) => {
                  const cell = table.cells[gameCase][column]
                  return (
                    <td key={column} className="px-2 py-2 align-top">
                      {cell ? (
                        <>
                          <span className="font-semibold text-sky-800">
                            {cell.endings.map(endingLabel).join(' / ')}
                            {cell.animacy && <sup title={g.animacy}>*</sup>}
                          </span>
                          <span className="block text-slate-600">
                            {cell.example}
                            <Listen text={cell.example} speak={speak} />
                          </span>
                        </>
                      ) : (
                        <span className="text-slate-300">—</span>
                      )}
                    </td>
                  )
                })}
              </tr>
            )
          })}
        </tbody>
      </table>
      {gameCase === 'accusative' && <p className="mt-1 text-xs text-slate-500">* {g.animacy}</p>}
    </div>
  )
}

/** Personal pronouns in every case, or in `only` one case. */
export function PersonalPronounsTable({ only }: { only?: Case }) {
  const { t } = useTranslation()
  const { speak } = useSpeech()
  const cases = only ? [only] : CASES
  return (
    <div className="overflow-x-auto">
      <table className="w-full border-collapse text-sm">
        <thead>
          <tr className="border-b border-slate-300 text-left">
            {cases.map((c) => (
              <th key={c} className="px-2 py-2 font-medium">
                {t(`cases.${c}`)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {PERSONAL_PRONOUNS.map((p) => (
            <tr key={p.lemma} className="border-b border-slate-200">
              {cases.map((c) => (
                <td key={c} className="px-2 py-1">
                  {only && <span className="mr-2 text-slate-500">{p.lemma} →</span>}
                  {p.forms[c]}
                  {p.forms[c] !== '—' && <Listen text={p.forms[c]} speak={speak} />}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
