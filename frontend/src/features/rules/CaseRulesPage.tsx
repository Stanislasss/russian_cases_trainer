import { useTranslation } from 'react-i18next'
import { Link, Navigate, NavLink, useLocation, useParams } from 'react-router'
import { CaseRules } from '../../shared/grammar/CaseRules'
import { type Case, CASES } from '../../shared/grammar/paradigms'
import { useGrammarTexts } from '../../shared/grammar/texts'

/** One case's rules, with links to the other cases and to practising this one. */
export function CaseRulesPage() {
  const { case: param } = useParams()
  const { t } = useTranslation()
  const g = useGrammarTexts()
  // A linked table row (#noun_a).
  const { hash } = useLocation()

  if (!CASES.includes(param as Case)) return <Navigate to="/rules" replace />
  const c = param as Case

  return (
    <div className="mx-auto max-w-4xl space-y-8">
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

      <CaseRules
        gameCase={c}
        anchor={hash.slice(1) || undefined}
        actions={
          <Link
            to={`/play/endings?case=${c}`}
            className="inline-block rounded bg-sky-600 px-4 py-2 text-sm font-medium text-white hover:bg-sky-700"
          >
            {g.practise}
          </Link>
        }
      />
    </div>
  )
}
