//! The sentence game: a sentence frame with one noun to put in the right form.
//! The player picks the case first, then the ending; each frame says which case
//! it needs and why (`domain::sentences`).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use domain::game::{AnswerKind, LevelId, LevelSpec};
use domain::grammar::{Case, Number, PartOfSpeech};
use domain::sentences::{TEMPLATES, Template};
use domain::user::UserId;
use rand::seq::SliceRandom;

use crate::game::{
    Attempt, AttemptRepository, ExerciseRepository, GameError, GameFilter, GameItem, MAX_BATCH,
    MAX_ZONES, MIN_ZONE_SHARE, check_attempts, rarer_known_by, sort_like_the_paradigm,
};
use crate::preferences::PreferenceRepository;

pub const EXERCISE_TYPE: &str = "sentences";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceItem {
    pub template: &'static Template,
    /// The text before the noun, adapted to it (о → об before a vowel).
    pub before: String,
    pub word: GameItem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceBatch {
    /// The ending buttons for each case asked in the batch.
    pub zones: Vec<(Case, Vec<String>)>,
    pub items: Vec<SentenceItem>,
}

#[derive(Clone)]
pub struct SentenceGame {
    exercises: Arc<dyn ExerciseRepository>,
    attempts: Arc<dyn AttemptRepository>,
    preferences: Arc<dyn PreferenceRepository>,
}

impl SentenceGame {
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

    /// Up to `count` sentences with nouns from `levels` (noun levels only), the
    /// cases taking turns. Frames no word fits are skipped, so fewer may come back.
    /// Words `player` knows come up less often, as in the endings game.
    pub async fn new_batch(
        &self,
        levels: &[LevelId],
        count: usize,
        player: Option<UserId>,
    ) -> Result<SentenceBatch, GameError> {
        if !(1..=MAX_BATCH).contains(&count) {
            return Err(GameError::InvalidCount);
        }
        let mut specs: Vec<LevelSpec> = levels.iter().map(|l| l.spec()).collect();
        specs.sort_by_key(|s| s.max_rank);
        specs.dedup();
        if specs.is_empty() {
            return Err(GameError::NoLevels);
        }
        if specs.iter().any(|s| s.pos != PartOfSpeech::Noun) {
            return Err(GameError::NounLevelsOnly);
        }

        // Only frames the levels can fill (no plural-only frames for singular levels).
        let fits = |t: &Template| {
            t.numbers
                .iter()
                .any(|n| specs.iter().any(|s| s.numbers.contains(n)))
        };
        let picked = pick_templates(count, fits);
        let known_by = rarer_known_by(self.preferences.as_ref(), player).await?;

        // The ending buttons of each case, shared by all its frames.
        let mut zones: HashMap<Case, Vec<String>> = HashMap::new();
        for case in picked.iter().map(|t| t.case) {
            if zones.contains_key(&case) {
                continue;
            }
            let filter = filter(&specs, case, Number::ALL, None);
            let mut case_zones = self
                .exercises
                .zones(&filter, MIN_ZONE_SHARE, MAX_ZONES)
                .await?;
            sort_like_the_paradigm(&mut case_zones, case, true);
            zones.insert(case, case_zones);
        }

        // One query per kind of frame (same words), with spares so that no word
        // comes twice.
        let mut groups: Vec<(&'static Template, usize, Vec<GameItem>)> = Vec::new();
        for template in &picked {
            match groups.iter_mut().find(|(t, _, _)| same_words(t, template)) {
                Some((_, n, _)) => *n += 1,
                None => groups.push((template, 1, Vec::new())),
            }
        }
        for (template, n, words) in &mut groups {
            let case_zones = &zones[&template.case];
            let filter = GameFilter {
                rarer_known_by: known_by,
                ..filter(&specs, template.case, template.numbers, template.animacy)
            };
            if case_zones.is_empty() || filter.levels.is_empty() {
                continue;
            }
            *words = self
                .exercises
                .random_items(&filter, case_zones, (*n * 2).min(MAX_BATCH))
                .await?;
        }

        let mut used = HashSet::new();
        let mut items = Vec::new();
        for template in picked {
            let group = groups
                .iter_mut()
                .find(|(t, _, _)| same_words(t, template))
                .map(|(_, _, w)| w);
            let Some(word) = group.and_then(|w| {
                let i = w.iter().position(|w| !used.contains(&w.lexeme_id))?;
                Some(w.remove(i))
            }) else {
                continue;
            };
            used.insert(word.lexeme_id);
            items.push(SentenceItem {
                before: template.before_for(&word.stressed_form),
                template,
                word,
            });
        }

        let mut zones: Vec<(Case, Vec<String>)> = zones
            .into_iter()
            .filter(|(case, _)| items.iter().any(|i| i.template.case == *case))
            .collect();
        zones.sort_by_key(|(case, _)| Case::ALL.iter().position(|c| c == case));
        Ok(SentenceBatch { zones, items })
    }

    /// `correct`: both the case and the ending were right the first time.
    pub async fn record_attempts(
        &self,
        user: UserId,
        attempts: &[Attempt],
    ) -> Result<(), GameError> {
        check_attempts(attempts)?;
        Ok(self.attempts.record(user, EXERCISE_TYPE, attempts).await?)
    }
}

/// `count` frames that `fit`, the cases taking turns in random order; a case's
/// frames repeat only once they're all used. Cases without such frames sit out.
fn pick_templates(count: usize, fit: impl Fn(&Template) -> bool) -> Vec<&'static Template> {
    let mut rng = rand::rng();
    let mut cases: Vec<Case> = Case::ALL
        .iter()
        .copied()
        .filter(|c| TEMPLATES.iter().any(|t| t.case == *c && fit(t)))
        .collect();
    if cases.is_empty() {
        return Vec::new();
    }
    cases.shuffle(&mut rng);
    let mut left: HashMap<Case, Vec<&'static Template>> = HashMap::new();
    (0..count)
        .map(|i| {
            let case = cases[i % cases.len()];
            let pool = left.entry(case).or_default();
            if pool.is_empty() {
                pool.extend(TEMPLATES.iter().filter(|t| t.case == case && fit(t)));
                pool.shuffle(&mut rng);
            }
            pool.pop().expect("the case has fitting frames")
        })
        .collect()
}

/// Frames that take the same words (same case, numbers and animacy).
fn same_words(a: &Template, b: &Template) -> bool {
    a.case == b.case && a.numbers == b.numbers && a.animacy == b.animacy
}

/// The levels restricted to the frame's numbers (a level without any of them
/// drops out), in one case.
fn filter(
    specs: &[LevelSpec],
    case: Case,
    numbers: &'static [Number],
    animacy: Option<domain::grammar::Animacy>,
) -> GameFilter {
    let levels = specs
        .iter()
        .filter_map(|spec| {
            let both = |n: &Number| spec.numbers.contains(n) && numbers.contains(n);
            let numbers: &'static [Number] = match (both(&Number::Singular), both(&Number::Plural))
            {
                (true, true) => Number::ALL,
                (true, false) => &[Number::Singular],
                (false, true) => &[Number::Plural],
                (false, false) => return None,
            };
            Some(LevelSpec { numbers, ..*spec })
        })
        .collect();
    GameFilter {
        levels,
        case,
        answer: AnswerKind::Ending,
        animacy,
        rarer_known_by: None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use domain::grammar::{Animacy, Locale};

    use super::*;
    use crate::ports::RepoError;
    use crate::preferences::FakePreferences;

    /// Words per (case, animacy); records the filters it was asked with.
    #[derive(Default)]
    struct FakeWords {
        filters: Mutex<Vec<GameFilter>>,
        /// Cases without any word.
        empty: Vec<Case>,
    }

    fn word(id: i64, case: Case, animacy: Option<Animacy>) -> GameItem {
        GameItem {
            lexeme_id: id,
            stressed_lemma: format!("{case}-{animacy:?}-{id}"),
            translations: vec![(Locale::En, "word".into())],
            number: Number::Singular,
            gender: None,
            animacy: None,
            answer: "а".into(),
            accepted: vec!["а".into()],
            stressed_form: "окно\u{301}".into(),
            paradigm: None,
        }
    }

    #[async_trait]
    impl ExerciseRepository for FakeWords {
        async fn zones(
            &self,
            filter: &GameFilter,
            _: f64,
            _: usize,
        ) -> Result<Vec<String>, RepoError> {
            Ok(if self.empty.contains(&filter.case) {
                vec![]
            } else {
                vec!["а".into(), "у".into()]
            })
        }

        async fn random_items(
            &self,
            filter: &GameFilter,
            _: &[String],
            count: usize,
        ) -> Result<Vec<GameItem>, RepoError> {
            self.filters.lock().unwrap().push(filter.clone());
            let base = Case::ALL.iter().position(|c| *c == filter.case).unwrap() as i64 * 1000
                + if filter.animacy.is_some() { 500 } else { 0 };
            // Overlapping ids between calls of the same case: exercises the dedup.
            Ok((0..count as i64)
                .map(|i| word(base + i, filter.case, filter.animacy))
                .collect())
        }
    }

    struct NoAttempts;

    #[async_trait]
    impl AttemptRepository for NoAttempts {
        async fn record(&self, _: UserId, _: &str, _: &[Attempt]) -> Result<(), RepoError> {
            Ok(())
        }
    }

    fn game(words: FakeWords) -> (SentenceGame, Arc<FakeWords>) {
        let words = Arc::new(words);
        (
            SentenceGame::new(
                words.clone(),
                Arc::new(NoAttempts),
                Arc::new(FakePreferences::default()),
            ),
            words,
        )
    }

    #[tokio::test]
    async fn the_cases_take_turns_and_no_word_comes_twice() {
        let (game, _) = game(FakeWords::default());
        let batch = game
            .new_batch(&[LevelId::IntermediateNouns], 12, None)
            .await
            .unwrap();
        assert_eq!(batch.items.len(), 12);
        for case in Case::ALL {
            let n = batch
                .items
                .iter()
                .filter(|i| i.template.case == *case)
                .count();
            assert_eq!(n, 2, "{case}");
        }
        let ids: HashSet<i64> = batch.items.iter().map(|i| i.word.lexeme_id).collect();
        assert_eq!(ids.len(), 12);
        assert_eq!(batch.zones.len(), 6);
        // Frames keep their requirements.
        for item in &batch.items {
            assert!(
                item.template.animacy.is_none() || item.word.stressed_lemma.contains("Some"),
                "{}",
                item.template.id
            );
        }
    }

    #[tokio::test]
    async fn frames_without_words_are_skipped() {
        let (game, _) = game(FakeWords {
            empty: vec![Case::Dative],
            ..Default::default()
        });
        let batch = game
            .new_batch(&[LevelId::IntermediateNouns], 12, None)
            .await
            .unwrap();
        assert_eq!(batch.items.len(), 10);
        assert!(batch.items.iter().all(|i| i.template.case != Case::Dative));
        assert!(batch.zones.iter().all(|(case, _)| *case != Case::Dative));
    }

    #[tokio::test]
    async fn singular_only_levels_get_no_plural_frames() {
        let (game, words) = game(FakeWords::default());
        let batch = game
            .new_batch(&[LevelId::BasicNouns], 12, None)
            .await
            .unwrap();
        // Nominative frames are plural only, and so are "many"/"five": they aren't
        // picked, and the round is still full.
        assert_eq!(batch.items.len(), 12);
        assert!(
            batch
                .items
                .iter()
                .all(|i| i.template.case != Case::Nominative)
        );
        assert!(
            batch
                .items
                .iter()
                .all(|i| i.template.numbers.contains(&Number::Singular))
        );
        for filter in words.filters.lock().unwrap().iter() {
            assert!(
                filter
                    .levels
                    .iter()
                    .all(|l| l.numbers == [Number::Singular])
            );
        }
    }

    #[tokio::test]
    async fn o_becomes_ob_before_a_vowel() {
        let (game, _) = game(FakeWords::default());
        let batch = game
            .new_batch(&[LevelId::IntermediateNouns], 30, None)
            .await
            .unwrap();
        for item in batch
            .items
            .iter()
            .filter(|i| i.template.before.ends_with(" о "))
        {
            assert!(item.before.ends_with(" об "), "{}", item.before); // окно́
        }
    }

    #[tokio::test]
    async fn only_noun_levels_can_be_played() {
        let (game, _) = game(FakeWords::default());
        assert!(matches!(
            game.new_batch(&[LevelId::BasicAdjectives], 5, None).await,
            Err(GameError::NounLevelsOnly)
        ));
        assert!(matches!(
            game.new_batch(&[], 5, None).await,
            Err(GameError::NoLevels)
        ));
        assert!(matches!(
            game.new_batch(&[LevelId::BasicNouns], 0, None).await,
            Err(GameError::InvalidCount)
        ));
    }

    #[tokio::test]
    async fn the_players_known_words_come_up_less_often() {
        let (game, words) = game(FakeWords::default());
        let anna = UserId(uuid::Uuid::new_v4());
        game.new_batch(&[LevelId::BasicNouns], 6, Some(anna))
            .await
            .unwrap();
        let filters = words.filters.lock().unwrap();
        assert!(!filters.is_empty());
        assert!(filters.iter().all(|f| f.rarer_known_by == Some(anna)));
    }
}
