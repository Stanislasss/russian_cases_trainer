import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { api, unwrap, type Schemas } from '../api/client'

export type AppConfig = Schemas['AppConfig']
export type SpeechStatus = Schemas['SpeechStatus']

const configKey = ['config'] as const

/**
 * What this installation offers: accounts (web) or one local user (desktop app),
 * and whether words can be pronounced. Polls while the speech pack downloads.
 */
export function useConfig() {
  return useQuery({
    queryKey: configKey,
    queryFn: () => unwrap(api.GET('/api/config')),
    staleTime: Infinity,
    refetchInterval: (query) => (query.state.data?.speech.state === 'installing' ? 500 : false),
    refetchIntervalInBackground: true,
  })
}

/** Whether words can be spoken right now (muting aside). */
export function speechAvailable(config: AppConfig | undefined) {
  const state = config?.speech.state
  return state === 'service' || state === 'ready'
}

/** Desktop app: download the speech pack, or delete it. */
export function useSpeechPack() {
  const queryClient = useQueryClient()
  const refresh = () => queryClient.invalidateQueries({ queryKey: configKey })
  const install = useMutation({
    mutationFn: () => unwrap(api.POST('/api/speech/pack')),
    onSettled: refresh,
  })
  const remove = useMutation({
    mutationFn: () => unwrap(api.DELETE('/api/speech/pack')),
    onSettled: refresh,
  })
  return { install, remove }
}
