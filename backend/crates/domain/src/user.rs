//! Users and the value objects that validate account input.

use std::fmt;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::grammar::Locale;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub display_name: DisplayName,
    pub ui_locale: Locale,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    #[error("not a valid email address")]
    InvalidEmail,
    #[error("the password must be between {min} and {max} characters", min = Password::MIN_CHARS, max = Password::MAX_CHARS)]
    PasswordLength,
    #[error("the display name must be between 1 and {max} characters", max = DisplayName::MAX_CHARS)]
    DisplayNameLength,
}

/// An email address, trimmed and lowercased so that it can be compared and stored as is.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    const MAX_LEN: usize = 254;

    /// A deliberately loose check (something@domain.tld); the real test is whether mail arrives.
    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let email = input.trim().to_lowercase();
        let valid = email.len() <= Self::MAX_LEN
            && !email.chars().any(char::is_whitespace)
            && match email.split_once('@') {
                Some((local, domain)) => {
                    !local.is_empty()
                        && !domain.contains('@')
                        && domain.contains('.')
                        && !domain.starts_with('.')
                        && !domain.ends_with('.')
                }
                None => false,
            };
        if valid {
            Ok(Email(email))
        } else {
            Err(ValidationError::InvalidEmail)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A plaintext password that satisfies the password policy. Never stored or logged.
#[derive(Clone)]
pub struct Password(String);

impl Password {
    pub const MIN_CHARS: usize = 8;
    /// Bounds the hashing cost of a single request.
    pub const MAX_CHARS: usize = 128;

    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let chars = input.chars().count();
        if (Self::MIN_CHARS..=Self::MAX_CHARS).contains(&chars) {
            Ok(Password(input.to_owned()))
        } else {
            Err(ValidationError::PasswordLength)
        }
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password(***)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayName(String);

impl DisplayName {
    pub const MAX_CHARS: usize = 50;

    pub fn parse(input: &str) -> Result<Self, ValidationError> {
        let name = input.trim();
        if (1..=Self::MAX_CHARS).contains(&name.chars().count()) {
            Ok(DisplayName(name.to_owned()))
        } else {
            Err(ValidationError::DisplayNameLength)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_is_normalised() {
        assert_eq!(
            Email::parse("  Anna@Example.COM ").unwrap().as_str(),
            "anna@example.com"
        );
    }

    #[test]
    fn malformed_emails_are_rejected() {
        for bad in [
            "",
            "anna",
            "anna@",
            "@example.com",
            "anna@example",
            "an na@example.com",
            "a@b@c.com",
            "anna@.com",
            "anna@example.",
        ] {
            assert_eq!(
                Email::parse(bad),
                Err(ValidationError::InvalidEmail),
                "{bad}"
            );
        }
    }

    #[test]
    fn password_length_is_counted_in_characters() {
        assert!(Password::parse("короткий").is_ok()); // 8 chars, 16 bytes
        assert_eq!(
            Password::parse("1234567").unwrap_err(),
            ValidationError::PasswordLength
        );
        assert_eq!(
            Password::parse(&"x".repeat(129)).unwrap_err(),
            ValidationError::PasswordLength
        );
    }

    #[test]
    fn password_is_not_printed() {
        assert_eq!(
            format!("{:?}", Password::parse("secret-password").unwrap()),
            "Password(***)"
        );
    }

    #[test]
    fn display_name_is_trimmed_and_required() {
        assert_eq!(DisplayName::parse("  Аня ").unwrap().as_str(), "Аня");
        assert_eq!(
            DisplayName::parse("   "),
            Err(ValidationError::DisplayNameLength)
        );
    }
}
