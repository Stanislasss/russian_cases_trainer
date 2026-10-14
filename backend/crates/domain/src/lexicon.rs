//! Words as they are loaded into the lexicon by the importer.

use crate::grammar::{Animacy, Case, Gender, Locale, Number, PartOfSpeech};
use crate::inflection::{Split, split_noun_form};
use crate::stress;

/// A word with all its forms and translations, identified by where it came from
/// so that re-importing updates it instead of duplicating it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewLexeme {
    pub source: String,
    pub source_id: String,
    pub lemma: String,
    pub stressed_lemma: Option<String>,
    pub pos: PartOfSpeech,
    pub gender: Option<Gender>,
    pub animacy: Option<Animacy>,
    pub indeclinable: bool,
    /// 1 = most frequent.
    pub frequency_rank: Option<i32>,
    pub forms: Vec<NewForm>,
    pub translations: Vec<NewTranslation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewForm {
    pub case: Case,
    pub number: Number,
    /// 0 for the main form, 1.. for alternatives.
    pub variant: i16,
    pub form: String,
    pub stressed_form: String,
    pub stem: Option<String>,
    pub ending: Option<String>,
    pub irregular: bool,
}

impl NewForm {
    /// Builds a noun form from its stressed spelling, splitting it into stem +
    /// ending. Forms of indeclinable nouns are always irregular: кофе would
    /// otherwise look like a regular dative коф|е.
    pub fn noun(
        lemma: &str,
        stressed_form: &str,
        case: Case,
        number: Number,
        variant: i16,
        indeclinable: bool,
    ) -> Self {
        let split = if indeclinable {
            Split::Irregular
        } else {
            split_noun_form(lemma, stressed_form, case, number)
        };
        let (stem, ending, irregular) = match split {
            Split::Regular { stem, ending } => (Some(stem), Some(ending), false),
            Split::Irregular => (None, None, true),
        };
        NewForm {
            case,
            number,
            variant,
            form: stress::strip(stressed_form),
            stressed_form: stressed_form.to_owned(),
            stem,
            ending,
            irregular,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTranslation {
    pub locale: Locale,
    pub position: i16,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noun_form_is_split_and_unstressed() {
        let form = NewForm::noun(
            "книга",
            "кни\u{301}гой",
            Case::Instrumental,
            Number::Singular,
            0,
            false,
        );
        assert_eq!(form.form, "книгой");
        assert_eq!(form.stressed_form, "кни\u{301}гой");
        assert_eq!(form.stem.as_deref(), Some("книг"));
        assert_eq!(form.ending.as_deref(), Some("ой"));
        assert!(!form.irregular);
    }

    #[test]
    fn indeclinable_nouns_are_never_split() {
        let form = NewForm::noun(
            "кофе",
            "ко\u{301}фе",
            Case::Dative,
            Number::Singular,
            0,
            true,
        );
        assert!(form.irregular);
        assert_eq!(form.stem, None);
    }
}
