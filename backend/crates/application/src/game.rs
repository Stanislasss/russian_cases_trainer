//! The endings game: a word in its dictionary form is dropped onto the ending
//! (or, for personal pronouns, the whole form) it takes in the target case.

use std::sync::Arc;

use async_trait::async_trait;
use domain::game::{AnswerKind, LevelId, LevelSpec};
use domain::grammar::{Animacy, Case, Gender, Locale, Number, PartOfSpeech};
use domain::inflection::{adjective_endings, noun_endings};
use domain::rules::Paradigm;
use domain::user::UserId;

use crate::ports::{BoxError, RepoError};
use crate::preferences::PreferenceRepository;

pub const EXERCISE_TYPE: &str = "endings";
pub const MAX_BATCH: usize = 50;
pub const MAX_ATTEMPTS: usize = 100;
/// Answers are endings or short pronoun forms (собо́ю); anything longer isn't real.
const MAX_ANSWER_CHARS: usize = 10;
/// An answer gets its own zone only if at least this share of the pool uses it;
/// words with rarer answers are left out so the board stays readable.
pub const MIN_ZONE_SHARE: f64 = 0.01;
/// At most this many zones, the most common answers; more don't fit on a phone.
pub const MAX_ZONES: usize = 12;
/// A word counts as known in a case once its last this many answers in that
/// case (in any game) were right; one mistake makes it new again.
pub const KNOWN_STREAK: usize = 3;

/// Which forms a game draws from: any selected level, in one case. The form equal
/// to the dictionary form shown (книга, новый, я) is never asked: it gives itself away.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameFilter {
    pub levels: Vec<LevelSpec>,
    pub case: Case,
    /// Shared by all levels of a game.
    pub answer: AnswerKind,
    /// Only nouns of this animacy (sentence frames that need someone); `None`: any word.
    pub animacy: Option<Animacy>,
    /// Words this player knows in `case` come up less often; `None`: all alike.
    pub rarer_known_by: Option<UserId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameItem {
    pub lexeme_id: i64,
    pub stressed_lemma: String,
    /// Up to three meanings per language (joined with "; "); the client shows
    /// the one for its UI language. For Russian these are definitions.
    pub translations: Vec<(Locale, String)>,
    pub number: Number,
    /// For adjective-like words: the gender (singular) and, in the accusative,
    /// the animacy of the noun the form agrees with.
    pub gender: Option<Gender>,
    pub animacy: Option<Animacy>,
    /// The ending (or whole form) of the main form.
    pub answer: String,
    /// `answer` plus those of alternative forms (книгой / книгою, его / него).
    pub accepted: Vec<String>,
    pub stressed_form: String,
    /// The declension pattern, for explaining mistakes; `None` if it fits none.
    pub paradigm: Option<Paradigm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    pub answer: AnswerKind,
    /// The zones on the board; the same for every batch of a game.
    pub zones: Vec<String>,
    pub items: Vec<GameItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    pub lexeme_id: i64,
    pub case: Case,
    pub number: Number,
    pub gender: Option<Gender>,
    pub animacy: Option<Animacy>,
    pub answer: String,
    pub correct: bool,
}

#[async_trait]
pub trait ExerciseRepository: Send + Sync {
    /// The `max_zones` most common answers among the forms matching `filter`,
    /// leaving out any used by less than `min_share` of them. Ordered by the
    /// frequency rank of the words that use them (я before ты before он).
    async fn zones(
        &self,
        filter: &GameFilter,
        min_share: f64,
        max_zones: usize,
    ) -> Result<Vec<String>, RepoError>;

    /// Up to `count` random forms matching `filter` whose answer is one of `zones`.
    /// Words `filter.rarer_known_by` knows (see `KNOWN_STREAK`) are drawn with
    /// a quarter of the weight of the others.
    async fn random_items(
        &self,
        filter: &GameFilter,
        zones: &[String],
        count: usize,
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
    /// Personal pronouns (whole forms) can't share a board with ending levels.
    #[error("these levels can't be played together")]
    MixedLevels,
    #[error("ask for between 1 and {MAX_BATCH} words")]
    InvalidCount,
    /// The sentence game only fills its frames with nouns.
    #[error("only noun levels can be played in sentences")]
    NounLevelsOnly,
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
    preferences: Arc<dyn PreferenceRepository>,
}

impl EndingsGame {
    pub fn new(
        exercises: Arc<dyn ExerciseRepository>,
        attempts: Arc<dyn AttemptRepository>,
        preferences: Arc<dyn PreferenceRepository>,
    ) -> Self {
        Self {
            exercises,
            attempts,
            preferences,
        }
    }

