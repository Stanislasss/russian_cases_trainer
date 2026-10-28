//! Behaviour every `ExerciseRepository` must share, run against each database by
//! `contract_tests!` in the adapter's own tests.

use application::game::{
    Attempt, AttemptRepository, ExerciseRepository, GameFilter, KNOWN_STREAK, MAX_ZONES,
    MIN_ZONE_SHARE,
};
use domain::game::LevelId;
use domain::grammar::{Animacy, Case, Gender, Locale, Number, PartOfSpeech};
use domain::lexicon::{NewForm, NewLexeme, NewTranslation, Slot};
use domain::user::UserId;

/// A fresh database holding `words`, seen through its exercise repository.
pub trait Seed {
    async fn seed(self, words: &[NewLexeme]) -> Box<dyn ExerciseRepository>;

    /// The same, plus a player and where their answers go.
    async fn seed_with_player(
        self,
        words: &[NewLexeme],
    ) -> (
        Box<dyn ExerciseRepository>,
        Box<dyn AttemptRepository>,
        UserId,
    );
}

/// One test per contract function below, each on a fresh database from `$seed`:
/// `contract_tests!(test attribute, (test fn arguments) => seed expression)`.
macro_rules! contract_tests {
    ($attr:meta, $args:tt => $seed:expr) => {
        $crate::contract::contract_tests!(@each $attr, $args => $seed;
            zones_come_from_the_selected_levels,
            zones_are_capped_to_the_most_common,
            rare_endings_get_no_zone,
            items_carry_answers_variants_and_translations,
            items_show_up_to_three_meanings_per_language,
            adjective_items_say_which_form_is_asked,
            a_batch_asks_each_word_once,
            the_dictionary_form_is_never_asked,
            personal_pronouns_are_asked_as_whole_forms,
            animacy_filters_keep_only_such_nouns,
            known_words_come_up_less_often,
        );
    };
    (@each $attr:meta, $args:tt => $seed:expr; $($name:ident),* $(,)?) => {
        $(
            #[$attr]
            async fn $name $args {
                $crate::contract::$name($seed).await
            }
        )*
    };
}
pub(crate) use contract_tests;

pub fn lexeme(lemma: &str, pos: PartOfSpeech, rank: i32, indeclinable: bool) -> NewLexeme {
    NewLexeme {
        source: "test".into(),
        source_id: lemma.into(),
        lemma: lemma.into(),
        stressed_lemma: None,
        pos,
        gender: None,
        animacy: None,
        indeclinable,
        frequency_rank: Some(rank),
        forms: Vec::new(),
        translations: Vec::new(),
    }
}

/// A noun with the given instrumental forms; stress marks optional.
pub fn noun(
    lemma: &str,
    rank: i32,
    indeclinable: bool,
    forms: &[(Number, &[&str])],
    translations: &[(Locale, &str)],
) -> NewLexeme {
    NewLexeme {
        forms: forms
            .iter()
            .flat_map(|&(number, variants)| {
                (0..).zip(variants.iter()).map(move |(variant, form)| {
                    let slot = Slot::new(Case::Instrumental, number);
                    NewForm::noun(lemma, form, slot, variant, indeclinable)
                })
            })
            .collect(),
        translations: (0..)
            .zip(translations)
            .map(|(position, &(locale, text))| NewTranslation {
                locale,
                position,
                text: text.into(),
            })
            .collect(),
        ..lexeme(lemma, PartOfSpeech::Noun, rank, indeclinable)
    }
}

pub fn nouns() -> Vec<NewLexeme> {
    use Number::*;
    vec![
        noun(
            "книга",
            1,
            false,
            &[
                (Singular, &["кни\u{301}гой", "кни\u{301}гою"]),
                (Plural, &["кни\u{301}гами"]),
            ],
            &[(Locale::En, "book")],
        ),
        noun("отец", 2, false, &[(Singular, &["отцо\u{301}м"])], &[]), // irregular
        noun("Москва", 3, false, &[(Singular, &["Москво\u{301}й"])], &[]), // proper noun
        noun("кофе", 4, true, &[(Singular, &["ко\u{301}фе"])], &[]),   // indeclinable
        noun(
            "стол",
            500,
            false,
            &[
                (Singular, &["столо\u{301}м"]),
                (Plural, &["стола\u{301}ми"]),
            ],
            &[(Locale::En, "table"), (Locale::Fr, "table (fr)")],
        ),
    ]
}

