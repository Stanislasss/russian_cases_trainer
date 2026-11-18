//! The noun in a sentence's translation, with the article, contraction and form
//! its frame asks for: "the dogs", "au chien", "dem Jungen", "keine Hunde".
//!
//! A frame marks the noun with a slot such as `{the}`, `{à le}` or `{der:dat}`
//! (see [`NounSlot::parse`]); the noun's [`NounForms`] say its gender and plural
//! in that language, as far as the dictionaries know them.

use crate::grammar::{Gender, Locale, Number};

/// A word's first meaning in one language, as a noun: its gender and plural
/// there (German Hund: masculine, Hunde). Unknown parts are `None`, and the
/// phrase then does without them (no article, a plural by rule).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NounForms {
    pub locale: Locale,
    /// "dog", "chien", "Hund"; a few words at most ("pomme de terre").
    pub singular: String,
    /// The whole phrase in the plural ("pommes de terre").
    pub plural: Option<String>,
    pub gender: Option<Gender>,
    /// Used only in the plural (Leute, ciseaux): plural articles, no plural ending.
    pub plural_only: bool,
    /// German weak nouns: the singular outside the nominative (der Junge → den
    /// Jungen, dem Jungen), when it differs.
    pub oblique: Option<String>,
}

impl NounForms {
    /// The noun of a dictionary meaning: "dog; hound" → dog, "chien (mâle)" →
    /// chien, "der Rat" → Rat (masculine: the article gives the gender away).
    /// `None` when no noun is left.
    pub fn from_meaning(locale: Locale, meaning: &str) -> Option<Self> {
        let first = meaning.split(';').next().unwrap_or_default();
        let mut text = String::new();
        let mut depth = 0usize;
        for c in first.chars() {
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => depth = depth.saturating_sub(1),
                _ if depth == 0 => text.push(c),
                _ => {}
            }
        }
        let text = text.split([',', '/']).next().unwrap_or_default();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut words = words.as_slice();
        let mut gender = None;
        let mut plural_only = false;
        let mut elided = None;
        if let [first, rest @ ..] = words
            && !rest.is_empty()
        {
            let article = match (locale, first.to_lowercase().as_str()) {
                (Locale::En, "the" | "a" | "an") => Some(None),
                (Locale::De, "der" | "ein") => Some(Some(Gender::Masculine)),
                (Locale::De, "die" | "eine") => Some(Some(Gender::Feminine)),
                (Locale::De, "das") => Some(Some(Gender::Neuter)),
                (Locale::Fr, "le" | "un") => Some(Some(Gender::Masculine)),
                (Locale::Fr, "la" | "une") => Some(Some(Gender::Feminine)),
                (Locale::Fr, "les") => {
                    plural_only = true;
                    Some(None)
                }
                _ => None,
            };
            if let Some(found) = article {
                gender = found;
                words = rest;
            }
        }
        // French l’arbre: the article is glued on.
        if locale == Locale::Fr
            && let [first, ..] = words
            && let Some(rest) = ["l’", "l'", "L’", "L'"]
                .iter()
                .find_map(|a| first.strip_prefix(a))
            && !rest.is_empty()
        {
            elided = Some(rest.to_owned());
        }
        // German adjectives in front (habilitierter Doktor) would need endings
        // to match the article: the noun alone does. Nouns are capitalised.
        if locale == Locale::De
            && let Some(noun) = words.iter().position(|w| w.starts_with(char::is_uppercase))
        {
            words = &words[noun..];
        }
        let mut singular = words.join(" ");
        if let Some(rest) = elided {
            singular = std::iter::once(rest.as_str())
                .chain(words[1..].iter().copied())
                .collect::<Vec<_>>()
                .join(" ");
        }
        let singular = singular.trim_end_matches('.').trim().to_owned();
        if singular.is_empty() || singular.chars().any(|c| c.is_ascii_digit()) {
            return None;
        }
        Some(NounForms {
            locale,
            singular,
            plural: None,
            gender,
            plural_only,
            oblique: None,
        })
    }

    /// The word that decides the gender and plural, to look up in a dictionary:
    /// French phrases start with it (pomme de terre), English and German ones
    /// end with it (fountain pen, kleiner Hund), unless an "of" comes first.
    pub fn head_word(&self) -> &str {
        head_word(self.locale, &self.singular)
    }

    /// The plural phrase: the dictionary's, or one by rule.
    pub fn plural_phrase(&self) -> String {
        if self.plural_only {
            return self.singular.clone();
        }
        self.plural
            .clone()
            .unwrap_or_else(|| pluralize_phrase(self.locale, &self.singular, None))
    }
}

