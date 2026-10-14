import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router'
import { api, unwrap } from '../../shared/api/client'

export function HomePage() {
  const { t } = useTranslation()
  const health = useQuery({
    queryKey: ['health'],
    queryFn: () => unwrap(api.GET('/api/health')),
  })

  return (
    <section className="space-y-4">
      <h1 className="text-3xl font-bold">{t('home.heading')}</h1>
      <p className="text-slate-600">{t('home.intro')}</p>
      <Link
        to="/play/endings"
        className="inline-block rounded bg-sky-600 px-4 py-2 font-medium text-white hover:bg-sky-700"
      >
        {t('home.play')}
      </Link>
      <p className="text-sm">
        {t('home.backend')}:{' '}
        {health.isPending ? '…' : health.isError ? t('home.offline') : health.data.status}
      </p>
    </section>
  )
}
