//! Registration, login, logout and session lookup.

use std::sync::Arc;

use chrono::{DateTime, TimeDelta, Utc};
use domain::grammar::Locale;
use domain::user::{DisplayName, Email, Password, User, ValidationError};
use sha2::{Digest, Sha256};

use crate::ports::{
    BoxError, Clock, NewPasswordUser, PasswordHasher, RepoError, SessionRepository, TokenGenerator,
    UserRepository,
};

pub const SESSION_TTL: TimeDelta = TimeDelta::days(30);

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("this email is already registered")]
    EmailTaken,
    /// Wrong email or wrong password; deliberately not saying which.
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error(transparent)]
    Internal(BoxError),
}

impl From<RepoError> for AuthError {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::Conflict => AuthError::EmailTaken,
            e @ RepoError::InvalidReference => AuthError::Internal(e.into()),
            RepoError::Other(e) => AuthError::Internal(e),
        }
    }
}

/// A freshly created session. `token` goes into the cookie; only its hash is stored.
#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub ui_locale: Locale,
}

#[derive(Clone)]
pub struct AuthService {
    users: Arc<dyn UserRepository>,
    sessions: Arc<dyn SessionRepository>,
    hasher: Arc<dyn PasswordHasher>,
    clock: Arc<dyn Clock>,
    tokens: Arc<dyn TokenGenerator>,
}

impl AuthService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        sessions: Arc<dyn SessionRepository>,
        hasher: Arc<dyn PasswordHasher>,
        clock: Arc<dyn Clock>,
        tokens: Arc<dyn TokenGenerator>,
    ) -> Self {
        Self {
            users,
            sessions,
            hasher,
            clock,
            tokens,
        }
    }

    /// Creates an account and logs it in.
    pub async fn register(&self, input: RegisterInput) -> Result<(User, Session), AuthError> {
        let email = Email::parse(&input.email)?;
        let password = Password::parse(&input.password)?;
        let display_name = DisplayName::parse(&input.display_name)?;

        let password_hash = self
            .hasher
            .hash(&password)
            .await
            .map_err(AuthError::Internal)?;
        let user = self
            .users
            .create_password_user(NewPasswordUser {
                email,
                display_name,
                ui_locale: input.ui_locale,
                password_hash,
            })
            .await?;
        let session = self.start_session(&user).await?;
        Ok((user, session))
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(User, Session), AuthError> {
        let (Ok(email), Ok(password)) = (Email::parse(email), Password::parse(password)) else {
            return Err(AuthError::InvalidCredentials);
        };

        let Some((user, hash)) = self.users.find_password_login(&email).await? else {
            // Spend the same time as a real check so response times don't reveal
            // which emails are registered.
            let _ = self.hasher.hash(&password).await;
            return Err(AuthError::InvalidCredentials);
        };
        let matches = self
            .hasher
            .verify(&password, &hash)
            .await
            .map_err(AuthError::Internal)?;
        if !matches {
            return Err(AuthError::InvalidCredentials);
        }

        let session = self.start_session(&user).await?;
        Ok((user, session))
    }

    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        Ok(self.sessions.delete(&hash_token(token)).await?)
    }

    /// The user owning this session token, if the session exists and hasn't expired.
    pub async fn current_user(&self, token: &str) -> Result<Option<User>, AuthError> {
        Ok(self
            .sessions
            .find_user(&hash_token(token), self.clock.now())
            .await?)
    }

    async fn start_session(&self, user: &User) -> Result<Session, AuthError> {
        let token = self.tokens.session_token();
        let expires_at = self.clock.now() + SESSION_TTL;
        self.sessions
            .create(&hash_token(&token), user.id, expires_at)
            .await?;
        Ok(Session { token, expires_at })
    }
}

