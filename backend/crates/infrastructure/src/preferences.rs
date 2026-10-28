//! Postgres implementation of the preferences port.

use application::ports::{BoxError, RepoError};
use application::preferences::{PreferenceRepository, Preferences};
use async_trait::async_trait;
use domain::user::UserId;
use sqlx::PgPool;

fn other(err: sqlx::Error) -> RepoError {
    RepoError::Other(BoxError::from(err))
}

#[derive(Clone)]
pub struct PgPreferenceRepository {
    pool: PgPool,
}

impl PgPreferenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PreferenceRepository for PgPreferenceRepository {
    async fn get(&self, user: UserId) -> Result<Preferences, RepoError> {
        let stored: Option<bool> =
            sqlx::query_scalar("SELECT fewer_known_words FROM user_preferences WHERE user_id = $1")
                .bind(user.0)
                .fetch_optional(&self.pool)
                .await
                .map_err(other)?;
        Ok(
            stored.map_or_else(Preferences::default, |fewer_known_words| Preferences {
                fewer_known_words,
            }),
        )
    }

    async fn save(&self, user: UserId, preferences: &Preferences) -> Result<(), RepoError> {
        sqlx::query(
            "INSERT INTO user_preferences (user_id, fewer_known_words) VALUES ($1, $2)
             ON CONFLICT (user_id) DO UPDATE SET fewer_known_words = excluded.fewer_known_words",
        )
        .bind(user.0)
        .bind(preferences.fewer_known_words)
        .execute(&self.pool)
        .await
        .map_err(other)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use application::ports::{NewPasswordUser, UserRepository};
    use domain::grammar::Locale;
    use domain::user::{DisplayName, Email};

    use super::*;
    use crate::users::PgUserRepository;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn preferences_start_as_the_defaults_and_are_kept(pool: PgPool) {
        let user = PgUserRepository::new(pool.clone())
            .create_password_user(NewPasswordUser {
                email: Email::parse("anna@example.com").unwrap(),
                display_name: DisplayName::parse("Anna").unwrap(),
                ui_locale: Locale::En,
                password_hash: "x".into(),
            })
            .await
            .unwrap();
        let repo = PgPreferenceRepository::new(pool);
        assert_eq!(repo.get(user.id).await.unwrap(), Preferences::default());

        for fewer_known_words in [false, true] {
            let preferences = Preferences { fewer_known_words };
            repo.save(user.id, &preferences).await.unwrap();
            assert_eq!(repo.get(user.id).await.unwrap(), preferences);
        }
    }
}
