//! Words as they are loaded into the lexicon by the importer.

use crate::grammar::{Animacy, Case, Gender, Locale, Number, PartOfSpeech};
use crate::inflection::{Split, split_adjective_form, split_noun_form, split_pronoun_form};
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
    /// 1 = most frequent (within its part of speech).
    pub frequency_rank: Option<i32>,
    pub forms: Vec<NewForm>,
    pub translations: Vec<NewTranslation>,
}

/// Which form of a word: case and number, plus gender and animacy for words that
/// agree with a noun (adjectives, pronouns, surnames). `None` = doesn't vary by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slot {
    pub case: Case,
    pub number: Number,
    pub gender: Option<Gender>,
    pub animacy: Option<Animacy>,
}

impl Slot {
    pub const fn new(case: Case, number: Number) -> Self {
        Slot {
            case,
            number,
            gender: None,
            animacy: None,
        }
    }

    pub const fn gender(self, gender: Gender) -> Self {
        Slot {
            gender: Some(gender),
            ..self
        }
    }

    pub const fn animacy(self, animacy: Animacy) -> Self {
        Slot {
            animacy: Some(animacy),
            ..self
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewForm {
    pub slot: Slot,
    /// 0 for the main form, 1.. for alternatives.
    pub variant: i16,
    pub form: String,
    pub stressed_form: String,
    pub stem: Option<String>,
    pub ending: Option<String>,
    /// No stem + ending split; such forms are only asked as whole forms.
    pub irregular: bool,
}

impl NewForm {
    /// A noun form, split against the lemma. Forms of indeclinable nouns are
    /// always irregular: кофе would otherwise look like a regular dative коф|е.
    pub fn noun(
        lemma: &str,
        stressed_form: &str,
        slot: Slot,
        variant: i16,
        indeclinable: bool,
    ) -> Self {
        let split = if indeclinable {
            Split::Irregular
        } else {
            split_noun_form(lemma, stressed_form, slot.case, slot.number)
        };
        Self::build(stressed_form, slot, variant, split)
    }

    /// An adjective form, split against the lemma (но́вый → нов|ого).
    pub fn adjective(lemma: &str, stressed_form: &str, slot: Slot, variant: i16) -> Self {
        let split = split_adjective_form(lemma, stressed_form, slot.case, slot.number, slot.gender);
        Self::build(stressed_form, slot, variant, split)
    }

    /// A pronoun form, split after the stem shared by all the pronoun's forms
    /// (see [`crate::inflection::common_stem`]).
    pub fn pronoun(stem: &str, stressed_form: &str, slot: Slot, variant: i16) -> Self {
        Self::build(
            stressed_form,
            slot,
            variant,
            split_pronoun_form(stem, stressed_form),
        )
    }

    /// A form whose stem and ending are known up front (generated surnames).
    pub fn presplit(stem: &str, ending: &str, slot: Slot, variant: i16) -> Self {
        let split = Split::Regular {
            stem: stem.to_owned(),
            ending: ending.to_owned(),
        };
        Self::build(&format!("{stem}{ending}"), slot, variant, split)
    }

    /// A form that is learnt as a whole (меня́, него́); never split.
    pub fn whole(stressed_form: &str, slot: Slot, variant: i16) -> Self {
        Self::build(stressed_form, slot, variant, Split::Irregular)
    }

    fn build(stressed_form: &str, slot: Slot, variant: i16, split: Split) -> Self {
        let (stem, ending, irregular) = match split {
            Split::Regular { stem, ending } => (Some(stem), Some(ending), false),
            Split::Irregular => (None, None, true),
        };
        NewForm {
            slot,
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
        let slot = Slot::new(Case::Instrumental, Number::Singular);
        let form = NewForm::noun("книга", "кни\u{301}гой", slot, 0, false);
        assert_eq!(form.form, "книгой");
        assert_eq!(form.stressed_form, "кни\u{301}гой");
        assert_eq!(form.stem.as_deref(), Some("книг"));
        assert_eq!(form.ending.as_deref(), Some("ой"));
        assert!(!form.irregular);
    }

    #[test]
    fn indeclinable_nouns_are_never_split() {
        let slot = Slot::new(Case::Dative, Number::Singular);
        let form = NewForm::noun("кофе", "ко\u{301}фе", slot, 0, true);
        assert!(form.irregular);
        assert_eq!(form.stem, None);
    }

    #[test]
    fn adjective_forms_keep_their_slot() {
        let slot = Slot::new(Case::Accusative, Number::Singular)
            .gender(Gender::Masculine)
            .animacy(Animacy::Animate);
        let form = NewForm::adjective("но\u{301}вый", "но\u{301}вого", slot, 0);
        assert_eq!(form.slot, slot);
        assert_eq!(form.ending.as_deref(), Some("ого"));
    }

    #[test]
    fn whole_and_presplit_forms() {
        let slot = Slot::new(Case::Genitive, Number::Singular);
        let menya = NewForm::whole("меня\u{301}", slot, 0);
        assert!(menya.irregular);
        assert_eq!(menya.form, "меня");

        let ivanova = NewForm::presplit("Иванов", "ой", slot.gender(Gender::Feminine), 0);
        assert_eq!(ivanova.form, "Ивановой");
        assert_eq!(ivanova.ending.as_deref(), Some("ой"));
    }
}
