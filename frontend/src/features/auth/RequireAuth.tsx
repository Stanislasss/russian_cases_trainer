import { useTranslation } from 'react-i18next'
import { Navigate, Outlet, useLocation } from 'react-router'
import { useSession } from './session'

/** Route guard: renders child routes for logged-in users, otherwise redirects to login. */
export function RequireAuth() {
  const { t } = useTranslation()
  const session = useSession()
  const location = useLocation()

  if (session.isPending) return <p className="text-slate-500">{t('common.loading')}</p>
  if (!session.data) {
    const next = encodeURIComponent(location.pathname + location.search)
    return <Navigate to={`/login?next=${next}`} replace />
  }
  return <Outlet />
}
