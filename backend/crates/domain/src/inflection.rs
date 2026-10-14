//! Splitting inflected noun forms into stem + ending for the drag-the-ending game.
//!
//! The stem is taken from the dictionary form (lemma) minus its nominative ending,
//! and a form splits cleanly only if it starts with that stem and the rest is a
//! known ending for the requested case and number. Anything else (fleeting vowels
//! as in отец → отца, stem changes as in имя → имени, irregular plurals as in
//! брат → братья) is reported as [`Split::Irregular`] and kept out of easy levels.
//!
//! Endings are deliberately learner-oriented rather than strictly morphological:
//! музей splits as музе|й → музе|я and дверь as двер|ь → двер|и.

use crate::grammar::{Case, Number};
use crate::stress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Split {
    /// Plain spelling (no stress marks) of the stem and lowercase ending; `ending` may be empty.
    Regular {
        stem: String,
        ending: String,
    },
    Irregular,
}

/// Letters that end a noun lemma and are therefore its nominative ending. ы and и
/// cover plural-only nouns (ножницы, брюки).
const NOMINATIVE_ENDINGS: &[char] = &['а', 'я', 'о', 'е', 'ё', 'ь', 'й', 'ы', 'и'];

/// Known noun endings per case and number. `""` is the zero ending (стол, книг).
pub fn noun_endings(case: Case, number: Number) -> &'static [&'static str] {
    use Case::*;
    use Number::*;
    match (case, number) {
        (Nominative, Singular) => &["", "а", "я", "о", "е", "ё", "ь", "й"],
        (Genitive, Singular) => &["а", "я", "ы", "и"],
        (Dative, Singular) => &["у", "ю", "е", "и"],
        (Accusative, Singular) => &["", "а", "я", "у", "ю", "о", "е", "ё", "ь", "й"],
        (Instrumental, Singular) => &["ом", "ем", "ём", "ой", "ей", "ёй", "ою", "ею", "ёю", "ью"],
        (Prepositional, Singular) => &["е", "и"],
        (Nominative, Plural) => &["ы", "и", "а", "я"],
        (Genitive, Plural) => &["", "ов", "ев", "ёв", "ей", "ь", "й"],
        (Dative, Plural) => &["ам", "ям"],
        (Accusative, Plural) => &["ы", "и", "а", "я", "", "ов", "ев", "ёв", "ей", "ь", "й"],
        (Instrumental, Plural) => &["ами", "ями", "ьми"],
        (Prepositional, Plural) => &["ах", "ях"],
    }
}

/// The part of a noun lemma that stays when it declines: книга → книг, стол → стол.
/// Stress marks are removed; capitalisation is kept (Москва → Москв).
pub fn noun_stem(lemma: &str) -> String {
    let lemma = plain(lemma);
    match lemma.chars().last() {
        Some(last) if NOMINATIVE_ENDINGS.contains(&last) => {
            lemma[..lemma.len() - last.len_utf8()].to_owned()
        }
        _ => lemma,
    }
}

/// Splits `form` (the `case`/`number` form of the noun `lemma`) into stem + ending.
/// Both inputs may carry stress marks; the result never does. Comparison ignores
/// letter case, but the stem keeps the form's own spelling.
pub fn split_noun_form(lemma: &str, form: &str, case: Case, number: Number) -> Split {
    let stem: Vec<char> = noun_stem(lemma).chars().collect();
    let form: Vec<char> = plain(form).chars().collect();

    if stem.is_empty() || form.len() < stem.len() {
        return Split::Irregular;
    }
    let same_stem = stem.iter().zip(&form).all(|(&a, &b)| fold(a) == fold(b));
    if !same_stem {
        return Split::Irregular;
    }

    let ending: String = form[stem.len()..]
        .iter()
        .flat_map(|c| c.to_lowercase())
        .collect();
    if noun_endings(case, number).contains(&ending.as_str()) {
        Split::Regular {
            stem: form[..stem.len()].iter().collect(),
            ending,
        }
    } else {
        Split::Irregular
    }
}

fn plain(word: &str) -> String {
    stress::strip(word.trim())
}

