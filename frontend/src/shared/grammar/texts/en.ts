import type { GrammarTexts } from './types'

export const en: GrammarTexts = {
  title: 'Grammar rules',
  intro: [
    'Russian nouns, adjectives and pronouns change their ending according to their role in the sentence. These forms are called cases, and there are six of them.',
    'The ending tells you who does what, whatever the word order: in «Ма́ма ви́дит па́пу» (Mum sees Dad), -а marks the one who sees and -у the one who is seen.',
    'Each case answers its own questions (кто? кого́? кому́? …) and is also required after certain prepositions and verbs. Pick a case to see what it is used for, its endings and its exceptions.',
  ],
  overview: 'The six cases',
  questions: 'Questions',
  uses: 'When to use it',
  endings: 'Endings',
  exceptions: 'Exceptions',
  spellingRules: 'Spelling rules',
  personalPronouns: 'Personal pronouns',
  nForms: 'After a preposition, он, она́, оно́ and они́ take an н-: у него́, к ней, с ни́ми.',
  indeclinable: {
    title: 'Indeclinable nouns',
    detail: 'Some borrowed nouns never change: the same form is used in every case.',
  },
  animacy: 'Like the nominative for things, like the genitive for people and animals.',
  practise: 'Practise this case',
  allRules: 'All rules',
  readMore: 'All the rules for this case',
  explanation: {
    title: 'Why?',
    correct: 'Correct form',
    yourChoice: 'Your choice',
    usedFor: (ending, owners) => `${ending} is used for: ${owners}.`,
    sameCell: (ending) =>
      `${ending} is the right ending family, but here the spelling rule decides the letter.`,
    noTable: 'This word declines irregularly: see the exceptions.',
    continue: 'Continue',
  },
  groups: { nouns: 'Nouns', adjectives: 'Adjectives', pronouns: 'Pronouns', surnames: 'Surnames' },
  columns: {
    singular: 'Singular',
    plural: 'Plural',
    masculine: 'Masculine',
    neuter: 'Neuter',
    feminine: 'Feminine',
  },
  paradigms: {
    noun_masculine_hard: 'Masculine nouns ending in a consonant',
    noun_masculine_sibilant: 'Masculine nouns in ж, ш, щ, ч',
    noun_masculine_soft_sign: 'Masculine nouns in -ь',
    noun_masculine_j: 'Masculine nouns in -й',
    noun_masculine_iy: 'Masculine nouns in -ий',
    noun_neuter_o: 'Neuter nouns in -о',
    noun_neuter_e: 'Neuter nouns in -е/-ё',
    noun_neuter_ie: 'Neuter nouns in -ие',
    noun_a: 'Nouns in -а/-я (mostly feminine)',
    noun_iya: 'Nouns in -ия',
    noun_feminine_soft_sign: 'Feminine nouns in -ь',
    noun_plural_only: 'Plural-only nouns',
    adjective_hard: 'Adjectives in -ый (hard)',
    adjective_soft: 'Adjectives in -ий (soft)',
    adjective_stressed: 'Adjectives in stressed -ой',
    pronoun_possessive: 'мой, твой, свой',
    pronoun_nash: 'наш, ваш',
    pronoun_etot: 'э́тот',
    personal_pronoun: 'Personal pronouns',
    surname_mixed: 'Surnames in -ов/-ев/-ин',
    surname_adjectival: 'Surnames in -ский/-ой',
  },
  spelling: {
    yi: {
      title: 'и, never ы, after г к х ж ш щ ч',
      detail: 'Where the ending would start with ы, write и after these letters.',
    },
    unstressedE: {
      title: 'е instead of unstressed о after ж ш щ ч ц',
      detail:
        'An ending starting with о is written with е after these letters when it is unstressed; stressed, it stays о: ножо́м, большо́го.',
    },
    au: {
      title: 'а, у instead of я, ю after ж ш щ ч ц',
      detail: 'These letters are never followed by я or ю in endings.',
    },
  },
  cases: {
    nominative: {
      purpose:
        'The basic form, found in dictionaries. It names the subject: the person or thing doing the action.',
      uses: {
        subject: {
          title: 'The subject of the sentence',
          translations: ['The student is reading.', 'The book is on the table.'],
        },
        predicate: {
          title: 'After «э́то» and in definitions',
          detail: 'In the present tense, «to be» is left out.',
          translations: ['This is my brother.', 'Moscow is the capital of Russia.'],
        },
        dictionary: {
          title: 'The dictionary form',
          detail: 'Words are listed in the nominative singular.',
          translations: ['table, book, window'],
        },
      },
      exceptions: {
        stressedA: {
          title: 'Plural in stressed -а́/-я́',
          detail: 'Some masculine nouns take -а́ instead of -ы in the plural.',
        },
        plural_ya: {
          title: 'Plural in -ья',
          detail:
            'A few masculine nouns add -ья and keep it in every plural case (бра́тьев, бра́тьям).',
        },
        anin: {
          title: '-анин/-янин → -ане/-яне',
          detail: 'Nouns for people in -ин drop it in the plural.',
        },
        enok: {
          title: '-ёнок → -я́та',
          detail: 'Words for young animals and children change their suffix in the plural.',
        },
        suppletive: { title: 'A different word in the plural' },
      },
    },
    genitive: {
      purpose:
        'Expresses belonging ("of"), absence and quantity. It is the case most often required by prepositions.',
      uses: {
        possession: {
          title: 'Belonging: "of"',
          translations: ["the brother's book", 'the city centre'],
        },
        absence: {
          title: 'Absence: нет, не́ было, не бу́дет',
          translations: ["I don't have time.", "There's no shop here."],
        },
        quantity: {
          title: 'Quantities and numbers',
          detail:
            'After мно́го, ма́ло, ско́лько and containers. After 2, 3, 4: genitive singular; from 5: genitive plural.',
          translations: ['many people', 'a glass of water', 'two tables, five tables'],
        },
        prepositions: {
          title: 'After у, из, от, до, без, для, о́коло, по́сле, кро́ме, вме́сто, с (from)',
          translations: [
            'near the house',
            'from Moscow',
            'without sugar',
            'for a friend',
            'after work',
          ],
        },
        verbs: {
          title: 'After some verbs',
          detail: 'боя́ться, жела́ть, избега́ть, достига́ть…',
          translations: ["I'm afraid of dogs.", 'I wish you happiness.'],
        },
      },
      exceptions: {
        fleeting: {
          title: 'Fleeting vowel',
          detail: 'The last о or е of the stem disappears when an ending is added.',
        },
        insertedVowel: {
          title: 'Inserted vowel in the genitive plural',
          detail: 'With the zero ending, an о or е appears between the last two consonants.',
        },
        irregularPlural: {
          title: 'Irregular genitive plural',
          detail:
            'Some common nouns have a short or different genitive plural, used after numbers.',
        },
        partitive: {
          title: 'Partitive -у/-ю',
          detail: '"Some" of a substance may take -у in everyday speech.',
        },
        materDocher: {
          title: 'мать, дочь',
          detail: 'They add -ер- in every form except the nominative and accusative singular.',
        },
        mya: {
          title: 'Neuter nouns in -мя',
          detail: 'Ten nouns add -ен-: и́мя, вре́мя, пле́мя, зна́мя…',
        },
      },
    },
    dative: {
      purpose:
        'Marks the recipient: the person to or for whom something is done. Also used for age and feelings.',
      uses: {
        recipient: {
          title: 'Indirect object: "to whom"',
          translations: ['I gave the book to a friend.', 'Call Mum.'],
        },
        impersonal: {
          title: 'Age, feelings, need',
          detail: 'The person who experiences something is in the dative.',
          translations: ["I'm twenty.", "I'm cold.", 'We have to go.'],
        },
        prepositions: {
          title: 'After к (towards), по (along, around), благодаря́, вопреки́',
          translations: ["I'm going to the doctor.", 'We walk in the park.'],
        },
        verbs: {
          title: 'After some verbs',
          detail: 'нра́виться, помога́ть, звони́ть, сове́товать, ве́рить, меша́ть…',
          translations: ['I like music.', 'Help your brother.'],
        },
      },
      exceptions: {
        materDocher: { title: 'мать, дочь', detail: 'They add -ер- and take -и.' },
        mya: { title: 'Neuter nouns in -мя', detail: 'They add -ен- and take -и.' },
        put: {
          title: 'путь',
          detail: 'Masculine, but -и in the genitive, dative and prepositional.',
        },
        suppletive: {
          title: 'лю́ди, де́ти',
          detail: 'Regular endings on an irregular plural.',
        },
      },
    },
    accusative: {
      purpose:
        'Marks the direct object: what the action is done to. With в and на it also gives the direction ("where to").',
      uses: {
        object: {
          title: 'Direct object',
          translations: ["I'm reading a book.", 'I see my brother.'],
        },
        direction: {
          title: 'Where to: в, на + accusative',
          detail: 'Compare в шко́лу (into, accusative) and в шко́ле (in, prepositional).',
          translations: ["I'm going to school.", 'Put the book on the table.'],
        },
        time: {
          title: 'Time',
          detail: 'Days of the week with в, and durations.',
          translations: ['on Wednesday', 'all week', 'every day'],
        },
        prepositions: {
          title: 'After че́рез, про, за (for); под and за with motion',
          translations: ['across the bridge', 'Thank you for the present.', 'a film about the war'],
        },
      },
      exceptions: {
        animate: {
          title: 'Animate nouns',
          detail:
            'For people and animals, masculine singular and all plural nouns take the genitive form; things keep the nominative form.',
        },
        feminineSoftSign: {
          title: 'Feminine nouns in -ь',
          detail: 'The accusative is the same as the nominative, even for people.',
        },
        masculineA: {
          title: 'Masculine nouns in -а/-я',
          detail: 'They decline like кни́га: -у/-ю.',
        },
      },
    },
    instrumental: {
      purpose:
        'Expresses the means ("with what"), company with с, and position after some prepositions.',
      uses: {
        means: {
          title: 'Tool or means: "with", "by"',
          translations: ['to write with a pen', 'to travel by train'],
        },
        with: {
          title: 'Together with: с',
          translations: ['tea with lemon', 'I walk with a friend.'],
        },
        location: {
          title: 'Position: над, под, пе́ред, за, ме́жду',
          translations: [
            'above the table',
            'under the bed',
            'in front of the house',
            'around the corner',
          ],
        },
        being: {
          title: 'Being and becoming',
          detail: 'After стать, быть (past and future), рабо́тать, явля́ться.',
          translations: ['He became a doctor.', 'She works as a teacher.'],
        },
        verbs: {
          title: 'After some verbs',
          detail: 'занима́ться, интересова́ться, горди́ться, по́льзоваться…',
          translations: ['to do sport', 'to be interested in history'],
        },
        time: {
          title: 'Times of day and seasons',
          translations: ['in the morning', 'in summer', 'in winter'],
        },
      },
      exceptions: {
        materDocher: { title: 'мать, дочь', detail: 'They add -ер- and take -ью.' },
        put: { title: 'путь', detail: 'It takes -ём.' },
        mya: { title: 'Neuter nouns in -мя', detail: 'They add -ен- and take -ем.' },
        lyudmi: { title: 'Plural in -ьми', detail: 'A few nouns take -ьми instead of -ями.' },
        long: { title: 'Long forms -ою/-ею', detail: 'Literary or poetic variants of -ой/-ей.' },
      },
    },
    prepositional: {
      purpose:
        'Only used after prepositions: location with в and на ("where"), and the topic with о ("about").',
      uses: {
        location: {
          title: 'Where: в, на + prepositional',
          translations: ['at school', 'on the table', 'in Moscow'],
        },
        about: {
          title: 'About: о (об, обо)',
          translations: ['to talk about work', "I'm thinking about my brother."],
        },
        transport: { title: 'Means of transport: на', translations: ['to go by bus', 'by bike'] },
        time: { title: 'Months and years: в', translations: ['in May', 'last year'] },
      },
      exceptions: {
        locative: {
          title: 'Locative in stressed -у́',
          detail: 'After в and на, some masculine nouns take -у́ (but -е after о).',
        },
        ii: { title: 'Nouns in -ие, -ия, -ий', detail: 'They take -и instead of -е.' },
        put: { title: 'путь, мать, дочь', detail: 'They take -и.' },
        ob: { title: 'о → об, обо', detail: 'об before a vowel, обо before мне and всём, всех.' },
      },
    },
  },
}