    /// `player`: who is playing, if anyone is logged in. Words they know come up
    /// less often, unless they turned that off.
    pub async fn new_batch(
        &self,
        levels: &[LevelId],
        case: Case,
        count: usize,
        player: Option<UserId>,
    ) -> Result<Batch, GameError> {
        if !(1..=MAX_BATCH).contains(&count) {
            return Err(GameError::InvalidCount);
        }
        let mut levels = levels.to_vec();
        levels.sort_by_key(|l| l.as_str());
        levels.dedup();
        let specs: Vec<LevelSpec> = levels.iter().map(|l| l.spec()).collect();
        let answer = match specs.first() {
            None => return Err(GameError::NoLevels),
            Some(first) if specs.iter().any(|s| s.answer != first.answer) => {
                return Err(GameError::MixedLevels);
            }
            Some(first) => first.answer,
        };
        let filter = GameFilter {
            levels: specs,
            case,
            answer,
            animacy: None,
            rarer_known_by: rarer_known_by(self.preferences.as_ref(), player).await?,
        };

        let mut zones = self
            .exercises
            .zones(&filter, MIN_ZONE_SHARE, MAX_ZONES)
            .await?;
        if answer == AnswerKind::Ending {
            let nouns_first = filter.levels.iter().any(|l| l.pos == PartOfSpeech::Noun);
            sort_like_the_paradigm(&mut zones, case, nouns_first);
        }
        let items = if zones.is_empty() {
            Vec::new()
        } else {
            self.exercises.random_items(&filter, &zones, count).await?
        };
        Ok(Batch {
            answer,
            zones,
            items,
        })
    }

