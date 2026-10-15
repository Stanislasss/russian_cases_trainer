import type { RouteObject } from 'react-router'
import { AccountPage } from '../features/auth/AccountPage'
import { AccountsOnly } from '../features/auth/AccountsOnly'
import { LoginPage } from '../features/auth/LoginPage'
import { RegisterPage } from '../features/auth/RegisterPage'
import { RequireAuth } from '../features/auth/RequireAuth'
import { EndingsGamePage } from '../features/endings-game/EndingsGamePage'
import { HomePage } from '../features/home/HomePage'
import { CaseRulesPage } from '../features/rules/CaseRulesPage'
import { RulesPage } from '../features/rules/RulesPage'
import { SettingsPage } from '../features/settings/SettingsPage'
import { Layout } from './Layout'

export const routes: RouteObject[] = [
  {
    element: <Layout />,
    children: [
      { index: true, element: <HomePage /> },
      { path: 'play/endings', element: <EndingsGamePage /> },
      { path: 'rules', element: <RulesPage /> },
      { path: 'rules/:case', element: <CaseRulesPage /> },
      { path: 'settings', element: <SettingsPage /> },
      {
        element: <AccountsOnly />,
        children: [
          { path: 'login', element: <LoginPage /> },
          { path: 'register', element: <RegisterPage /> },
          {
            element: <RequireAuth />,
            children: [{ path: 'account', element: <AccountPage /> }],
          },
        ],
      },
    ],
  },
]
