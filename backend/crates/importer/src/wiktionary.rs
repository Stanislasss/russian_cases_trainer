//! Translations and definitions from Wiktionary, via kaikki.org's extracts of its
//! Russian entries (CC BY-SA): French translations from the French Wiktionary,
//! Russian definitions from the Russian one. One JSON object per line; only the
//! word, its part of speech and its definitions ("glosses") are read.

use std::collections::HashMap;
use std::io::BufRead;

use domain::grammar::{Locale, PartOfSpeech};
use domain::lexicon::{NewLexeme, NewTranslation};
use serde::Deserialize;

/// A Wiktionary language edition and how to tidy its definitions.
pub struct Edition {
    pub locale: Locale,
    pub url: &'static str,
    /// File name in the download cache.
    pub cache_file: &'static str,
    /// French glosses start like sentences ("Livre."), but are used as translations.
    lowercase_first: bool,
    /// Longer definitions are explanations rather than something to show on a card.
    max_chars: usize,
    /// Meanings kept per word. Russian definitions are whole sentences, so one
    /// fits a card; short translations can list a few.
    max_meanings: usize,
}

pub const FRENCH: Edition = Edition {
    locale: Locale::Fr,
    url: "https://kaikki.org/frwiktionary/Russe/kaikki.org-dictionary-Russe.jsonl",
    cache_file: "fr-wiktionary-russe.jsonl",
    lowercase_first: true,
    max_chars: 80,
    max_meanings: 3,
};

pub const RUSSIAN: Edition = Edition {
    locale: Locale::Ru,
    url: "https://kaikki.org/ruwiktionary/%D0%A0%D1%83%D1%81%D1%81%D0%BA%D0%B8%D0%B9/kaikki.org-dictionary-%D0%A0%D1%83%D1%81%D1%81%D0%BA%D0%B8%D0%B9.jsonl",
    cache_file: "ru-wiktionary-russkiy.jsonl",
    lowercase_first: false,
    max_chars: 120,
    max_meanings: 1,
};

#[derive(Deserialize)]
struct Entry {
    word: String,
    pos: String,
    #[serde(default)]
    senses: Vec<Sense>,
}

#[derive(Deserialize)]
struct Sense {
    #[serde(default)]
    glosses: Vec<String>,
    /// Set on senses like "Génitif singulier de …", which only point to another word.
    #[serde(default)]
    form_of: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Group {
    Noun,
    Adjective,
    Pronoun,
}

fn group_of_entry(pos: &str) -> Option<Group> {
    match pos {
        "noun" => Some(Group::Noun),
        "adj" => Some(Group::Adjective),
        "pron" => Some(Group::Pronoun),
        _ => None,
    }
}

/// Where to look a lexeme up, in order. Possessives like мой are filed under
/// adjectives in some editions and pronouns in others.
fn groups_of_lexeme(pos: PartOfSpeech) -> &'static [Group] {
    match pos {
        PartOfSpeech::Noun => &[Group::Noun],
        PartOfSpeech::Adjective => &[Group::Adjective],
        PartOfSpeech::Pronoun => &[Group::Adjective, Group::Pronoun],
        PartOfSpeech::PersonalPronoun => &[Group::Pronoun],
        PartOfSpeech::Surname => &[],
    }
}

/// Meanings by word in one language, read from a JSONL extract. Unparsable lines are skipped.
pub struct Dictionary {
    locale: Locale,
    meanings: HashMap<(Group, String), Vec<String>>,
}

impl Dictionary {
    pub fn parse(edition: &Edition, reader: impl BufRead) -> std::io::Result<Self> {
        let mut meanings: HashMap<(Group, String), Vec<String>> = HashMap::new();
        for line in reader.lines() {
            let line = line?;
            let Ok(entry) = serde_json::from_str::<Entry>(&line) else {
                continue;
            };
            let Some(group) = group_of_entry(&entry.pos) else {
                continue;
            };
            let found = meanings.entry((group, entry.word)).or_default();
            let glosses = entry
                .senses
                .iter()
                .filter(|s| s.form_of.is_empty())
                .filter_map(|s| s.glosses.first())
                .filter_map(|g| clean(edition, g));
            for gloss in glosses {
                if found.len() < edition.max_meanings && !found.contains(&gloss) {
                    found.push(gloss);
                }
            }
        }
        meanings.retain(|_, m| !m.is_empty());
        Ok(Dictionary {
            locale: edition.locale,
            meanings,
        })
    }

    pub fn len(&self) -> usize {
        self.meanings.len()
    }

    /// Adds meanings to lexemes that have none in this language yet; returns how many got some.
    pub fn translate(&self, lexemes: &mut [NewLexeme]) -> usize {
        let mut translated = 0;
        for lexeme in lexemes {
            if lexeme.translations.iter().any(|t| t.locale == self.locale) {
                continue;
            }
            let found = groups_of_lexeme(lexeme.pos)
                .iter()
                .find_map(|&group| self.meanings.get(&(group, lexeme.lemma.clone())));
            let Some(meanings) = found else {
                continue;
            };
            lexeme
                .translations
                .extend((0..).zip(meanings).map(|(position, text)| NewTranslation {
                    locale: self.locale,
                    position,
                    text: text.clone(),
                }));
            translated += 1;
        }
        translated
    }
}

