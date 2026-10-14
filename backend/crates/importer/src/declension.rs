//! Building the forms of adjective-like words (adjectives and the pronouns that
//! decline like them) from a table of cells, whatever the table's source.

use domain::grammar::{Animacy, Case, Gender, Number, PartOfSpeech};
use domain::inflection::common_stem;
use domain::lexicon::{NewForm, Slot};

/// The four columns of an adjective paradigm, with their OpenRussian names.
pub const COLUMNS: [(Number, Option<Gender>, &str); 4] = [
    (Number::Singular, Some(Gender::Masculine), "m"),
    (Number::Singular, Some(Gender::Feminine), "f"),
    (Number::Singular, Some(Gender::Neuter), "n"),
    (Number::Plural, None, "pl"),
];

pub const CASES: [(Case, &str); 6] = [
    (Case::Nominative, "nom"),
    (Case::Genitive, "gen"),
    (Case::Dative, "dat"),
    (Case::Accusative, "acc"),
    (Case::Instrumental, "inst"),
    (Case::Prepositional, "prep"),
];

/// All forms of an adjective-like word. `cell(number, gender, case)` returns the
/// stressed variants of one form.
///
/// The masculine and plural accusative is the nominative form with inanimate
/// nouns and the genitive form with animate ones, so those slots are built from
/// the nominative and genitive cells (the accusative cells list both in no
/// reliable order). Pronouns are split after the stem shared by all their forms,
/// adjectives against their lemma.
pub fn adjective_like_forms(
    lemma: &str,
    pos: PartOfSpeech,
    cell: impl Fn(Number, Option<Gender>, Case) -> Vec<String>,
) -> Vec<NewForm> {
    let mut slots: Vec<(Slot, Vec<String>)> = Vec::new();
    for (number, gender, _) in COLUMNS {
        for (case, _) in CASES {
            let slot = Slot {
                gender,
                ..Slot::new(case, number)
            };
            // Masculine singular, or plural (no gender).
            let depends_on_animacy =
                case == Case::Accusative && matches!(gender, Some(Gender::Masculine) | None);
            if depends_on_animacy {
                slots.push((
                    slot.animacy(Animacy::Inanimate),
                    cell(number, gender, Case::Nominative),
                ));
                slots.push((
                    slot.animacy(Animacy::Animate),
                    cell(number, gender, Case::Genitive),
                ));
            } else {
                slots.push((slot, cell(number, gender, case)));
            }
        }
    }

    let stem = if pos == PartOfSpeech::Pronoun {
        let all: Vec<&String> = slots.iter().flat_map(|(_, forms)| forms).collect();
        common_stem(&all)
    } else {
        String::new()
    };
    slots
        .iter()
        .flat_map(|(slot, forms)| {
            let stem = &stem;
            (0..).zip(forms).map(move |(variant, form)| match pos {
                PartOfSpeech::Pronoun => NewForm::pronoun(stem, form, *slot, variant),
                _ => NewForm::adjective(lemma, form, *slot, variant),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The masculine/plural columns of но́вый; other cells empty.
    fn novy(number: Number, gender: Option<Gender>, case: Case) -> Vec<String> {
        let form = match (number, gender, case) {
            (Number::Singular, Some(Gender::Masculine), Case::Nominative) => "но\u{301}вый",
            (Number::Singular, Some(Gender::Masculine), Case::Genitive) => "но\u{301}вого",
            // Listed in the "wrong" order on purpose: the cell is ignored.
            (Number::Singular, Some(Gender::Masculine), Case::Accusative) => {
                "но\u{301}вого, но\u{301}вый"
            }
            (Number::Plural, _, Case::Nominative) => "но\u{301}вые",
            (Number::Plural, _, Case::Genitive) => "но\u{301}вых",
            _ => return Vec::new(),
        };
        form.split(", ").map(str::to_owned).collect()
    }

    fn find(forms: &[NewForm], slot: Slot) -> Vec<&str> {
        forms
            .iter()
            .filter(|f| f.slot == slot)
            .map(|f| f.form.as_str())
            .collect()
    }

    #[test]
    fn accusative_follows_animacy() {
        let forms = adjective_like_forms("новый", PartOfSpeech::Adjective, novy);
        let acc = Slot::new(Case::Accusative, Number::Singular).gender(Gender::Masculine);
        assert_eq!(find(&forms, acc.animacy(Animacy::Inanimate)), ["новый"]);
        assert_eq!(find(&forms, acc.animacy(Animacy::Animate)), ["нового"]);
        let acc_pl = Slot::new(Case::Accusative, Number::Plural);
        assert_eq!(find(&forms, acc_pl.animacy(Animacy::Animate)), ["новых"]);
        assert!(forms.iter().all(|f| !f.irregular));
    }

    #[test]
    fn pronouns_split_after_their_common_stem() {
        let nash = |number, gender, case| -> Vec<String> {
            let form = match (number, gender, case) {
                (Number::Singular, Some(Gender::Masculine), Case::Nominative) => "наш",
                (Number::Singular, Some(Gender::Masculine), Case::Genitive) => "на\u{301}шего",
                (Number::Singular, Some(Gender::Feminine), Case::Nominative) => "на\u{301}ша",
                _ => return Vec::new(),
            };
            vec![form.to_owned()]
        };
        let forms = adjective_like_forms("наш", PartOfSpeech::Pronoun, nash);
        let genitive = forms.iter().find(|f| f.form == "нашего").unwrap();
        assert_eq!(
            (genitive.stem.as_deref(), genitive.ending.as_deref()),
            (Some("наш"), Some("его"))
        );
    }
}
