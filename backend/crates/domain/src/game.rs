//! Game configuration: the difficulty levels players pick with checkboxes.

use crate::grammar::{Number, PartOfSpeech, string_enum};

string_enum! {
    /// A selectable difficulty level. Adjective levels are added once adjectives are imported.
    LevelId, "level" {
        BasicNouns => "basic_nouns",
        IntermediateNouns => "intermediate_nouns",
        AdvancedNouns => "advanced_nouns",
    }
}

/// Which word forms a level draws from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelSpec {
    pub pos: PartOfSpeech,
    /// Only the `max_rank` most frequent words (frequency rank 1 = most common).
    pub max_rank: i32,
    pub numbers: &'static [Number],
}

const SINGULAR: &[Number] = &[Number::Singular];
const BOTH_NUMBERS: &[Number] = &[Number::Singular, Number::Plural];

impl LevelId {
    pub const fn spec(self) -> LevelSpec {
        match self {
            LevelId::BasicNouns => LevelSpec {
                pos: PartOfSpeech::Noun,
                max_rank: 300,
                numbers: SINGULAR,
            },
            LevelId::IntermediateNouns => LevelSpec {
                pos: PartOfSpeech::Noun,
                max_rank: 2000,
                numbers: BOTH_NUMBERS,
            },
            LevelId::AdvancedNouns => LevelSpec {
                pos: PartOfSpeech::Noun,
                max_rank: 10_000,
                numbers: BOTH_NUMBERS,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_get_harder() {
        let ranks: Vec<i32> = LevelId::ALL.iter().map(|l| l.spec().max_rank).collect();
        assert!(ranks.is_sorted(), "{ranks:?}");
        assert_eq!(LevelId::BasicNouns.spec().numbers, [Number::Singular]);
    }

    #[test]
    fn level_ids_parse() {
        assert_eq!(
            "advanced_nouns".parse::<LevelId>(),
            Ok(LevelId::AdvancedNouns)
        );
    }
}
