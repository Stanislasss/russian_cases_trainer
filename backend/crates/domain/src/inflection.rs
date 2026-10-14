//! Splitting inflected forms into stem + ending for the drag-the-ending game.
//! Nouns and adjectives take their stem from the dictionary form; pronouns, which
//! don't follow one rule, from the beginning all their forms share.
//!
//! The stem is taken from the dictionary form (lemma) minus its nominative ending,
//! and a form splits cleanly only if it starts with that stem and the rest is a
//! known ending for the requested case and number. Anything else (fleeting vowels
//! as in отец → отца, stem changes as in имя → имени, irregular plurals as in
//! брат → братья) is reported as [`Split::Irregular`] and kept out of easy levels.
//!
//! Endings are deliberately learner-oriented rather than strictly morphological:
//! музей splits as музе|й → музе|я and дверь as двер|ь → двер|и.

use crate::grammar::{Case, Gender, Number};
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
    split_with(&noun_stem(lemma), form, noun_endings(case, number))
}

/// Known adjective endings. Singular forms depend on the gender (`None` =
/// masculine); the masculine accusative is the nominative or genitive form
/// depending on animacy, so both are allowed.
pub fn adjective_endings(
    case: Case,
    number: Number,
    gender: Option<Gender>,
) -> &'static [&'static str] {
    use Case::*;
    match (number, gender) {
        (Number::Plural, _) => match case {
            Nominative => &["ые", "ие"],
            Genitive | Prepositional => &["ых", "их"],
            Dative => &["ым", "им"],
            Accusative => &["ые", "ие", "ых", "их"],
            Instrumental => &["ыми", "ими"],
        },
        (Number::Singular, Some(Gender::Feminine)) => match case {
            Nominative => &["ая", "яя"],
            Accusative => &["ую", "юю"],
            Instrumental => &["ой", "ей", "ою", "ею"],
            Genitive | Dative | Prepositional => &["ой", "ей"],
        },
        (Number::Singular, Some(Gender::Neuter)) => match case {
            Nominative | Accusative => &["ое", "ее"],
            _ => masculine_neuter_oblique(case),
        },
        (Number::Singular, _) => match case {
            Nominative => &["ый", "ий", "ой"],
            Accusative => &["ый", "ий", "ой", "ого", "его"],
            _ => masculine_neuter_oblique(case),
        },
    }
}

fn masculine_neuter_oblique(case: Case) -> &'static [&'static str] {
    match case {
        Case::Genitive => &["ого", "его"],
        Case::Dative => &["ому", "ему"],
        Case::Instrumental => &["ым", "им"],
        Case::Prepositional => &["ом", "ем"],
        Case::Nominative | Case::Accusative => &[],
    }
}

/// новый → нов, синий → син, большой → больш. `None` for anything else
/// (participles in -ся, oddities), whose forms are then irregular.
pub fn adjective_stem(lemma: &str) -> Option<String> {
    let lemma = plain(lemma);
    ["ый", "ий", "ой"]
        .iter()
        .find_map(|ending| lemma.strip_suffix(ending))
        .filter(|stem| !stem.is_empty())
        .map(str::to_owned)
}

pub fn split_adjective_form(
    lemma: &str,
    form: &str,
    case: Case,
    number: Number,
    gender: Option<Gender>,
) -> Split {
    match adjective_stem(lemma) {
        Some(stem) => split_with(&stem, form, adjective_endings(case, number, gender)),
        None => Split::Irregular,
    }
}

/// Endings of the adjective-like pronouns (мо|его, наш|ему, э́т|ого, сам|ом, …).
const PRONOUN_ENDINGS: &[&str] = &[
    "", "й", "а", "я", "о", "е", "ё", "у", "ю", "и", "от", "ого", "его", "ому", "ему", "ом", "ем",
    "ём", "ой", "ей", "ою", "ею", "им", "ым", "их", "ых", "ими", "ыми", "ий", "ый", "ая", "ое",
    "ую", "ие", "ые",
];