fn filter(levels: &[LevelId], case: Case) -> GameFilter {
    let levels: Vec<_> = levels.iter().map(|l| l.spec()).collect();
    GameFilter {
        answer: levels[0].answer,
        levels,
        case,
        animacy: None,
        rarer_known_by: None,
    }
}

pub async fn zones_come_from_the_selected_levels(db: impl Seed) {
    let repo = db.seed(&nouns()).await;
    let instrumental = |levels: &[LevelId]| filter(levels, Case::Instrumental);

    let mut basic = repo
        .zones(
            &instrumental(&[LevelId::BasicNouns]),
            MIN_ZONE_SHARE,
            MAX_ZONES,
        )
        .await
        .unwrap();
    basic.sort();
    // Only книга qualifies: отец is irregular, Москва a proper noun, кофе indeclinable,
    // стол too rare, and plurals aren't in the basic level.
    assert_eq!(basic, ["ой"]);

    let mut wider = repo
        .zones(
            &instrumental(&[LevelId::BasicNouns, LevelId::IntermediateNouns]),
            MIN_ZONE_SHARE,
            MAX_ZONES,
        )
        .await
        .unwrap();
    wider.sort();
    assert_eq!(wider, ["ами", "ой", "ом"]);
}

pub async fn zones_are_capped_to_the_most_common(db: impl Seed) {
    let repo = db.seed(&nouns()).await;
    // Intermediate pool: ой, ом, ами, ами → ами is the most common.
    let zones = repo
        .zones(
            &filter(&[LevelId::IntermediateNouns], Case::Instrumental),
            0.0,
            1,
        )
        .await
        .unwrap();
    assert_eq!(zones, ["ами"]);
}

pub async fn rare_endings_get_no_zone(db: impl Seed) {
    let repo = db.seed(&nouns()).await;
    // Intermediate pool: ой, ом, ами, ами → only ами reaches 40%.
    let zones = repo
        .zones(
            &filter(&[LevelId::IntermediateNouns], Case::Instrumental),
            0.4,
            MAX_ZONES,
        )
        .await
        .unwrap();
    assert_eq!(zones, ["ами"]);
}

pub async fn items_carry_answers_variants_and_translations(db: impl Seed) {
    let repo = db.seed(&nouns()).await;

    let basic = filter(&[LevelId::BasicNouns], Case::Instrumental);
    let items = repo.random_items(&basic, &["ой".into()], 10).await.unwrap();
    assert_eq!(items.len(), 1);
    let kniga = &items[0];
    assert_eq!(kniga.stressed_lemma, "книга"); // no stressed lemma seeded
    assert_eq!(
        (kniga.number, kniga.gender, kniga.animacy),
        (Number::Singular, None, None)
    );
    assert_eq!(kniga.answer, "ой");
    let mut accepted = kniga.accepted.clone();
    accepted.sort();
    assert_eq!(accepted, ["ой", "ою"]);
    assert_eq!(kniga.stressed_form, "кни\u{301}гой");
    assert_eq!(kniga.paradigm, Some(domain::rules::Paradigm::NounA));
    assert_eq!(kniga.translations, [(Locale::En, "book".to_owned())]);

    let intermediate = filter(&[LevelId::IntermediateNouns], Case::Instrumental);
    let items = repo
        .random_items(&intermediate, &["ом".into()], 10)
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    let mut translations = items[0].translations.clone();
    translations.sort_by_key(|(locale, _)| locale.as_str());
    assert_eq!(
        translations,
        [
            (Locale::En, "table".to_owned()),
            (Locale::Fr, "table (fr)".to_owned())
        ]
    );
}

pub async fn items_show_up_to_three_meanings_per_language(db: impl Seed) {
    use Number::*;
    let meanings = [
        (Locale::En, "hand"),
        (Locale::En, "arm"),
        (Locale::En, "handwriting"),
        (Locale::En, "side"),
    ];
    // noun() numbers translations 0.. across locales, as one list here.
    let ruka = noun("рука", 1, false, &[(Singular, &["руко́й"])], &meanings);
    let items = db
        .seed(&[ruka])
        .await
        .random_items(
            &filter(&[LevelId::BasicNouns], Case::Instrumental),
            &["ой".into()],
            10,
        )
        .await
        .unwrap();
    assert_eq!(
        items[0].translations,
        [(Locale::En, "hand; arm; handwriting".to_owned())]
    );
}

