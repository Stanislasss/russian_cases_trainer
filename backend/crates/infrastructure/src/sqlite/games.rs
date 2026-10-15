//! SQLite implementation of the game ports. The queries mirror the Postgres ones
//! in `crate::games`; arrays travel as JSON.

use application::game::{Attempt, AttemptRepository, ExerciseRepository, GameFilter, GameItem};
use application::ports::{BoxError, RepoError};
use async_trait::async_trait;
use domain::game::AnswerKind;
use domain::user::UserId;
use sqlx::SqlitePool;

use crate::games::{ItemRow, level_rows};

/// Askable forms for a filter, as the CTE `eligible(lexeme_id, number, gender,
/// animacy, answer, rank)`. Binds: ?1 levels as JSON [[pos, max_rank, number], …],
/// ?2 case, ?3 whole forms (bool).
const ELIGIBLE: &str = "
    WITH levels AS (
        SELECT value ->> 0 AS pos, value ->> 1 AS max_rank, value ->> 2 AS number
          FROM json_each(?1)
    ),
    eligible AS (
        SELECT f.lexeme_id, f.number, f.gender, f.animacy,
               CASE WHEN ?3 THEN f.form ELSE f.ending END AS answer,
               l.frequency_rank AS rank
          FROM forms f
          JOIN lexemes l ON l.id = f.lexeme_id
         WHERE f.grammatical_case = ?2
           AND f.variant = 0
           AND (?3 OR NOT f.irregular)
           AND NOT l.indeclinable
           -- no place names and the like (surnames are capitalised on purpose)
           AND (l.pos <> 'noun' OR NOT l.proper)
           -- the dictionary form shown would give the answer away
           AND NOT (f.grammatical_case = 'nominative' AND f.is_lemma)
           AND EXISTS (SELECT 1 FROM levels v
                        WHERE v.pos = l.pos AND v.number = f.number
                          AND l.frequency_rank <= v.max_rank)
    )";

fn other(err: impl Into<BoxError>) -> RepoError {
    RepoError::Other(err.into())
}

/// The filter as the parameters ?1..?3 of `ELIGIBLE`.
fn filter_binds(filter: &GameFilter) -> (String, &'static str, bool) {
    (
        serde_json::to_string(&level_rows(filter)).expect("plain data"),
        filter.case.as_str(),
        filter.answer == AnswerKind::WholeForm,
    )
}

#[derive(Clone)]
pub struct SqliteExerciseRepository {
    pool: SqlitePool,
}

impl SqliteExerciseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ExerciseRepository for SqliteExerciseRepository {
    async fn zones(
        &self,
        filter: &GameFilter,
        min_share: f64,
        max_zones: usize,
    ) -> Result<Vec<String>, RepoError> {
        let (levels, case, whole) = filter_binds(filter);
        sqlx::query_scalar(&format!(
            "{ELIGIBLE}
             SELECT answer FROM (
                 SELECT answer, min(rank) AS first_rank, count(*) AS uses FROM eligible
                  GROUP BY answer
                 HAVING count(*) >= ?4 * (SELECT count(*) FROM eligible)
                  ORDER BY uses DESC, answer
                  LIMIT ?5
             ) common
             ORDER BY first_rank, answer"
        ))
        .bind(levels)
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
            /// JSON [[locale, meanings], …]
            translations: String,
            number: String,
            gender: String,
            animacy: String,
            answer: String,
            /// JSON [answer, …]
            accepted: String,
            stressed_form: String,
            pos: String,
            lemma: String,
            lexeme_gender: Option<String>,
        }

        let (levels, case, whole) = filter_binds(filter);
        let rows: Vec<Row> = sqlx::query_as(&format!(
            "{ELIGIBLE},
             -- one form per word, so a batch never asks the same word twice
             picked AS (
                 SELECT * FROM (
                     SELECT *, row_number() OVER (PARTITION BY lexeme_id ORDER BY random()) AS nth
                       FROM eligible
                      WHERE answer IN (SELECT value FROM json_each(?4))
                 ) one_per_word
                  WHERE nth = 1
                  ORDER BY random() LIMIT ?5
             )
             SELECT p.lexeme_id,
                    coalesce(l.stressed_lemma, l.lemma) AS stressed_lemma,
                    -- the first meanings in each language
                    (SELECT json_group_array(json_array(g.locale, g.text))
                       FROM (SELECT t.locale, group_concat(t.text, '; ' ORDER BY t.position) AS text
                               FROM translations t
                              WHERE t.lexeme_id = p.lexeme_id
                              GROUP BY t.locale) g) AS translations,
                    p.number, p.gender, p.animacy, p.answer,
                    (SELECT json_group_array(DISTINCT CASE WHEN ?3 THEN v.form ELSE v.ending END)
                       FROM forms v
                      WHERE v.lexeme_id = p.lexeme_id AND v.grammatical_case = ?2
                        AND v.number = p.number AND v.gender = p.gender
                        AND v.animacy = p.animacy AND (?3 OR NOT v.irregular)) AS accepted,
                    coalesce(f.stressed_form, f.form) AS stressed_form,
                    l.pos, l.lemma, l.gender AS lexeme_gender
               FROM picked p
               JOIN lexemes l ON l.id = p.lexeme_id
               JOIN forms f ON f.lexeme_id = p.lexeme_id AND f.grammatical_case = ?2
                           AND f.number = p.number AND f.gender = p.gender
                           AND f.animacy = p.animacy AND f.variant = 0"
        ))
        .bind(levels)
        .bind(case)
        .bind(whole)
        .bind(serde_json::to_string(zones).expect("plain data"))
        .bind(i64::try_from(count).unwrap_or(i64::MAX))
        .fetch_all(&self.pool)
        .await
        .map_err(other)?;

