import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api, ApiError, unwrap, type Schemas } from '../../shared/api/client'
import i18n from '../../shared/i18n'

export type User = Schemas['UserDto']

const sessionKey = ['session'] as const

/** The logged-in user, or `null` when logged out. */
export function useSession() {
  return useQuery({
    queryKey: sessionKey,
    queryFn: async (): Promise<User | null> => {
      try {
        return await unwrap(api.GET('/api/me'))
      } catch (e) {
        if (e instanceof ApiError && e.code === 'unauthenticated') return null
        throw e
      }
    },
    staleTime: Infinity,
    retry: false,
  })
}

/** Stores the user returned by login/register and switches the UI to their language. */
function useOnLoggedIn() {
  const queryClient = useQueryClient()
  return (user: User) => {
    queryClient.setQueryData(sessionKey, user)
    void i18n.changeLanguage(user.ui_locale)
  }
}

export function useLogin() {
  const onLoggedIn = useOnLoggedIn()
  return useMutation({
    mutationFn: (body: Schemas['LoginRequest']) => unwrap(api.POST('/api/auth/login', { body })),
    onSuccess: onLoggedIn,
  })
}

export function useRegister() {
  const onLoggedIn = useOnLoggedIn()
  return useMutation({
    mutationFn: (body: Schemas['RegisterRequest']) =>
      unwrap(api.POST('/api/auth/register', { body })),
    onSuccess: onLoggedIn,
  })
}

export function useLogout() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => unwrap(api.POST('/api/auth/logout')),
    onSuccess: () => {
      // Drop everything cached for the previous user.
      queryClient.clear()
      queryClient.setQueryData(sessionKey, null)
    },
  })
}
