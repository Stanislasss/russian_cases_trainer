import { useTranslation } from 'react-i18next'
import { Link, useNavigate } from 'react-router'
import { Button } from '../../shared/ui/Button'
import { useConfig } from '../../shared/config/config'
import { useLogout, useSession } from '../../shared/session/session'

export function UserMenu() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const session = useSession()
  const logout = useLogout()
  const config = useConfig()

  // The desktop app has one local user, who never logs in or out.
  if (session.isPending || config.data?.accounts !== true) return null
  if (!session.data) {
    return (
      <nav className="flex items-center gap-3 text-sm">
        <Link to="/login" className="hover:underline">
          {t('auth.login.title')}
        </Link>
        <Link to="/register" className="rounded bg-sky-600 px-3 py-1.5 text-white hover:bg-sky-700">
          {t('auth.register.title')}
        </Link>
      </nav>
    )
  }
  return (
    <nav className="flex items-center gap-3 text-sm">
      <Link to="/account" className="font-medium hover:underline">
        {session.data.display_name}
      </Link>
      <Button
        variant="ghost"
        disabled={logout.isPending}
        onClick={() => logout.mutate(undefined, { onSuccess: () => navigate('/') })}
      >
        {t('auth.logout')}
      </Button>
    </nav>
  )
}
