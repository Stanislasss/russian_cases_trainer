//! The translations' nouns as nouns in their own language: gender and plural
//! of German and French words (Hund: masculine, Hunde), from kaikki.org's
//! extracts of the English Wiktionary (CC BY-SA), so translated sentences can
//! put them in with the right article. English needs no gender, and its plurals
//! mostly follow rules (`domain::noun_phrase`), so it has no download.
//!
//! One JSON object per line; only the word, its head line ("Hund m (strong,
//! genitive Hundes, plural Hunde)"), its forms and whether it is just a form of
//! another word are read.

use std::collections::{HashMap, HashSet};
use std::io::BufRead;

use domain::grammar::{Gender, Locale, PartOfSpeech};
use domain::lexicon::NewLexeme;
use domain::noun_phrase::{NounForms, pluralize_phrase};
use serde::Deserialize;

/// The languages translated sentences are shown in.
const LOCALES: [Locale; 3] = [Locale::En, Locale::Fr, Locale::De];

/// A noun list for one language.
pub struct Source {
    pub locale: Locale,
    pub url: &'static str,
    /// File name in the download cache.
    pub cache_file: &'static str,
}

pub const GERMAN: Source = Source {
    locale: Locale::De,
    url: "https://kaikki.org/dictionary/German/pos-noun/kaikki.org-dictionary-German-by-pos-noun.jsonl",
    cache_file: "en-wiktionary-german-nouns.jsonl",
};

pub const FRENCH: Source = Source {
    locale: Locale::Fr,
    url: "https://kaikki.org/dictionary/French/pos-noun/kaikki.org-dictionary-French-by-pos-noun.jsonl",
    cache_file: "en-wiktionary-french-nouns.jsonl",
};

#[derive(Deserialize)]
struct Entry {
    word: String,
    #[serde(default)]
    pos: String,
    #[serde(default)]
    head_templates: Vec<Head>,
    #[serde(default)]
    forms: Vec<Form>,
    #[serde(default)]
    senses: Vec<Sense>,
}

#[derive(Deserialize)]
struct Head {
    #[serde(default)]
    expansion: String,
}

#[derive(Deserialize)]
struct Form {
    form: String,
    #[serde(default)]
    tags: Vec<String>,
    /// "declension" for the forms of the declension table; none for the head line's.
    #[serde(default)]
    source: Option<String>,
}

#[derive(Deserialize)]
struct Sense {
    #[serde(default)]
    tags: Vec<String>,
}

/// What the noun list says about one word.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Grammar {
    gender: Option<Gender>,
    plural_only: bool,
    plural: Option<String>,
    oblique: Option<String>,
}

impl Grammar {
    fn of(entry: &Entry) -> Self {
        let head = entry
            .head_templates
            .first()
            .map(|h| h.expansion.as_str())
            .unwrap_or_default();
        // "Hund m (strong, …)", "ciseaux m pl", "élève m or f (plural élèves)"
        let marks = head
            .strip_prefix(entry.word.as_str())
            .unwrap_or(head)
            .split('(')
            .next()
            .unwrap_or_default();
        let marks: Vec<&str> = marks.split_whitespace().collect();
        let gender = marks.iter().find_map(|m| match *m {
            "m" => Some(Gender::Masculine),
            "f" => Some(Gender::Feminine),
            "n" => Some(Gender::Neuter),
            _ => None,
        });
        // Other genders of the word (Hündin) and the like aren't its plural.
        const NOT_PLURAL: &[&str] = &[
            "feminine",
            "masculine",
            "neuter",
            "diminutive",
            "augmentative",
            "genitive",
            "dative",
            "accusative",
            "archaic",
            "dated",
            "rare",
        ];
        let plural = entry
            .forms
            .iter()
            .filter(|f| f.source.is_none())
            .find(|f| {
                f.tags.iter().any(|t| t == "plural")
                    && !f.tags.iter().any(|t| NOT_PLURAL.contains(&t.as_str()))
                    && usable(&f.form)
            })
            .map(|f| f.form.clone());
        // German weak nouns: den Jungen.
        let oblique = entry
            .forms
            .iter()
            .filter(|f| f.source.as_deref() == Some("declension"))
            .find(|f| {
                f.tags.iter().any(|t| t == "accusative") && f.tags.iter().any(|t| t == "singular")
            })
            .map(|f| f.form.clone())
            .filter(|form| usable(form) && *form != entry.word);
        Grammar {
            gender,
            plural_only: marks.contains(&"pl"),
            plural,
            oblique,
        }
    }
}