fn head_word(locale: Locale, phrase: &str) -> &str {
    let words: Vec<&str> = phrase.split(' ').collect();
    words[head_index(locale, &words)]
}

/// Where the head word is in `words`.
fn head_index(locale: Locale, words: &[&str]) -> usize {
    match locale {
        Locale::Fr => 0,
        // cup of tea, Blatt eines Baums: the head comes before what describes it.
        _ => {
            let attribute: &[&str] = match locale {
                Locale::De => &[
                    "eines", "einer", "des", "der", "von", "für", "mit", "aus", "zum", "zur",
                ],
                _ => &["of", "for", "with"],
            };
            // Searching from the second word, the attribute's position there
            // is the index of the word before it.
            words
                .iter()
                .skip(1)
                .position(|w| attribute.contains(w))
                .unwrap_or(words.len() - 1)
        }
    }
}

/// `phrase` in the plural: the head word becomes `head_plural` (or a plural by
/// rule; German has no rule, so it stays), and French adjectives agree with it
/// (chien mâle → chiens mâles, but pommes de terre).
pub fn pluralize_phrase(locale: Locale, phrase: &str, head_plural: Option<&str>) -> String {
    let words: Vec<&str> = phrase.split(' ').collect();
    let head = head_index(locale, &words);
    let agree_until = match locale {
        Locale::Fr => words
            .iter()
            .position(|w| matches!(*w, "de" | "à" | "du" | "des") || w.starts_with("d’"))
            .unwrap_or(words.len()),
        _ => head + 1,
    };
    words
        .iter()
        .enumerate()
        .map(|(i, word)| {
            if i == head {
                match head_plural {
                    Some(plural) => plural.to_owned(),
                    None => plural_by_rule(locale, word),
                }
            } else if locale == Locale::Fr && i < agree_until {
                french_plural(word)
            } else {
                (*word).to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn plural_by_rule(locale: Locale, word: &str) -> String {
    match locale {
        Locale::En => english_plural(word),
        Locale::Fr => french_plural(word),
        // German plurals don't follow a rule one could write down here.
        _ => word.to_owned(),
    }
}

/// dog → dogs, box → boxes, city → cities, knife → knives, child → children.
pub fn english_plural(word: &str) -> String {
    const IRREGULAR: &[(&str, &str)] = &[
        ("man", "men"),
        ("woman", "women"),
        ("child", "children"),
        ("person", "people"),
        ("foot", "feet"),
        ("tooth", "teeth"),
        ("goose", "geese"),
        ("mouse", "mice"),
        ("louse", "lice"),
        ("ox", "oxen"),
        ("sheep", "sheep"),
        ("fish", "fish"),
        ("deer", "deer"),
        ("series", "series"),
        ("species", "species"),
        ("aircraft", "aircraft"),
        ("people", "people"),
        ("police", "police"),
        ("cattle", "cattle"),
        ("criterion", "criteria"),
        ("phenomenon", "phenomena"),
        ("analysis", "analyses"),
        ("crisis", "crises"),
        ("thesis", "theses"),
        ("cactus", "cacti"),
        ("knife", "knives"),
        ("wife", "wives"),
        ("life", "lives"),
        ("leaf", "leaves"),
        ("half", "halves"),
        ("wolf", "wolves"),
        ("shelf", "shelves"),
        ("loaf", "loaves"),
        ("thief", "thieves"),
        ("calf", "calves"),
        ("elf", "elves"),
        ("self", "selves"),
        ("potato", "potatoes"),
        ("tomato", "tomatoes"),
        ("hero", "heroes"),
        ("echo", "echoes"),
        ("veto", "vetoes"),
        ("torpedo", "torpedoes"),
    ];
    // Not compounds of man: human, German, Roman, …
    const NOT_MAN: &[&str] = &["human", "german", "roman", "shaman", "talisman", "caiman"];
    let lower = word.to_lowercase();
    if let Some((_, plural)) = IRREGULAR.iter().find(|(s, _)| *s == lower) {
        return keep_capital(word, plural);
    }
    if let Some(stem) = word.strip_suffix("man")
        && !NOT_MAN.contains(&lower.as_str())
    {
        return format!("{stem}men");
    }
    if let Some(stem) = word.strip_suffix("woman") {
        return format!("{stem}women");
    }
    let ends_with = |s: &str| lower.ends_with(s);
    if ["s", "x", "z", "ch", "sh"].iter().any(|s| ends_with(s)) {
        return format!("{word}es");
    }
    if let Some(stem) = word.strip_suffix('y')
        && !stem.ends_with(['a', 'e', 'i', 'o', 'u'])
    {
        return format!("{stem}ies");
    }
    format!("{word}s")
}

/// chien → chiens, journal → journaux, gâteau → gâteaux, prix → prix.
pub fn french_plural(word: &str) -> String {
    const IRREGULAR: &[(&str, &str)] = &[
        ("œil", "yeux"),
        ("ciel", "cieux"),
        ("bal", "bals"),
        ("carnaval", "carnavals"),
        ("festival", "festivals"),
        ("récital", "récitals"),
        ("pneu", "pneus"),
        ("bleu", "bleus"),
        ("travail", "travaux"),
        ("vitrail", "vitraux"),
        ("corail", "coraux"),
        ("émail", "émaux"),
        ("bijou", "bijoux"),
        ("caillou", "cailloux"),
        ("chou", "choux"),
        ("genou", "genoux"),
        ("hibou", "hiboux"),
        ("joujou", "joujoux"),
        ("pou", "poux"),
        ("monsieur", "messieurs"),
        ("madame", "mesdames"),
        ("mademoiselle", "mesdemoiselles"),
    ];
    if let Some((_, plural)) = IRREGULAR.iter().find(|(s, _)| *s == word) {
        return (*plural).to_owned();
    }
    if word.ends_with(['s', 'x', 'z']) {
        return word.to_owned();
    }
    if let Some(stem) = word.strip_suffix("al") {
        return format!("{stem}aux");
    }
    if word.ends_with("au") || word.ends_with("eu") {
        return format!("{word}x");
    }
    format!("{word}s")
}

fn keep_capital(original: &str, plural: &str) -> String {
    if original.starts_with(char::is_uppercase) {
        capitalize(plural)
    } else {
        plural.to_owned()
    }
}

/// The first letter in upper case: a slot at the start of a sentence.
pub fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The article (or preposition + article) a frame puts before its noun.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Determiner {
    /// No article: "five dogs", "devenir médecin", "Hier wohnen Hunde".
    Bare,
    /// en `{the}`, fr `{le}`, de `{der:…}`.
    Definite,
    /// en `{a}`, fr `{un}`, de `{ein:…}`; nothing (fr: des) in the plural.
    Indefinite,
    /// de `{kein:…}`.
    Negative,
    /// fr `{à le}`: au, à la, à l’, aux.
    ToTheFr,
    /// fr `{de le}`: du, de la, de l’, des.
    OfTheFr,
    /// fr `{de}`: de / d’ without article (pas de chien, beaucoup de chiens).
    OfFr,
    /// de `{zu der:dat}`: zum, zur, zu den.
    ToTheDe,
}

/// The German case of the noun (Russian and German cases often differ: позвони́ть
/// кому́, jemanden anrufen).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GermanCase {
    Nominative,
    Accusative,
    Dative,
}

/// Where the noun goes in a frame's translation and in what shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NounSlot {
    pub locale: Locale,
    pub determiner: Determiner,
    /// German only; nominative elsewhere.
    pub case: GermanCase,
}

impl NounSlot {
    /// A slot as written in a frame, without its braces:
    /// - en: `the`, `a`, `` (bare)
    /// - fr: `le`, `un`, `à le`, `de le`, `de`, ``
    /// - de: `der:acc`, `ein:dat`, `kein:acc`, `zu der:dat`, `:nom` (bare), with
    ///   case `nom`, `acc` or `dat`
    pub fn parse(locale: Locale, text: &str) -> Option<Self> {
        let (determiner, case) = match locale {
            Locale::En => (
                match text {
                    "" => Determiner::Bare,
                    "the" => Determiner::Definite,
                    "a" => Determiner::Indefinite,
                    _ => return None,
                },
                GermanCase::Nominative,
            ),
            Locale::Fr => (
                match text {
                    "" => Determiner::Bare,
                    "le" => Determiner::Definite,
                    "un" => Determiner::Indefinite,
                    "à le" => Determiner::ToTheFr,
                    "de le" => Determiner::OfTheFr,
                    "de" => Determiner::OfFr,
                    _ => return None,
                },
                GermanCase::Nominative,
            ),
            Locale::De => {
                let (article, case) = text.split_once(':')?;
                let case = match case {
                    "nom" => GermanCase::Nominative,
                    "acc" => GermanCase::Accusative,
                    "dat" => GermanCase::Dative,
                    _ => return None,
                };
                let determiner = match article {
                    "" => Determiner::Bare,
                    "der" => Determiner::Definite,
                    "ein" => Determiner::Indefinite,
                    "kein" => Determiner::Negative,
                    "zu der" if case == GermanCase::Dative => Determiner::ToTheDe,
                    _ => return None,
                };
                (determiner, case)
            }
            Locale::Ru => return None,
        };
        Some(NounSlot {
            locale,
            determiner,
            case,
        })
    }

    /// `noun` with its article, in `number` (a plural-only noun is always plural).
    pub fn phrase(&self, noun: &NounForms, number: Number) -> String {
        let plural = noun.plural_only || number == Number::Plural;
        let mut text = if plural {
            noun.plural_phrase()
        } else {
            noun.singular.clone()
        };
        if self.locale == Locale::De {
            text = german_case_form(noun, text, plural, self.case);
        }
        let article = match self.locale {
            Locale::En => english_article(self.determiner, &text, plural),
            Locale::Fr => french_article(self.determiner, noun.gender, &text, plural),
            Locale::De => german_article(self.determiner, self.case, noun.gender, plural),
            Locale::Ru => String::new(),
        };
        format!("{article}{text}")
    }
}

/// Dative plural -n (den Hunden, Kindern, Vögeln, but den Autos, Frauen) and
/// weak nouns (dem Jungen). Plurals taking the -n end in -e, -er or -el; any
/// other ending means the plural isn't really known (Gesundheit).
fn german_case_form(noun: &NounForms, text: String, plural: bool, case: GermanCase) -> String {
    if plural {
        if case == GermanCase::Dative && text.ends_with(['e', 'r', 'l']) {
            return format!("{text}n");
        }
        return text;
    }
    match (&noun.oblique, case) {
        (Some(oblique), GermanCase::Dative) => oblique.clone(),
        (Some(oblique), GermanCase::Accusative) if noun.gender == Some(Gender::Masculine) => {
            oblique.clone()
        }
        _ => text,
    }
}

fn english_article(determiner: Determiner, text: &str, plural: bool) -> String {
    match determiner {
        Determiner::Definite => "the ".into(),
        Determiner::Indefinite if !plural => {
            if english_vowel_sound(text) {
                "an ".into()
            } else {
                "a ".into()
            }
        }
        _ => String::new(),
    }
}

/// an apple, an hour, but a university, a one-way street.
fn english_vowel_sound(text: &str) -> bool {
    let lower = text.to_lowercase();
    const SILENT_H: &[&str] = &["hour", "honest", "honour", "honor", "heir"];
    const CONSONANT_SOUND: &[&str] = &["uni", "use", "usu", "ute", "eu", "one", "once", "ewe"];
    if SILENT_H.iter().any(|w| lower.starts_with(w)) {
        return true;
    }
    if CONSONANT_SOUND.iter().any(|w| lower.starts_with(w)) {
        return false;
    }
    lower.starts_with(['a', 'e', 'i', 'o', 'u'])
}

/// Whether a French article is elided before `word` (l’arbre, l’homme, but le héros).
fn french_elides(word: &str) -> bool {
    const ASPIRATED_H: &[&str] = &[
        "hache",
        "haie",
        "haine",
        "hall",
        "hamac",
        "hameau",
        "hamster",
        "hanche",
        "handicap",
        "hangar",
        "hareng",
        "haricot",
        "harpe",
        "hasard",
        "hausse",
        "haut",
        "hauteur",
        "hérisson",
        "héros",
        "hêtre",
        "hibou",
        "hockey",
        "homard",
        "honte",
        "hoquet",
        "hotte",
        "houx",
        "housse",
        "hublot",
        "huit",
        "hurlement",
        "hutte",
    ];
    let lower = word.to_lowercase();
    if lower.starts_with('h') {
        return !ASPIRATED_H.iter().any(|h| lower.starts_with(h));
    }
    lower.starts_with([
        'a', 'e', 'i', 'o', 'u', 'y', 'à', 'â', 'é', 'è', 'ê', 'ë', 'î', 'ï', 'ô', 'û', 'ü', 'œ',
        'æ',
    ])
}

fn french_article(
    determiner: Determiner,
    gender: Option<Gender>,
    text: &str,
    plural: bool,
) -> String {
    let elides = french_elides(text);
    let feminine = match gender {
        Some(Gender::Feminine) => Some(true),
        Some(_) => Some(false),
        None => None,
    };
    let article = match (determiner, plural, elides, feminine) {
        (Determiner::Bare, ..) => "",
        (Determiner::OfFr, _, true, _) => "d’",
        (Determiner::OfFr, ..) => "de ",

        (Determiner::Definite, true, ..) => "les ",
        (Determiner::Definite, _, true, _) => "l’",
        (Determiner::Definite, _, _, Some(false)) => "le ",
        (Determiner::Definite, _, _, Some(true)) => "la ",
        (Determiner::Definite, ..) => "",

        (Determiner::ToTheFr, true, ..) => "aux ",
        (Determiner::ToTheFr, _, true, _) => "à l’",
        (Determiner::ToTheFr, _, _, Some(false)) => "au ",
        (Determiner::ToTheFr, _, _, Some(true)) => "à la ",
        (Determiner::ToTheFr, ..) => "à ",

        (Determiner::OfTheFr, true, ..) => "des ",
        (Determiner::OfTheFr, _, true, _) => "de l’",
        (Determiner::OfTheFr, _, _, Some(false)) => "du ",
        (Determiner::OfTheFr, _, _, Some(true)) => "de la ",
        (Determiner::OfTheFr, ..) => "de ",

        (Determiner::Indefinite, true, ..) => "des ",
        (Determiner::Indefinite, _, _, Some(false)) => "un ",
        (Determiner::Indefinite, _, _, Some(true)) => "une ",
        (Determiner::Indefinite, ..) => "",

        (Determiner::Negative | Determiner::ToTheDe, ..) => "",
    };
    article.to_owned()
}

fn german_article(
    determiner: Determiner,
    case: GermanCase,
    gender: Option<Gender>,
    plural: bool,
) -> String {
    use GermanCase::*;
    // Columns: masculine, feminine, neuter, plural.
    let (definite, indefinite) = match case {
        Nominative => (["der", "die", "das", "die"], ["ein", "eine", "ein", ""]),
        Accusative => (["den", "die", "das", "die"], ["einen", "eine", "ein", ""]),
        Dative => (
            ["dem", "der", "dem", "den"],
            ["einem", "einer", "einem", ""],
        ),
    };
    let column = match (plural, gender) {
        (true, _) => Some(3),
        (false, Some(Gender::Masculine)) => Some(0),
        (false, Some(Gender::Feminine)) => Some(1),
        (false, Some(Gender::Neuter)) => Some(2),
        (false, None) => None,
    };
    let word = match (determiner, column) {
        (Determiner::Definite, Some(c)) => definite[c].to_owned(),
        (Determiner::Indefinite, Some(c)) => indefinite[c].to_owned(),
        // keine Hunde, keinen Hunden
        (Determiner::Negative, Some(3)) => {
            if case == Dative { "keinen" } else { "keine" }.to_owned()
        }
        (Determiner::Negative, Some(c)) => format!("k{}", indefinite[c]),
        (Determiner::ToTheDe, Some(c)) => match definite[c] {
            "dem" => "zum".to_owned(),
            "der" => "zur".to_owned(),
            other => format!("zu {other}"),
        },
        // Gender unknown: the noun alone, or both endings.
        (Determiner::Indefinite, None) => "ein(e)".to_owned(),
        (Determiner::Negative, None) => "kein(e)".to_owned(),
        (Determiner::ToTheDe, None) => "zu".to_owned(),
        _ => String::new(),
    };
    if word.is_empty() {
        word
    } else {
        format!("{word} ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Gender::*;
    use Number::*;

    fn noun(
        locale: Locale,
        singular: &str,
        plural: Option<&str>,
        gender: Option<Gender>,
    ) -> NounForms {
        NounForms {
            locale,
            singular: singular.into(),
            plural: plural.map(Into::into),
            gender,
            plural_only: false,
            oblique: None,
        }
    }

    fn phrase(locale: Locale, slot: &str, noun: &NounForms, number: Number) -> String {
        NounSlot::parse(locale, slot)
            .unwrap_or_else(|| panic!("{slot}"))
            .phrase(noun, number)
    }

    #[test]
    fn meanings_become_nouns() {
        let en = |m| NounForms::from_meaning(Locale::En, m).map(|n| n.singular);
        assert_eq!(en("dog; hound").as_deref(), Some("dog"));
        assert_eq!(en("the Kremlin").as_deref(), Some("Kremlin"));
        assert_eq!(en("cup (of tea), mug").as_deref(), Some("cup"));
        assert_eq!(en("  "), None);
        assert_eq!(en("(only a note)"), None);

        let rat = NounForms::from_meaning(Locale::De, "der Rat").unwrap();
        assert_eq!(
            (rat.singular.as_str(), rat.gender),
            ("Rat", Some(Masculine))
        );
        let doktor = NounForms::from_meaning(Locale::De, "habilitierter Doktor").unwrap();
        assert_eq!(doktor.singular, "Doktor");
        // A lone "die" is not an article.
        assert_eq!(
            NounForms::from_meaning(Locale::De, "Die").unwrap().singular,
            "Die"
        );

        let arbre = NounForms::from_meaning(Locale::Fr, "l’arbre").unwrap();
        assert_eq!((arbre.singular.as_str(), arbre.gender), ("arbre", None));
        let ciseaux = NounForms::from_meaning(Locale::Fr, "les ciseaux").unwrap();
        assert!(ciseaux.plural_only);
        let chienne = NounForms::from_meaning(Locale::Fr, "chienne (femelle du chien)").unwrap();
        assert_eq!(chienne.singular, "chienne");
    }

    #[test]
    fn head_words() {
        let head = |locale, s| noun(locale, s, None, None).head_word().to_owned();
        assert_eq!(head(Locale::Fr, "pomme de terre"), "pomme");
        assert_eq!(head(Locale::En, "fountain pen"), "pen");
        assert_eq!(head(Locale::En, "cup of tea"), "cup");
        assert_eq!(head(Locale::De, "kleiner Hund"), "Hund");
        assert_eq!(head(Locale::De, "Blatt eines Baums"), "Blatt");
        assert_eq!(head(Locale::En, "case for glasses"), "case");
    }

    #[test]
    fn plurals_by_rule() {
        for (s, p) in [
            ("dog", "dogs"),
            ("box", "boxes"),
            ("church", "churches"),
            ("city", "cities"),
            ("boy", "boys"),
            ("knife", "knives"),
            ("child", "children"),
            ("policeman", "policemen"),
            ("human", "humans"),
            ("Man", "Men"),
        ] {
            assert_eq!(english_plural(s), p);
        }
        for (s, p) in [
            ("chien", "chiens"),
            ("journal", "journaux"),
            ("gâteau", "gâteaux"),
            ("prix", "prix"),
            ("œil", "yeux"),
            ("bijou", "bijoux"),
        ] {
            assert_eq!(french_plural(s), p);
        }
        assert_eq!(
            pluralize_phrase(Locale::Fr, "pomme de terre", None),
            "pommes de terre"
        );
        assert_eq!(
            pluralize_phrase(Locale::Fr, "chien mâle", None),
            "chiens mâles"
        );
        assert_eq!(
            pluralize_phrase(Locale::En, "cup of tea", None),
            "cups of tea"
        );
        assert_eq!(
            pluralize_phrase(Locale::De, "kleiner Hund", Some("Hunde")),
            "kleiner Hunde"
        );
        assert_eq!(pluralize_phrase(Locale::De, "Hund", None), "Hund");
    }

    #[test]
    fn english_phrases() {
        let dog = noun(Locale::En, "dog", None, None);
        assert_eq!(phrase(Locale::En, "the", &dog, Singular), "the dog");
        assert_eq!(phrase(Locale::En, "the", &dog, Plural), "the dogs");
        assert_eq!(phrase(Locale::En, "a", &dog, Singular), "a dog");
        assert_eq!(phrase(Locale::En, "a", &dog, Plural), "dogs");
        assert_eq!(phrase(Locale::En, "", &dog, Plural), "dogs");
        let a = |s| phrase(Locale::En, "a", &noun(Locale::En, s, None, None), Singular);
        assert_eq!(a("apple"), "an apple");
        assert_eq!(a("hour"), "an hour");
        assert_eq!(a("university"), "a university");
        assert_eq!(a("house"), "a house");
    }

    #[test]
    fn french_phrases() {
        let chien = noun(Locale::Fr, "chien", Some("chiens"), Some(Masculine));
        let table = noun(Locale::Fr, "table", None, Some(Feminine));
        let oiseau = noun(Locale::Fr, "oiseau", Some("oiseaux"), Some(Masculine));
        let heros = noun(Locale::Fr, "héros", None, Some(Masculine));
        let fr = |slot, n: &NounForms, number| phrase(Locale::Fr, slot, n, number);
        assert_eq!(fr("à le", &chien, Singular), "au chien");
        assert_eq!(fr("à le", &table, Singular), "à la table");
        assert_eq!(fr("à le", &oiseau, Singular), "à l’oiseau");
        assert_eq!(fr("à le", &chien, Plural), "aux chiens");
        assert_eq!(fr("de le", &chien, Singular), "du chien");
        assert_eq!(fr("de le", &oiseau, Plural), "des oiseaux");
        assert_eq!(fr("de", &chien, Singular), "de chien");
        assert_eq!(fr("de", &oiseau, Singular), "d’oiseau");
        assert_eq!(fr("le", &heros, Singular), "le héros");
        assert_eq!(fr("le", &table, Plural), "les tables");
        assert_eq!(fr("un", &table, Singular), "une table");
        assert_eq!(fr("un", &table, Plural), "des tables");
        assert_eq!(fr("", &chien, Plural), "chiens");
        // Gender unknown: no article where it would need one.
        let unknown = noun(Locale::Fr, "chat", None, None);
        assert_eq!(fr("à le", &unknown, Singular), "à chat");
        assert_eq!(fr("à le", &unknown, Plural), "aux chats");
        // Plural only.
        let ciseaux = NounForms {
            plural_only: true,
            ..noun(Locale::Fr, "ciseaux", None, Some(Masculine))
        };
        assert_eq!(fr("le", &ciseaux, Singular), "les ciseaux");
    }

    #[test]
    fn german_phrases() {
        let hund = noun(Locale::De, "Hund", Some("Hunde"), Some(Masculine));
        let auto = noun(Locale::De, "Auto", Some("Autos"), Some(Neuter));
        let katze = noun(Locale::De, "Katze", Some("Katzen"), Some(Feminine));
        let junge = NounForms {
            oblique: Some("Jungen".into()),
            ..noun(Locale::De, "Junge", Some("Jungen"), Some(Masculine))
        };
        let herz = NounForms {
            oblique: Some("Herzen".into()),
            ..noun(Locale::De, "Herz", Some("Herzen"), Some(Neuter))
        };
        let de = |slot, n: &NounForms, number| phrase(Locale::De, slot, n, number);
        assert_eq!(de("der:acc", &hund, Singular), "den Hund");
        assert_eq!(de("der:dat", &hund, Singular), "dem Hund");
        assert_eq!(de("der:dat", &hund, Plural), "den Hunden");
        assert_eq!(de("der:dat", &auto, Plural), "den Autos");
        let kind = noun(Locale::De, "Kind", Some("Kinder"), Some(Neuter));
        assert_eq!(de("der:dat", &kind, Plural), "den Kindern");
        let frau = noun(Locale::De, "Frau", Some("Frauen"), Some(Feminine));
        assert_eq!(de("der:dat", &frau, Plural), "den Frauen");
        // A plural that is really the singular gets no -n.
        let gesundheit = noun(Locale::De, "Gesundheit", Some("Gesundheit"), Some(Feminine));
        assert_eq!(de("der:dat", &gesundheit, Plural), "den Gesundheit");
        assert_eq!(de("der:nom", &katze, Plural), "die Katzen");
        assert_eq!(de("kein:acc", &hund, Singular), "keinen Hund");
        assert_eq!(de("kein:acc", &katze, Plural), "keine Katzen");
        assert_eq!(de("kein:acc", &auto, Singular), "kein Auto");
        assert_eq!(de("ein:dat", &katze, Singular), "einer Katze");
        assert_eq!(de("ein:acc", &hund, Plural), "Hunde");
        assert_eq!(de("zu der:dat", &hund, Singular), "zum Hund");
        assert_eq!(de("zu der:dat", &katze, Singular), "zur Katze");
        assert_eq!(de("zu der:dat", &katze, Plural), "zu den Katzen");
        assert_eq!(de(":nom", &hund, Plural), "Hunde");
        assert_eq!(de("der:acc", &junge, Singular), "den Jungen");
        assert_eq!(de("der:dat", &junge, Singular), "dem Jungen");
        assert_eq!(de("der:acc", &herz, Singular), "das Herz");
        assert_eq!(de("der:dat", &herz, Singular), "dem Herzen");
        // Gender unknown.
        let unknown = noun(Locale::De, "Ding", None, None);
        assert_eq!(de("der:acc", &unknown, Singular), "Ding");
        assert_eq!(de("kein:acc", &unknown, Singular), "kein(e) Ding");
        assert_eq!(de("der:acc", &unknown, Plural), "die Ding");
    }

    #[test]
    fn slots_are_checked() {
        assert!(NounSlot::parse(Locale::En, "le").is_none());
        assert!(NounSlot::parse(Locale::De, "der").is_none());
        assert!(NounSlot::parse(Locale::De, "der:gen").is_none());
        assert!(NounSlot::parse(Locale::De, "zu der:acc").is_none());
        assert!(NounSlot::parse(Locale::Ru, "").is_none());
    }
}
