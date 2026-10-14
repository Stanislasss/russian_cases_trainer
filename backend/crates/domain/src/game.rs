//! Game configuration: the difficulty levels players pick with checkboxes.

use crate::grammar::{Number, PartOfSpeech, string_enum};

string_enum! {
    /// A selectable difficulty level.
    LevelId, "level" {
        BasicNouns => "basic_nouns",
        IntermediateNouns => "intermediate_nouns",
        AdvancedNouns => "advanced_nouns",
        BasicAdjectives => "basic_adjectives",
        IntermediateAdjectives => "intermediate_adjectives",
        Pronouns => "pronouns",
        PersonalPronouns => "personal_pronouns",
        Surnames => "surnames",
    }
}

string_enum! {
    /// What the player drops a word onto.
    AnswerKind, "answer kind" {
        /// The form's ending (кни́г|ой → -ой).
        Ending => "ending",
        /// The whole form, for words without a stem to keep (я → мной).
        WholeForm => "whole_form",
    }
}

/// Which word forms a level draws from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelSpec {
    pub pos: PartOfSpeech,
    /// Only the `max_rank` most frequent words of that part of speech.
    pub max_rank: i32,
    pub numbers: &'static [Number],
    pub answer: AnswerKind,
}

const SINGULAR: &[Number] = &[Number::Singular];
const BOTH_NUMBERS: &[Number] = &[Number::Singular, Number::Plural];

impl LevelId {
    pub const fn spec(self) -> LevelSpec {
        use PartOfSpeech::*;
        let (pos, max_rank, numbers, answer) = match self {
            LevelId::BasicNouns => (Noun, 300, SINGULAR, AnswerKind::Ending),
            LevelId::IntermediateNouns => (Noun, 2000, BOTH_NUMBERS, AnswerKind::Ending),
            LevelId::AdvancedNouns => (Noun, 10_000, BOTH_NUMBERS, AnswerKind::Ending),
            LevelId::BasicAdjectives => (Adjective, 300, SINGULAR, AnswerKind::Ending),
            LevelId::IntermediateAdjectives => (Adjective, 2000, BOTH_NUMBERS, AnswerKind::Ending),
            LevelId::Pronouns => (Pronoun, i32::MAX, BOTH_NUMBERS, AnswerKind::Ending),
            LevelId::PersonalPronouns => (
                PersonalPronoun,
                i32::MAX,
                BOTH_NUMBERS,
                AnswerKind::WholeForm,
            ),
            LevelId::Surnames => (Surname, i32::MAX, BOTH_NUMBERS, AnswerKind::Ending),
        };
        LevelSpec {
            pos,
            max_rank,
            numbers,
            answer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noun_levels_get_harder() {
        let nouns = [
            LevelId::BasicNouns,
            LevelId::IntermediateNouns,
            LevelId::AdvancedNouns,
        ];
        let ranks: Vec<i32> = nouns.iter().map(|l| l.spec().max_rank).collect();
        assert!(ranks.is_sorted(), "{ranks:?}");
        assert_eq!(LevelId::BasicNouns.spec().numbers, [Number::Singular]);
    }

    #[test]
    fn only_personal_pronouns_are_whole_forms() {
        for &level in LevelId::ALL {
            let whole = level.spec().answer == AnswerKind::WholeForm;
            assert_eq!(whole, level == LevelId::PersonalPronouns, "{level}");
        }
    }

    #[test]
    fn level_ids_parse() {
        assert_eq!(
            "advanced_nouns".parse::<LevelId>(),
            Ok(LevelId::AdvancedNouns)
        );
    }
}