    pub async fn record_attempts(
        &self,
        user: UserId,
        attempts: &[Attempt],
    ) -> Result<(), GameError> {
        check_attempts(attempts)?;
        Ok(self.attempts.record(user, EXERCISE_TYPE, attempts).await?)
    }
}

/// The player whose known words should come up less often, if any.
pub(crate) async fn rarer_known_by(
    preferences: &dyn PreferenceRepository,
    player: Option<UserId>,
) -> Result<Option<UserId>, GameError> {
    let Some(user) = player else {
        return Ok(None);
    };
    let wanted = preferences.get(user).await?.fewer_known_words;
    Ok(wanted.then_some(user))
}

/// Between 1 and `MAX_ATTEMPTS` attempts, each with a plausible answer.
pub(crate) fn check_attempts(attempts: &[Attempt]) -> Result<(), GameError> {
    let sane = |a: &Attempt| a.answer.chars().count() <= MAX_ANSWER_CHARS;
    if !(1..=MAX_ATTEMPTS).contains(&attempts.len()) || !attempts.iter().all(sane) {
        return Err(GameError::InvalidAttempts);
    }
    Ok(())
}

/// The order grammar tables list endings in: nouns singular then plural, and
/// adjectives masculine, feminine, neuter, plural. Noun order comes first when a
/// noun level is played (-ей is then a genitive plural, not a feminine ending).
/// Unknown endings go last.
pub(crate) fn sort_like_the_paradigm(zones: &mut [String], case: Case, nouns_first: bool) {
    let nouns: Vec<&str> = [Number::Singular, Number::Plural]
        .into_iter()
        .flat_map(|number| noun_endings(case, number).iter().copied())
        .collect();
    let adjectives: Vec<&str> = [
        (Number::Singular, Some(Gender::Masculine)),
        (Number::Singular, Some(Gender::Feminine)),
        (Number::Singular, Some(Gender::Neuter)),
        (Number::Plural, None),
    ]
    .into_iter()
    .flat_map(|(number, gender)| adjective_endings(case, number, gender).iter().copied())
    .collect();
    let order = if nouns_first {
        [nouns, adjectives].concat()
    } else {
        [adjectives, nouns].concat()
    };
    zones.sort_by_key(|z| order.iter().position(|o| o == z).unwrap_or(usize::MAX));
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use uuid::Uuid;

    use super::*;
    use crate::preferences::{FakePreferences, Preferences};

    #[derive(Default)]
    struct FakeExercises {
        zones: Vec<String>,
        seen_filter: Mutex<Option<GameFilter>>,
    }

    #[async_trait]
    impl ExerciseRepository for FakeExercises {
        async fn zones(
            &self,
            filter: &GameFilter,
            _min_share: f64,
            _max_zones: usize,
        ) -> Result<Vec<String>, RepoError> {
            *self.seen_filter.lock().unwrap() = Some(filter.clone());
            Ok(self.zones.clone())
        }

        async fn random_items(
            &self,
            _filter: &GameFilter,
            zones: &[String],
            count: usize,
        ) -> Result<Vec<GameItem>, RepoError> {
            Ok((0..count)
                .map(|i| GameItem {
                    lexeme_id: i as i64,
                    stressed_lemma: "кни\u{301}га".into(),
                    translations: vec![(Locale::En, "book".into())],
                    number: Number::Singular,
                    gender: None,
                    animacy: None,
                    answer: zones[0].clone(),
                    accepted: vec![zones[0].clone()],
                    stressed_form: "кни\u{301}гой".into(),
                    paradigm: Some(Paradigm::NounA),
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
            EndingsGame::new(
                exercises.clone(),
                attempts.clone(),
                Arc::new(FakePreferences::default()),
            ),
            exercises,
            attempts,
        )
    }

    #[tokio::test]
    async fn zones_follow_the_paradigm_order() {
        let (game, _, _) = game(&["ами", "ой", "ом", "ью", "ей"]);
        let batch = game
            .new_batch(&[LevelId::BasicNouns], Case::Instrumental, 5, None)
            .await
            .unwrap();
        assert_eq!(batch.answer, AnswerKind::Ending);
        assert_eq!(batch.zones, ["ом", "ой", "ей", "ью", "ами"]);
        assert_eq!(batch.items.len(), 5);
    }

    #[tokio::test]
    async fn adjective_zones_follow_noun_ones() {
        let (game, _, _) = game(&["ых", "ого", "а", "ой"]);
        let batch = game
            .new_batch(
                &[LevelId::BasicNouns, LevelId::BasicAdjectives],
                Case::Genitive,
                5,
                None,
            )
            .await
            .unwrap();
        assert_eq!(batch.zones, ["а", "ого", "ой", "ых"]);
    }

    #[tokio::test]
    async fn adjective_games_use_the_adjective_order() {
        // -ей is a noun genitive plural ending, but here the feminine one.
        let (game, _, _) = game(&["ей", "ого", "его", "ой"]);
        let batch = game
            .new_batch(&[LevelId::BasicAdjectives], Case::Genitive, 5, None)
            .await
            .unwrap();
        assert_eq!(batch.zones, ["ого", "его", "ой", "ей"]);
    }

    #[tokio::test]
    async fn whole_form_zones_keep_the_repository_order() {
        let (game, exercises, _) = game(&["меня", "тебя", "его"]);
        let batch = game
            .new_batch(&[LevelId::PersonalPronouns], Case::Genitive, 5, None)
            .await
            .unwrap();
        assert_eq!(batch.answer, AnswerKind::WholeForm);
        assert_eq!(batch.zones, ["меня", "тебя", "его"]);
        let filter = exercises.seen_filter.lock().unwrap().clone().unwrap();
        assert_eq!(filter.answer, AnswerKind::WholeForm);
    }

    #[tokio::test]
    async fn whole_form_and_ending_levels_dont_mix() {
        let (game, _, _) = game(&["ой"]);
        assert!(matches!(
            game.new_batch(
                &[LevelId::PersonalPronouns, LevelId::BasicNouns],
                Case::Genitive,
                5,
                None
            )
            .await,
            Err(GameError::MixedLevels)
        ));
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
            None,
        )
        .await
        .unwrap();

        let filter = exercises.seen_filter.lock().unwrap().clone().unwrap();
        assert_eq!(filter.case, Case::Genitive);
        assert_eq!(filter.levels.len(), 2);
    }

    #[tokio::test]
    async fn known_words_come_up_less_often_unless_turned_off() {
        let (game, exercises, _) = game(&["ой"]);
        let anna = UserId(Uuid::new_v4());
        let known_by = |player| {
            let game = game.clone();
            let exercises = exercises.clone();
            async move {
                game.new_batch(&[LevelId::BasicNouns], Case::Genitive, 3, player)
                    .await
                    .unwrap();
                exercises
                    .seen_filter
                    .lock()
                    .unwrap()
                    .clone()
                    .unwrap()
                    .rarer_known_by
            }
        };

        assert_eq!(known_by(Some(anna)).await, Some(anna));
        assert_eq!(known_by(None).await, None);

        let off = Preferences {
            fewer_known_words: false,
        };
        game.preferences.save(anna, &off).await.unwrap();
        assert_eq!(known_by(Some(anna)).await, None);
    }

    #[tokio::test]
    async fn empty_pool_gives_an_empty_batch() {
        let (game, _, _) = game(&[]);
        let batch = game
            .new_batch(&[LevelId::BasicNouns], Case::Dative, 10, None)
            .await
            .unwrap();
        assert!(batch.zones.is_empty() && batch.items.is_empty());
    }

    #[tokio::test]
    async fn batch_requests_are_validated() {
        let (game, _, _) = game(&["ой"]);
        assert!(matches!(
            game.new_batch(&[], Case::Dative, 10, None).await,
            Err(GameError::NoLevels)
        ));
        for count in [0, MAX_BATCH + 1] {
            assert!(matches!(
                game.new_batch(&[LevelId::BasicNouns], Case::Dative, count, None)
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
            gender: Some(Gender::Feminine),
            animacy: None,
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