/// Letter case is ignored, and ё matches е: ё is often written as е, and it
/// alternates within paradigms (жена́ → жёны).
fn fold(c: char) -> char {
    match c.to_lowercase().next().unwrap_or(c) {
        'ё' => 'е',
        lower => lower,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Case::*;
    use Number::*;

    fn regular(stem: &str, ending: &str) -> Split {
        Split::Regular {
            stem: stem.to_owned(),
            ending: ending.to_owned(),
        }
    }

    #[test]
    fn stem_drops_the_nominative_ending() {
        assert_eq!(noun_stem("книга"), "книг");
        assert_eq!(noun_stem("стол"), "стол");
        assert_eq!(noun_stem("окно"), "окн");
        assert_eq!(noun_stem("дверь"), "двер");
        assert_eq!(noun_stem("музей"), "музе");
        assert_eq!(noun_stem("здание"), "здани");
        assert_eq!(noun_stem("кни\u{301}га"), "книг");
        assert_eq!(noun_stem("Москва"), "Москв");
        assert_eq!(noun_stem("ножницы"), "ножниц");
    }

    #[test]
    fn plural_only_nouns() {
        assert_eq!(
            split_noun_form("ножницы", "ножницами", Instrumental, Plural),
            regular("ножниц", "ами")
        );
        assert_eq!(
            split_noun_form("брюки", "брюк", Genitive, Plural),
            regular("брюк", "")
        );
        assert_eq!(
            split_noun_form("деньги", "денег", Genitive, Plural),
            Split::Irregular
        );
    }

    #[test]
    fn proper_nouns_keep_their_capital() {
        assert_eq!(
            split_noun_form("Москва", "Москвой", Instrumental, Singular),
            regular("Москв", "ой")
        );
    }

    #[test]
    fn feminine_a_declension() {
        assert_eq!(
            split_noun_form("книга", "книга", Nominative, Singular),
            regular("книг", "а")
        );
        assert_eq!(
            split_noun_form("книга", "книги", Genitive, Singular),
            regular("книг", "и")
        );
        assert_eq!(
            split_noun_form("книга", "книгу", Accusative, Singular),
            regular("книг", "у")
        );
        assert_eq!(
            split_noun_form("книга", "книгой", Instrumental, Singular),
            regular("книг", "ой")
        );
        assert_eq!(
            split_noun_form("книга", "книг", Genitive, Plural),
            regular("книг", "")
        );
        assert_eq!(
            split_noun_form("книга", "книгам", Dative, Plural),
            regular("книг", "ам")
        );
    }

    #[test]
    fn masculine_zero_ending() {
        assert_eq!(
            split_noun_form("стол", "стол", Nominative, Singular),
            regular("стол", "")
        );
        assert_eq!(
            split_noun_form("стол", "стола", Genitive, Singular),
            regular("стол", "а")
        );
        assert_eq!(
            split_noun_form("стол", "столами", Instrumental, Plural),
            regular("стол", "ами")
        );
        assert_eq!(
            split_noun_form("стол", "столов", Genitive, Plural),
            regular("стол", "ов")
        );
    }

    #[test]
    fn soft_stems() {
        assert_eq!(
            split_noun_form("музей", "музея", Genitive, Singular),
            regular("музе", "я")
        );
        assert_eq!(
            split_noun_form("музей", "музеем", Instrumental, Singular),
            regular("музе", "ем")
        );
        assert_eq!(
            split_noun_form("дверь", "дверью", Instrumental, Singular),
            regular("двер", "ью")
        );
        assert_eq!(
            split_noun_form("дверь", "дверей", Genitive, Plural),
            regular("двер", "ей")
        );
        assert_eq!(
            split_noun_form("неделя", "недель", Genitive, Plural),
            regular("недел", "ь")
        );
    }

    #[test]
    fn nouns_in_ie_and_iya() {
        assert_eq!(
            split_noun_form("здание", "здании", Prepositional, Singular),
            regular("здани", "и")
        );
        assert_eq!(
            split_noun_form("здание", "зданий", Genitive, Plural),
            regular("здани", "й")
        );
        assert_eq!(
            split_noun_form("армия", "армии", Dative, Singular),
            regular("арми", "и")
        );
    }

    #[test]
    fn stress_marks_are_ignored_and_dropped() {
        assert_eq!(
            split_noun_form("кни\u{301}га", "кни\u{301}гой", Instrumental, Singular),
            regular("книг", "ой")
        );
        assert_eq!(
            split_noun_form("окно\u{301}", "о\u{301}кна", Nominative, Plural),
            regular("окн", "а")
        );
    }

    #[test]
    fn yo_alternation_keeps_the_forms_spelling() {
        assert_eq!(
            split_noun_form("ёж", "ежа", Genitive, Singular),
            regular("еж", "а")
        );
        assert_eq!(
            split_noun_form("жена", "жёны", Nominative, Plural),
            regular("жён", "ы")
        );
    }

    #[test]
    fn fleeting_vowels_are_irregular() {
        assert_eq!(
            split_noun_form("отец", "отца", Genitive, Singular),
            Split::Irregular
        );
        assert_eq!(
            split_noun_form("день", "дня", Genitive, Singular),
            Split::Irregular
        );
        assert_eq!(
            split_noun_form("окно", "окон", Genitive, Plural),
            Split::Irregular
        );
    }

    #[test]
    fn stem_changes_and_odd_endings_are_irregular() {
        assert_eq!(
            split_noun_form("имя", "имени", Genitive, Singular),
            Split::Irregular
        );
        assert_eq!(
            split_noun_form("мать", "матери", Genitive, Singular),
            Split::Irregular
        );
        assert_eq!(
            split_noun_form("брат", "братья", Nominative, Plural),
            Split::Irregular
        );
        assert_eq!(
            split_noun_form("человек", "люди", Nominative, Plural),
            Split::Irregular
        );
    }

    #[test]
    fn ending_must_belong_to_the_requested_case() {
        // "а" is a genitive singular ending, not a dative one.
        assert_eq!(
            split_noun_form("стол", "стола", Dative, Singular),
            Split::Irregular
        );
    }
}
