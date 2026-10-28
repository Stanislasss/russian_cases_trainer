//! Postgres implementation of the game ports.

use application::game::{
    Attempt, AttemptRepository, ExerciseRepository, GameFilter, GameItem, KNOWN_STREAK,
};
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
/// per level × number), $4 case, $5 whole forms (bool), $8 lexeme animacy (text or
/// NULL for any; $8 so that $6 and $7 stay free for the queries using it).
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
           AND ($8::text IS NULL OR l.animacy = $8)
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

/// The filter's levels as (part of speech, max rank, number) rows, one per level
/// and number.
pub(crate) fn level_rows(filter: &GameFilter) -> Vec<(&'static str, i32, &'static str)> {
    filter
        .levels
        .iter()
        .flat_map(|spec| {
            spec.numbers
                .iter()
                .map(move |n| (spec.pos.as_str(), spec.max_rank, n.as_str()))
        })
        .collect()
}

/// The filter as the parameters $1..$5 of `ELIGIBLE`.
fn filter_binds(filter: &GameFilter) -> FilterBinds {
    let rows = level_rows(filter);
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
        value.parse().map(Some).map_err(invalid)
    }
}

fn invalid(err: impl std::error::Error + Send + Sync + 'static) -> RepoError {
    RepoError::Other(Box::new(err))
}

/// One picked word as stored, the same for Postgres and SQLite.
pub(crate) struct ItemRow {
    pub lexeme_id: i64,
    pub stressed_lemma: String,
    /// (locale, the first meanings joined by "; ")
    pub translations: Vec<(String, String)>,
    pub number: String,
    pub gender: String,
    pub animacy: String,
    pub answer: String,
    pub accepted: Vec<String>,
    pub stressed_form: String,
    pub pos: String,
    pub lemma: String,
    pub lexeme_gender: Option<String>,
}

impl ItemRow {
    pub(crate) fn into_item(self) -> Result<GameItem, RepoError> {
        let lexeme_gender = self
            .lexeme_gender
            .as_deref()
            .map(str::parse::<Gender>)
            .transpose()
            .map_err(invalid)?;
        let pos = self.pos.parse().map_err(invalid)?;
        Ok(GameItem {
            paradigm: paradigm(pos, &self.lemma, lexeme_gender),
            lexeme_id: self.lexeme_id,
            stressed_lemma: self.stressed_lemma,
            translations: self
                .translations
                .into_iter()
                .map(|(locale, text)| Ok((locale.parse::<Locale>().map_err(invalid)?, text)))
                .collect::<Result<_, RepoError>>()?,
            number: self.number.parse().map_err(invalid)?,
            gender: optional::<Gender>(&self.gender)?,
            animacy: optional::<Animacy>(&self.animacy)?,
            answer: self.answer,
            accepted: self.accepted,
            stressed_form: self.stressed_form,
        })
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
        .bind(filter.animacy.map(|a| a.as_str()))
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
             -- words whose last $10 answers in this case by player $9 were right
             known AS (
                 SELECT lexeme_id FROM (
                     SELECT lexeme_id, correct,
                            row_number() OVER (PARTITION BY lexeme_id ORDER BY id DESC) AS recent
                       FROM exercise_attempts
                      WHERE user_id = $9 AND grammatical_case = $4
                 ) latest
                  WHERE recent <= $10
                  GROUP BY lexeme_id
                 HAVING count(*) = $10 AND bool_and(correct)
             ),
             -- one form per word, so a batch never asks the same word twice; then
             -- weighted sampling: the highest draw^(1/weight) win, known words weigh 1/4
             picked AS (
                 SELECT * FROM (
                     SELECT DISTINCT ON (lexeme_id) *, random() AS draw FROM eligible
                      WHERE answer = ANY($6)
                      ORDER BY lexeme_id, random()
                 ) one_per_word
                  ORDER BY CASE WHEN lexeme_id IN (SELECT lexeme_id FROM known)
                                THEN power(draw, 4) ELSE draw END DESC
                  LIMIT $7
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
        .bind(filter.animacy.map(|a| a.as_str()))
        .bind(filter.rarer_known_by.map(|u| u.0))
        .bind(i64::try_from(KNOWN_STREAK).expect("small"))
        .fetch_all(&self.pool)
        .await
        .map_err(other)?;

        rows.into_iter()
            .map(|r| {
                ItemRow {
                    lexeme_id: r.lexeme_id,
                    stressed_lemma: r.stressed_lemma,
                    translations: r
                        .translation_locales
                        .unwrap_or_default()
                        .into_iter()
                        .zip(r.translation_texts.unwrap_or_default())
                        .collect(),
                    number: r.number,
                    gender: r.gender,
                    animacy: r.animacy,
                    answer: r.answer,
                    accepted: r.accepted,
                    stressed_form: r.stressed_form,
                    pos: r.pos,
                    lemma: r.lemma,
                    lexeme_gender: r.lexeme_gender,
                }
                .into_item()
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
    use application::ports::{NewPasswordUser, UserRepository};
    use domain::grammar::{Case, Number};
    use domain::lexicon::NewLexeme;
    use domain::user::{DisplayName, Email};

    use super::*;
    use crate::contract::{Seed, contract_tests, nouns};
    use crate::lexicon::upsert_lexemes;
    use crate::users::PgUserRepository;

    async fn seed(pool: &PgPool, words: &[NewLexeme]) {
        let mut tx = pool.begin().await.unwrap();
        upsert_lexemes(&mut tx, words).await.unwrap();
        tx.commit().await.unwrap();
    }

    struct Pg(PgPool);

    impl Seed for Pg {
        async fn seed(self, words: &[NewLexeme]) -> Box<dyn ExerciseRepository> {
            seed(&self.0, words).await;
            Box::new(PgExerciseRepository::new(self.0))
        }

        async fn seed_with_player(
            self,
            words: &[NewLexeme],
        ) -> (
            Box<dyn ExerciseRepository>,
            Box<dyn AttemptRepository>,
            UserId,
        ) {
            seed(&self.0, words).await;
            let player = anna(&self.0).await;
            (
                Box::new(PgExerciseRepository::new(self.0.clone())),
                Box::new(PgAttemptRepository::new(self.0)),
                player,
            )
        }
    }

    async fn anna(pool: &PgPool) -> UserId {
        PgUserRepository::new(pool.clone())
            .create_password_user(NewPasswordUser {
                email: Email::parse("anna@example.com").unwrap(),
                display_name: DisplayName::parse("Anna").unwrap(),
                ui_locale: Locale::De,
                password_hash: "x".into(),
            })
            .await
            .unwrap()
            .id
    }

    contract_tests!(sqlx::test(migrator = "crate::MIGRATOR"), (pool: PgPool) => Pg(pool));

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn attempts_are_stored(pool: PgPool) {
        seed(&pool, &nouns()).await;
        let user = anna(&pool).await;
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
                user,
                "endings",
                &[attempt("ой", true), attempt("ом", false)],
            )
            .await
            .unwrap();

        let stored: Vec<(String, bool, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT answer, correct, gender, animacy FROM exercise_attempts
              WHERE user_id = $1 AND exercise_type = 'endings' ORDER BY id",
        )
        .bind(user.0)
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
            .record(user, "endings", &[unknown_word])
            .await
            .unwrap_err();
        assert!(matches!(err, RepoError::InvalidReference));
    }
}
