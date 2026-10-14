//! Password hashing, session tokens and the clock.

use application::ports::{BoxError, Clock, PasswordHasher, TokenGenerator};
use argon2::Argon2;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::password_hash::{self, PasswordHash, PasswordVerifier, SaltString};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::user::Password;

/// Argon2id with the crate's default (OWASP-recommended) parameters. Hashing is
/// CPU-heavy, so it runs on the blocking thread pool.
#[derive(Clone, Default)]
pub struct Argon2Hasher;

#[async_trait]
impl PasswordHasher for Argon2Hasher {
    async fn hash(&self, password: &Password) -> Result<String, BoxError> {
        let password = password.clone();
        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            argon2::PasswordHasher::hash_password(
                &Argon2::default(),
                password.expose().as_bytes(),
                &salt,
            )
            .map(|hash| hash.to_string())
            .map_err(|e| BoxError::from(e.to_string()))
        })
        .await?
    }

    async fn verify(&self, password: &Password, hash: &str) -> Result<bool, BoxError> {
        let password = password.clone();
        let hash = hash.to_owned();
        tokio::task::spawn_blocking(move || {
            let parsed = PasswordHash::new(&hash).map_err(|e| BoxError::from(e.to_string()))?;
            match Argon2::default().verify_password(password.expose().as_bytes(), &parsed) {
                Ok(()) => Ok(true),
                Err(password_hash::Error::Password) => Ok(false),
                Err(e) => Err(BoxError::from(e.to_string())),
            }
        })
        .await?
    }
}

/// 256-bit random tokens from the OS, hex-encoded.
#[derive(Clone, Default)]
pub struct OsTokenGenerator;

impl TokenGenerator for OsTokenGenerator {
    fn session_token(&self) -> String {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[derive(Clone, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hashes_verify_only_the_right_password() {
        let hasher = Argon2Hasher;
        let password = Password::parse("correct horse").unwrap();
        let hash = hasher.hash(&password).await.unwrap();

        assert!(hash.starts_with("$argon2id$"));
        assert!(hasher.verify(&password, &hash).await.unwrap());
        let wrong = Password::parse("wrong horse").unwrap();
        assert!(!hasher.verify(&wrong, &hash).await.unwrap());
    }

    #[test]
    fn tokens_are_random_hex() {
        let a = OsTokenGenerator.session_token();
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, OsTokenGenerator.session_token());
    }
}