/// Sessions are stored by token hash, so a leaked database doesn't leak live sessions.
pub fn hash_token(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU32, Ordering};

    use async_trait::async_trait;
    use domain::user::UserId;
    use uuid::Uuid;

    use super::*;

    #[derive(Default)]
    struct FakeUsers(Mutex<Vec<(User, String)>>);

    #[async_trait]
    impl UserRepository for FakeUsers {
        async fn create_password_user(&self, new: NewPasswordUser) -> Result<User, RepoError> {
            let mut users = self.0.lock().unwrap();
            if users.iter().any(|(u, _)| u.email == new.email) {
                return Err(RepoError::Conflict);
            }
            let user = User {
                id: UserId(Uuid::new_v4()),
                email: new.email,
                display_name: new.display_name,
                ui_locale: new.ui_locale,
                created_at: DateTime::UNIX_EPOCH,
            };
            users.push((user.clone(), new.password_hash));
            Ok(user)
        }

        async fn find_password_login(
            &self,
            email: &Email,
        ) -> Result<Option<(User, String)>, RepoError> {
            Ok(self
                .0
                .lock()
                .unwrap()
                .iter()
                .find(|(u, _)| &u.email == email)
                .cloned())
        }
    }

    /// token hash → (owner, expiry)
    type SessionMap = HashMap<Vec<u8>, (UserId, DateTime<Utc>)>;

    #[derive(Default)]
    struct FakeSessions {
        users: Arc<FakeUsers>,
        sessions: Mutex<SessionMap>,
    }

    #[async_trait]
    impl SessionRepository for FakeSessions {
        async fn create(
            &self,
            token_hash: &[u8],
            user_id: UserId,
            expires_at: DateTime<Utc>,
        ) -> Result<(), RepoError> {
            self.sessions
                .lock()
                .unwrap()
                .insert(token_hash.to_vec(), (user_id, expires_at));
            Ok(())
        }

        async fn find_user(
            &self,
            token_hash: &[u8],
            now: DateTime<Utc>,
        ) -> Result<Option<User>, RepoError> {
            let session = self.sessions.lock().unwrap().get(token_hash).copied();
            Ok(session
                .filter(|&(_, expires_at)| expires_at > now)
                .and_then(|(id, _)| {
                    let users = self.users.0.lock().unwrap();
                    users
                        .iter()
                        .find(|(u, _)| u.id == id)
                        .map(|(u, _)| u.clone())
                }))
        }

        async fn delete(&self, token_hash: &[u8]) -> Result<(), RepoError> {
            self.sessions.lock().unwrap().remove(token_hash);
            Ok(())
        }
    }

    struct FakeHasher;

    #[async_trait]
    impl PasswordHasher for FakeHasher {
        async fn hash(&self, password: &Password) -> Result<String, BoxError> {
            Ok(format!("hashed:{}", password.expose()))
        }
        async fn verify(&self, password: &Password, hash: &str) -> Result<bool, BoxError> {
            Ok(hash == format!("hashed:{}", password.expose()))
        }
    }

    struct FakeClock(Mutex<DateTime<Utc>>);

    impl Clock for FakeClock {
        fn now(&self) -> DateTime<Utc> {
            *self.0.lock().unwrap()
        }
    }

    #[derive(Default)]
    struct CountingTokens(AtomicU32);

    impl TokenGenerator for CountingTokens {
        fn session_token(&self) -> String {
            format!("token-{}", self.0.fetch_add(1, Ordering::Relaxed))
        }
    }

    struct Harness {
        auth: AuthService,
        sessions: Arc<FakeSessions>,
        clock: Arc<FakeClock>,
    }

    fn harness() -> Harness {
        let users = Arc::new(FakeUsers::default());
        let sessions = Arc::new(FakeSessions {
            users: users.clone(),
            ..Default::default()
        });
        let clock = Arc::new(FakeClock(Mutex::new(DateTime::UNIX_EPOCH)));
        let auth = AuthService::new(
            users,
            sessions.clone(),
            Arc::new(FakeHasher),
            clock.clone(),
            Arc::new(CountingTokens::default()),
        );
        Harness {
            auth,
            sessions,
            clock,
        }
    }

    fn anna() -> RegisterInput {
        RegisterInput {
            email: " Anna@Example.com".into(),
            password: "correct horse".into(),
            display_name: "Anna".into(),
            ui_locale: Locale::Fr,
        }
    }

    #[tokio::test]
    async fn register_creates_a_logged_in_user() {
        let h = harness();
        let (user, session) = h.auth.register(anna()).await.unwrap();

        assert_eq!(user.email.as_str(), "anna@example.com");
        assert_eq!(user.ui_locale, Locale::Fr);
        assert_eq!(session.expires_at, DateTime::UNIX_EPOCH + SESSION_TTL);
        let current = h.auth.current_user(&session.token).await.unwrap();
        assert_eq!(current.map(|u| u.id), Some(user.id));
    }

    #[tokio::test]
    async fn sessions_are_stored_by_hash_only() {
        let h = harness();
        let (_, session) = h.auth.register(anna()).await.unwrap();

        let stored = h.sessions.sessions.lock().unwrap();
        assert!(stored.contains_key(&hash_token(&session.token)));
        assert!(!stored.contains_key(session.token.as_bytes()));
    }

    #[tokio::test]
    async fn email_can_only_be_registered_once() {
        let h = harness();
        h.auth.register(anna()).await.unwrap();
        let again = RegisterInput {
            email: "anna@example.COM".into(),
            ..anna()
        };
        assert!(matches!(
            h.auth.register(again).await,
            Err(AuthError::EmailTaken)
        ));
    }

    #[tokio::test]
    async fn register_validates_input() {
        let h = harness();
        let short = RegisterInput {
            password: "short".into(),
            ..anna()
        };
        assert!(matches!(
            h.auth.register(short).await,
            Err(AuthError::Validation(ValidationError::PasswordLength))
        ));
    }

    #[tokio::test]
    async fn login_checks_the_password() {
        let h = harness();
        let (user, _) = h.auth.register(anna()).await.unwrap();

        let (logged_in, _) = h
            .auth
            .login("ANNA@example.com", "correct horse")
            .await
            .unwrap();
        assert_eq!(logged_in.id, user.id);
        assert!(matches!(
            h.auth.login("anna@example.com", "wrong password").await,
            Err(AuthError::InvalidCredentials)
        ));
        assert!(matches!(
            h.auth.login("nobody@example.com", "correct horse").await,
            Err(AuthError::InvalidCredentials)
        ));
        assert!(matches!(
            h.auth.login("not an email", "x").await,
            Err(AuthError::InvalidCredentials)
        ));
    }

    #[tokio::test]
    async fn logout_ends_the_session() {
        let h = harness();
        let (_, session) = h.auth.register(anna()).await.unwrap();

        h.auth.logout(&session.token).await.unwrap();
        assert!(h.auth.current_user(&session.token).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn sessions_expire() {
        let h = harness();
        let (_, session) = h.auth.register(anna()).await.unwrap();

        *h.clock.0.lock().unwrap() = session.expires_at;
        assert!(h.auth.current_user(&session.token).await.unwrap().is_none());
    }
}
