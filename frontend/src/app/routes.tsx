import type { RouteObject } from 'react-router'
import { AccountPage } from '../features/auth/AccountPage'
import { LoginPage } from '../features/auth/LoginPage'
import { RegisterPage } from '../features/auth/RegisterPage'
import { RequireAuth } from '../features/auth/RequireAuth'
import { EndingsGamePage } from '../features/endings-game/EndingsGamePage'
import { HomePage } from '../features/home/HomePage'
import { CaseRulesPage } from '../features/rules/CaseRulesPage'
import { RulesPage } from '../features/rules/RulesPage'
import { Layout } from './Layout'

export const routes: RouteObject[] = [
  {
    element: <Layout />,
    children: [
      { index: true, element: <HomePage /> },
      { path: 'login', element: <LoginPage /> },
      { path: 'register', element: <RegisterPage /> },
      { path: 'play/endings', element: <EndingsGamePage /> },
      { path: 'rules', element: <RulesPage /> },
      { path: 'rules/:case', element: <CaseRulesPage /> },
      {
        element: <RequireAuth />,
        children: [{ path: 'account', element: <AccountPage /> }],
      },
    ],
  },
]
