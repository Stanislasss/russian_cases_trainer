//! Postgres implementations of the account ports.

use application::ports::{BoxError, NewPasswordUser, RepoError, SessionRepository, UserRepository};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::user::{DisplayName, Email, User, UserId};
use sqlx::PgPool;
use uuid::Uuid;

const USER_COLUMNS: &str = "u.id, u.email, u.display_name, u.ui_locale, u.created_at";

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    display_name: String,
    ui_locale: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = RepoError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        let corrupt = |e: &dyn std::fmt::Display| {
            RepoError::Other(format!("invalid user row {}: {e}", row.id).into())
        };
        Ok(User {
            id: UserId(row.id),
            email: Email::parse(&row.email).map_err(|e| corrupt(&e))?,
            display_name: DisplayName::parse(&row.display_name).map_err(|e| corrupt(&e))?,
            ui_locale: row.ui_locale.parse().map_err(|e| corrupt(&e))?,
            created_at: row.created_at,
        })
    }
}

fn db_error(err: sqlx::Error) -> RepoError {
    match &err {
        sqlx::Error::Database(db) if db.is_unique_violation() => RepoError::Conflict,
        _ => RepoError::Other(BoxError::from(err)),
    }
}

#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create_password_user(&self, new: NewPasswordUser) -> Result<User, RepoError> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;

        let row: UserRow = sqlx::query_as(&format!(
            "INSERT INTO users AS u (email, display_name, ui_locale) VALUES ($1, $2, $3)
             RETURNING {USER_COLUMNS}"
        ))
        .bind(new.email.as_str())
        .bind(new.display_name.as_str())
        .bind(new.ui_locale.as_str())
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;

        sqlx::query(
            "INSERT INTO user_identities (provider, provider_subject, user_id)
             VALUES ('password', $1, $2)",
        )
        .bind(row.id.to_string())
        .bind(row.id)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;

        sqlx::query("INSERT INTO password_credentials (user_id, password_hash) VALUES ($1, $2)")
            .bind(row.id)
            .bind(&new.password_hash)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;

        tx.commit().await.map_err(db_error)?;
        row.try_into()
    }

    async fn find_password_login(
        &self,
        email: &Email,
    ) -> Result<Option<(User, String)>, RepoError> {
        #[derive(sqlx::FromRow)]
        struct Row {
            #[sqlx(flatten)]
            user: UserRow,
            password_hash: String,
        }

        let row: Option<Row> = sqlx::query_as(&format!(
            "SELECT {USER_COLUMNS}, pc.password_hash
               FROM users u JOIN password_credentials pc ON pc.user_id = u.id
              WHERE u.email = $1"
        ))
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;

        row.map(|r| Ok((r.user.try_into()?, r.password_hash)))
            .transpose()
    }
}

#[derive(Clone)]
pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn create(
        &self,
        token_hash: &[u8],
        user_id: UserId,
        expires_at: DateTime<Utc>,
    ) -> Result<(), RepoError> {
        sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at) VALUES ($1, $2, $3)")
            .bind(token_hash)
            .bind(user_id.0)
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .map_err(db_error)?;
        Ok(())
    }

    async fn find_user(
        &self,
        token_hash: &[u8],
        now: DateTime<Utc>,
    ) -> Result<Option<User>, RepoError> {
        let row: Option<UserRow> = sqlx::query_as(&format!(
            "SELECT {USER_COLUMNS}
               FROM sessions s JOIN users u ON u.id = s.user_id
              WHERE s.token_hash = $1 AND s.expires_at > $2"
        ))
        .bind(token_hash)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;
        row.map(User::try_from).transpose()
    }

    async fn delete(&self, token_hash: &[u8]) -> Result<(), RepoError> {
        sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(db_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use domain::grammar::Locale;

    use super::*;

    fn new_user(email: &str) -> NewPasswordUser {
        NewPasswordUser {
            email: Email::parse(email).unwrap(),
            display_name: DisplayName::parse("Anna").unwrap(),
            ui_locale: Locale::Ru,
            password_hash: "$argon2id$fake".into(),
        }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn creates_and_finds_password_users(pool: PgPool) {
        let repo = PgUserRepository::new(pool.clone());
        let user = repo
            .create_password_user(new_user("anna@example.com"))
            .await
            .unwrap();

        let (found, hash) = repo
            .find_password_login(&Email::parse("anna@example.com").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found, user);
        assert_eq!(found.ui_locale, Locale::Ru);
        assert_eq!(hash, "$argon2id$fake");

        let identities: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM user_identities WHERE provider = 'password' AND user_id = $1",
        )
        .bind(user.id.0)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(identities, 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn duplicate_email_is_a_conflict(pool: PgPool) {
        let repo = PgUserRepository::new(pool.clone());
        repo.create_password_user(new_user("anna@example.com"))
            .await
            .unwrap();

        let err = repo
            .create_password_user(new_user("anna@example.com"))
            .await
            .unwrap_err();
        assert!(matches!(err, RepoError::Conflict));
        // The failed registration left nothing behind.
        let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(users, 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn sessions_find_their_user_until_they_expire(pool: PgPool) {
        let user = PgUserRepository::new(pool.clone())
            .create_password_user(new_user("anna@example.com"))
            .await
            .unwrap();
        let sessions = PgSessionRepository::new(pool);
        let now = Utc::now();
        sessions
            .create(b"hash", user.id, now + TimeDelta::hours(1))
            .await
            .unwrap();

        let found = sessions.find_user(b"hash", now).await.unwrap();
        assert_eq!(found.map(|u| u.id), Some(user.id));
        assert!(sessions.find_user(b"other", now).await.unwrap().is_none());
        assert!(
            sessions
                .find_user(b"hash", now + TimeDelta::hours(2))
                .await
                .unwrap()
                .is_none()
        );

        sessions.delete(b"hash").await.unwrap();
        assert!(sessions.find_user(b"hash", now).await.unwrap().is_none());
    }
}
