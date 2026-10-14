//! Which declension pattern ("paradigm") a word follows. The rules pages and the
//! wrong-answer explanations show the ending table of the word's paradigm.

use crate::grammar::{Gender, PartOfSpeech, string_enum};
use crate::stress;

string_enum! {
    Paradigm, "paradigm" {
        /// стол, оте́ц: masculine, ending in a consonant (not ж ш щ ч).
        NounMasculineHard => "noun_masculine_hard",
        /// нож, врач, това́рищ: masculine in ж ш щ ч.
        NounMasculineSibilant => "noun_masculine_sibilant",
        /// слова́рь, учи́тель: masculine in -ь.
        NounMasculineSoftSign => "noun_masculine_soft_sign",
        /// музе́й, геро́й: masculine in -й.
        NounMasculineJ => "noun_masculine_j",
        /// ге́ний, санато́рий: masculine in -ий (prepositional -ии).
        NounMasculineIy => "noun_masculine_iy",
        /// окно́, ме́сто: neuter in -о.
        NounNeuterO => "noun_neuter_o",
        /// по́ле, мо́ре, ружьё: neuter in -е/-ё.
        NounNeuterE => "noun_neuter_e",
        /// зда́ние: neuter in -ие.
        NounNeuterIe => "noun_neuter_ie",
        /// кни́га, неде́ля, па́па: nouns in -а/-я, mostly feminine.
        NounA => "noun_a",
        /// а́рмия, ста́нция: nouns in -ия.
        NounIya => "noun_iya",
        /// дверь, ночь: feminine in -ь.
        NounFeminineSoftSign => "noun_feminine_soft_sign",
        /// но́жницы, брю́ки: only used in the plural.
        NounPluralOnly => "noun_plural_only",
        /// но́вый; also ру́сский, хоро́ший (spelling rules).
        AdjectiveHard => "adjective_hard",
        /// си́ний.
        AdjectiveSoft => "adjective_soft",
        /// большо́й, друго́й: stressed ending -ой.
        AdjectiveStressed => "adjective_stressed",
        /// мой, твой, свой.
        PronounPossessive => "pronoun_possessive",
        /// наш, ваш.
        PronounNash => "pronoun_nash",
        /// э́тот.
        PronounEtot => "pronoun_etot",
        /// я, ты, он, …: whole forms.
        PersonalPronoun => "personal_pronoun",
        /// Ивано́в, Пу́шкин: -ов/-ев/-ин.
        SurnameMixed => "surname_mixed",
        /// Чайко́вский, Толсто́й: like adjectives.
        SurnameAdjectival => "surname_adjectival",
    }
}

const VELARS_AND_SIBILANTS: &[char] = &['г', 'к', 'х', 'ж', 'ш', 'щ', 'ч'];
const SIBILANTS: &[char] = &['ж', 'ш', 'щ', 'ч'];

/// The paradigm of a word, from its part of speech, dictionary form and gender.
/// `None` for words that follow none of the tables (имя, тот, весь, …).
pub fn paradigm(pos: PartOfSpeech, lemma: &str, gender: Option<Gender>) -> Option<Paradigm> {
    let lemma = stress::strip(lemma).to_lowercase();
    match pos {
        PartOfSpeech::Noun => noun_paradigm(&lemma, gender),
        PartOfSpeech::Adjective => adjective_paradigm(&lemma),
        PartOfSpeech::Pronoun => match lemma.as_str() {
            "мой" | "твой" | "свой" => Some(Paradigm::PronounPossessive),
            "наш" | "ваш" => Some(Paradigm::PronounNash),
            "этот" => Some(Paradigm::PronounEtot),
            // како́й, кото́рый, ка́ждый, … decline like adjectives.
            _ => adjective_paradigm(&lemma),
        },
        PartOfSpeech::PersonalPronoun => Some(Paradigm::PersonalPronoun),
        PartOfSpeech::Surname => Some(if lemma.ends_with("ий") || lemma.ends_with("ой") {
            Paradigm::SurnameAdjectival
        } else {
            Paradigm::SurnameMixed
        }),
    }
}

fn noun_paradigm(lemma: &str, gender: Option<Gender>) -> Option<Paradigm> {
    use Paradigm::*;
    let ends = |suffix: &str| lemma.ends_with(suffix);
    let last = lemma.chars().last()?;
    Some(if ends("мя") && gender == Some(Gender::Neuter) {
        return None; // и́мя → и́мени: irregular
    } else if ends("ия") {
        NounIya
    } else if ends("ие") {
        NounNeuterIe
    } else if ends("ий") {
        NounMasculineIy
    } else if matches!(last, 'а' | 'я') {
        NounA
    } else if matches!(last, 'ы' | 'и') {
        NounPluralOnly
    } else if last == 'о' {
        NounNeuterO
    } else if matches!(last, 'е' | 'ё') {
        NounNeuterE
    } else if last == 'ь' {
        match gender? {
            Gender::Feminine => NounFeminineSoftSign,
            Gender::Masculine => NounMasculineSoftSign,
            Gender::Neuter => return None,
        }
    } else if last == 'й' {
        NounMasculineJ
    } else if SIBILANTS.contains(&last) {
        NounMasculineSibilant
    } else {
        NounMasculineHard
    })
}

