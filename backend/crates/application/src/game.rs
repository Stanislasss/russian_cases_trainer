//! The endings game: a word in its dictionary form is dropped onto the ending
//! it takes in the target case.

use std::sync::Arc;

use async_trait::async_trait;
use domain::game::{LevelId, LevelSpec};
use domain::grammar::{Case, Locale, Number};
use domain::inflection::noun_endings;
use domain::user::UserId;

use crate::ports::{BoxError, RepoError};

pub const EXERCISE_TYPE: &str = "endings";
pub const MAX_BATCH: usize = 50;
pub const MAX_ATTEMPTS: usize = 100;
const MAX_ANSWER_CHARS: usize = 10;
/// An ending gets its own zone only if at least this share of the pool uses it;
/// words with rarer endings are left out so the board stays readable.
pub const MIN_ZONE_SHARE: f64 = 0.01;

/// Which forms a game draws from: any selected level, in one case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndingsFilter {
    pub levels: Vec<LevelSpec>,
    pub case: Case,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameItem {
    pub lexeme_id: i64,
    pub stressed_lemma: String,
    /// In the requested locale, else English, else none.
    pub translation: Option<String>,
    pub number: Number,
    /// The ending of the main form.
    pub answer: String,
    /// `answer` plus the endings of alternative forms (книгой / книгою).
    pub accepted: Vec<String>,
    pub stressed_form: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    /// The ending zones on the board; the same for every batch of a game.
    pub zones: Vec<String>,
    pub items: Vec<GameItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub lexeme_id: i64,
    pub case: Case,
    pub number: Number,
    pub answer: String,
    pub correct: bool,
}

#[async_trait]
pub trait ExerciseRepository: Send + Sync {
    /// Endings used by at least `min_share` of the forms matching `filter`.
    async fn ending_zones(
        &self,
        filter: &EndingsFilter,
        min_share: f64,
    ) -> Result<Vec<String>, RepoError>;

    /// Up to `count` random forms matching `filter` whose ending is one of `zones`.
    async fn random_ending_items(
        &self,
        filter: &EndingsFilter,
        zones: &[String],
        count: usize,
        locale: Locale,
    ) -> Result<Vec<GameItem>, RepoError>;
}

#[async_trait]
pub trait AttemptRepository: Send + Sync {
    async fn record(
        &self,
        user: UserId,
        exercise_type: &str,
        attempts: &[Attempt],
    ) -> Result<(), RepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("select at least one level")]
    NoLevels,
    #[error("ask for between 1 and {MAX_BATCH} words")]
    InvalidCount,
    /// Too few/many attempts, or one refers to an unknown word.
    #[error("send between 1 and {MAX_ATTEMPTS} attempts for existing words")]
    InvalidAttempts,
    #[error(transparent)]
    Internal(BoxError),
}

impl From<RepoError> for GameError {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::InvalidReference => GameError::InvalidAttempts,
            e @ RepoError::Conflict => GameError::Internal(e.into()),
            RepoError::Other(e) => GameError::Internal(e),
        }
    }
}

#[derive(Clone)]
pub struct EndingsGame {
    exercises: Arc<dyn ExerciseRepository>,
    attempts: Arc<dyn AttemptRepository>,
}

impl EndingsGame {
    pub fn new(
        exercises: Arc<dyn ExerciseRepository>,
        attempts: Arc<dyn AttemptRepository>,
    ) -> Self {
        Self {
            exercises,
            attempts,
        }
    }

