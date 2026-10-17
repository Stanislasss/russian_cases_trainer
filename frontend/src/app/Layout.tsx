import { useTranslation } from 'react-i18next'
import { Link, NavLink, Outlet } from 'react-router'
import { UserMenu } from '../features/auth/UserMenu'
import { useConfig } from '../shared/config/config'
import { LanguageSwitcher } from '../shared/i18n/LanguageSwitcher'

/** Width and side padding shared by the header, the page and the footer. */
const CONTAINER = 'mx-auto w-full max-w-7xl px-4 sm:px-6 lg:px-8'

export function Layout() {
  const { t } = useTranslation()
  // The desktop app has settings (the speech pack) instead of accounts.
  const desktop = useConfig().data?.accounts === false

  return (
    <div className="flex min-h-screen flex-col bg-slate-50 text-slate-900">
      <header className="border-b border-slate-200 bg-white">
        {/* Wraps onto two lines in narrow windows instead of overflowing. The
            home page leads everywhere, so the title is the only way there. */}
        <div
          className={`${CONTAINER} flex flex-wrap items-center justify-between gap-x-6 gap-y-2 py-3`}
        >
          <Link to="/" className="text-lg font-semibold">
            {t('app.title')}
          </Link>
          <div className="flex items-center gap-4">
            <UserMenu />
            {desktop && (
              <NavLink
                to="/settings"
                aria-label={t('nav.settings')}
                title={t('nav.settings')}
                className={({ isActive }) =>
                  `rounded px-1.5 text-xl hover:bg-slate-100 ${isActive ? 'bg-slate-100' : ''}`
                }
              >
                ⚙
              </NavLink>
            )}
            <LanguageSwitcher />
          </div>
        </div>
      </header>
      <main className={`${CONTAINER} flex-1 py-6 sm:py-8`}>
        <Outlet />
      </main>
      <footer className="border-t border-slate-200 py-3 text-center text-xs text-slate-500">
        <div className={CONTAINER}>
          {t('app.attribution')}{' '}
          <a className="underline" href="https://en.openrussian.org">
            OpenRussian.org
          </a>{' '}
          &amp;{' '}
          <a className="underline" href="https://fr.wiktionary.org">
            Wiktionary
          </a>{' '}
          (CC BY-SA 4.0)
        </div>
      </footer>
    </div>
  )
}
