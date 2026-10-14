//! What the use cases need from the outside world. Implemented in `infrastructure`
//! (and by in-memory fakes in tests).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::grammar::Locale;
use domain::user::{DisplayName, Email, Password, User, UserId};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    /// A uniqueness constraint was violated (e.g. the email is already registered).
    #[error("already exists")]
    Conflict,
    #[error(transparent)]
    Other(BoxError),
}

#[derive(Debug, Clone)]
pub struct NewPasswordUser {
    pub email: Email,
    pub display_name: DisplayName,
    pub ui_locale: Locale,
    pub password_hash: String,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Creates the user, its `password` identity and its credentials atomically.
    async fn create_password_user(&self, user: NewPasswordUser) -> Result<User, RepoError>;

    /// The user registered with this email and their password hash, if they have one.
    async fn find_password_login(&self, email: &Email)
    -> Result<Option<(User, String)>, RepoError>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(
        &self,
        token_hash: &[u8],
        user_id: UserId,
        expires_at: DateTime<Utc>,
    ) -> Result<(), RepoError>;

    /// The owner of an unexpired session.
    async fn find_user(
        &self,
        token_hash: &[u8],
        now: DateTime<Utc>,
    ) -> Result<Option<User>, RepoError>;

    async fn delete(&self, token_hash: &[u8]) -> Result<(), RepoError>;
}

#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, password: &Password) -> Result<String, BoxError>;
    async fn verify(&self, password: &Password, hash: &str) -> Result<bool, BoxError>;
}

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub trait TokenGenerator: Send + Sync {
    /// A new unguessable session token (at least 128 bits of randomness).
    fn session_token(&self) -> String;
}
