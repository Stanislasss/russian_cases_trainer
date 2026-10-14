import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Link, Navigate, NavLink, useLocation, useParams } from 'react-router'
import { EXCEPTIONS, QUESTIONS, USES } from '../../shared/grammar/cases'
import { type Case, CASES, GROUPS } from '../../shared/grammar/paradigms'
import { Listen } from '../../shared/grammar/RuleCard'
import { useGrammarTexts } from '../../shared/grammar/texts'
import type { Entry, UseEntry } from '../../shared/grammar/texts/types'
import { useSpeech } from '../../shared/speech/useSpeech'
import { EndingsTable, PersonalPronounsTable } from './tables'

/** One case: what it's for, its endings in every table, and its exceptions. */
export function CaseRulesPage() {
  const { case: param } = useParams()
  const { t } = useTranslation()
  const g = useGrammarTexts()
  const { speak } = useSpeech()
  const { hash } = useLocation()

  // Bring a linked table row (#noun_a) into view once the page has rendered.
  useEffect(() => {
    if (hash) document.getElementById(hash.slice(1))?.scrollIntoView({ block: 'center' })
  }, [hash, param])

  if (!CASES.includes(param as Case)) return <Navigate to="/rules" replace />
  const c = param as Case
  const texts = g.cases[c]
  // The ids are typed per case; here we only need them as entries.
  const uses = texts.uses as Record<string, UseEntry>
  const exceptions = texts.exceptions as Record<string, Entry>

  return (
    <article className="mx-auto max-w-4xl space-y-8">
      <nav className="flex flex-wrap gap-2 text-sm">
        <Link to="/rules" className="mr-2 text-sky-700 underline">
          {g.allRules}
        </Link>
        {CASES.map((other) => (
          <NavLink
            key={other}
            to={`/rules/${other}`}
            className={({ isActive }) =>
              `rounded px-2 py-0.5 ${isActive ? 'bg-sky-600 text-white' : 'bg-slate-100 hover:bg-slate-200'}`
            }
          >
            {t(`cases.${other}`)}
          </NavLink>
        ))}
      </nav>

      <header className="space-y-2">
        <h1 className="text-3xl font-bold">{t(`cases.${c}`)}</h1>
        <p className="text-slate-500">
          {g.questions}: {QUESTIONS[c]}
        </p>
        <p className="text-lg">{texts.purpose}</p>
        <Link
          to={`/play/endings?case=${c}`}
          className="inline-block rounded bg-sky-600 px-4 py-2 text-sm font-medium text-white hover:bg-sky-700"
        >
          {g.practise}
        </Link>
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
                  <Listen text={example} speak={speak} />
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
                  <Listen text={example} speak={speak} />
                </li>
              ))}
            </ul>
          </div>
        ))}
      </section>
    </article>
  )
}
