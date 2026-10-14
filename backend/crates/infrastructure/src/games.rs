//! Postgres implementation of the game ports.

use application::game::{Attempt, AttemptRepository, ExerciseRepository, GameFilter, GameItem};
use application::ports::{BoxError, RepoError};
use async_trait::async_trait;
use domain::game::AnswerKind;
use domain::grammar::{Animacy, Gender, Locale};
use domain::rules::paradigm;
use domain::user::UserId;
use sqlx::PgPool;

use crate::lexicon::ANY;

/// Askable forms for a filter, as the CTE `eligible(lexeme_id, number, gender,
/// animacy, answer, rank)`. Binds: $1 pos[], $2 max_rank[], $3 number[] (one row
/// per level × number), $4 case, $5 whole forms (bool).
const ELIGIBLE: &str = "
    WITH levels AS (
        SELECT * FROM UNNEST($1::text[], $2::int4[], $3::text[]) AS t(pos, max_rank, number)
    ),
    eligible AS (
        SELECT f.lexeme_id, f.number, f.gender, f.animacy,
               CASE WHEN $5 THEN f.form ELSE f.ending END AS answer,
               l.frequency_rank AS rank
          FROM forms f
          JOIN lexemes l ON l.id = f.lexeme_id
         WHERE f.grammatical_case = $4
           AND f.variant = 0
           AND ($5 OR NOT f.irregular)
           AND NOT l.indeclinable
           -- no place names and the like (surnames are capitalised on purpose)
           AND (l.pos <> 'noun' OR l.lemma = lower(l.lemma))
           -- the dictionary form shown would give the answer away
           AND NOT (f.grammatical_case = 'nominative' AND lower(f.form) = lower(l.lemma))
           AND EXISTS (SELECT 1 FROM levels v
                        WHERE v.pos = l.pos AND v.number = f.number
                          AND l.frequency_rank <= v.max_rank)
    )";

fn other(err: sqlx::Error) -> RepoError {
    RepoError::Other(BoxError::from(err))
}

type FilterBinds = (
    Vec<&'static str>,
    Vec<i32>,
    Vec<&'static str>,
    &'static str,
    bool,
);

/// The filter as the parameters $1..$5 of `ELIGIBLE`.
fn filter_binds(filter: &GameFilter) -> FilterBinds {
    let rows: Vec<_> = filter
        .levels
        .iter()
        .flat_map(|spec| {
            spec.numbers
                .iter()
                .map(move |n| (spec.pos.as_str(), spec.max_rank, n.as_str()))
        })
        .collect();
    (
        rows.iter().map(|r| r.0).collect(),
        rows.iter().map(|r| r.1).collect(),
        rows.iter().map(|r| r.2).collect(),
        filter.case.as_str(),
        filter.answer == AnswerKind::WholeForm,
    )
}

/// `forms.gender` / `forms.animacy` back into the domain; 'any' is `None`.
fn optional<T: std::str::FromStr>(value: &str) -> Result<Option<T>, RepoError>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    if value == ANY {
        Ok(None)
    } else {
        value
            .parse()
            .map(Some)
            .map_err(|e| RepoError::Other(Box::new(e)))
    }
}

#[derive(Clone)]
pub struct PgExerciseRepository {
    pool: PgPool,
}

impl PgExerciseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExerciseRepository for PgExerciseRepository {
    async fn zones(
        &self,
        filter: &GameFilter,
        min_share: f64,
        max_zones: usize,
    ) -> Result<Vec<String>, RepoError> {
        let (pos, ranks, numbers, case, whole) = filter_binds(filter);
        sqlx::query_scalar(&format!(
            "{ELIGIBLE}
             SELECT answer FROM (
                 SELECT answer, min(rank) AS first_rank, count(*) AS uses FROM eligible
                  GROUP BY answer
                 HAVING count(*) >= $6 * (SELECT count(*) FROM eligible)
                  ORDER BY uses DESC, answer
                  LIMIT $7
             ) common
             ORDER BY first_rank, answer"
        ))
        .bind(pos)
        .bind(ranks)
        .bind(numbers)
        .bind(case)
        .bind(whole)
        .bind(min_share)
        .bind(i64::try_from(max_zones).unwrap_or(i64::MAX))
        .fetch_all(&self.pool)
        .await
        .map_err(other)
    }