fn adjective_paradigm(lemma: &str) -> Option<Paradigm> {
    if lemma.ends_with("ой") {
        Some(Paradigm::AdjectiveStressed)
    } else if lemma.ends_with("ый") {
        Some(Paradigm::AdjectiveHard)
    } else if let Some(stem) = lemma.strip_suffix("ий") {
        // ру́сский, хоро́ший are hard; only the spelling rules make them look soft.
        let before = stem.chars().last()?;
        Some(if VELARS_AND_SIBILANTS.contains(&before) {
            Paradigm::AdjectiveHard
        } else {
            Paradigm::AdjectiveSoft
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Paradigm::*;
    use PartOfSpeech::*;

    fn noun(lemma: &str, gender: Gender) -> Option<Paradigm> {
        paradigm(Noun, lemma, Some(gender))
    }

    #[test]
    fn nouns() {
        use Gender::*;
        assert_eq!(noun("стол", Masculine), Some(NounMasculineHard));
        assert_eq!(noun("оте\u{301}ц", Masculine), Some(NounMasculineHard));
        assert_eq!(noun("нож", Masculine), Some(NounMasculineSibilant));
        assert_eq!(noun("врач", Masculine), Some(NounMasculineSibilant));
        assert_eq!(noun("слова́рь", Masculine), Some(NounMasculineSoftSign));
        assert_eq!(noun("музей", Masculine), Some(NounMasculineJ));
        assert_eq!(noun("гений", Masculine), Some(NounMasculineIy));
        assert_eq!(noun("окно", Neuter), Some(NounNeuterO));
        assert_eq!(noun("поле", Neuter), Some(NounNeuterE));
        assert_eq!(noun("ружьё", Neuter), Some(NounNeuterE));
        assert_eq!(noun("здание", Neuter), Some(NounNeuterIe));
        assert_eq!(noun("книга", Feminine), Some(NounA));
        assert_eq!(noun("неделя", Feminine), Some(NounA));
        assert_eq!(noun("папа", Masculine), Some(NounA));
        assert_eq!(noun("армия", Feminine), Some(NounIya));
        assert_eq!(noun("дверь", Feminine), Some(NounFeminineSoftSign));
        assert_eq!(paradigm(Noun, "ножницы", None), Some(NounPluralOnly));
    }

    #[test]
    fn irregular_nouns_have_no_paradigm() {
        assert_eq!(noun("имя", Gender::Neuter), None);
        assert_eq!(paradigm(Noun, "сирота", None), Some(NounA)); // common gender in -а is fine
        assert_eq!(paradigm(Noun, "тюль", None), None); // -ь without gender
    }

    #[test]
    fn adjectives() {
        assert_eq!(paradigm(Adjective, "новый", None), Some(AdjectiveHard));
        assert_eq!(paradigm(Adjective, "русский", None), Some(AdjectiveHard));
        assert_eq!(paradigm(Adjective, "хороший", None), Some(AdjectiveHard));
        assert_eq!(paradigm(Adjective, "синий", None), Some(AdjectiveSoft));
        assert_eq!(
            paradigm(Adjective, "большо\u{301}й", None),
            Some(AdjectiveStressed)
        );
        assert_eq!(paradigm(Adjective, "являющийся", None), None);
    }

    #[test]
    fn pronouns_and_surnames() {
        assert_eq!(paradigm(Pronoun, "мой", None), Some(PronounPossessive));
        assert_eq!(paradigm(Pronoun, "ваш", None), Some(PronounNash));
        assert_eq!(paradigm(Pronoun, "этот", None), Some(PronounEtot));
        assert_eq!(paradigm(Pronoun, "какой", None), Some(AdjectiveStressed));
        assert_eq!(paradigm(Pronoun, "который", None), Some(AdjectiveHard));
        assert_eq!(paradigm(Pronoun, "тот", None), None);
        assert_eq!(
            paradigm(PartOfSpeech::PersonalPronoun, "я", None),
            Some(Paradigm::PersonalPronoun)
        );
        assert_eq!(paradigm(Surname, "Иванов", None), Some(SurnameMixed));
        assert_eq!(paradigm(Surname, "Пушкин", None), Some(SurnameMixed));
        assert_eq!(
            paradigm(Surname, "Чайковский", None),
            Some(SurnameAdjectival)
        );
        assert_eq!(paradigm(Surname, "Толстой", None), Some(SurnameAdjectival));
    }
}
