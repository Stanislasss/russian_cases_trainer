import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { api, unwrap } from '../../shared/api/client'
import { QUESTIONS } from '../../shared/grammar/cases'
import { CASES } from '../endings-game/settings'

/** The three ways to learn, as cards: read, drag endings, find the case in context. */
const CARDS = [
  { id: 'rules', to: '/rules', icon: '📖' },
  { id: 'endings', to: '/play/endings', icon: '🎯' },
  { id: 'sentences', to: '/play/sentences', icon: '💬' },
] as const

export function HomePage() {
  const { t } = useTranslation()
  const health = useQuery({
    queryKey: ['health'],
    queryFn: () => unwrap(api.GET('/api/health')),
  })

  return (
    <div className="space-y-10">
      <section className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-sky-600 to-indigo-700 px-6 py-10 text-white shadow-sm sm:px-10 sm:py-14">
        {/* Decoration: "cases" in Russian. */}
        <p
          aria-hidden
          lang="ru"
          className="pointer-events-none absolute -right-2 -bottom-2 text-[4rem] leading-none font-bold text-white/10 select-none sm:text-[6rem] lg:text-[8rem]"
        >
          падежи
        </p>
        <h1 className="relative max-w-2xl text-3xl font-bold sm:text-4xl lg:text-5xl">
          {t('home.heading')}
        </h1>
        <p className="relative mt-3 max-w-2xl text-lg text-sky-100">{t('home.intro')}</p>
        <div className="relative mt-6 flex flex-wrap gap-3">
          <Link
            to="/play/endings"
            className="rounded-lg bg-white px-5 py-2.5 font-semibold text-sky-700 hover:bg-sky-50"
          >
            {t('home.play')}
          </Link>
          <Link
            to="/rules"
            className="rounded-lg border border-white/60 px-5 py-2.5 font-semibold hover:bg-white/10"
          >
            {t('home.readRules')}
          </Link>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-3">
        {CARDS.map((card) => (
          <Link
            key={card.id}
            to={card.to}
            className="group flex flex-col rounded-xl border border-slate-200 bg-white p-5 shadow-sm transition hover:-translate-y-0.5 hover:border-sky-300 hover:shadow-md"
          >
            <span aria-hidden className="text-3xl">
              {card.icon}
            </span>
            <h2 className="mt-3 text-lg font-semibold">{t(`home.cards.${card.id}.title`)}</h2>
            <p className="mt-1 flex-1 text-sm text-slate-600">{t(`home.cards.${card.id}.text`)}</p>
            <span className="mt-4 text-sm font-medium text-sky-700 group-hover:underline">
              {t(`home.cards.${card.id}.action`)} →
            </span>
          </Link>
        ))}
      </section>

      <section>
        <h2 className="mb-3 text-lg font-semibold">{t('home.casesTitle')}</h2>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
          {CASES.map((c) => (
            <Link
              key={c}
              to={`/rules/${c}`}
              className="rounded-lg border border-slate-200 bg-white p-3 transition hover:border-sky-300 hover:bg-sky-50"
            >
              <span className="block font-semibold">{t(`cases.${c}`)}</span>
              <span lang="ru" className="block text-sm text-slate-500">
                {QUESTIONS[c]}
              </span>
            </Link>
          ))}
        </div>
      </section>

      {health.isError && (
        <p role="alert" className="rounded bg-red-50 p-3 text-sm text-red-700">
          {t('home.offline')}
        </p>
      )}
    </div>
  )
}
