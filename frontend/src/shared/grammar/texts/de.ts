import type { GrammarTexts } from './types'

export const de: GrammarTexts = {
  title: 'Grammatikregeln',
  intro: [
    'Im Russischen ändern Substantive, Adjektive und Pronomen ihre Endung je nach ihrer Rolle im Satz. Diese Formen heißen Fälle (Kasus), und es gibt sechs davon.',
    'Die Endung zeigt, wer was tut, egal in welcher Wortstellung: In „Ма́ма ви́дит па́пу“ (Mama sieht Papa) kennzeichnet -а die Person, die sieht, und -у die, die gesehen wird.',
    'Jeder Fall beantwortet eigene Fragen (кто? кого́? кому́? …) und steht auch nach bestimmten Präpositionen und Verben. Wähle einen Fall, um zu sehen, wofür er gebraucht wird, welche Endungen und welche Ausnahmen er hat.',
  ],
  overview: 'Die sechs Fälle',
  questions: 'Fragen',
  uses: 'Wann man ihn verwendet',
  endings: 'Endungen',
  exceptions: 'Ausnahmen',
  spellingRules: 'Rechtschreibregeln',
  personalPronouns: 'Personalpronomen',
  nForms: 'Nach einer Präposition bekommen он, она́, оно́ und они́ ein н-: у него́, к ней, с ни́ми.',
  indeclinable: {
    title: 'Unveränderliche Substantive',
    detail: 'Manche Lehnwörter ändern sich nie: In allen Fällen gilt dieselbe Form.',
  },
  animacy: 'Wie der Nominativ bei Dingen, wie der Genitiv bei Menschen und Tieren.',
  practise: 'Diesen Fall üben',
  allRules: 'Alle Regeln',
  readMore: 'Alle Regeln zu diesem Fall',
  explanation: {
    title: 'Warum?',
    correct: 'Richtige Form',
    yourChoice: 'Deine Wahl',
    usedFor: (ending, owners) => `${ending} steht bei: ${owners}.`,
    sameCell: (ending) =>
      `${ending} gehört zur richtigen Endungsgruppe, aber hier entscheidet die Rechtschreibregel über den Buchstaben.`,
    noTable: 'Dieses Wort wird unregelmäßig dekliniert: siehe die Ausnahmen.',
    continue: 'Weiter',
  },
  groups: {
    nouns: 'Substantive',
    adjectives: 'Adjektive',
    pronouns: 'Pronomen',
    surnames: 'Familiennamen',
  },
  columns: {
    singular: 'Singular',
    plural: 'Plural',
    masculine: 'Maskulinum',
    neuter: 'Neutrum',
    feminine: 'Femininum',
  },
  paradigms: {
    noun_masculine_hard: 'Maskulina auf Konsonant',
    noun_masculine_sibilant: 'Maskulina auf ж, ш, щ, ч',
    noun_masculine_soft_sign: 'Maskulina auf -ь',
    noun_masculine_j: 'Maskulina auf -й',
    noun_masculine_iy: 'Maskulina auf -ий',
    noun_neuter_o: 'Neutra auf -о',
    noun_neuter_e: 'Neutra auf -е/-ё',
    noun_neuter_ie: 'Neutra auf -ие',
    noun_a: 'Substantive auf -а/-я (meist feminin)',
    noun_iya: 'Substantive auf -ия',
    noun_feminine_soft_sign: 'Feminina auf -ь',
    noun_plural_only: 'Substantive nur im Plural',
    adjective_hard: 'Adjektive auf -ый (hart)',
    adjective_soft: 'Adjektive auf -ий (weich)',
    adjective_stressed: 'Adjektive auf betontes -ой',
    pronoun_possessive: 'мой, твой, свой',
    pronoun_nash: 'наш, ваш',
    pronoun_etot: 'э́тот',
    personal_pronoun: 'Personalpronomen',
    surname_mixed: 'Familiennamen auf -ов/-ев/-ин',
    surname_adjectival: 'Familiennamen auf -ский/-ой',
  },
  spelling: {
    yi: {
      title: 'и, nie ы, nach г к х ж ш щ ч',
      detail: 'Wo die Endung mit ы beginnen würde, schreibt man nach diesen Buchstaben и.',
    },
    unstressedE: {
      title: 'е statt unbetontem о nach ж ш щ ч ц',
      detail:
        'Eine Endung mit о wird nach diesen Buchstaben mit е geschrieben, wenn sie unbetont ist; betont bleibt sie о: ножо́м, большо́го.',
    },
    au: {
      title: 'а, у statt я, ю nach ж ш щ ч ц',
      detail: 'Auf diese Buchstaben folgt in Endungen nie я oder ю.',
    },
  },
  cases: {
    nominative: {
      purpose:
        'Die Grundform, die im Wörterbuch steht. Er bezeichnet das Subjekt: die Person oder Sache, die handelt.',
      uses: {
        subject: {
          title: 'Das Subjekt des Satzes',
          translations: ['Der Student liest.', 'Das Buch liegt auf dem Tisch.'],
        },
        predicate: {
          title: 'Nach „э́то“ und in Definitionen',
          detail: 'Im Präsens fällt „sein“ weg.',
          translations: ['Das ist mein Bruder.', 'Moskau ist die Hauptstadt Russlands.'],
        },
        dictionary: {
          title: 'Die Wörterbuchform',
          detail: 'Wörter stehen im Nominativ Singular im Wörterbuch.',
          translations: ['Tisch, Buch, Fenster'],
        },
      },
      exceptions: {
        stressedA: {
          title: 'Plural auf betontes -а́/-я́',
          detail: 'Manche Maskulina haben im Plural -а́ statt -ы.',
        },
        plural_ya: {
          title: 'Plural auf -ья',
          detail:
            'Einige Maskulina fügen -ья an und behalten es in allen Pluralfällen (бра́тьев, бра́тьям).',
        },
        anin: {
          title: '-анин/-янин → -ане/-яне',
          detail: 'Personenbezeichnungen auf -ин verlieren es im Plural.',
        },
        enok: {
          title: '-ёнок → -я́та',
          detail: 'Wörter für Tierjunge und Kinder ändern im Plural ihr Suffix.',
        },
        suppletive: { title: 'Ein anderes Wort im Plural' },
      },
    },
    genitive: {
      purpose:
        'Drückt Zugehörigkeit („von“), Fehlen und Menge aus. Er ist der Fall, den Präpositionen am häufigsten verlangen.',
      uses: {
        possession: {
          title: 'Zugehörigkeit: „von“',
          translations: ['das Buch des Bruders', 'das Stadtzentrum'],
        },
        absence: {
          title: 'Fehlen: нет, не́ было, не бу́дет',
          translations: ['Ich habe keine Zeit.', 'Hier gibt es kein Geschäft.'],
        },
        quantity: {
          title: 'Mengen und Zahlen',
          detail:
            'Nach мно́го, ма́ло, ско́лько und Behältern. Nach 2, 3, 4: Genitiv Singular; ab 5: Genitiv Plural.',
          translations: ['viele Leute', 'ein Glas Wasser', 'zwei Tische, fünf Tische'],
        },
        prepositions: {
          title: 'Nach у, из, от, до, без, для, о́коло, по́сле, кро́ме, вме́сто, с (von … herab)',
          translations: [
            'beim Haus',
            'aus Moskau',
            'ohne Zucker',
            'für einen Freund',
            'nach der Arbeit',
          ],
        },
        verbs: {
          title: 'Nach einigen Verben',
          detail: 'боя́ться, жела́ть, избега́ть, достига́ть…',
          translations: ['Ich habe Angst vor Hunden.', 'Ich wünsche dir Glück.'],
        },
      },
      exceptions: {
        fleeting: {
          title: 'Flüchtiger Vokal',
          detail: 'Das letzte о oder е des Stammes fällt weg, wenn eine Endung hinzukommt.',
        },
        insertedVowel: {
          title: 'Eingeschobener Vokal im Genitiv Plural',
          detail:
            'Bei der Nullendung erscheint ein о oder е zwischen den letzten beiden Konsonanten.',
        },
        irregularPlural: {
          title: 'Unregelmäßiger Genitiv Plural',
          detail:
            'Einige häufige Substantive haben einen kurzen oder anderen Genitiv Plural, der nach Zahlen steht.',
        },
        partitive: {
          title: 'Partitiv auf -у/-ю',
          detail: '„Etwas von“ einem Stoff kann umgangssprachlich -у bekommen.',
        },
        materDocher: {
          title: 'мать, дочь',
          detail: 'Sie fügen in allen Formen außer Nominativ und Akkusativ Singular -ер- ein.',
        },
        mya: {
          title: 'Neutra auf -мя',
          detail: 'Zehn Substantive fügen -ен- ein: и́мя, вре́мя, пле́мя, зна́мя…',
        },
      },
    },
    dative: {
      purpose:
        'Bezeichnet den Empfänger: die Person, der oder für die etwas getan wird. Auch für Alter und Empfindungen.',
      uses: {
        recipient: {
          title: 'Indirektes Objekt: „wem“',
          translations: ['Ich habe einem Freund das Buch gegeben.', 'Ruf Mama an.'],
        },
        impersonal: {
          title: 'Alter, Empfindungen, Notwendigkeit',
          detail: 'Die Person, die etwas empfindet, steht im Dativ.',
          translations: ['Ich bin zwanzig.', 'Mir ist kalt.', 'Wir müssen gehen.'],
        },
        prepositions: {
          title: 'Nach к (zu), по (entlang, durch), благодаря́, вопреки́',
          translations: ['Ich gehe zum Arzt.', 'Wir gehen im Park spazieren.'],
        },
        verbs: {
          title: 'Nach einigen Verben',
          detail: 'нра́виться, помога́ть, звони́ть, сове́товать, ве́рить, меша́ть…',
          translations: ['Mir gefällt Musik.', 'Hilf deinem Bruder.'],
        },
      },
      exceptions: {
        materDocher: { title: 'мать, дочь', detail: 'Sie fügen -ер- ein und enden auf -и.' },
        mya: { title: 'Neutra auf -мя', detail: 'Sie fügen -ен- ein und enden auf -и.' },
        put: {
          title: 'путь',
          detail: 'Maskulin, aber -и im Genitiv, Dativ und Präpositiv.',
        },
        suppletive: {
          title: 'лю́ди, де́ти',
          detail: 'Regelmäßige Endungen an einem unregelmäßigen Plural.',
        },
      },
    },
    accusative: {
      purpose:
        'Bezeichnet das direkte Objekt: worauf sich die Handlung richtet. Mit в und на gibt er auch die Richtung an („wohin“).',
      uses: {
        object: {
          title: 'Direktes Objekt',
          translations: ['Ich lese ein Buch.', 'Ich sehe meinen Bruder.'],
        },
        direction: {
          title: 'Wohin: в, на + Akkusativ',
          detail: 'Vergleiche в шко́лу (in … hinein, Akkusativ) und в шко́ле (in, Präpositiv).',
          translations: ['Ich gehe in die Schule.', 'Leg das Buch auf den Tisch.'],
        },
        time: {
          title: 'Zeit',
          detail: 'Wochentage mit в und Zeitdauer.',
          translations: ['am Mittwoch', 'die ganze Woche', 'jeden Tag'],
        },
        prepositions: {
          title: 'Nach че́рез, про, за (für); под und за bei Bewegung',
          translations: ['über die Brücke', 'Danke für das Geschenk.', 'ein Film über den Krieg'],
        },
      },
      exceptions: {
        animate: {
          title: 'Belebte Substantive',
          detail:
            'Bei Menschen und Tieren haben Maskulina im Singular und alle Plurale die Form des Genitivs; Dinge behalten die Form des Nominativs.',
        },
        feminineSoftSign: {
          title: 'Feminina auf -ь',
          detail: 'Der Akkusativ ist gleich dem Nominativ, auch bei Personen.',
        },
        masculineA: {
          title: 'Maskulina auf -а/-я',
          detail: 'Sie werden wie кни́га dekliniert: -у/-ю.',
        },
      },
    },
    instrumental: {
      purpose:
        'Drückt das Mittel aus („womit“), die Begleitung mit с und die Lage nach einigen Präpositionen.',
      uses: {
        means: {
          title: 'Werkzeug oder Mittel: „mit“',
          translations: ['mit einem Stift schreiben', 'mit dem Zug fahren'],
        },
        with: {
          title: 'Zusammen mit: с',
          translations: ['Tee mit Zitrone', 'Ich gehe mit einem Freund spazieren.'],
        },
        location: {
          title: 'Lage: над, под, пе́ред, за, ме́жду',
          translations: ['über dem Tisch', 'unter dem Bett', 'vor dem Haus', 'um die Ecke'],
        },
        being: {
          title: 'Sein und werden',
          detail: 'Nach стать, быть (Vergangenheit und Zukunft), рабо́тать, явля́ться.',
          translations: ['Er ist Arzt geworden.', 'Sie arbeitet als Lehrerin.'],
        },
        verbs: {
          title: 'Nach einigen Verben',
          detail: 'занима́ться, интересова́ться, горди́ться, по́льзоваться…',
          translations: ['Sport treiben', 'sich für Geschichte interessieren'],
        },
        time: {
          title: 'Tageszeiten und Jahreszeiten',
          translations: ['morgens', 'im Sommer', 'im Winter'],
        },
      },
      exceptions: {
        materDocher: { title: 'мать, дочь', detail: 'Sie fügen -ер- ein und enden auf -ью.' },
        put: { title: 'путь', detail: 'Endet auf -ём.' },
        mya: { title: 'Neutra auf -мя', detail: 'Sie fügen -ен- ein und enden auf -ем.' },
        lyudmi: { title: 'Plural auf -ьми', detail: 'Einige Substantive haben -ьми statt -ями.' },
        long: {
          title: 'Langformen -ою/-ею',
          detail: 'Literarische oder poetische Varianten von -ой/-ей.',
        },
      },
    },
    prepositional: {
      purpose:
        'Steht nur nach Präpositionen: der Ort mit в und на („wo“) und das Thema mit о („über“).',
      uses: {
        location: {
          title: 'Wo: в, на + Präpositiv',
          translations: ['in der Schule', 'auf dem Tisch', 'in Moskau'],
        },
        about: {
          title: 'Über: о (об, обо)',
          translations: ['über die Arbeit sprechen', 'Ich denke an meinen Bruder.'],
        },
        transport: {
          title: 'Verkehrsmittel: на',
          translations: ['mit dem Bus fahren', 'mit dem Fahrrad'],
        },
        time: { title: 'Monate und Jahre: в', translations: ['im Mai', 'letztes Jahr'] },
      },
      exceptions: {
        locative: {
          title: 'Lokativ auf betontes -у́',
          detail: 'Nach в und на haben einige Maskulina -у́ (aber -е nach о).',
        },
        ii: { title: 'Substantive auf -ие, -ия, -ий', detail: 'Sie enden auf -и statt -е.' },
        put: { title: 'путь, мать, дочь', detail: 'Sie enden auf -и.' },
        ob: { title: 'о → об, обо', detail: 'об vor einem Vokal, обо vor мне und всём, всех.' },
      },
    },
  },
}
