import type { ExceptionId, SpellingRule, UseId } from '../cases'
import type { Case, Column, Group, Paradigm } from '../paradigms'

export interface Entry {
  title: string
  detail?: string
}

export interface UseEntry extends Entry {
  /** Translations of the use's Russian examples, in the same order (none in Russian). */
  translations?: string[]
}

export type CaseTexts = {
  [C in Case]: {
    purpose: string
    uses: Record<UseId<C>, UseEntry>
    exceptions: Record<ExceptionId<C>, Entry>
  }
}

/** Everything the rules pages and explanations say, in one UI language. */
export interface GrammarTexts {
  title: string
  intro: string[]
  overview: string
  questions: string
  uses: string
  endings: string
  exceptions: string
  spellingRules: string
  personalPronouns: string
  nForms: string
  indeclinable: Entry
  animacy: string
  practise: string
  allRules: string
  readMore: string
  explanation: {
    title: string
    correct: string
    yourChoice: string
    /** "-ом is used for: …" */
    usedFor: (ending: string, owners: string) => string
    /** The choice is one of the right cell's endings, but the wrong one here. */
    sameCell: (ending: string) => string
    noTable: string
    continue: string
  }
  groups: Record<Group, string>
  columns: Record<Column, string>
  paradigms: Record<Paradigm, string>
  spelling: Record<SpellingRule, Entry>
  cases: CaseTexts
}
