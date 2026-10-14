import { useTranslation } from 'react-i18next'
import { Link, Outlet } from 'react-router'
import { UserMenu } from '../features/auth/UserMenu'
import { LanguageSwitcher } from '../shared/i18n/LanguageSwitcher'

export function Layout() {
  const { t } = useTranslation()

  return (
    <div className="flex min-h-screen flex-col bg-slate-50 text-slate-900">
      <header className="border-b border-slate-200 bg-white">
        <div className="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
          <Link to="/" className="text-lg font-semibold">
            {t('app.title')}
          </Link>
          <div className="flex items-center gap-4">
            <UserMenu />
            <LanguageSwitcher />
          </div>
        </div>
      </header>
      <main className="mx-auto w-full max-w-5xl flex-1 px-4 py-8">
        <Outlet />
      </main>
      <footer className="border-t border-slate-200 px-4 py-3 text-center text-xs text-slate-500">
        {t('app.attribution')}{' '}
        <a className="underline" href="https://en.openrussian.org">
          OpenRussian.org
        </a>{' '}
        (CC BY-SA 4.0)
      </footer>
    </div>
  )
}
