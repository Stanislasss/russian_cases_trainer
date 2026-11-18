//! Sentence frames for the sentence game: a short Russian sentence with one slot,
//! filled with a noun from the word list. The frame decides the case and why
//! (`reason`, one of the frontend's case uses), so a wrong guess can be explained.
//!
//! The noun is picked at random, so frames must make sense with almost any noun
//! of the allowed animacy; place frames (в ___) are left out for that reason.

use crate::grammar::{Animacy, Case, Locale, Number};
use crate::noun_phrase::{NounForms, NounSlot, capitalize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Template {
    pub id: &'static str,
    /// Stress-marked Russian before and after the slot.
    pub before: &'static str,
    pub after: &'static str,
    pub case: Case,
    /// Why this case: a use id of `case` in `frontend/src/shared/grammar/cases.ts`.
    pub reason: &'static str,
    pub numbers: &'static [Number],
    /// Only nouns of this animacy fit (позвони́ть ___ needs someone).
    pub animacy: Option<Animacy>,
    /// en, fr, de; a slot in braces marks the noun and says which article and
    /// case it takes there (`{the}`, `{à le}`, `{der:dat}`: see
    /// [`NounSlot::parse`]). Russian needs none.
    pub translations: [(Locale, &'static str); 3],
}

const BOTH: &[Number] = &[Number::Singular, Number::Plural];
const PLURAL: &[Number] = &[Number::Plural];
const ANIMATE: Option<Animacy> = Some(Animacy::Animate);
const INANIMATE: Option<Animacy> = Some(Animacy::Inanimate);

impl Template {
    /// The text before the slot for a given form: о becomes об before a vowel
    /// (об окне́, об у́лице).
    pub fn before_for(&self, form: &str) -> String {
        let starts_with_vowel = form
            .chars()
            .next()
            .is_some_and(|c| "аэиоуАЭИОУ".contains(c));
        match self.before.strip_suffix(" о ") {
            Some(start) if starts_with_vowel => format!("{start} об "),
            _ => self.before.to_owned(),
        }
    }

    /// The translation in `locale`, split around its slot: (before, slot, after).
    fn split(&self, locale: Locale) -> Option<(&'static str, &'static str, &'static str)> {
        let text = self.translations.iter().find(|(l, _)| *l == locale)?.1;
        let (before, rest) = text.split_once('{')?;
        let (slot, after) = rest.split_once('}')?;
        Some((before, slot, after))
    }

    /// The translation in `locale` with `___` where the noun goes.
    pub fn frame(&self, locale: Locale) -> Option<String> {
        let (before, _, after) = self.split(locale)?;
        Some(format!("{before}___{after}"))
    }

    /// The noun as it goes into the translation in `locale`, article included
    /// ("au chien", "den Hunden"); capitalised at the start of the sentence.
    pub fn noun_phrase(&self, noun: &NounForms, number: Number) -> Option<String> {
        let (before, slot, _) = self.split(noun.locale)?;
        let phrase = NounSlot::parse(noun.locale, slot)?.phrase(noun, number);
        Some(if before.is_empty() {
            capitalize(&phrase)
        } else {
            phrase
        })
    }
}

macro_rules! t {
    ($id:literal, $before:literal, $after:literal, $case:ident.$reason:ident, $numbers:expr,
     $animacy:expr, en: $en:literal, fr: $fr:literal, de: $de:literal) => {
        Template {
            id: $id,
            before: $before,
            after: $after,
            case: Case::$case,
            reason: stringify!($reason),
            numbers: $numbers,
            animacy: $animacy,
            translations: [(Locale::En, $en), (Locale::Fr, $fr), (Locale::De, $de)],
        }
    };
}

pub const TEMPLATES: &[Template] = &[
    // nominative: plural only, the singular is the dictionary form shown
    t!("nom.these", "Э́то мои́ ", ".", Nominative.predicate, PLURAL, None,
       en: "These are my {}.", fr: "Ce sont mes {}.", de: "Das sind meine {:nom}."),
    t!("nom.where", "Где ", "?", Nominative.subject, PLURAL, None,
       en: "Where are {the}?", fr: "Où sont {le} ?", de: "Wo sind {der:nom}?"),
    t!("nom.all", "Все ", " здесь.", Nominative.subject, PLURAL, None,
       en: "All {the} are here.", fr: "{le} sont ici, sans exception.",
       de: "Alle {:nom} sind hier."),
    t!("nom.live", "Здесь живу́т ", ".", Nominative.subject, PLURAL, ANIMATE,
       en: "{} live here.", fr: "Ici vivent {un}.", de: "Hier wohnen {:nom}."),
    // genitive
    t!("gen.none", "У меня́ нет ", ".", Genitive.absence, BOTH, None,
       en: "I have no {}.", fr: "Je n’ai pas {de}.", de: "Ich habe {kein:acc}."),
    t!("gen.wasnt", "Вчера́ здесь не́ было ", ".", Genitive.absence, BOTH, None,
       en: "Yesterday we had no {} here.", fr: "Hier, il n’y avait pas {de} ici.",
       de: "Gestern gab es hier {kein:acc}."),
    t!("gen.afraid", "Я бою́сь ", ".", Genitive.verbs, BOTH, None,
       en: "I am afraid of {the}.", fr: "J’ai peur {de le}.", de: "Ich habe Angst vor {der:dat}."),
    t!("gen.without", "Я не могу́ жить без ", ".", Genitive.prepositions, BOTH, None,
       en: "I can’t live without {the}.", fr: "Je ne peux pas vivre sans {le}.",
       de: "Ich kann nicht ohne {der:acc} leben."),
    t!("gen.near", "Мы сиди́м о́коло ", ".", Genitive.prepositions, BOTH, None,
       en: "We are sitting near {the}.", fr: "Nous sommes assis près {de le}.",
       de: "Wir sitzen neben {der:dat}."),
    t!("gen.for", "Э́то пода́рок для ", ".", Genitive.prepositions, BOTH, ANIMATE,
       en: "This is a present for {the}.", fr: "C’est un cadeau pour {le}.",
       de: "Das ist ein Geschenk für {der:acc}."),
    t!("gen.house", "Э́то дом ", ".", Genitive.possession, BOTH, ANIMATE,
       en: "This is the house of {the}.", fr: "C’est la maison {de le}.",
       de: "Dieses Haus gehört {der:dat}."),
    t!("gen.many", "Здесь мно́го ", ".", Genitive.quantity, PLURAL, None,
       en: "There are many {} here.", fr: "Il y a beaucoup {de} ici.",
       de: "Hier gibt es viele {:acc}."),
    t!("gen.five", "У нас пять ", ".", Genitive.quantity, PLURAL, None,
       en: "We have five {}.", fr: "Nous avons cinq {}.", de: "Wir haben fünf {:acc}."),
    // dative
    t!("dat.towards", "Мы идём к ", ".", Dative.prepositions, BOTH, None,
       en: "We are going to {the}.", fr: "Nous allons vers {le}.", de: "Wir gehen {zu der:dat}."),
    t!("dat.miss", "Я скуча́ю по ", ".", Dative.prepositions, BOTH, None,
       en: "I miss {the}.", fr: "Je m’ennuie {de le}.", de: "Ich vermisse {der:acc}."),
    t!("dat.thanks", "Всё получи́лось благодаря́ ", ".", Dative.prepositions, BOTH, None,
       en: "It all worked out thanks to {the}.", fr: "Tout a réussi grâce {à le}.",
       de: "Alles hat dank {der:dat} geklappt."),
    t!("dat.call", "Я позвони́л ", ".", Dative.verbs, BOTH, ANIMATE,
       en: "I called {the}.", fr: "J’ai téléphoné {à le}.", de: "Ich habe {der:acc} angerufen."),
    t!("dat.help", "Я помога́ю ", ".", Dative.verbs, BOTH, ANIMATE,
       en: "I help {the}.", fr: "J’aide {le}.", de: "Ich helfe {der:dat}."),
    t!("dat.give", "Я дал кни́гу ", ".", Dative.recipient, BOTH, ANIMATE,
       en: "I gave the book to {the}.", fr: "J’ai donné le livre {à le}.",
       de: "Ich habe {der:dat} das Buch gegeben."),
    t!("dat.letter", "Она́ пи́шет письмо́ ", ".", Dative.recipient, BOTH, ANIMATE,
       en: "She is writing a letter to {the}.", fr: "Elle écrit une lettre {à le}.",
       de: "Sie schreibt {der:dat} einen Brief."),
    // accusative
    t!("acc.see", "Я ви́жу ", ".", Accusative.object, BOTH, None,
       en: "I see {a}.", fr: "Je vois {un}.", de: "Ich sehe {ein:acc}."),
    t!("acc.love", "Я люблю́ ", ".", Accusative.object, BOTH, None,
       en: "I love {the}.", fr: "J’aime {le}.", de: "Ich liebe {der:acc}."),
    t!("acc.wait", "Мы ждём ", ".", Accusative.object, BOTH, None,
       en: "We are waiting for {the}.", fr: "Nous attendons {le}.",
       de: "Wir warten auf {der:acc}."),
    t!("acc.draw", "Он рису́ет ", ".", Accusative.object, BOTH, None,
       en: "He is drawing {a}.", fr: "Il dessine {un}.", de: "Er zeichnet {ein:acc}."),
    t!("acc.tell", "Расскажи́ мне про ", ".", Accusative.prepositions, BOTH, None,
       en: "Tell me about {the}.", fr: "Parle-moi {de le}.",
       de: "Erzähl mir etwas über {der:acc}."),
    t!("acc.thanks", "Спаси́бо за ", ".", Accusative.prepositions, BOTH, None,
       en: "Thank you for {the}.", fr: "Merci pour {le}.", de: "Danke für {der:acc}."),
    // instrumental
    t!("ins.proud", "Я горжу́сь ", ".", Instrumental.verbs, BOTH, None,
       en: "I am proud of {the}.", fr: "Je suis fier {de le}.",
       de: "Ich bin stolz auf {der:acc}."),
    t!("ins.interested", "Он интересу́ется ", ".", Instrumental.verbs, BOTH, None,
       en: "He is interested in {the}.", fr: "Il s’intéresse {à le}.",
       de: "Er interessiert sich für {der:acc}."),
    t!("ins.admires", "Она́ любу́ется ", ".", Instrumental.verbs, BOTH, None,
       en: "She is admiring {the}.", fr: "Elle admire {le}.", de: "Sie bewundert {der:acc}."),
    t!("ins.talk", "Я говорю́ с ", ".", Instrumental.with, BOTH, ANIMATE,
       en: "I am talking with {the}.", fr: "Je parle avec {le}.",
       de: "Ich spreche mit {der:dat}."),
    t!("ins.met", "Я познако́мился с ", ".", Instrumental.with, BOTH, ANIMATE,
       en: "I got to know {the}.", fr: "J’ai fait la connaissance {de le}.",
       de: "Ich habe {der:acc} kennengelernt."),
    t!("ins.become", "Он хо́чет стать ", ".", Instrumental.being, BOTH, ANIMATE,
       en: "He wants to become {a}.", fr: "Il veut devenir {}.", de: "Er will {:nom} werden."),
    t!("ins.under", "Ко́шка спит под ", ".", Instrumental.location, BOTH, INANIMATE,
       en: "The cat is sleeping under {the}.", fr: "Le chat dort sous {le}.",
       de: "Die Katze schläft unter {der:dat}."),
    t!("ins.behind", "Мы стои́м за ", ".", Instrumental.location, BOTH, None,
       en: "We are standing behind {the}.", fr: "Nous sommes derrière {le}.",
       de: "Wir stehen hinter {der:dat}."),
    // prepositional (о becomes об before a vowel: Template::before_for)
    t!("prep.think", "Я ду́маю о ", ".", Prepositional.about, BOTH, None,
       en: "I am thinking about {the}.", fr: "Je pense {à le}.", de: "Ich denke an {der:acc}."),
    t!("prep.talk", "Мы говори́м о ", ".", Prepositional.about, BOTH, None,
       en: "We are talking about {the}.", fr: "Nous parlons {de le}.",
       de: "Wir sprechen über {der:acc}."),
    t!("prep.dream", "Она́ мечта́ет о ", ".", Prepositional.about, BOTH, None,
       en: "She dreams of {a}.", fr: "Elle rêve {de le}.", de: "Sie träumt von {ein:dat}."),
    t!("prep.book", "Э́то кни́га о ", ".", Prepositional.about, BOTH, None,
       en: "This is a book about {the}.", fr: "C’est un livre sur {le}.",
       de: "Das ist ein Buch über {der:acc}."),
    t!("prep.forgot", "Я забы́л о ", ".", Prepositional.about, BOTH, None,
       en: "I forgot about {the}.", fr: "J’ai oublié {le}.", de: "Ich habe {der:acc} vergessen."),
];

/// The use ids of each case, as in `USES` in `frontend/src/shared/grammar/cases.ts`.
/// A template's `reason` must be one of its case's.
pub fn reasons(case: Case) -> &'static [&'static str] {
    match case {
        Case::Nominative => &["subject", "predicate", "dictionary"],
        Case::Genitive => &["possession", "absence", "quantity", "prepositions", "verbs"],
        Case::Dative => &["recipient", "impersonal", "prepositions", "verbs"],
        Case::Accusative => &["object", "direction", "time", "prepositions"],
        Case::Instrumental => &["means", "with", "location", "being", "verbs", "time"],
        Case::Prepositional => &["location", "about", "transport", "time"],
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::grammar::Gender;

    #[test]
    fn every_case_has_frames() {
        for case in Case::ALL {
            let count = TEMPLATES.iter().filter(|t| t.case == *case).count();
            assert!(count >= 4, "{case}: {count}");
        }
    }

    #[test]
    fn frames_are_well_formed() {
        let mut ids = HashSet::new();
        for t in TEMPLATES {
            assert!(ids.insert(t.id), "duplicate {}", t.id);
            assert!(
                reasons(t.case).contains(&t.reason),
                "{}: {}",
                t.id,
                t.reason
            );
            assert!(
                !t.before.trim().is_empty() || !t.after.trim().is_empty(),
                "{}",
                t.id
            );
            assert!(t.before.is_empty() || t.before.ends_with(' '), "{}", t.id);
            assert!(t.after.chars().all(|c| !c.is_alphabetic()) || t.after.starts_with(' '));
            for (locale, text) in t.translations {
                assert_eq!(text.matches('{').count(), 1, "{}: {text}", t.id);
                assert_eq!(text.matches('}').count(), 1, "{}: {text}", t.id);
                let (_, slot, _) = t.split(locale).unwrap();
                assert!(NounSlot::parse(locale, slot).is_some(), "{}: {text}", t.id);
            }
        }
    }

    #[test]
    fn stress_is_marked_on_words_of_several_syllables() {
        // Unstressed after a stressed не́: не́ было.
        const CLITICS: &[&str] = &["было"];
        for t in TEMPLATES {
            let text = format!("{}{}", t.before, t.after);
            for word in text.split(|c: char| !c.is_alphabetic() && c != '\u{301}') {
                let vowels = word
                    .chars()
                    .filter(|c| "аэыуояеёюиАЭЫУОЯЕЁЮИ".contains(*c))
                    .count();
                let marked = word.contains('\u{301}') || word.contains(['ё', 'Ё']);
                assert!(
                    vowels <= 1 || marked || CLITICS.contains(&word),
                    "{}: {word}",
                    t.id
                );
            }
        }
    }

    #[test]
    fn translations_get_the_noun_with_its_article() {
        let frame = |id: &str| TEMPLATES.iter().find(|t| t.id == id).unwrap();
        let sentence = |id: &str, noun: &NounForms, number| {
            let t = frame(id);
            t.frame(noun.locale)
                .unwrap()
                .replace("___", &t.noun_phrase(noun, number).unwrap())
        };
        let forms = |locale, singular: &str, plural: &str, gender| NounForms {
            locale,
            singular: singular.into(),
            plural: Some(plural.into()),
            gender,
            plural_only: false,
            oblique: None,
        };
        let dog = NounForms::from_meaning(Locale::En, "dog; hound").unwrap();
        let chien = forms(Locale::Fr, "chien", "chiens", Some(Gender::Masculine));
        let hund = forms(Locale::De, "Hund", "Hunde", Some(Gender::Masculine));
        use Number::*;
        assert_eq!(sentence("dat.call", &dog, Singular), "I called the dog.");
        assert_eq!(
            sentence("dat.call", &chien, Singular),
            "J’ai téléphoné au chien."
        );
        assert_eq!(
            sentence("dat.call", &hund, Singular),
            "Ich habe den Hund angerufen."
        );
        assert_eq!(sentence("dat.help", &hund, Plural), "Ich helfe den Hunden.");
        assert_eq!(sentence("gen.five", &dog, Plural), "We have five dogs.");
        assert_eq!(
            sentence("gen.none", &chien, Singular),
            "Je n’ai pas de chien."
        );
        assert_eq!(
            sentence("dat.towards", &hund, Singular),
            "Wir gehen zum Hund."
        );
        // At the start of the sentence.
        assert_eq!(sentence("nom.live", &dog, Plural), "Dogs live here.");
        assert_eq!(
            sentence("nom.all", &chien, Plural),
            "Les chiens sont ici, sans exception."
        );
        // No German frame in Russian.
        assert_eq!(frame("dat.call").frame(Locale::Ru), None);
    }

    #[test]
    fn o_becomes_ob_before_a_vowel() {
        let think = TEMPLATES.iter().find(|t| t.id == "prep.think").unwrap();
        assert_eq!(think.before_for("окне́"), "Я ду́маю об ");
        assert_eq!(think.before_for("кни́ге"), "Я ду́маю о ");
        assert_eq!(think.before_for("ёжике"), "Я ду́маю о "); // о ёжике
        let see = TEMPLATES.iter().find(|t| t.id == "acc.see").unwrap();
        assert_eq!(see.before_for("окно́"), "Я ви́жу ");
    }
}
