//! The desktop app's preferences, in the `settings` table of `progress.sqlite`.

use application::ports::{BoxError, RepoError};
use application::preferences::{PreferenceRepository, Preferences};
use async_trait::async_trait;
use domain::user::UserId;
use sqlx::SqlitePool;

const FEWER_KNOWN_WORDS: &str = "fewer_known_words";

fn other(err: sqlx::Error) -> RepoError {
    RepoError::Other(BoxError::from(err))
}

/// Preferences of the desktop app's one local user.
#[derive(Clone)]
pub struct SqlitePreferenceRepository {
    pool: SqlitePool,
}

impl SqlitePreferenceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PreferenceRepository for SqlitePreferenceRepository {
    async fn get(&self, _user: UserId) -> Result<Preferences, RepoError> {
        let stored: Option<String> =
            sqlx::query_scalar("SELECT value FROM settings WHERE key = ?1")
                .bind(FEWER_KNOWN_WORDS)
                .fetch_optional(&self.pool)
                .await
                .map_err(other)?;
        let mut preferences = Preferences::default();
        if let Some(value) = stored {
            preferences.fewer_known_words = value == "true";
        }
        Ok(preferences)
    }

    async fn save(&self, _user: UserId, preferences: &Preferences) -> Result<(), RepoError> {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        )
        .bind(FEWER_KNOWN_WORDS)
        .bind(preferences.fewer_known_words.to_string())
        .execute(&self.pool)
        .await
        .map_err(other)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::contract::nouns;
    use crate::sqlite::lexicon::write_lexicon;

    #[tokio::test]
    async fn preferences_start_as_the_defaults_and_are_kept() {
        let dir = tempfile::tempdir().unwrap();
        let lexicon = dir.path().join("lexicon.sqlite");
        write_lexicon(&lexicon, &nouns()).await.unwrap();
        let pool = crate::sqlite::open(&lexicon, &dir.path().join("progress.sqlite"))
            .await
            .unwrap();
        let repo = SqlitePreferenceRepository::new(pool);
        let me = UserId(Uuid::nil());
        assert_eq!(repo.get(me).await.unwrap(), Preferences::default());

        for fewer_known_words in [false, true] {
            let preferences = Preferences { fewer_known_words };
            repo.save(me, &preferences).await.unwrap();
            assert_eq!(repo.get(me).await.unwrap(), preferences);
        }
    }
}
