import { type ReactNode, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useSpeech } from '../speech/useSpeech'
import { EXCEPTIONS, QUESTIONS, USES } from './cases'
import { Listen } from './Listen'
import { type Case, GROUPS } from './paradigms'
import { EndingsTable, PersonalPronounsTable } from './tables'
import { useGrammarTexts } from './texts'
import type { Entry, UseEntry } from './texts/types'

interface Props {
  gameCase: Case
  /** A table row (a paradigm id, e.g. noun_a) to bring into view. */
  anchor?: string
  /** Shown under the purpose, e.g. a link to practise the case. */
  actions?: ReactNode
}

/** One case: what it's for, its endings in every table, and its exceptions. */
export function CaseRules({ gameCase: c, anchor, actions }: Props) {
  const { t } = useTranslation()
  const g = useGrammarTexts()
  const { listen } = useSpeech()
  const texts = g.cases[c]
  // The ids are typed per case; here we only need them as entries.
  const uses = texts.uses as Record<string, UseEntry>
  const exceptions = texts.exceptions as Record<string, Entry>

  // Bring the linked table row into view once the tables have rendered.
  useEffect(() => {
    if (anchor) document.getElementById(anchor)?.scrollIntoView?.({ block: 'center' })
  }, [anchor, c])

  return (
    <article className="space-y-8">
      <header className="space-y-2">
        <h1 className="text-3xl font-bold">{t(`cases.${c}`)}</h1>
        <p className="text-slate-500">
          {g.questions}: {QUESTIONS[c]}
        </p>
        <p className="text-lg">{texts.purpose}</p>
        {actions}
      </header>

      <section className="space-y-3">
        <h2 className="text-xl font-semibold">{g.uses}</h2>
        {Object.entries(USES[c]).map(([id, examples]) => (
          <div key={id} className="rounded border border-slate-200 bg-white p-3">
            <p className="font-medium">{uses[id].title}</p>
            {uses[id].detail && <p className="text-sm text-slate-600">{uses[id].detail}</p>}
            <ul className="mt-1 space-y-0.5 text-sm">
              {(examples as readonly string[]).map((example, i) => (
                <li key={example}>
                  <span lang="ru">{example}</span>
                  <Listen text={example} speak={listen} />
                  {uses[id].translations?.[i] && (
                    <span className="ml-2 text-slate-500">— {uses[id].translations?.[i]}</span>
                  )}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </section>

      <section className="space-y-4">
        <h2 className="text-xl font-semibold">{g.endings}</h2>
        {GROUPS.map((group) => (
          <EndingsTable key={group} group={group} gameCase={c} />
        ))}
        <div>
          <h3 className="mb-1 font-medium">{g.personalPronouns}</h3>
          <PersonalPronounsTable only={c} />
          <p className="mt-1 text-xs text-slate-500">{g.nForms}</p>
        </div>
      </section>

      <section className="space-y-3">
        <h2 className="text-xl font-semibold">{g.exceptions}</h2>
        {Object.entries(EXCEPTIONS[c]).map(([id, examples]) => (
          <div key={id} className="rounded border border-slate-200 bg-white p-3">
            <p className="font-medium">{exceptions[id].title}</p>
            {exceptions[id].detail && (
              <p className="text-sm text-slate-600">{exceptions[id].detail}</p>
            )}
            <ul className="mt-1 flex flex-wrap gap-x-4 text-sm">
              {(examples as readonly string[]).map((example) => (
                <li key={example} lang="ru">
                  {example}
                  <Listen text={example} speak={listen} />
                </li>
              ))}
            </ul>
          </div>
        ))}
      </section>
    </article>
  )
}