    pub async fn new_batch(
        &self,
        levels: &[LevelId],
        case: Case,
        count: usize,
        locale: Locale,
    ) -> Result<Batch, GameError> {
        if levels.is_empty() {
            return Err(GameError::NoLevels);
        }
        if !(1..=MAX_BATCH).contains(&count) {
            return Err(GameError::InvalidCount);
        }
        let mut levels = levels.to_vec();
        levels.sort_by_key(|l| l.as_str());
        levels.dedup();
        let filter = EndingsFilter {
            levels: levels
                .iter()
                .filter_map(|l| playable_spec(l.spec(), case))
                .collect(),
            case,
        };
        if filter.levels.is_empty() {
            return Ok(Batch {
                zones: Vec::new(),
                items: Vec::new(),
            });
        }

        let mut zones = self.exercises.ending_zones(&filter, MIN_ZONE_SHARE).await?;
        sort_like_the_paradigm(&mut zones, case);
        let items = if zones.is_empty() {
            Vec::new()
        } else {
            self.exercises
                .random_ending_items(&filter, &zones, count, locale)
                .await?
        };
        Ok(Batch { zones, items })
    }

    pub async fn record_attempts(
        &self,
        user: UserId,
        attempts: &[Attempt],
    ) -> Result<(), GameError> {
        // Answers are endings, so a few letters; anything longer isn't a real answer.
        let sane = |a: &Attempt| a.answer.chars().count() <= MAX_ANSWER_CHARS;
        if !(1..=MAX_ATTEMPTS).contains(&attempts.len()) || !attempts.iter().all(sane) {
            return Err(GameError::InvalidAttempts);
        }
        Ok(self.attempts.record(user, EXERCISE_TYPE, attempts).await?)
    }
}

/// The nominative singular *is* the dictionary form shown to the player, so it
/// would give the answer away: in the nominative only plurals are asked.
fn playable_spec(mut spec: LevelSpec, case: Case) -> Option<LevelSpec> {
    const PLURAL: &[Number] = &[Number::Plural];
    if case == Case::Nominative {
        if !spec.numbers.contains(&Number::Plural) {
            return None;
        }
        spec.numbers = PLURAL;
    }
    Some(spec)
}

