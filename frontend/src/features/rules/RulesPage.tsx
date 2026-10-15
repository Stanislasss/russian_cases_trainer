import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import {
  INDECLINABLE,
  QUESTIONS,
  SPELLING_EXAMPLES,
  type SpellingRule,
} from '../../shared/grammar/cases'
import { CASES } from '../../shared/grammar/paradigms'
import { Listen } from '../../shared/grammar/Listen'
import { useGrammarTexts } from '../../shared/grammar/texts'
import { useSpeech } from '../../shared/speech/useSpeech'
import { PersonalPronounsTable } from '../../shared/grammar/tables'

const SPELLING_RULES: SpellingRule[] = ['yi', 'unstressedE', 'au']

/** What cases are, an overview of the six, and the rules that apply to all of them. */
export function RulesPage() {
  const { t } = useTranslation()
  const g = useGrammarTexts()
  const { speak } = useSpeech()

  return (
    <article className="mx-auto max-w-3xl space-y-8">
      <header className="space-y-3">
        <h1 className="text-3xl font-bold">{g.title}</h1>
        {g.intro.map((p) => (
          <p key={p} className="text-slate-700">
            {p}
          </p>
        ))}
      </header>

      <section>
        <h2 className="mb-3 text-xl font-semibold">{g.overview}</h2>
        <ul className="grid gap-3 sm:grid-cols-2">
          {CASES.map((c) => (
            <li key={c}>
              <Link
                to={`/rules/${c}`}
                className="block h-full rounded-lg border border-slate-200 bg-white p-4 hover:border-sky-400"
              >
                <span className="block font-semibold text-sky-800">{t(`cases.${c}`)}</span>
                <span className="block text-sm text-slate-500">{QUESTIONS[c]}</span>
                <span className="mt-1 block text-sm">{g.cases[c].purpose}</span>
              </Link>
            </li>
          ))}
        </ul>
      </section>

      <section className="space-y-2">
        <h2 className="text-xl font-semibold">{g.spellingRules}</h2>
        {SPELLING_RULES.map((rule) => (
          <div key={rule} className="rounded border border-slate-200 bg-white p-3">
            <p className="font-medium">{g.spelling[rule].title}</p>
            <p className="text-sm text-slate-700">{g.spelling[rule].detail}</p>
            <p className="text-sm">
              {SPELLING_EXAMPLES[rule].join(', ')}
              <Listen text={SPELLING_EXAMPLES[rule].join(', ')} speak={speak} />
            </p>
          </div>
        ))}
      </section>

      <section className="space-y-2">
        <h2 className="text-xl font-semibold">{g.personalPronouns}</h2>
        <PersonalPronounsTable />
        <p className="text-sm text-slate-600">{g.nForms}</p>
      </section>

      <section className="space-y-1">
        <h2 className="text-xl font-semibold">{g.indeclinable.title}</h2>
        <p className="text-slate-700">{g.indeclinable.detail}</p>
        <p>
          {INDECLINABLE.join(', ')}
          <Listen text={INDECLINABLE.join(', ')} speak={speak} />
        </p>
      </section>
    </article>
  )
}
