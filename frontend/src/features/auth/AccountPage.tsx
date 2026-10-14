import { useTranslation } from 'react-i18next'
import { useSession } from './session'

/** Rendered behind `RequireAuth`, so the session is always loaded here. */
export function AccountPage() {
  const { t } = useTranslation()
  const user = useSession().data
  if (!user) return null

  return (
    <section className="space-y-4">
      <h1 className="text-2xl font-bold">{t('auth.account.title')}</h1>
      <dl className="grid max-w-md grid-cols-[auto_1fr] gap-x-6 gap-y-2">
        <dt className="text-slate-500">{t('auth.displayName')}</dt>
        <dd>{user.display_name}</dd>
        <dt className="text-slate-500">{t('auth.email')}</dt>
        <dd>{user.email}</dd>
      </dl>
    </section>
  )
}