        rows.into_iter()
            .map(|r| {
                ItemRow {
                    lexeme_id: r.lexeme_id,
                    stressed_lemma: r.stressed_lemma,
                    translations: serde_json::from_str(&r.translations).map_err(other)?,
                    number: r.number,
                    gender: r.gender,
                    animacy: r.animacy,
                    answer: r.answer,
                    accepted: serde_json::from_str(&r.accepted).map_err(other)?,
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

/// Attempts of the desktop app's one local user.
#[derive(Clone)]
pub struct SqliteAttemptRepository {
    pool: SqlitePool,
}

impl SqliteAttemptRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AttemptRepository for SqliteAttemptRepository {
    async fn record(
        &self,
        _user: UserId,
        exercise_type: &str,
        attempts: &[Attempt],
    ) -> Result<(), RepoError> {
        let mut tx = self.pool.begin().await.map_err(other)?;
        for a in attempts {
            // No foreign key into the attached word list: check by hand.
            let inserted = sqlx::query(
                "INSERT INTO exercise_attempts
                     (exercise_type, lexeme_id, grammatical_case, number, gender, animacy,
                      answer, correct)
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8
                  WHERE EXISTS (SELECT 1 FROM lexemes WHERE id = ?2)",
            )
            .bind(exercise_type)
            .bind(a.lexeme_id)
            .bind(a.case.as_str())
            .bind(a.number.as_str())
            .bind(a.gender.map(|g| g.as_str()))
            .bind(a.animacy.map(|g| g.as_str()))
            .bind(&a.answer)
            .bind(a.correct)
            .execute(&mut *tx)
            .await
            .map_err(other)?;
            if inserted.rows_affected() == 0 {
                return Err(RepoError::InvalidReference); // rolls back
            }
        }
        tx.commit().await.map_err(other)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use domain::grammar::{Case, Gender, Number};
    use domain::lexicon::NewLexeme;
    use tempfile::TempDir;

    use super::*;
    use crate::contract::{Seed, contract_tests, nouns};
    use crate::sqlite::lexicon::{lexeme_id, write_lexicon};

    /// A word list and progress database in a temporary folder.
    async fn database(words: &[NewLexeme]) -> (SqlitePool, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let lexicon = dir.path().join("lexicon.sqlite");
        write_lexicon(&lexicon, words).await.unwrap();
        let pool = crate::sqlite::open(&lexicon, &dir.path().join("progress.sqlite"))
            .await
            .unwrap();
        (pool, dir)
    }

    /// Keeps the temporary folder for as long as the repository is used.
    struct Held(SqliteExerciseRepository, #[allow(dead_code)] TempDir);

    #[async_trait]
    impl ExerciseRepository for Held {
        async fn zones(
            &self,
            filter: &GameFilter,
            min_share: f64,
            max_zones: usize,
        ) -> Result<Vec<String>, RepoError> {
            self.0.zones(filter, min_share, max_zones).await
        }

        async fn random_items(
            &self,
            filter: &GameFilter,
            zones: &[String],
            count: usize,
        ) -> Result<Vec<GameItem>, RepoError> {
            self.0.random_items(filter, zones, count).await
        }
    }

    struct Sqlite;

    impl Seed for Sqlite {
        async fn seed(self, words: &[NewLexeme]) -> Box<dyn ExerciseRepository> {
            let (pool, dir) = database(words).await;
            Box::new(Held(SqliteExerciseRepository::new(pool), dir))
        }
    }

    contract_tests!(tokio::test, () => Sqlite);

    #[tokio::test]
    async fn only_askable_words_are_shipped() {
        let mut rare = nouns().remove(0);
        rare.source_id = "rare".into();
        rare.frequency_rank = Some(1_000_000);
        let (pool, _dir) = database(&[nouns().remove(0), rare]).await;
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM lexemes")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn the_word_list_is_read_only() {
        let (pool, _dir) = database(&nouns()).await;
        let err = sqlx::query("DELETE FROM lexemes").execute(&pool).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn attempts_are_stored() {
        let (pool, _dir) = database(&nouns()).await;
        let attempt = |answer: &str, correct| Attempt {
            lexeme_id: lexeme_id("test", "книга"),
            case: Case::Instrumental,
            number: Number::Singular,
            gender: Some(Gender::Feminine),
            animacy: None,
            answer: answer.into(),
            correct,
        };
        let repo = SqliteAttemptRepository::new(pool.clone());
        let user = UserId(uuid::Uuid::nil());

        repo.record(
            user,
            "endings",
            &[attempt("ой", true), attempt("ом", false)],
        )
        .await
        .unwrap();

        let stored: Vec<(String, bool, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT answer, correct, gender, animacy FROM exercise_attempts
              WHERE exercise_type = 'endings' ORDER BY id",
        )
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
        let err = repo
            .record(user, "endings", &[attempt("ой", true), unknown_word])
            .await
            .unwrap_err();
        assert!(matches!(err, RepoError::InvalidReference));
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM exercise_attempts")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(
            count, 2,
            "a batch with an unknown word is not stored at all"
        );
    }

    #[tokio::test]
    async fn progress_survives_reopening() {
        let (pool, dir) = database(&nouns()).await;
        sqlx::query("INSERT INTO settings (key, value) VALUES ('k', 'v')")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
        let lexicon: &Path = &dir.path().join("lexicon.sqlite");
        let pool = crate::sqlite::open(lexicon, &dir.path().join("progress.sqlite"))
            .await
            .unwrap();
        let value: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'k'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(value, "v");
    }
}
