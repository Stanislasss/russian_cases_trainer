// What each case is used for, and its exceptions: ids and Russian examples.
// The explanations, in each UI language, live in ./texts and are typed against
// these ids, so every use and exception must be translated.
import type { Case } from './paradigms'

/** The questions a case answers. */
export const QUESTIONS: Record<Case, string> = {
  nominative: 'кто? что?',
  genitive: 'кого́? чего́?',
  dative: 'кому́? чему́?',
  accusative: 'кого́? что?',
  instrumental: 'кем? чем?',
  prepositional: 'о ком? о чём?',
}

// prettier-ignore
export const USES = {
  nominative: {
    subject: ['Студе́нт чита́ет.', 'Кни́га лежи́т на столе́.'],
    predicate: ['Э́то мой брат.', 'Москва́ — столи́ца Росси́и.'],
    dictionary: ['стол, кни́га, окно́'],
  },
  genitive: {
    possession: ['кни́га бра́та', 'центр го́рода'],
    absence: ['У меня́ нет вре́мени.', 'Здесь нет магази́на.'],
    quantity: ['мно́го люде́й', 'стака́н воды́', 'два стола́, пять столо́в'],
    prepositions: ['у до́ма', 'из Москвы́', 'без са́хара', 'для дру́га', 'по́сле рабо́ты'],
    verbs: ['Я бою́сь соба́к.', 'Жела́ю тебе́ сча́стья.'],
  },
  dative: {
    recipient: ['Я дал кни́гу дру́гу.', 'Позвони́ ма́ме.'],
    impersonal: ['Мне два́дцать лет.', 'Мне хо́лодно.', 'Нам на́до идти́.'],
    prepositions: ['Я иду́ к врачу́.', 'Мы гуля́ем по па́рку.'],
    verbs: ['Мне нра́вится му́зыка.', 'Помоги́ бра́ту.'],
  },
  accusative: {
    object: ['Я чита́ю кни́гу.', 'Я ви́жу бра́та.'],
    direction: ['Я иду́ в шко́лу.', 'Положи́ кни́гу на стол.'],
    time: ['в сре́ду', 'всю неде́лю', 'ка́ждый день'],
    prepositions: ['че́рез мост', 'Спаси́бо за пода́рок.', 'фильм про войну́'],
  },
  instrumental: {
    means: ['писа́ть ру́чкой', 'е́хать по́ездом'],
    with: ['чай с лимо́ном', 'Я гуля́ю с дру́гом.'],
    location: ['над столо́м', 'под крова́тью', 'пе́ред до́мом', 'за угло́м'],
    being: ['Он стал врачо́м.', 'Она́ рабо́тает учи́тельницей.'],
    verbs: ['занима́ться спо́ртом', 'интересова́ться исто́рией'],
    time: ['у́тром', 'ле́том', 'зимо́й'],
  },
  prepositional: {
    location: ['в шко́ле', 'на столе́', 'в Москве́'],
    about: ['говори́ть о рабо́те', 'Я ду́маю о бра́те.'],
    transport: ['е́хать на авто́бусе', 'на велосипе́де'],
    time: ['в ма́е', 'в про́шлом году́'],
  },
} as const satisfies Record<Case, Record<string, readonly string[]>>

// prettier-ignore
export const EXCEPTIONS = {
  nominative: {
    stressedA: ['дом → дома́', 'го́род → города́', 'глаз → глаза́', 'учи́тель → учителя́'],
    plural_ya: ['брат → бра́тья', 'стул → сту́лья', 'друг → друзья́', 'лист → ли́стья'],
    anin: ['англича́нин → англича́не', 'граждани́н → гра́ждане'],
    enok: ['котёнок → котя́та', 'ребёнок → ребя́та'],
    suppletive: ['челове́к → лю́ди', 'ребёнок → де́ти'],
  },
  genitive: {
    fleeting: ['оте́ц → отца́', 'день → дня', 'потоло́к → потолка́'],
    insertedVowel: ['окно́ → о́кон', 'сестра́ → сестёр', 'де́вушка → де́вушек', 'письмо́ → пи́сем'],
    irregularPlural: ['челове́к → люде́й', 'год → лет', 'раз → раз', 'глаз → глаз', 'солда́т → солда́т'],
    partitive: ['ча́шка ча́ю', 'мно́го наро́ду'],
    materDocher: ['мать → ма́тери', 'дочь → до́чери'],
    mya: ['и́мя → и́мени', 'вре́мя → вре́мени'],
  },
  dative: {
    materDocher: ['мать → ма́тери', 'дочь → до́чери'],
    mya: ['и́мя → и́мени'],
    put: ['путь → пути́'],
    suppletive: ['лю́ди → лю́дям', 'де́ти → де́тям'],
  },
  accusative: {
    animate: ['Я ви́жу стол. — Я ви́жу студе́нта.', 'Я ви́жу кни́ги. — Я ви́жу сестёр.'],
    feminineSoftSign: ['Я ви́жу дверь.', 'Я люблю́ мать.'],
    masculineA: ['Я ви́жу па́пу.', 'Я ви́жу дя́дю.'],
  },
  instrumental: {
    materDocher: ['мать → ма́терью', 'дочь → до́черью'],
    put: ['путь → путём'],
    mya: ['и́мя → и́менем'],
    lyudmi: ['лю́ди → людьми́', 'де́ти → детьми́', 'ло́шадь → лошадьми́'],
    long: ['кни́гой / кни́гою', 'мной / мно́ю'],
  },
  prepositional: {
    locative: ['лес → в лесу́ (о ле́се)', 'сад → в саду́', 'пол → на полу́', 'год → в году́', 'мост → на мосту́'],
    ii: ['зда́ние → о зда́нии', 'а́рмия → об а́рмии', 'ге́ний → о ге́нии'],
    put: ['путь → в пути́', 'мать → о ма́тери'],
    ob: ['об отце́, об э́том', 'обо мне, обо всём'],
  },
} as const satisfies Record<Case, Record<string, readonly string[]>>

/** Nouns that never change (borrowings mostly). */
export const INDECLINABLE = ['ко́фе', 'метро́', 'такси́', 'кино́', 'ра́дио', 'пальто́']

export type SpellingRule = 'yi' | 'unstressedE' | 'au'

/** The three spelling rules that make endings look irregular. */
export const SPELLING_EXAMPLES: Record<SpellingRule, string[]> = {
  yi: ['кни́ги', 'ру́сские', 'ножи́'],
  unstressedE: ['ме́сяцем', 'хоро́шего', 'ту́чей'],
  au: ['хоро́шая', 'ноча́м', 'врача́'],
}

export type UseId<C extends Case> = keyof (typeof USES)[C]
export type ExceptionId<C extends Case> = keyof (typeof EXCEPTIONS)[C]
