import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api, unwrap, type Schemas } from '../api/client'

export type Preferences = Schemas['Preferences']

const preferencesKey = ['preferences'] as const

/** The player's preferences, kept with their progress (desktop app, or logged in). */
export function usePreferences() {
  return useQuery({
    queryKey: preferencesKey,
    queryFn: () => unwrap(api.GET('/api/preferences')),
    staleTime: Infinity,
  })
}

/** Saves the preferences; the change shows at once and is undone if saving fails. */
export function useSavePreferences() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (body: Preferences) => unwrap(api.PUT('/api/preferences', { body })),
    onMutate: async (body) => {
      await queryClient.cancelQueries({ queryKey: preferencesKey })
      const previous = queryClient.getQueryData<Preferences>(preferencesKey)
      queryClient.setQueryData(preferencesKey, body)
      return { previous }
    },
    onError: (_error, _body, context) => {
      queryClient.setQueryData(preferencesKey, context?.previous)
    },
  })
}
