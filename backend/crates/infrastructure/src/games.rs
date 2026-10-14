//! Postgres implementation of the game ports.

use application::game::{Attempt, AttemptRepository, EndingsFilter, ExerciseRepository, GameItem};
use application::ports::{BoxError, RepoError};
use async_trait::async_trait;
use domain::grammar::Locale;
use domain::user::UserId;
use sqlx::PgPool;

/// Eligible forms for a filter, as the CTE `eligible(lexeme_id, number, ending)`.
/// Binds: $1 pos[], $2 max_rank[], $3 number[] (one row per level × number), $4 case.
const ELIGIBLE: &str = "
    WITH levels AS (
        SELECT * FROM UNNEST($1::text[], $2::int4[], $3::text[]) AS t(pos, max_rank, number)
    ),
    eligible AS (
        SELECT f.lexeme_id, f.number, f.ending
          FROM forms f
          JOIN lexemes l ON l.id = f.lexeme_id
         WHERE f.grammatical_case = $4
           AND f.variant = 0
           AND NOT f.irregular
           AND NOT l.indeclinable
           AND l.lemma = lower(l.lemma)  -- no proper nouns
           AND EXISTS (SELECT 1 FROM levels v
                        WHERE v.pos = l.pos AND v.number = f.number
                          AND l.frequency_rank <= v.max_rank)
    )";

fn other(err: sqlx::Error) -> RepoError {
    RepoError::Other(BoxError::from(err))
}