    async fn random_items(
        &self,
        filter: &GameFilter,
        zones: &[String],
        count: usize,
    ) -> Result<Vec<GameItem>, RepoError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            lexeme_id: i64,
            stressed_lemma: String,
            /// Parallel arrays; NULL when the word has no translations at all.
            translation_locales: Option<Vec<String>>,
            translation_texts: Option<Vec<String>>,
            number: String,
            gender: String,
            animacy: String,
            answer: String,
            accepted: Vec<String>,
            stressed_form: String,
            pos: String,
            lemma: String,
            lexeme_gender: Option<String>,
        }

        let (pos, ranks, numbers, case, whole) = filter_binds(filter);
        let rows: Vec<Row> = sqlx::query_as(&format!(
            "{ELIGIBLE},
             -- one form per word, so a batch never asks the same word twice
             picked AS (
                 SELECT * FROM (
                     SELECT DISTINCT ON (lexeme_id) * FROM eligible
                      WHERE answer = ANY($6)
                      ORDER BY lexeme_id, random()
                 ) one_per_word
                  ORDER BY random() LIMIT $7
             )
             SELECT p.lexeme_id,
                    coalesce(l.stressed_lemma, l.lemma) AS stressed_lemma,
                    tr.translation_locales, tr.translation_texts,
                    p.number, p.gender, p.animacy, p.answer,
                    (SELECT array_agg(DISTINCT CASE WHEN $5 THEN v.form ELSE v.ending END)
                       FROM forms v
                      WHERE v.lexeme_id = p.lexeme_id AND v.grammatical_case = $4
                        AND v.number = p.number AND v.gender = p.gender
                        AND v.animacy = p.animacy AND ($5 OR NOT v.irregular)) AS accepted,
                    coalesce(f.stressed_form, f.form) AS stressed_form,
                    l.pos, l.lemma, l.gender AS lexeme_gender
               FROM picked p
               JOIN lexemes l ON l.id = p.lexeme_id
               JOIN forms f ON f.lexeme_id = p.lexeme_id AND f.grammatical_case = $4
                           AND f.number = p.number AND f.gender = p.gender
                           AND f.animacy = p.animacy AND f.variant = 0
               -- the first three meanings in each language
               CROSS JOIN LATERAL (
                   SELECT array_agg(g.locale) AS translation_locales,
                          array_agg(g.text) AS translation_texts
                     FROM (SELECT t.locale, string_agg(t.text, '; ' ORDER BY t.position) AS text
                             FROM translations t
                            WHERE t.lexeme_id = p.lexeme_id AND t.position < 3
                            GROUP BY t.locale) g
               ) tr"
        ))
        .bind(pos)
        .bind(ranks)
        .bind(numbers)
        .bind(case)
        .bind(whole)
        .bind(zones)
        .bind(i64::try_from(count).unwrap_or(i64::MAX))
        .fetch_all(&self.pool)
        .await
        .map_err(other)?;

        rows.into_iter()
            .map(|r| {
                let lexeme_gender = r
                    .lexeme_gender
                    .as_deref()
                    .map(str::parse::<Gender>)
                    .transpose()
                    .map_err(|e| RepoError::Other(Box::new(e)))?;
                let pos = r.pos.parse().map_err(|e| RepoError::Other(Box::new(e)))?;
                Ok(GameItem {
                    paradigm: paradigm(pos, &r.lemma, lexeme_gender),
                    lexeme_id: r.lexeme_id,
                    stressed_lemma: r.stressed_lemma,
                    translations: r
                        .translation_locales
                        .unwrap_or_default()
                        .iter()
                        .zip(r.translation_texts.unwrap_or_default())
                        .map(|(locale, text)| {
                            Ok((
                                locale
                                    .parse::<Locale>()
                                    .map_err(|e| RepoError::Other(Box::new(e)))?,
                                text,
                            ))
                        })
                        .collect::<Result<_, RepoError>>()?,
                    number: r
                        .number
                        .parse()
                        .map_err(|e| RepoError::Other(Box::new(e)))?,
                    gender: optional::<Gender>(&r.gender)?,
                    animacy: optional::<Animacy>(&r.animacy)?,
                    answer: r.answer,
                    accepted: r.accepted,
                    stressed_form: r.stressed_form,
                })
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct PgAttemptRepository {
    pool: PgPool,
}

impl PgAttemptRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttemptRepository for PgAttemptRepository {
    async fn record(
        &self,
        user: UserId,
        exercise_type: &str,
        attempts: &[Attempt],
    ) -> Result<(), RepoError> {
        sqlx::query(
            "INSERT INTO exercise_attempts
                 (user_id, exercise_type, lexeme_id, grammatical_case, number, gender, animacy,
                  answer, correct)
             SELECT $1, $2, * FROM UNNEST($3::int8[], $4::text[], $5::text[], $6::text[],
                                          $7::text[], $8::text[], $9::bool[])",
        )
        .bind(user.0)
        .bind(exercise_type)
        .bind(attempts.iter().map(|a| a.lexeme_id).collect::<Vec<_>>())
        .bind(attempts.iter().map(|a| a.case.as_str()).collect::<Vec<_>>())
        .bind(
            attempts
                .iter()
                .map(|a| a.number.as_str())
                .collect::<Vec<_>>(),
        )
        .bind(
            attempts
                .iter()
                .map(|a| a.gender.map(|g| g.as_str()))
                .collect::<Vec<_>>(),
        )
        .bind(
            attempts
                .iter()
                .map(|a| a.animacy.map(|g| g.as_str()))
                .collect::<Vec<_>>(),
        )
        .bind(
            attempts
                .iter()
                .map(|a| a.answer.clone())
                .collect::<Vec<_>>(),
        )
        .bind(attempts.iter().map(|a| a.correct).collect::<Vec<_>>())
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db) if db.is_foreign_key_violation() => {
                RepoError::InvalidReference
            }
            _ => other(e),
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use application::game::{MAX_ZONES, MIN_ZONE_SHARE};
    use application::ports::{NewPasswordUser, UserRepository};
    use domain::game::LevelId;
    use domain::grammar::{Case, Number, PartOfSpeech};
    use domain::lexicon::{NewForm, NewLexeme, NewTranslation, Slot};
    use domain::user::{DisplayName, Email};

    use super::*;
    use crate::lexicon::upsert_lexemes;
    use crate::users::PgUserRepository;

    fn lexeme(lemma: &str, pos: PartOfSpeech, rank: i32, indeclinable: bool) -> NewLexeme {
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
    fn noun(
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

    async fn seed(pool: &PgPool, words: &[NewLexeme]) {
        let mut tx = pool.begin().await.unwrap();
        upsert_lexemes(&mut tx, words).await.unwrap();
        tx.commit().await.unwrap();
    }

    async fn seed_nouns(pool: &PgPool) {
        use Number::*;
        seed(
            pool,
            &[
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
            ],
        )
        .await;
    }

    fn filter(levels: &[LevelId], case: Case) -> GameFilter {
        let levels: Vec<_> = levels.iter().map(|l| l.spec()).collect();
        GameFilter {
            answer: levels[0].answer,
            levels,
            case,
        }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn zones_come_from_the_selected_levels(pool: PgPool) {
        seed_nouns(&pool).await;
        let repo = PgExerciseRepository::new(pool);
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

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn zones_are_capped_to_the_most_common(pool: PgPool) {
        seed_nouns(&pool).await;
        let repo = PgExerciseRepository::new(pool);
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

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn rare_endings_get_no_zone(pool: PgPool) {
        seed_nouns(&pool).await;
        let repo = PgExerciseRepository::new(pool);
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

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn items_carry_answers_variants_and_translations(pool: PgPool) {
        seed_nouns(&pool).await;
        let repo = PgExerciseRepository::new(pool);

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

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn items_show_up_to_three_meanings_per_language(pool: PgPool) {
        use Number::*;
        let meanings = [
            (Locale::En, "hand"),
            (Locale::En, "arm"),
            (Locale::En, "handwriting"),
            (Locale::En, "side"),
        ];
        // noun() numbers translations 0.. across locales, as one list here.
        let ruka = noun("рука", 1, false, &[(Singular, &["руко́й"])], &meanings);
        seed(&pool, &[ruka]).await;

        let items = PgExerciseRepository::new(pool)
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

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn adjective_items_say_which_form_is_asked(pool: PgPool) {
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
        seed(&pool, &[novy]).await;
        let repo = PgExerciseRepository::new(pool);
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

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn a_batch_asks_each_word_once(pool: PgPool) {
        let slot = |gender| Slot::new(Case::Genitive, Number::Singular).gender(gender);
        let novy = NewLexeme {
            forms: vec![
                NewForm::adjective("новый", "но\u{301}вого", slot(Gender::Masculine), 0),
                NewForm::adjective("новый", "но\u{301}вого", slot(Gender::Neuter), 0),
                NewForm::adjective("новый", "но\u{301}вой", slot(Gender::Feminine), 0),
            ],
            ..lexeme("новый", PartOfSpeech::Adjective, 1, false)
        };
        seed(&pool, &[novy]).await;

        let items = PgExerciseRepository::new(pool)
            .random_items(
                &filter(&[LevelId::BasicAdjectives], Case::Genitive),
                &["ого".into(), "ой".into()],
                10,
            )
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn the_dictionary_form_is_never_asked(pool: PgPool) {
        let nom = |number| Slot::new(Case::Nominative, number);
        let kniga = NewLexeme {
            forms: vec![
                NewForm::noun("книга", "кни\u{301}га", nom(Number::Singular), 0, false),
                NewForm::noun("книга", "кни\u{301}ги", nom(Number::Plural), 0, false),
            ],
            ..lexeme("книга", PartOfSpeech::Noun, 1, false)
        };
        seed(&pool, &[kniga]).await;
        let zones = PgExerciseRepository::new(pool)
            .zones(
                &filter(&[LevelId::IntermediateNouns], Case::Nominative),
                MIN_ZONE_SHARE,
                MAX_ZONES,
            )
            .await
            .unwrap();
        assert_eq!(zones, ["и"]);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn personal_pronouns_are_asked_as_whole_forms(pool: PgPool) {
        let genitive = Slot::new(Case::Genitive, Number::Singular);
        let pronoun = |lemma: &str, rank, forms: &[&str]| NewLexeme {
            forms: (0..)
                .zip(forms)
                .map(|(variant, form)| NewForm::whole(form, genitive, variant))
                .collect(),
            ..lexeme(lemma, PartOfSpeech::PersonalPronoun, rank, false)
        };
        seed(
            &pool,
            &[
                pronoun("я", 1, &["меня\u{301}"]),
                pronoun("он", 3, &["его\u{301}", "него\u{301}"]),
                pronoun("ты", 2, &["тебя\u{301}"]),
            ],
        )
        .await;
        let repo = PgExerciseRepository::new(pool);
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

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn attempts_are_stored(pool: PgPool) {
        seed_nouns(&pool).await;
        let user = PgUserRepository::new(pool.clone())
            .create_password_user(NewPasswordUser {
                email: Email::parse("anna@example.com").unwrap(),
                display_name: DisplayName::parse("Anna").unwrap(),
                ui_locale: Locale::De,
                password_hash: "x".into(),
            })
            .await
            .unwrap();
        let lexeme_id: i64 = sqlx::query_scalar("SELECT id FROM lexemes WHERE lemma = 'книга'")
            .fetch_one(&pool)
            .await
            .unwrap();
        let attempt = |answer: &str, correct| Attempt {
            lexeme_id,
            case: Case::Instrumental,
            number: Number::Singular,
            gender: Some(Gender::Feminine),
            animacy: None,
            answer: answer.into(),
            correct,
        };

        PgAttemptRepository::new(pool.clone())
            .record(
                user.id,
                "endings",
                &[attempt("ой", true), attempt("ом", false)],
            )
            .await
            .unwrap();

        let stored: Vec<(String, bool, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT answer, correct, gender, animacy FROM exercise_attempts
              WHERE user_id = $1 AND exercise_type = 'endings' ORDER BY id",
        )
        .bind(user.id.0)
        .fetch_all(&pool)
        .await
        .unwrap();
        let feminine = || Some("feminine".to_owned());
        assert_eq!(
            stored,
            [
                ("ой".to_owned(), true, feminine(), None),
                ("ом".to_owned(), false, feminine(), None)
            ]
        );

        let unknown_word = Attempt {
            lexeme_id: -1,
            ..attempt("ой", true)
        };
        let err = PgAttemptRepository::new(pool)
            .record(user.id, "endings", &[unknown_word])
            .await
            .unwrap_err();
        assert!(matches!(err, RepoError::InvalidReference));
    }
}