/// The longest beginning shared by all `forms` of a word, e.g. мо for мой,
/// моего́, мое́й, … Pronouns don't follow one lemma rule, so their stem comes from
/// their forms. Comparison ignores stress, case and ё/е; the result is plain.
pub fn common_stem<S: AsRef<str>>(forms: &[S]) -> String {
    let mut forms = forms.iter().map(|f| plain(f.as_ref()));
    let Some(first) = forms.next() else {
        return String::new();
    };
    let mut stem: Vec<char> = first.chars().collect();
    for form in forms {
        let shared = stem
            .iter()
            .zip(form.chars())
            .take_while(|&(&a, b)| fold(a) == fold(b))
            .count();
        stem.truncate(shared);
    }
    stem.into_iter().collect()
}

/// Splits a pronoun form after its `common_stem`. A one-letter stem (т|ого,
/// в|сего) teaches nothing, so such pronouns count as irregular.
pub fn split_pronoun_form(stem: &str, form: &str) -> Split {
    if stem.chars().count() < 2 {
        return Split::Irregular;
    }
    split_with(stem, form, PRONOUN_ENDINGS)
}

/// `form` splits if it starts with `stem` and the rest is one of `endings`.
fn split_with(stem: &str, form: &str, endings: &[&str]) -> Split {
    let stem: Vec<char> = stem.chars().collect();
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
    if endings.contains(&ending.as_str()) {
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

    #[test]
    fn adjective_stems() {
        assert_eq!(adjective_stem("но\u{301}вый").as_deref(), Some("нов"));
        assert_eq!(adjective_stem("си\u{301}ний").as_deref(), Some("син"));
        assert_eq!(adjective_stem("большо\u{301}й").as_deref(), Some("больш"));
        assert_eq!(adjective_stem("являющийся"), None);
    }

    #[test]
    fn adjectives_split_by_gender() {
        use Gender::*;
        let split =
            |form, case, number, gender| split_adjective_form("новый", form, case, number, gender);
        assert_eq!(
            split("нового", Genitive, Singular, None),
            regular("нов", "ого")
        );
        assert_eq!(
            split("новой", Genitive, Singular, Some(Feminine)),
            regular("нов", "ой")
        );
        assert_eq!(
            split("новое", Accusative, Singular, Some(Neuter)),
            regular("нов", "ое")
        );
        assert_eq!(
            split("новыми", Instrumental, Plural, None),
            regular("нов", "ыми")
        );
        // A feminine ending isn't a masculine one.
        assert_eq!(
            split("новой", Genitive, Singular, Some(Masculine)),
            Split::Irregular
        );
        assert_eq!(
            split_adjective_form("синий", "синюю", Accusative, Singular, Some(Feminine)),
            regular("син", "юю")
        );
        assert_eq!(
            split_adjective_form("русский", "русского", Accusative, Singular, Some(Masculine)),
            regular("русск", "ого")
        );
    }

    #[test]
    fn pronoun_stems_come_from_their_forms() {
        let moy = [
            "мо\u{301}й",
            "моего\u{301}",
            "моему\u{301}",
            "моём",
            "моя\u{301}",
            "мои\u{301}ми",
        ];
        assert_eq!(common_stem(&moy), "мо");
        assert_eq!(
            split_pronoun_form("мо", "моего\u{301}"),
            regular("мо", "его")
        );
        assert_eq!(split_pronoun_form("мо", "мой"), regular("мо", "й"));
        assert_eq!(
            common_stem(&["э\u{301}тот", "э\u{301}того", "э\u{301}та"]),
            "эт"
        );
        assert_eq!(split_pronoun_form("эт", "этот"), regular("эт", "от"));
        assert_eq!(common_stem(&["наш", "на\u{301}шего", "на\u{301}ша"]), "наш");
        assert_eq!(split_pronoun_form("наш", "наш"), regular("наш", ""));
    }

    #[test]
    fn one_letter_pronoun_stems_are_irregular() {
        let tot = ["тот", "того\u{301}", "те"];
        assert_eq!(common_stem(&tot), "т");
        assert_eq!(split_pronoun_form("т", "того"), Split::Irregular);
        assert_eq!(common_stem::<&str>(&[]), "");
    }
}