/// The filter flattened into the parallel arrays bound as $1..$4.
fn filter_binds(
    filter: &EndingsFilter,
) -> (Vec<&'static str>, Vec<i32>, Vec<&'static str>, &'static str) {
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
    )
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
    async fn ending_zones(
        &self,
        filter: &EndingsFilter,
        min_share: f64,
    ) -> Result<Vec<String>, RepoError> {
        let (pos, ranks, numbers, case) = filter_binds(filter);
        sqlx::query_scalar(&format!(
            "{ELIGIBLE}
             SELECT ending FROM eligible
              GROUP BY ending
             HAVING count(*) >= $5 * (SELECT count(*) FROM eligible)"
        ))
        .bind(pos)
        .bind(ranks)
        .bind(numbers)
        .bind(case)
        .bind(min_share)
        .fetch_all(&self.pool)
        .await
        .map_err(other)
    }

    async fn random_ending_items(
        &self,
        filter: &EndingsFilter,
        zones: &[String],
        count: usize,
        locale: Locale,
    ) -> Result<Vec<GameItem>, RepoError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            lexeme_id: i64,
            stressed_lemma: String,
            translation: Option<String>,
            number: String,
            answer: String,
            accepted: Vec<String>,
            stressed_form: String,
        }

        let (pos, ranks, numbers, case) = filter_binds(filter);
        let rows: Vec<Row> = sqlx::query_as(&format!(
            "{ELIGIBLE},
             picked AS (
                 SELECT * FROM eligible WHERE ending = ANY($5) ORDER BY random() LIMIT $6
             )
             SELECT p.lexeme_id,
                    coalesce(l.stressed_lemma, l.lemma) AS stressed_lemma,
                    (SELECT t.text FROM translations t
                      WHERE t.lexeme_id = p.lexeme_id AND t.locale IN ($7, 'en')
                      ORDER BY t.locale = $7 DESC, t.position
                      LIMIT 1) AS translation,
                    p.number,
                    p.ending AS answer,
                    (SELECT array_agg(DISTINCT v.ending) FROM forms v
                      WHERE v.lexeme_id = p.lexeme_id AND v.grammatical_case = $4
                        AND v.number = p.number AND NOT v.irregular) AS accepted,
                    coalesce(f.stressed_form, f.form) AS stressed_form
               FROM picked p
               JOIN lexemes l ON l.id = p.lexeme_id
               JOIN forms f ON f.lexeme_id = p.lexeme_id AND f.grammatical_case = $4
                           AND f.number = p.number AND f.variant = 0"
        ))
        .bind(pos)
        .bind(ranks)
        .bind(numbers)
        .bind(case)
        .bind(zones)
        .bind(i64::try_from(count).unwrap_or(i64::MAX))
        .bind(locale.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(other)?;

        rows.into_iter()
            .map(|r| {
                Ok(GameItem {
                    lexeme_id: r.lexeme_id,
                    stressed_lemma: r.stressed_lemma,
                    translation: r.translation,
                    number: r
                        .number
                        .parse()
                        .map_err(|e| RepoError::Other(Box::new(e)))?,
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
                 (user_id, exercise_type, lexeme_id, grammatical_case, number, answer, correct)
             SELECT $1, $2, * FROM UNNEST($3::int8[], $4::text[], $5::text[], $6::text[], $7::bool[])",
        )
        .bind(user.0)
        .bind(exercise_type)
        .bind(attempts.iter().map(|a| a.lexeme_id).collect::<Vec<_>>())
        .bind(attempts.iter().map(|a| a.case.as_str()).collect::<Vec<_>>())
        .bind(attempts.iter().map(|a| a.number.as_str()).collect::<Vec<_>>())
        .bind(attempts.iter().map(|a| a.answer.clone()).collect::<Vec<_>>())
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
    use application::game::{EndingsFilter, MIN_ZONE_SHARE};
    use application::ports::{NewPasswordUser, UserRepository};
    use domain::game::LevelId;
    use domain::grammar::{Case, Gender, Number, PartOfSpeech};
    use domain::lexicon::{NewForm, NewLexeme, NewTranslation};
    use domain::user::{DisplayName, Email};

    use super::*;
    use crate::lexicon::upsert_lexemes;
    use crate::users::PgUserRepository;

    /// A noun with the given instrumental forms; stress marks optional.
    fn noun(
        lemma: &str,
        rank: i32,
        indeclinable: bool,
        forms: &[(Number, &[&str])],
        translations: &[(Locale, &str)],
    ) -> NewLexeme {
        NewLexeme {
            source: "test".into(),
            source_id: lemma.into(),
            lemma: lemma.into(),
            stressed_lemma: None,
            pos: PartOfSpeech::Noun,
            gender: Some(Gender::Feminine),
            animacy: None,
            indeclinable,
            frequency_rank: Some(rank),
            forms: forms
                .iter()
                .flat_map(|&(number, variants)| {
                    (0..).zip(variants.iter()).map(move |(variant, form)| {
                        NewForm::noun(
                            lemma,
                            form,
                            Case::Instrumental,
                            number,
                            variant,
                            indeclinable,
                        )
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
        }
    }

    async fn seed(pool: &PgPool) {
        use Number::*;
        let words = [
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
        ];
        let mut tx = pool.begin().await.unwrap();
        upsert_lexemes(&mut tx, &words).await.unwrap();
        tx.commit().await.unwrap();
    }

    fn filter(levels: &[LevelId]) -> EndingsFilter {
        EndingsFilter {
            levels: levels.iter().map(|l| l.spec()).collect(),
            case: Case::Instrumental,
        }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn zones_come_from_the_selected_levels(pool: PgPool) {
        seed(&pool).await;
        let repo = PgExerciseRepository::new(pool);

        let mut basic = repo
            .ending_zones(&filter(&[LevelId::BasicNouns]), MIN_ZONE_SHARE)
            .await
            .unwrap();
        basic.sort();
        // Only книга qualifies: отец is irregular, Москва a proper noun, кофе indeclinable,
        // стол too rare, and plurals aren't in the basic level.
        assert_eq!(basic, ["ой"]);

        let mut wider = repo
            .ending_zones(
                &filter(&[LevelId::BasicNouns, LevelId::IntermediateNouns]),
                MIN_ZONE_SHARE,
            )
            .await
            .unwrap();
        wider.sort();
        assert_eq!(wider, ["ами", "ой", "ом"]);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn rare_endings_get_no_zone(pool: PgPool) {
        seed(&pool).await;
        let repo = PgExerciseRepository::new(pool);
        // Intermediate pool: ой, ом, ами, ами → only ами reaches 40%.
        let zones = repo
            .ending_zones(&filter(&[LevelId::IntermediateNouns]), 0.4)
            .await
            .unwrap();
        assert_eq!(zones, ["ами"]);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn items_carry_answers_variants_and_a_fallback_translation(pool: PgPool) {
        seed(&pool).await;
        let repo = PgExerciseRepository::new(pool);

        let items = repo
            .random_ending_items(
                &filter(&[LevelId::BasicNouns]),
                &["ой".into()],
                10,
                Locale::Fr,
            )
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        let kniga = &items[0];
        assert_eq!(kniga.stressed_lemma, "книга"); // no stressed lemma seeded
        assert_eq!(kniga.number, Number::Singular);
        assert_eq!(kniga.answer, "ой");
        let mut accepted = kniga.accepted.clone();
        accepted.sort();
        assert_eq!(accepted, ["ой", "ою"]);
        assert_eq!(kniga.stressed_form, "кни\u{301}гой");
        assert_eq!(kniga.translation.as_deref(), Some("book")); // no French → English

        let items = repo
            .random_ending_items(
                &filter(&[LevelId::IntermediateNouns]),
                &["ом".into()],
                10,
                Locale::Fr,
            )
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].translation.as_deref(), Some("table (fr)"));
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn attempts_are_stored(pool: PgPool) {
        seed(&pool).await;
        let user = PgUserRepository::new(pool.clone())
            .create_password_user(NewPasswordUser {
                email: Email::parse("anna@example.com").unwrap(),
                display_name: DisplayName::parse("Anna").unwrap(),
                ui_locale: Locale::En,
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

        let stored: Vec<(String, bool)> = sqlx::query_as(
            "SELECT answer, correct FROM exercise_attempts
              WHERE user_id = $1 AND exercise_type = 'endings' ORDER BY id",
        )
        .bind(user.id.0)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(stored, [("ой".to_owned(), true), ("ом".to_owned(), false)]);

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