/// Singular endings first, then plural, in the order grammar tables list them.
fn sort_like_the_paradigm(zones: &mut [String], case: Case) {
    let order: Vec<&str> = noun_endings(case, Number::Singular)
        .iter()
        .chain(noun_endings(case, Number::Plural))
        .copied()
        .collect();
    zones.sort_by_key(|z| order.iter().position(|o| o == z).unwrap_or(usize::MAX));
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use uuid::Uuid;

    use super::*;

    #[derive(Default)]
    struct FakeExercises {
        zones: Vec<String>,
        seen_filter: Mutex<Option<EndingsFilter>>,
    }

    #[async_trait]
    impl ExerciseRepository for FakeExercises {
        async fn ending_zones(
            &self,
            filter: &EndingsFilter,
            _min_share: f64,
        ) -> Result<Vec<String>, RepoError> {
            *self.seen_filter.lock().unwrap() = Some(filter.clone());
            Ok(self.zones.clone())
        }

        async fn random_ending_items(
            &self,
            _filter: &EndingsFilter,
            zones: &[String],
            count: usize,
            _locale: Locale,
        ) -> Result<Vec<GameItem>, RepoError> {
            Ok((0..count)
                .map(|i| GameItem {
                    lexeme_id: i as i64,
                    stressed_lemma: "кни\u{301}га".into(),
                    translation: Some("book".into()),
                    number: Number::Singular,
                    answer: zones[0].clone(),
                    accepted: vec![zones[0].clone()],
                    stressed_form: "кни\u{301}гой".into(),
                })
                .collect())
        }
    }

    #[derive(Default)]
    struct FakeAttempts(Mutex<Vec<(UserId, String, Attempt)>>);

    #[async_trait]
    impl AttemptRepository for FakeAttempts {
        async fn record(
            &self,
            user: UserId,
            exercise_type: &str,
            attempts: &[Attempt],
        ) -> Result<(), RepoError> {
            let mut stored = self.0.lock().unwrap();
            stored.extend(
                attempts
                    .iter()
                    .map(|a| (user, exercise_type.to_owned(), a.clone())),
            );
            Ok(())
        }
    }

    fn game(zones: &[&str]) -> (EndingsGame, Arc<FakeExercises>, Arc<FakeAttempts>) {
        let exercises = Arc::new(FakeExercises {
            zones: zones.iter().map(|z| z.to_string()).collect(),
            ..Default::default()
        });
        let attempts = Arc::new(FakeAttempts::default());
        (
            EndingsGame::new(exercises.clone(), attempts.clone()),
            exercises,
            attempts,
        )
    }

    #[tokio::test]
    async fn zones_follow_the_paradigm_order() {
        let (game, _, _) = game(&["ами", "ой", "ом", "ью", "ей"]);
        let batch = game
            .new_batch(&[LevelId::BasicNouns], Case::Instrumental, 5, Locale::En)
            .await
            .unwrap();
        assert_eq!(batch.zones, ["ом", "ой", "ей", "ью", "ами"]);
        assert_eq!(batch.items.len(), 5);
    }

    #[tokio::test]
    async fn duplicate_levels_are_merged_into_one_filter() {
        let (game, exercises, _) = game(&["ой"]);
        game.new_batch(
            &[
                LevelId::BasicNouns,
                LevelId::AdvancedNouns,
                LevelId::BasicNouns,
            ],
            Case::Genitive,
            3,
            Locale::Fr,
        )
        .await
        .unwrap();

        let filter = exercises.seen_filter.lock().unwrap().clone().unwrap();
        assert_eq!(filter.case, Case::Genitive);
        assert_eq!(filter.levels.len(), 2);
    }

    #[tokio::test]
    async fn nominative_only_asks_plurals() {
        let (game, exercises, _) = game(&["ы"]);

        let basic_only = game
            .new_batch(&[LevelId::BasicNouns], Case::Nominative, 5, Locale::En)
            .await
            .unwrap();
        assert!(basic_only.zones.is_empty() && basic_only.items.is_empty());
        assert!(
            exercises.seen_filter.lock().unwrap().is_none(),
            "no query needed"
        );

        game.new_batch(
            &[LevelId::BasicNouns, LevelId::IntermediateNouns],
            Case::Nominative,
            5,
            Locale::En,
        )
        .await
        .unwrap();
        let filter = exercises.seen_filter.lock().unwrap().clone().unwrap();
        assert_eq!(filter.levels.len(), 1);
        assert_eq!(filter.levels[0].numbers, [Number::Plural]);
    }

    #[tokio::test]
    async fn empty_pool_gives_an_empty_batch() {
        let (game, _, _) = game(&[]);
        let batch = game
            .new_batch(&[LevelId::BasicNouns], Case::Dative, 10, Locale::En)
            .await
            .unwrap();
        assert!(batch.zones.is_empty() && batch.items.is_empty());
    }

    #[tokio::test]
    async fn batch_requests_are_validated() {
        let (game, _, _) = game(&["ой"]);
        assert!(matches!(
            game.new_batch(&[], Case::Dative, 10, Locale::En).await,
            Err(GameError::NoLevels)
        ));
        for count in [0, MAX_BATCH + 1] {
            assert!(matches!(
                game.new_batch(&[LevelId::BasicNouns], Case::Dative, count, Locale::En)
                    .await,
                Err(GameError::InvalidCount)
            ));
        }
    }

    #[tokio::test]
    async fn attempts_are_recorded_for_the_user() {
        let (game, _, attempts) = game(&["ой"]);
        let user = UserId(Uuid::new_v4());
        let attempt = Attempt {
            lexeme_id: 7,
            case: Case::Instrumental,
            number: Number::Singular,
            answer: "ой".into(),
            correct: true,
        };

        game.record_attempts(user, std::slice::from_ref(&attempt))
            .await
            .unwrap();
        assert_eq!(
            *attempts.0.lock().unwrap(),
            [(user, EXERCISE_TYPE.to_owned(), attempt.clone())]
        );
        assert!(matches!(
            game.record_attempts(user, &[]).await,
            Err(GameError::InvalidAttempts)
        ));
        let junk = Attempt {
            answer: "x".repeat(500),
            ..attempt
        };
        assert!(matches!(
            game.record_attempts(user, &[junk]).await,
            Err(GameError::InvalidAttempts)
        ));
    }
}
