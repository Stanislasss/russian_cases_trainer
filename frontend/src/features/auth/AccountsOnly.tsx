import { Navigate, Outlet } from 'react-router'
import { useConfig } from '../../shared/config/config'

/** Account pages; the desktop app has one local user and none of them. */
export function AccountsOnly() {
  const config = useConfig()
  if (config.data?.accounts === false) return <Navigate to="/" replace />
  return <Outlet />
}