/// "Livre." → "livre"; `None` for empty or overly long definitions.
fn clean(edition: &Edition, gloss: &str) -> Option<String> {
    let gloss = gloss.trim().trim_end_matches('.').trim();
    if gloss.is_empty() || gloss.chars().count() > edition.max_chars {
        return None;
    }
    let mut chars = gloss.chars();
    let first = chars.next()?;
    // Lowercase a capitalised first word, but leave acronyms and names alone.
    let second_is_lower = chars.next().is_some_and(char::is_lowercase);
    Some(
        if edition.lowercase_first && first.is_uppercase() && second_is_lower {
            first.to_lowercase().chain(gloss.chars().skip(1)).collect()
        } else {
            gloss.to_owned()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRENCH_SAMPLE: &str = r#"{"word": "книга", "pos": "noun", "senses": [{"glosses": ["Livre."]}]}
{"word": "мой", "pos": "verb", "senses": [{"glosses": ["Impératif singulier de мыть."], "form_of": [{"word": "мыть"}]}]}
{"word": "мой", "pos": "adj", "senses": [{"glosses": ["Mon."]}]}
{"word": "время", "pos": "noun", "senses": [{"glosses": ["Temps qui passe, époque, saison."]}, {"glosses": ["Heure, moment."]}, {"glosses": ["Temps."]}, {"glosses": ["Quatrième sens."]}]}
{"word": "книги", "pos": "noun", "senses": [{"glosses": ["Génitif singulier de книга."], "form_of": [{"word": "книга"}]}]}
not json
{"word": "стол", "pos": "noun", "senses": [{"glosses": ["Table."]}]}"#;

    const RUSSIAN_SAMPLE: &str = r#"{"word": "мальчик", "pos": "noun", "senses": [{"glosses": ["ребёнок либо подросток мужского пола"]}, {"glosses": ["разг. то же, что сын"]}]}
{"word": "я", "pos": "pron", "senses": [{"glosses": ["Указывает на говорящего"]}]}"#;

    fn lexeme(lemma: &str, pos: PartOfSpeech) -> NewLexeme {
        NewLexeme {
            source: "test".into(),
            source_id: lemma.into(),
            lemma: lemma.into(),
            stressed_lemma: None,
            pos,
            gender: None,
            animacy: None,
            indeclinable: false,
            frequency_rank: None,
            forms: Vec::new(),
            translations: Vec::new(),
        }
    }

    fn texts(l: &NewLexeme, locale: Locale) -> Vec<&str> {
        l.translations
            .iter()
            .filter(|t| t.locale == locale)
            .map(|t| t.text.as_str())
            .collect()
    }

    #[test]
    fn french_meanings_skip_form_of_senses() {
        let dict = Dictionary::parse(&FRENCH, FRENCH_SAMPLE.as_bytes()).unwrap();
        // книги only had a form-of sense; the verb мой isn't a noun or adjective.
        assert_eq!(dict.len(), 4);

        let mut words = vec![
            lexeme("книга", PartOfSpeech::Noun),
            lexeme("мой", PartOfSpeech::Pronoun),
            lexeme("время", PartOfSpeech::Noun),
            lexeme("стол", PartOfSpeech::Adjective), // wrong group: no match
            lexeme("Иванов", PartOfSpeech::Surname),
        ];
        assert_eq!(dict.translate(&mut words), 3);
        assert_eq!(texts(&words[0], Locale::Fr), ["livre"]);
        assert_eq!(texts(&words[1], Locale::Fr), ["mon"]);
        assert_eq!(
            texts(&words[2], Locale::Fr),
            ["temps qui passe, époque, saison", "heure, moment", "temps"]
        );
        assert!(texts(&words[3], Locale::Fr).is_empty());
    }

    #[test]
    fn russian_definitions_keep_their_case() {
        let dict = Dictionary::parse(&RUSSIAN, RUSSIAN_SAMPLE.as_bytes()).unwrap();
        let mut words = vec![
            lexeme("мальчик", PartOfSpeech::Noun),
            lexeme("я", PartOfSpeech::PersonalPronoun),
        ];
        assert_eq!(dict.translate(&mut words), 2);
        assert_eq!(
            texts(&words[0], Locale::Ru),
            ["ребёнок либо подросток мужского пола"] // one definition per word
        );
        assert_eq!(texts(&words[1], Locale::Ru), ["Указывает на говорящего"]);
    }

    #[test]
    fn existing_meanings_are_kept() {
        let dict = Dictionary::parse(&FRENCH, FRENCH_SAMPLE.as_bytes()).unwrap();
        let mut kniga = lexeme("книга", PartOfSpeech::Noun);
        kniga.translations.push(NewTranslation {
            locale: Locale::Fr,
            position: 0,
            text: "bouquin".into(),
        });
        let mut words = [kniga];
        assert_eq!(dict.translate(&mut words), 0);
        assert_eq!(texts(&words[0], Locale::Fr), ["bouquin"]);
    }

    #[test]
    fn glosses_are_tidied() {
        assert_eq!(clean(&FRENCH, "Livre.").as_deref(), Some("livre"));
        assert_eq!(clean(&FRENCH, "URSS.").as_deref(), Some("URSS"));
        assert_eq!(clean(&FRENCH, "  "), None);
        assert_eq!(clean(&FRENCH, &"x".repeat(100)), None);
        assert_eq!(
            clean(&RUSSIAN, &"x".repeat(100)).map(|s| s.len()),
            Some(100)
        );
    }
}
