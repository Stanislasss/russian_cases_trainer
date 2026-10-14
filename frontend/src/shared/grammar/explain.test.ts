import { describe, expect, test } from 'vitest'
import { explain, type Item } from './explain'

function item(over: Partial<Item>): Item {
  return {
    lexeme_id: 1,
    stressed_lemma: 'кни́га',
    translations: {},
    number: 'singular',
    gender: null,
    animacy: null,
    answer: 'ой',
    accepted: ['ой', 'ою'],
    stressed_form: 'кни́гой',
    paradigm: 'noun_a',
    ...over,
  }
}

describe('nouns', () => {
  test('points at the right cell and where the choice belongs', () => {
    const e = explain(item({}), 'ом', 'instrumental', 'ending')
    expect(e.column).toBe('singular')
    expect(e.cell?.endings).toContain('ой')
    expect(e.sameCell).toBe(false)
    expect(e.usedFor.map((o) => o.paradigm)).toEqual([
      'noun_masculine_hard',
      'noun_masculine_sibilant',
      'noun_neuter_o',
    ])
  })

  test('plural choices are compared with plural cells only', () => {
    const e = explain(
      item({ number: 'plural', answer: 'ами', stressed_form: 'кни́гами' }),
      'ах',
      'instrumental',
      'ending',
    )
    expect(e.column).toBe('plural')
    expect(e.usedFor).toEqual([])
  })

  test('и after a velar is the spelling rule', () => {
    const e = explain(
      item({ answer: 'и', stressed_form: 'кни́ги', accepted: ['и'] }),
      'ы',
      'genitive',
      'ending',
    )
    expect(e.spelling).toEqual(['yi'])
    expect(e.sameCell).toBe(true)
  })

  test('unstressed о becomes е after ц, stressed it stays', () => {
    const month = item({
      stressed_lemma: 'ме́сяц',
      paradigm: 'noun_masculine_hard',
      answer: 'ем',
      stressed_form: 'ме́сяцем',
    })
    expect(explain(month, 'ом', 'instrumental', 'ending').spelling).toEqual(['unstressedE'])

    const knife = item({
      stressed_lemma: 'нож',
      paradigm: 'noun_masculine_sibilant',
      answer: 'ом',
      stressed_form: 'ножо́м',
    })
    expect(explain(knife, 'ем', 'instrumental', 'ending').spelling).toEqual([])
  })

  test('irregular words have no table', () => {
    const e = explain(item({ paradigm: null }), 'ом', 'instrumental', 'ending')
    expect(e.cell).toBeUndefined()
    expect(e.paradigm).toBeNull()
  })
})

describe('adjectives', () => {
  test('the column is the gender, and the accusative depends on animacy', () => {
    const e = explain(
      item({
        stressed_lemma: 'но́вый',
        paradigm: 'adjective_hard',
        gender: 'masculine',
        animacy: 'animate',
        answer: 'ого',
        stressed_form: 'но́вого',
      }),
      'ый',
      'accusative',
      'ending',
    )
    expect(e.column).toBe('masculine')
    expect(e.cell?.animacy).toBe(true)
    expect(e.sameCell).toBe(true)
  })

  test('хоро́шего: е after ш, unstressed', () => {
    const e = explain(
      item({
        stressed_lemma: 'хоро́ший',
        paradigm: 'adjective_hard',
        gender: 'masculine',
        answer: 'его',
        stressed_form: 'хоро́шего',
      }),
      'ого',
      'genitive',
      'ending',
    )
    expect(e.spelling).toEqual(['unstressedE'])
  })
})

test('personal pronouns are explained with their row', () => {
  const e = explain(
    item({
      stressed_lemma: 'я',
      paradigm: 'personal_pronoun',
      answer: 'мне',
      stressed_form: 'мне',
    }),
    'меня',
    'dative',
    'whole_form',
  )
  expect(e.pronoun?.lemma).toBe('я')
  expect(e.chosenPronoun).toEqual([
    { lemma: 'я', case: 'genitive' },
    { lemma: 'я', case: 'accusative' },
  ])
})
