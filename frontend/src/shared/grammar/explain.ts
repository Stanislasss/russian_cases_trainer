// Why an answer in the endings game is wrong, worked out from the ending tables.
import type { Schemas } from '../api/client'
import type { SpellingRule } from './cases'
import type { AnswerKind } from './format'
import {
  type Case,
  type Cell,
  type Column,
  type Group,
  PERSONAL_PRONOUNS,
  type Paradigm,
  TABLES,
  type TableParadigm,
} from './paradigms'

export type Item = Schemas['EndingsItem']

export interface Owner {
  paradigm: TableParadigm
  column: Column
  example: string
}

export interface Explanation {
  /** The table the word follows; null for irregular words. */
  paradigm: Paradigm | null
  column?: Column
  /** The cell of the right answer in that table. */
  cell?: Cell
  /** Spelling rules that shaped the right ending (кни́ги: и after г). */
  spelling: SpellingRule[]
  /** The choice is one of the cell's endings, just not the one this word takes. */
  sameCell: boolean
  /** Other places where the chosen ending belongs (-ом: стол, окно). */
  usedFor: Owner[]
  /** For personal pronouns: the word's row, and the rows/cases the choice belongs to. */
  pronoun?: (typeof PERSONAL_PRONOUNS)[number]
  chosenPronoun: { lemma: string; case: Case }[]
}

const STRESS = '́'
const plain = (s: string) => s.replaceAll(STRESS, '').toLowerCase()

/** Adjective-like tables are compared with each other, noun tables with nouns. */
const FAMILY: Record<Group, 'nouns' | 'agreeing'> = {
  nouns: 'nouns',
  adjectives: 'agreeing',
  pronouns: 'agreeing',
  surnames: 'agreeing',
}

/** The column of a table that an item's form sits in. */
export function columnOf(paradigm: TableParadigm, item: Item): Column | undefined {
  const columns = TABLES[paradigm].columns
  if (columns.includes('singular') || item.number === 'plural') return item.number
  return item.gender ?? undefined
}

export function explain(
  item: Item,
  chosen: string,
  gameCase: Case,
  answerKind: AnswerKind,
): Explanation {
  const base: Explanation = {
    paradigm: item.paradigm ?? null,
    spelling: [],
    sameCell: false,
    usedFor: [],
    chosenPronoun: [],
  }
  if (answerKind === 'whole_form' || item.paradigm === 'personal_pronoun') {
    return { ...base, ...explainPronoun(item, chosen) }
  }
  if (!item.paradigm) return base

  const paradigm = item.paradigm
  const column = columnOf(paradigm, item)
  const cell = column && TABLES[paradigm].cells[gameCase][column]
  if (!column || !cell) return base

  const family = FAMILY[TABLES[paradigm].group]
  const usedFor: Owner[] = []
  for (const [other, table] of Object.entries(TABLES) as [
    TableParadigm,
    (typeof TABLES)[TableParadigm],
  ][]) {
    if (FAMILY[table.group] !== family) continue
    for (const [col, otherCell] of Object.entries(table.cells[gameCase]) as [Column, Cell][]) {
      const sameNumber = family !== 'nouns' || col === column
      const isOwnCell = other === paradigm && col === column
      if (sameNumber && !isOwnCell && otherCell.endings.includes(chosen)) {
        if (!usedFor.some((o) => o.paradigm === other)) {
          usedFor.push({ paradigm: other, column: col, example: otherCell.example })
        }
      }
    }
  }

  return {
    ...base,
    column,
    cell,
    spelling: spellingRules(item, cell),
    sameCell: cell.endings.includes(chosen),
    usedFor: usedFor.slice(0, 3),
  }
}

function spellingRules(item: Item, cell: Cell): SpellingRule[] {
  const answer = item.answer
  if (!answer) return []
  const form = plain(item.stressed_form)
  const stemLast = form.at(-answer.length - 1) ?? ''
  const rest = answer.slice(1)
  const rules: SpellingRule[] = []
  if (
    answer.startsWith('и') &&
    'гкхжшщч'.includes(stemLast) &&
    cell.endings.some((e) => e.startsWith('ы'))
  )
    rules.push('yi')
  if (
    answer.startsWith('е') &&
    'жшщчц'.includes(stemLast) &&
    cell.endings.includes('о' + rest) &&
    !endingIsStressed(item.stressed_form, answer.length)
  )
    rules.push('unstressedE')
  const softPair = { а: 'я', у: 'ю' }[answer[0]]
  if (softPair && 'жшщчц'.includes(stemLast) && cell.endings.includes(softPair + rest))
    rules.push('au')
  return rules
}

/**
 * Whether the stress mark falls on the last `length` letters of the form. The
 * mark follows its vowel, so walking back from the end meets it before that
 * vowel is counted (ножо́м: м, mark → stressed; ме́сяцем: м, е → not).
 */
function endingIsStressed(stressedForm: string, length: number): boolean {
  let letters = 0
  for (let i = stressedForm.length - 1; i >= 0 && letters < length; i--) {
    if (stressedForm[i] === STRESS) return true
    letters++
  }
  return false
}

function explainPronoun(item: Item, chosen: string): Partial<Explanation> {
  const words = (text: string) =>
    plain(text)
      .split(/[^а-яё]+/)
      .filter(Boolean)
  const lemma = plain(item.stressed_lemma)
  const pronoun = PERSONAL_PRONOUNS.find((p) => words(p.lemma).includes(lemma))
  const chosenPronoun = PERSONAL_PRONOUNS.flatMap((p) =>
    (Object.entries(p.forms) as [Case, string][])
      .filter(([, forms]) => words(forms).includes(plain(chosen)))
      .map(([c]) => ({ lemma: p.lemma, case: c })),
  )
  return { pronoun, chosenPronoun }
}