pub async fn adjective_items_say_which_form_is_asked(db: impl Seed) {
    let m_anim = Slot::new(Case::Accusative, Number::Singular)
        .gender(Gender::Masculine)
        .animacy(Animacy::Animate);
    let m_inan = m_anim.animacy(Animacy::Inanimate);
    let f = Slot::new(Case::Accusative, Number::Singular).gender(Gender::Feminine);
    let novy = NewLexeme {
        forms: vec![
            NewForm::adjective("новый", "но\u{301}вого", m_anim, 0),
            NewForm::adjective("новый", "но\u{301}вый", m_inan, 0),
            NewForm::adjective("новый", "но\u{301}вую", f, 0),
        ],
        ..lexeme("новый", PartOfSpeech::Adjective, 1, false)
    };
    let repo = db.seed(&[novy]).await;
    let accusative = filter(&[LevelId::BasicAdjectives], Case::Accusative);

    let mut zones = repo
        .zones(&accusative, MIN_ZONE_SHARE, MAX_ZONES)
        .await
        .unwrap();
    zones.sort();
    assert_eq!(zones, ["ого", "ую", "ый"]);

    let items = repo
        .random_items(&accusative, &["ого".into()], 10)
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].gender, Some(Gender::Masculine));
    assert_eq!(items[0].animacy, Some(Animacy::Animate));
    assert_eq!(items[0].accepted, ["ого"]); // not the inanimate ый
}

pub async fn a_batch_asks_each_word_once(db: impl Seed) {
    let slot = |gender| Slot::new(Case::Genitive, Number::Singular).gender(gender);
    let novy = NewLexeme {
        forms: vec![
            NewForm::adjective("новый", "но\u{301}вого", slot(Gender::Masculine), 0),
            NewForm::adjective("новый", "но\u{301}вого", slot(Gender::Neuter), 0),
            NewForm::adjective("новый", "но\u{301}вой", slot(Gender::Feminine), 0),
        ],
        ..lexeme("новый", PartOfSpeech::Adjective, 1, false)
    };
    let items = db
        .seed(&[novy])
        .await
        .random_items(
            &filter(&[LevelId::BasicAdjectives], Case::Genitive),
            &["ого".into(), "ой".into()],
            10,
        )
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
}

pub async fn the_dictionary_form_is_never_asked(db: impl Seed) {
    let nom = |number| Slot::new(Case::Nominative, number);
    let kniga = NewLexeme {
        forms: vec![
            NewForm::noun("книга", "кни\u{301}га", nom(Number::Singular), 0, false),
            NewForm::noun("книга", "кни\u{301}ги", nom(Number::Plural), 0, false),
        ],
        ..lexeme("книга", PartOfSpeech::Noun, 1, false)
    };
    let zones = db
        .seed(&[kniga])
        .await
        .zones(
            &filter(&[LevelId::IntermediateNouns], Case::Nominative),
            MIN_ZONE_SHARE,
            MAX_ZONES,
        )
        .await
        .unwrap();
    assert_eq!(zones, ["и"]);
}

pub async fn personal_pronouns_are_asked_as_whole_forms(db: impl Seed) {
    let genitive = Slot::new(Case::Genitive, Number::Singular);
    let pronoun = |lemma: &str, rank, forms: &[&str]| NewLexeme {
        forms: (0..)
            .zip(forms)
            .map(|(variant, form)| NewForm::whole(form, genitive, variant))
            .collect(),
        ..lexeme(lemma, PartOfSpeech::PersonalPronoun, rank, false)
    };
    let repo = db
        .seed(&[
            pronoun("я", 1, &["меня\u{301}"]),
            pronoun("он", 3, &["его\u{301}", "него\u{301}"]),
            pronoun("ты", 2, &["тебя\u{301}"]),
        ])
        .await;
    let genitive = filter(&[LevelId::PersonalPronouns], Case::Genitive);

    // Ordered like the pronouns themselves: я, ты, он.
    let zones = repo
        .zones(&genitive, MIN_ZONE_SHARE, MAX_ZONES)
        .await
        .unwrap();
    assert_eq!(zones, ["меня", "тебя", "его"]);

    let items = repo
        .random_items(&genitive, &["его".into()], 10)
        .await
        .unwrap();
    let mut accepted = items[0].accepted.clone();
    accepted.sort();
    assert_eq!(accepted, ["его", "него"]);
}

