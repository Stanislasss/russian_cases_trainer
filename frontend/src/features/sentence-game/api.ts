import { api, unwrap, type Schemas } from '../../shared/api/client'

export type SentenceItem = Schemas['SentenceItem']
export type SentencesBatch = Schemas['SentencesBatch']
export type Level = Schemas['Level']
export type GrammaticalCase = Schemas['GrammaticalCase']
export type AttemptPayload = Schemas['EndingsAttempt']

export function fetchSentences(levels: Level[], count: number) {
  return unwrap(api.POST('/api/games/sentences/batch', { body: { levels, count } }))
}

export function postSentenceAttempts(attempts: AttemptPayload[]) {
  return unwrap(api.POST('/api/games/sentences/attempts', { body: { attempts } }))
}
