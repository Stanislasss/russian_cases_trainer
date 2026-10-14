//! Declension of Russian surnames, which follows fixed rules by suffix:
//! - -ов/-ев/-ёв/-ин/-ын: the "mixed" type (Ивано́в, Ивано́ва, Ивано́вым; f. Ивано́вой);
//! - -ский/-цкий and -ой: like adjectives (Чайко́вский, Чайко́вского; f. Чайко́вская).
//!
//! The lemma is the masculine form; each gender has its own singular forms and
//! the plural is shared. Surnames name people, so the accusative is animate.

use crate::grammar::{Case, Gender, Number};
use crate::lexicon::{NewForm, Slot};

/// Endings in the order nominative, genitive, dative, accusative, instrumental, prepositional.
type Paradigm = [&'static str; 6];

struct Pattern {
    /// Removed from the lemma to get the stem.
    lemma_ending: &'static str,
    masculine: Paradigm,
    feminine: Paradigm,
    plural: Paradigm,
}

const MIXED: Pattern = Pattern {
    lemma_ending: "",
    masculine: ["", "а", "у", "а", "ым", "е"],
    feminine: ["а", "ой", "ой", "у", "ой", "ой"],
    plural: ["ы", "ых", "ым", "ых", "ыми", "ых"],
};

const SOFT_ADJECTIVAL: Pattern = Pattern {
    lemma_ending: "ий",
    masculine: ["ий", "ого", "ому", "ого", "им", "ом"],
    feminine: ["ая", "ой", "ой", "ую", "ой", "ой"],
    plural: ["ие", "их", "им", "их", "ими", "их"],
};

const STRESSED_ADJECTIVAL: Pattern = Pattern {
    lemma_ending: "ой",
    masculine: ["ой", "ого", "ому", "ого", "ым", "ом"],
    feminine: ["ая", "ой", "ой", "ую", "ой", "ой"],
    plural: ["ые", "ых", "ым", "ых", "ыми", "ых"],
};

const CASES: [Case; 6] = [
    Case::Nominative,
    Case::Genitive,
    Case::Dative,
    Case::Accusative,
    Case::Instrumental,
    Case::Prepositional,
];

fn pattern(lemma: &str) -> Option<&'static Pattern> {
    if ["ов", "ев", "ёв", "ин", "ын"]
        .iter()
        .any(|s| lemma.ends_with(s))
    {
        Some(&MIXED)
    } else if lemma.ends_with("ский") || lemma.ends_with("цкий") {
        Some(&SOFT_ADJECTIVAL)
    } else if lemma.ends_with("ой") {
        Some(&STRESSED_ADJECTIVAL)
    } else {
        None
    }
}

/// All 18 forms of a surname given in its masculine form, already split into
/// stem + ending; `None` if it doesn't follow one of the patterns above.
pub fn decline(lemma: &str) -> Option<Vec<NewForm>> {
    let pattern = pattern(lemma)?;
    let stem = lemma.strip_suffix(pattern.lemma_ending)?;
    let paradigms = [
        (
            Number::Singular,
            Some(Gender::Masculine),
            &pattern.masculine,
        ),
        (Number::Singular, Some(Gender::Feminine), &pattern.feminine),
        (Number::Plural, None, &pattern.plural),
    ];
    Some(
        paradigms
            .iter()
            .flat_map(|&(number, gender, endings)| {
                CASES.iter().zip(endings).map(move |(&case, ending)| {
                    let slot = Slot {
                        gender,
                        ..Slot::new(case, number)
                    };
                    NewForm::presplit(stem, ending, slot, 0)
                })
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(forms: &[NewForm], case: Case, number: Number, gender: Option<Gender>) -> String {
        forms
            .iter()
            .find(|f| f.slot.case == case && f.slot.number == number && f.slot.gender == gender)
            .map(|f| f.form.clone())
            .unwrap()
    }

    #[test]
    fn mixed_type() {
        let forms = decline("Иванов").unwrap();
        assert_eq!(forms.len(), 18);
        let m = Some(Gender::Masculine);
        let f = Some(Gender::Feminine);
        assert_eq!(
            form(&forms, Case::Instrumental, Number::Singular, m),
            "Ивановым"
        );
        assert_eq!(
            form(&forms, Case::Prepositional, Number::Singular, m),
            "Иванове"
        );
        assert_eq!(
            form(&forms, Case::Accusative, Number::Singular, m),
            "Иванова"
        );
        assert_eq!(
            form(&forms, Case::Nominative, Number::Singular, f),
            "Иванова"
        );
        assert_eq!(
            form(&forms, Case::Accusative, Number::Singular, f),
            "Иванову"
        );
        assert_eq!(
            form(&forms, Case::Instrumental, Number::Plural, None),
            "Ивановыми"
        );
        assert_eq!(
            form(
                &decline("Пушкин").unwrap(),
                Case::Genitive,
                Number::Plural,
                None
            ),
            "Пушкиных"
        );
    }

    #[test]
    fn adjectival_types() {
        let forms = decline("Чайковский").unwrap();
        let ending = |case, number, gender| {
            forms
                .iter()
                .find(|f| f.slot.case == case && f.slot.number == number && f.slot.gender == gender)
                .and_then(|f| f.ending.clone())
                .unwrap()
        };
        assert_eq!(
            ending(Case::Genitive, Number::Singular, Some(Gender::Masculine)),
            "ого"
        );
        assert_eq!(
            ending(Case::Nominative, Number::Singular, Some(Gender::Feminine)),
            "ая"
        );
        assert_eq!(
            form(&forms, Case::Dative, Number::Plural, None),
            "Чайковским"
        );

        let tolstoy = decline("Толстой").unwrap();
        assert_eq!(
            form(
                &tolstoy,
                Case::Instrumental,
                Number::Singular,
                Some(Gender::Masculine)
            ),
            "Толстым"
        );
        assert_eq!(
            form(
                &tolstoy,
                Case::Nominative,
                Number::Singular,
                Some(Gender::Feminine)
            ),
            "Толстая"
        );
    }

    #[test]
    fn other_surnames_are_not_declined_by_rule() {
        assert!(decline("Шевченко").is_none());
        assert!(decline("Дюма").is_none());
    }
}