fn usable(form: &str) -> bool {
    !form.is_empty() && form != "-" && !form.starts_with("no-")
}

/// Genders and plurals by word in one language. Unparsable lines are skipped.
pub struct Nouns {
    locale: Locale,
    words: HashMap<String, Grammar>,
}

impl Nouns {
    /// Reads the words in `wanted` from a JSONL noun list. A word's first noun
    /// entry with a gender wins (der Leiter before die Leiter); entries that
    /// are only inflected forms of other words (Hunde) don't count. Wiktionary
    /// also files words like Lehrer ("agent noun of lehren") as forms of others,
    /// but those have a head line with a gender of their own.
    pub fn parse(
        source: &Source,
        reader: impl BufRead,
        wanted: &HashSet<String>,
    ) -> std::io::Result<Self> {
        let mut words: HashMap<String, Grammar> = HashMap::new();
        for line in reader.lines() {
            let line = line?;
            let Ok(entry) = serde_json::from_str::<Entry>(&line) else {
                continue;
            };
            if entry.pos != "noun" || !wanted.contains(&entry.word) || entry.senses.is_empty() {
                continue;
            }
            let grammar = Grammar::of(&entry);
            let only_a_form = entry
                .senses
                .iter()
                .all(|s| s.tags.iter().any(|t| t == "form-of"));
            if only_a_form && grammar.gender.is_none() && !grammar.plural_only {
                continue;
            }
            // A later entry wins only with more to say: a gender where there was
            // none, or a plural (der Nutzen) over a verb used as a noun without
            // one (das Nutzen).
            let better = match words.get(&entry.word) {
                None => true,
                Some(found) => {
                    grammar.gender.is_some()
                        && (found.gender.is_none()
                            || (found.plural.is_none()
                                && !found.plural_only
                                && grammar.plural.is_some()))
                }
            };
            if better {
                words.insert(entry.word, grammar);
            }
        }
        Ok(Nouns {
            locale: source.locale,
            words,
        })
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Fills in the gender and plural of the lexemes' noun forms in this
    /// language; returns how many were found.
    pub fn apply(&self, lexemes: &mut [NewLexeme]) -> usize {
        let mut found = 0;
        for noun in lexemes
            .iter_mut()
            .flat_map(|l| &mut l.noun_forms)
            .filter(|n| n.locale == self.locale)
        {
            let head = noun.head_word();
            let Some(grammar) = self
                .words
                .get(head)
                .or_else(|| self.words.get(&head.to_lowercase()))
            else {
                continue;
            };
            let single_word = !noun.singular.contains(' ');
            // An article in the meaning (der Rat) beats the dictionary's first entry.
            noun.gender = noun.gender.or(grammar.gender);
            noun.plural_only |= grammar.plural_only;
            noun.plural = match &grammar.plural {
                Some(plural) if single_word => Some(plural.clone()),
                plural => Some(pluralize_phrase(
                    self.locale,
                    &noun.singular,
                    plural.as_deref(),
                )),
            };
            if single_word {
                noun.oblique = grammar.oblique.clone();
            }
            found += 1;
        }
        found
    }
}

/// Gives every noun its first meaning in each sentence language as a noun
/// there, as far as the meaning tells (the dictionaries add more later).
pub fn add_noun_forms(lexemes: &mut [NewLexeme]) {
    for lexeme in lexemes.iter_mut().filter(|l| l.pos == PartOfSpeech::Noun) {
        lexeme.noun_forms = LOCALES
            .iter()
            .filter_map(|&locale| {
                let first = lexeme
                    .translations
                    .iter()
                    .filter(|t| t.locale == locale)
                    .min_by_key(|t| t.position)?;
                NounForms::from_meaning(locale, &first.text)
            })
            .collect();
    }
}

/// The words to look up in `locale`'s noun list.
pub fn wanted(lexemes: &[NewLexeme], locale: Locale) -> HashSet<String> {
    lexemes
        .iter()
        .flat_map(|l| &l.noun_forms)
        .filter(|n| n.locale == locale)
        .flat_map(|n| {
            let head = n.head_word();
            [head.to_owned(), head.to_lowercase()]
        })
        .collect()
}

/// Per locale: (nouns, with a gender, with a plural from a dictionary).
pub fn coverage(lexemes: &[NewLexeme]) -> Vec<(&'static str, usize, usize, usize)> {
    LOCALES
        .iter()
        .map(|&locale| {
            let nouns: Vec<&NounForms> = lexemes
                .iter()
                .flat_map(|l| &l.noun_forms)
                .filter(|n| n.locale == locale)
                .collect();
            (
                locale.as_str(),
                nouns.len(),
                nouns.iter().filter(|n| n.gender.is_some()).count(),
                nouns.iter().filter(|n| n.plural.is_some()).count(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::lexicon::NewTranslation;

    const GERMAN_SAMPLE: &str = r#"{"word": "Hund", "pos": "noun", "head_templates": [{"expansion": "Hund m (strong, genitive Hundes or Hunds, plural Hunde, diminutive Hündchen n)"}], "forms": [{"form": "Hundes", "tags": ["genitive"]}, {"form": "Hunde", "tags": ["plural"]}, {"form": "Hündchen", "tags": ["diminutive", "neuter"]}, {"form": "Hündin", "tags": ["feminine"]}, {"form": "Hund", "tags": ["accusative", "singular"], "source": "declension"}], "senses": [{"glosses": ["dog"]}]}
{"word": "Hunde", "pos": "noun", "head_templates": [{"expansion": "Hunde"}], "senses": [{"tags": ["form-of", "plural"]}]}
{"word": "Junge", "pos": "noun", "head_templates": [{"expansion": "Junge m (weak, genitive Jungen, plural Jungen)"}], "forms": [{"form": "Jungen", "tags": ["plural"]}, {"form": "Junge", "tags": ["nominative", "singular"], "source": "declension"}, {"form": "Jungen", "tags": ["accusative", "singular"], "source": "declension"}], "senses": [{}]}
{"word": "Leiter", "pos": "noun", "head_templates": [{"expansion": "Leiter"}], "senses": [{}]}
{"word": "Leiter", "pos": "noun", "head_templates": [{"expansion": "Leiter m (strong, plural Leiter)"}], "forms": [{"form": "Leiter", "tags": ["plural"]}], "senses": [{}]}
{"word": "Leiter", "pos": "noun", "head_templates": [{"expansion": "Leiter f (plural Leitern)"}], "forms": [{"form": "Leitern", "tags": ["plural"]}], "senses": [{}]}
{"word": "Lehrer", "pos": "noun", "head_templates": [{"expansion": "Lehrer m (strong, genitive Lehrers, plural Lehrer, feminine Lehrerin)"}], "forms": [{"form": "Lehrer", "tags": ["plural"]}, {"form": "Lehrerin", "tags": ["feminine"]}], "senses": [{"tags": ["agent", "form-of", "masculine", "strong"]}]}
{"word": "Nutzen", "pos": "noun", "head_templates": [{"expansion": "Nutzen n (strong, genitive Nutzens, no plural)"}], "senses": [{}]}
{"word": "Nutzen", "pos": "noun", "head_templates": [{"expansion": "Nutzen m (strong, genitive Nutzens, plural Nutzen)"}], "forms": [{"form": "Nutzen", "tags": ["plural"]}], "senses": [{}]}
{"word": "Leute", "pos": "noun", "head_templates": [{"expansion": "Leute pl (plural only)"}], "senses": [{}]}
not json
{"word": "Katze", "pos": "noun", "head_templates": [{"expansion": "Katze f (plural Katzen)"}], "forms": [{"form": "Katzen", "tags": ["plural"]}], "senses": [{}]}"#;

    const FRENCH_SAMPLE: &str = r#"{"word": "chien", "pos": "noun", "head_templates": [{"expansion": "chien m (plural chiens, feminine chienne)"}], "forms": [{"form": "chiens", "tags": ["plural"]}, {"form": "chienne", "tags": ["feminine"]}], "senses": [{}]}
{"word": "pomme", "pos": "noun", "head_templates": [{"expansion": "pomme f (plural pommes)"}], "forms": [{"form": "pommes", "tags": ["plural"]}], "senses": [{}]}
{"word": "ciseaux", "pos": "noun", "head_templates": [{"expansion": "ciseaux m pl"}], "senses": [{}]}"#;

    fn noun(lemma: &str, meanings: &[(Locale, &str)]) -> NewLexeme {
        NewLexeme {
            source: "test".into(),
            source_id: lemma.into(),
            lemma: lemma.into(),
            stressed_lemma: None,
            pos: PartOfSpeech::Noun,
            gender: None,
            animacy: None,
            indeclinable: false,
            frequency_rank: None,
            forms: Vec::new(),
            translations: (0..)
                .zip(meanings)
                .map(|(position, &(locale, text))| NewTranslation {
                    locale,
                    position,
                    text: text.into(),
                })
                .collect(),
            noun_forms: Vec::new(),
        }
    }

    fn forms(lexeme: &NewLexeme, locale: Locale) -> &NounForms {
        lexeme
            .noun_forms
            .iter()
            .find(|n| n.locale == locale)
            .unwrap()
    }

    #[test]
    fn meanings_become_noun_forms() {
        let mut words = [noun(
            "собака",
            &[
                (Locale::En, "dog"),
                (Locale::En, "hound"),
                (Locale::De, "der Hund"),
                (Locale::Ru, "животное"),
            ],
        )];
        add_noun_forms(&mut words);
        let locales: Vec<Locale> = words[0].noun_forms.iter().map(|n| n.locale).collect();
        assert_eq!(locales, [Locale::En, Locale::De]);
        assert_eq!(forms(&words[0], Locale::En).singular, "dog");
        let hund = forms(&words[0], Locale::De);
        assert_eq!(
            (hund.singular.as_str(), hund.gender),
            ("Hund", Some(Gender::Masculine))
        );
    }

    #[test]
    fn german_genders_and_plurals() {
        let mut words = [
            noun("собака", &[(Locale::De, "Hund")]),
            noun("мальчик", &[(Locale::De, "Junge")]),
            noun("лестница", &[(Locale::De, "Leiter")]),
            noun("люди", &[(Locale::De, "Leute")]),
            noun("кошка", &[(Locale::De, "kleine Katze")]),
            noun("нечто", &[(Locale::De, "Unbekanntes")]),
            noun("учитель", &[(Locale::De, "Lehrer")]),
            noun("польза", &[(Locale::De, "Nutzen")]),
        ];
        add_noun_forms(&mut words);
        let wanted = wanted(&words, Locale::De);
        assert!(wanted.contains("Katze"));
        let nouns = Nouns::parse(&GERMAN, GERMAN_SAMPLE.as_bytes(), &wanted).unwrap();
        assert_eq!(nouns.len(), 7); // Hunde is only a form
        assert_eq!(nouns.apply(&mut words), 7);

        let hund = forms(&words[0], Locale::De);
        assert_eq!(hund.gender, Some(Gender::Masculine));
        assert_eq!(hund.plural.as_deref(), Some("Hunde"));
        assert_eq!(hund.oblique, None);
        let junge = forms(&words[1], Locale::De);
        assert_eq!(junge.oblique.as_deref(), Some("Jungen"));
        // The first entry with a gender.
        let leiter = forms(&words[2], Locale::De);
        assert_eq!(leiter.gender, Some(Gender::Masculine));
        assert_eq!(leiter.plural.as_deref(), Some("Leiter"));
        assert!(forms(&words[3], Locale::De).plural_only);
        let katze = forms(&words[4], Locale::De);
        assert_eq!(katze.gender, Some(Gender::Feminine));
        // The adjective is dropped: it would need endings.
        assert_eq!(katze.singular, "Katze");
        assert_eq!(katze.plural.as_deref(), Some("Katzen"));
        assert_eq!(forms(&words[5], Locale::De).gender, None);
        // Filed as a form of lehren, but a noun of its own.
        let lehrer = forms(&words[6], Locale::De);
        assert_eq!(lehrer.gender, Some(Gender::Masculine));
        assert_eq!(lehrer.plural.as_deref(), Some("Lehrer"));
        // der Nutzen, with a plural, over das Nutzen (a verb used as a noun).
        assert_eq!(forms(&words[7], Locale::De).gender, Some(Gender::Masculine));
    }

    #[test]
    fn french_genders_and_plurals() {
        let mut words = [
            noun("собака", &[(Locale::Fr, "chien")]),
            noun("картофель", &[(Locale::Fr, "pomme de terre")]),
            noun("ножницы", &[(Locale::Fr, "ciseaux")]),
        ];
        add_noun_forms(&mut words);
        let nouns = Nouns::parse(
            &FRENCH,
            FRENCH_SAMPLE.as_bytes(),
            &wanted(&words, Locale::Fr),
        )
        .unwrap();
        assert_eq!(nouns.apply(&mut words), 3);
        let chien = forms(&words[0], Locale::Fr);
        assert_eq!(chien.gender, Some(Gender::Masculine));
        assert_eq!(chien.plural.as_deref(), Some("chiens")); // not chienne
        let pomme = forms(&words[1], Locale::Fr);
        assert_eq!(pomme.gender, Some(Gender::Feminine));
        assert_eq!(pomme.plural.as_deref(), Some("pommes de terre"));
        assert!(forms(&words[2], Locale::Fr).plural_only);
    }
}