pub async fn animacy_filters_keep_only_such_nouns(db: impl Seed) {
    use Number::*;
    let with_animacy = |mut word: NewLexeme, animacy| {
        word.animacy = Some(animacy);
        word
    };
    let repo = db
        .seed(&[
            with_animacy(
                noun("брат", 1, false, &[(Singular, &["бра\u{301}том"])], &[]),
                Animacy::Animate,
            ),
            with_animacy(
                noun("стол", 2, false, &[(Singular, &["столо\u{301}м"])], &[]),
                Animacy::Inanimate,
            ),
        ])
        .await;
    let only = |animacy| GameFilter {
        animacy: Some(animacy),
        ..filter(&[LevelId::BasicNouns], Case::Instrumental)
    };
    let lemmas = |items: Vec<application::game::GameItem>| -> Vec<String> {
        items.into_iter().map(|i| i.stressed_lemma).collect()
    };

    let animate = repo
        .random_items(&only(Animacy::Animate), &["ом".into()], 10)
        .await
        .unwrap();
    assert_eq!(lemmas(animate), ["брат"]);
    let inanimate = repo
        .random_items(&only(Animacy::Inanimate), &["ом".into()], 10)
        .await
        .unwrap();
    assert_eq!(lemmas(inanimate), ["стол"]);
    let mut any = lemmas(
        repo.random_items(
            &filter(&[LevelId::BasicNouns], Case::Instrumental),
            &["ом".into()],
            10,
        )
        .await
        .unwrap(),
    );
    any.sort();
    assert_eq!(any, ["брат", "стол"]);
    let zones = repo
        .zones(&only(Animacy::Animate), MIN_ZONE_SHARE, MAX_ZONES)
        .await
        .unwrap();
    assert_eq!(zones, ["ом"]);
}

/// Known words get a quarter of the weight: of книга (known) and стол, one word
/// drawn is книга 1 time in 5, and 1 in 2 when all words are alike.
pub async fn known_words_come_up_less_often(db: impl Seed) {
    use Number::*;
    let (repo, attempts, player) = db
        .seed_with_player(&[
            noun("книга", 1, false, &[(Singular, &["кни\u{301}гой"])], &[]),
            noun("стол", 2, false, &[(Singular, &["столо\u{301}м"])], &[]),
        ])
        .await;
    let zones = ["ой".to_owned(), "ом".to_owned()];
    let all_alike = filter(&[LevelId::BasicNouns], Case::Instrumental);
    let ids: Vec<(String, i64)> = repo
        .random_items(&all_alike, &zones, 10)
        .await
        .unwrap()
        .into_iter()
        .map(|i| (i.stressed_lemma, i.lexeme_id))
        .collect();
    let id = |lemma: &str| ids.iter().find(|(l, _)| l == lemma).unwrap().1;
    let answer = |lemma, case, correct| Attempt {
        lexeme_id: id(lemma),
        case,
        number: Singular,
        gender: None,
        animacy: None,
        answer: "ой".into(),
        correct,
    };
    let record = async |answers: Vec<Attempt>| attempts.record(player, "endings", &answers).await;

    // книга: an old mistake, then right `KNOWN_STREAK` times. стол: right one
    // time less, and right in another case, which doesn't count.
    let mut kniga = vec![answer("книга", Case::Instrumental, false)];
    kniga.extend((0..KNOWN_STREAK).map(|_| answer("книга", Case::Instrumental, true)));
    record(kniga).await.unwrap();
    let mut stol: Vec<_> = (1..KNOWN_STREAK)
        .map(|_| answer("стол", Case::Instrumental, true))
        .collect();
    stol.extend((0..KNOWN_STREAK).map(|_| answer("стол", Case::Genitive, true)));
    record(stol).await.unwrap();

    const DRAWS: usize = 400;
    let kniga_drawn = async |filter: &GameFilter| {
        let mut n = 0;
        for _ in 0..DRAWS {
            let items = repo.random_items(filter, &zones, 1).await.unwrap();
            n += usize::from(items[0].stressed_lemma == "книга");
        }
        n
    };
    let rarer = GameFilter {
        rarer_known_by: Some(player),
        ..all_alike.clone()
    };
    // Expected 80 (sd 8) against 200 (sd 10).
    let rarely = DRAWS * 32 / 100;
    let evenly = DRAWS * 38 / 100..=DRAWS * 62 / 100;

    let n = kniga_drawn(&rarer).await;
    assert!(n < rarely, "known word drawn {n} times of {DRAWS}");
    let n = kniga_drawn(&all_alike).await;
    assert!(evenly.contains(&n), "with the setting off: {n} of {DRAWS}");

    // One mistake and it's new again.
    record(vec![answer("книга", Case::Instrumental, false)])
        .await
        .unwrap();
    let n = kniga_drawn(&rarer).await;
    assert!(evenly.contains(&n), "after a mistake: {n} of {DRAWS}");
}
