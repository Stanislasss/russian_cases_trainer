import { api, unwrap, type Schemas } from '../../shared/api/client'
import { playableCases, type Settings } from './settings'

export type SentenceItem = Schemas['SentenceItem']
export type SentencesBatch = Schemas['SentencesBatch']
export type Level = Schemas['Level']
export type GrammaticalCase = Schemas['GrammaticalCase']
export type GrammaticalNumber = Schemas['GrammaticalNumber']
export type AttemptPayload = Schemas['EndingsAttempt']

const NUMBERS: Record<Settings['number'], GrammaticalNumber[]> = {
  singular: ['singular'],
  plural: ['plural'],
  both: ['singular', 'plural'],
}

export function fetchSentences(settings: Settings) {
  const body = {
    levels: settings.levels,
    cases: playableCases(settings),
    numbers: NUMBERS[settings.number],
    count: settings.roundLength,
  }
  return unwrap(api.POST('/api/games/sentences/batch', { body }))
}

export function postSentenceAttempts(attempts: AttemptPayload[]) {
  return unwrap(api.POST('/api/games/sentences/attempts', { body: { attempts } }))
}
