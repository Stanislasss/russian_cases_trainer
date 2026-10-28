//! The player's own settings for how games are put together.

use async_trait::async_trait;
use domain::user::UserId;

use crate::ports::RepoError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preferences {
    /// Words the player keeps getting right come up less often
    /// (see `ExerciseRepository::random_items`).
    pub fewer_known_words: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            fewer_known_words: true,
        }
    }
}

#[async_trait]
pub trait PreferenceRepository: Send + Sync {
    /// The player's preferences; the defaults until they change any.
    async fn get(&self, user: UserId) -> Result<Preferences, RepoError>;

    async fn save(&self, user: UserId, preferences: &Preferences) -> Result<(), RepoError>;
}

/// Preferences kept in memory, for the use cases' tests.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct FakePreferences(std::sync::Mutex<std::collections::HashMap<UserId, Preferences>>);

#[cfg(test)]
#[async_trait]
impl PreferenceRepository for FakePreferences {
    async fn get(&self, user: UserId) -> Result<Preferences, RepoError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .get(&user)
            .copied()
            .unwrap_or_default())
    }

    async fn save(&self, user: UserId, preferences: &Preferences) -> Result<(), RepoError> {
        self.0.lock().unwrap().insert(user, *preferences);
        Ok(())
    }
}
