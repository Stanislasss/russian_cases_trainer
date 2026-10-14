import { api, unwrap, type Schemas } from '../../shared/api/client'

export type Level = Schemas['Level']
export type GrammaticalCase = Schemas['GrammaticalCase']
export type BatchRequest = Schemas['EndingsBatchRequest']
export type AttemptPayload = Schemas['EndingsAttempt']

export function fetchBatch(body: BatchRequest) {
  return unwrap(api.POST('/api/games/endings/batch', { body }))
}

export function postAttempts(attempts: AttemptPayload[]) {
  return unwrap(api.POST('/api/games/endings/attempts', { body: { attempts } }))
}
