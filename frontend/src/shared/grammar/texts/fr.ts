import type { GrammarTexts } from './types'

export const fr: GrammarTexts = {
  title: 'Règles de grammaire',
  intro: [
    'En russe, les noms, adjectifs et pronoms changent de terminaison selon leur rôle dans la phrase. Ces formes s’appellent des cas, et il y en a six.',
    'La terminaison indique qui fait quoi, quel que soit l’ordre des mots : dans « Ма́ма ви́дит па́пу » (Maman voit Papa), -а marque celle qui voit et -у celui qui est vu.',
    'Chaque cas répond à ses propres questions (кто ? кого́ ? кому́ ? …) et s’emploie aussi après certaines prépositions et certains verbes. Choisissez un cas pour voir à quoi il sert, ses terminaisons et ses exceptions.',
  ],
  overview: 'Les six cas',
  questions: 'Questions',
  uses: 'Quand l’employer',
  endings: 'Terminaisons',
  exceptions: 'Exceptions',
  spellingRules: 'Règles d’orthographe',
  personalPronouns: 'Pronoms personnels',
  nForms: 'Après une préposition, он, она́, оно́ et они́ prennent un н- : у него́, к ней, с ни́ми.',
  indeclinable: {
    title: 'Noms invariables',
    detail: 'Certains noms empruntés ne changent jamais : la même forme sert à tous les cas.',
  },
  animacy:
    'Comme le nominatif pour les choses, comme le génitif pour les personnes et les animaux.',
  practise: 'S’entraîner sur ce cas',
  allRules: 'Toutes les règles',
  readMore: 'Toutes les règles de ce cas',
  explanation: {
    title: 'Pourquoi ?',
    correct: 'Forme correcte',
    yourChoice: 'Votre choix',
    usedFor: (ending, owners) => `${ending} s’emploie pour : ${owners}.`,
    sameCell: (ending) =>
      `${ending} est la bonne famille de terminaisons, mais ici la règle d’orthographe décide de la lettre.`,
    noTable: 'Ce mot se décline de façon irrégulière : voir les exceptions.',
    continue: 'Continuer',
  },
  groups: {
    nouns: 'Noms',
    adjectives: 'Adjectifs',
    pronouns: 'Pronoms',
    surnames: 'Noms de famille',
  },
  columns: {
    singular: 'Singulier',
    plural: 'Pluriel',
    masculine: 'Masculin',
    neuter: 'Neutre',
    feminine: 'Féminin',
  },
  paradigms: {
    noun_masculine_hard: 'Noms masculins terminés par une consonne',
    noun_masculine_sibilant: 'Noms masculins en ж, ш, щ, ч',
    noun_masculine_soft_sign: 'Noms masculins en -ь',
    noun_masculine_j: 'Noms masculins en -й',
    noun_masculine_iy: 'Noms masculins en -ий',
    noun_neuter_o: 'Noms neutres en -о',
    noun_neuter_e: 'Noms neutres en -е/-ё',
    noun_neuter_ie: 'Noms neutres en -ие',
    noun_a: 'Noms en -а/-я (surtout féminins)',
    noun_iya: 'Noms en -ия',
    noun_feminine_soft_sign: 'Noms féminins en -ь',
    noun_plural_only: 'Noms toujours au pluriel',
    adjective_hard: 'Adjectifs en -ый (durs)',
    adjective_soft: 'Adjectifs en -ий (mous)',
    adjective_stressed: 'Adjectifs en -ой accentué',
    pronoun_possessive: 'мой, твой, свой',
    pronoun_nash: 'наш, ваш',
    pronoun_etot: 'э́тот',
    personal_pronoun: 'Pronoms personnels',
    surname_mixed: 'Noms de famille en -ов/-ев/-ин',
    surname_adjectival: 'Noms de famille en -ский/-ой',
  },
  spelling: {
    yi: {
      title: 'и, jamais ы, après г к х ж ш щ ч',
      detail: 'Quand la terminaison commencerait par ы, on écrit и après ces lettres.',
    },
    unstressedE: {
      title: 'е au lieu de о non accentué après ж ш щ ч ц',
      detail:
        'Une terminaison en о s’écrit avec е après ces lettres quand elle n’est pas accentuée ; accentuée, elle reste о : ножо́м, большо́го.',
    },
    au: {
      title: 'а, у au lieu de я, ю après ж ш щ ч ц',
      detail: 'Ces lettres ne sont jamais suivies de я ou ю dans les terminaisons.',
    },
  },
  cases: {
    nominative: {
      purpose:
        'La forme de base, celle des dictionnaires. Il désigne le sujet : la personne ou la chose qui fait l’action.',
      uses: {
        subject: {
          title: 'Le sujet de la phrase',
          translations: ['L’étudiant lit.', 'Le livre est sur la table.'],
        },
        predicate: {
          title: 'Après « э́то » et dans les définitions',
          detail: 'Au présent, le verbe « être » est omis.',
          translations: ['C’est mon frère.', 'Moscou est la capitale de la Russie.'],
        },
        dictionary: {
          title: 'La forme du dictionnaire',
          detail: 'Les mots sont donnés au nominatif singulier.',
          translations: ['table, livre, fenêtre'],
        },
      },
      exceptions: {
        stressedA: {
          title: 'Pluriel en -а́/-я́ accentué',
          detail: 'Certains noms masculins prennent -а́ au lieu de -ы au pluriel.',
        },
        plural_ya: {
          title: 'Pluriel en -ья',
          detail:
            'Quelques noms masculins ajoutent -ья et le gardent à tous les cas du pluriel (бра́тьев, бра́тьям).',
        },
        anin: {
          title: '-анин/-янин → -ане/-яне',
          detail: 'Les noms de personnes en -ин le perdent au pluriel.',
        },
        enok: {
          title: '-ёнок → -я́та',
          detail: 'Les noms de petits d’animaux et d’enfants changent de suffixe au pluriel.',
        },
        suppletive: { title: 'Un autre mot au pluriel' },
      },
    },
    genitive: {
      purpose:
        'Exprime l’appartenance (« de »), l’absence et la quantité. C’est le cas le plus souvent exigé par les prépositions.',
      uses: {
        possession: {
          title: 'Appartenance : « de »',
          translations: ['le livre du frère', 'le centre de la ville'],
        },
        absence: {
          title: 'Absence : нет, не́ было, не бу́дет',
          translations: ['Je n’ai pas le temps.', 'Il n’y a pas de magasin ici.'],
        },
        quantity: {
          title: 'Quantités et nombres',
          detail:
            'Après мно́го, ма́ло, ско́лько et les contenants. Après 2, 3, 4 : génitif singulier ; à partir de 5 : génitif pluriel.',
          translations: ['beaucoup de gens', 'un verre d’eau', 'deux tables, cinq tables'],
        },
        prepositions: {
          title: 'Après у, из, от, до, без, для, о́коло, по́сле, кро́ме, вме́сто, с (depuis)',
          translations: [
            'près de la maison',
            'de Moscou',
            'sans sucre',
            'pour un ami',
            'après le travail',
          ],
        },
        verbs: {
          title: 'Après certains verbes',
          detail: 'боя́ться, жела́ть, избега́ть, достига́ть…',
          translations: ['J’ai peur des chiens.', 'Je te souhaite du bonheur.'],
        },
      },
      exceptions: {
        fleeting: {
          title: 'Voyelle mobile',
          detail: 'Le dernier о ou е du radical disparaît quand on ajoute une terminaison.',
        },
        insertedVowel: {
          title: 'Voyelle insérée au génitif pluriel',
          detail:
            'Avec la terminaison zéro, un о ou un е apparaît entre les deux dernières consonnes.',
        },
        irregularPlural: {
          title: 'Génitif pluriel irrégulier',
          detail:
            'Quelques noms courants ont un génitif pluriel court ou différent, employé après les nombres.',
        },
        partitive: {
          title: 'Partitif en -у/-ю',
          detail: '« Un peu de » une matière peut prendre -у dans la langue courante.',
        },
        materDocher: {
          title: 'мать, дочь',
          detail:
            'Ils ajoutent -ер- à toutes les formes sauf le nominatif et l’accusatif singuliers.',
        },
        mya: {
          title: 'Noms neutres en -мя',
          detail: 'Dix noms ajoutent -ен- : и́мя, вре́мя, пле́мя, зна́мя…',
        },
      },
    },
    dative: {
      purpose:
        'Désigne le destinataire : la personne à qui ou pour qui l’on fait quelque chose. Sert aussi pour l’âge et les sensations.',
      uses: {
        recipient: {
          title: 'Complément d’attribution : « à qui »',
          translations: ['J’ai donné le livre à un ami.', 'Appelle maman.'],
        },
        impersonal: {
          title: 'Âge, sensations, besoin',
          detail: 'La personne qui ressent quelque chose est au datif.',
          translations: ['J’ai vingt ans.', 'J’ai froid.', 'Nous devons y aller.'],
        },
        prepositions: {
          title: 'Après к (vers), по (le long de, dans), благодаря́, вопреки́',
          translations: ['Je vais chez le médecin.', 'Nous nous promenons dans le parc.'],
        },
        verbs: {
          title: 'Après certains verbes',
          detail: 'нра́виться, помога́ть, звони́ть, сове́товать, ве́рить, меша́ть…',
          translations: ['J’aime la musique.', 'Aide ton frère.'],
        },
      },
      exceptions: {
        materDocher: { title: 'мать, дочь', detail: 'Ils ajoutent -ер- et prennent -и.' },
        mya: { title: 'Noms neutres en -мя', detail: 'Ils ajoutent -ен- et prennent -и.' },
        put: {
          title: 'путь',
          detail: 'Masculin, mais -и au génitif, au datif et au prépositionnel.',
        },
        suppletive: {
          title: 'лю́ди, де́ти',
          detail: 'Des terminaisons régulières sur un pluriel irrégulier.',
        },
      },
    },
    accusative: {
      purpose:
        'Désigne le complément d’objet direct : ce que l’action touche. Avec в et на, il indique aussi la direction (« où l’on va »).',
      uses: {
        object: {
          title: 'Complément d’objet direct',
          translations: ['Je lis un livre.', 'Je vois mon frère.'],
        },
        direction: {
          title: 'Où l’on va : в, на + accusatif',
          detail: 'Comparez в шко́лу (vers, accusatif) et в шко́ле (dans, prépositionnel).',
          translations: ['Je vais à l’école.', 'Pose le livre sur la table.'],
        },
        time: {
          title: 'Le temps',
          detail: 'Les jours de la semaine avec в, et les durées.',
          translations: ['mercredi', 'toute la semaine', 'chaque jour'],
        },
        prepositions: {
          title: 'Après че́рез, про, за (pour) ; под et за avec mouvement',
          translations: ['par le pont', 'Merci pour le cadeau.', 'un film sur la guerre'],
        },
      },
      exceptions: {
        animate: {
          title: 'Noms animés',
          detail:
            'Pour les personnes et les animaux, le masculin singulier et tous les pluriels prennent la forme du génitif ; les choses gardent celle du nominatif.',
        },
        feminineSoftSign: {
          title: 'Noms féminins en -ь',
          detail: 'L’accusatif est identique au nominatif, même pour les personnes.',
        },
        masculineA: {
          title: 'Noms masculins en -а/-я',
          detail: 'Ils se déclinent comme кни́га : -у/-ю.',
        },
      },
    },
    instrumental: {
      purpose:
        'Exprime le moyen (« avec quoi »), l’accompagnement avec с, et la position après certaines prépositions.',
      uses: {
        means: {
          title: 'Outil ou moyen : « avec », « en »',
          translations: ['écrire avec un stylo', 'voyager en train'],
        },
        with: {
          title: 'Avec quelqu’un ou quelque chose : с',
          translations: ['du thé au citron', 'Je me promène avec un ami.'],
        },
        location: {
          title: 'Position : над, под, пе́ред, за, ме́жду',
          translations: [
            'au-dessus de la table',
            'sous le lit',
            'devant la maison',
            'au coin de la rue',
          ],
        },
        being: {
          title: 'Être et devenir',
          detail: 'Après стать, быть (passé et futur), рабо́тать, явля́ться.',
          translations: ['Il est devenu médecin.', 'Elle travaille comme institutrice.'],
        },
        verbs: {
          title: 'Après certains verbes',
          detail: 'занима́ться, интересова́ться, горди́ться, по́льзоваться…',
          translations: ['faire du sport', 's’intéresser à l’histoire'],
        },
        time: {
          title: 'Moments de la journée et saisons',
          translations: ['le matin', 'en été', 'en hiver'],
        },
      },
      exceptions: {
        materDocher: { title: 'мать, дочь', detail: 'Ils ajoutent -ер- et prennent -ью.' },
        put: { title: 'путь', detail: 'Il prend -ём.' },
        mya: { title: 'Noms neutres en -мя', detail: 'Ils ajoutent -ен- et prennent -ем.' },
        lyudmi: {
          title: 'Pluriel en -ьми',
          detail: 'Quelques noms prennent -ьми au lieu de -ями.',
        },
        long: {
          title: 'Formes longues -ою/-ею',
          detail: 'Variantes littéraires ou poétiques de -ой/-ей.',
        },
      },
    },
    prepositional: {
      purpose:
        'S’emploie uniquement après une préposition : le lieu avec в et на (« où »), et le sujet avec о (« à propos de »).',
      uses: {
        location: {
          title: 'Où : в, на + prépositionnel',
          translations: ['à l’école', 'sur la table', 'à Moscou'],
        },
        about: {
          title: 'À propos de : о (об, обо)',
          translations: ['parler du travail', 'Je pense à mon frère.'],
        },
        transport: { title: 'Moyens de transport : на', translations: ['aller en bus', 'à vélo'] },
        time: { title: 'Mois et années : в', translations: ['en mai', 'l’année dernière'] },
      },
      exceptions: {
        locative: {
          title: 'Locatif en -у́ accentué',
          detail: 'Après в et на, certains noms masculins prennent -у́ (mais -е après о).',
        },
        ii: { title: 'Noms en -ие, -ия, -ий', detail: 'Ils prennent -и au lieu de -е.' },
        put: { title: 'путь, мать, дочь', detail: 'Ils prennent -и.' },
        ob: {
          title: 'о → об, обо',
          detail: 'об devant une voyelle, обо devant мне et всём, всех.',
        },
      },
    },
  },
}
