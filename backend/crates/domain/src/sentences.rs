//! Sentence frames for the sentence game: a short Russian sentence with one slot,
//! filled with a noun from the word list. The frame decides the case and why
//! (`reason`, one of the frontend's case uses), so a wrong guess can be explained.
//!
//! The noun is picked at random, so frames must make sense with almost any noun
//! of the allowed animacy; place frames (в ___) are left out for that reason.

use crate::grammar::{Animacy, Case, Locale, Number};

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
    /// en, fr, de; `___` marks the noun. Russian needs none.
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
       en: "These are my ___.", fr: "Ce sont mes ___.", de: "Das sind meine ___."),
    t!("nom.where", "Где ", "?", Nominative.subject, PLURAL, None,
       en: "Where are the ___?", fr: "Où sont les ___ ?", de: "Wo sind die ___?"),
    t!("nom.all", "Все ", " здесь.", Nominative.subject, PLURAL, None,
       en: "All the ___ are here.", fr: "Tous les ___ sont ici.", de: "Alle ___ sind hier."),
    t!("nom.live", "Здесь живу́т ", ".", Nominative.subject, PLURAL, ANIMATE,
       en: "___ live here.", fr: "Ici vivent des ___.", de: "Hier wohnen ___."),
    // genitive
    t!("gen.none", "У меня́ нет ", ".", Genitive.absence, BOTH, None,
       en: "I have no ___.", fr: "Je n’ai pas de ___.", de: "Ich habe kein(e) ___."),
    t!("gen.wasnt", "Вчера́ здесь не́ было ", ".", Genitive.absence, BOTH, None,
       en: "Yesterday there was no ___ here.", fr: "Hier, il n’y avait pas de ___ ici.",
       de: "Gestern gab es hier kein(e) ___."),
    t!("gen.afraid", "Я бою́сь ", ".", Genitive.verbs, BOTH, None,
       en: "I am afraid of ___.", fr: "J’ai peur de ___.", de: "Ich habe Angst vor ___."),
    t!("gen.without", "Я не могу́ жить без ", ".", Genitive.prepositions, BOTH, None,
       en: "I can’t live without ___.", fr: "Je ne peux pas vivre sans ___.",
       de: "Ich kann nicht ohne ___ leben."),
    t!("gen.near", "Мы сиди́м о́коло ", ".", Genitive.prepositions, BOTH, None,
       en: "We are sitting near ___.", fr: "Nous sommes assis près de ___.",
       de: "Wir sitzen in der Nähe von ___."),
    t!("gen.for", "Э́то пода́рок для ", ".", Genitive.prepositions, BOTH, ANIMATE,
       en: "This is a present for ___.", fr: "C’est un cadeau pour ___.",
       de: "Das ist ein Geschenk für ___."),
    t!("gen.house", "Э́то дом ", ".", Genitive.possession, BOTH, ANIMATE,
       en: "This is the house of ___.", fr: "C’est la maison de ___.",
       de: "Das ist das Haus von ___."),
    t!("gen.many", "Здесь мно́го ", ".", Genitive.quantity, PLURAL, None,
       en: "There are many ___ here.", fr: "Il y a beaucoup de ___ ici.",
       de: "Hier gibt es viele ___."),
    t!("gen.five", "У нас пять ", ".", Genitive.quantity, PLURAL, None,
       en: "We have five ___.", fr: "Nous avons cinq ___.", de: "Wir haben fünf ___."),
    // dative
    t!("dat.towards", "Мы идём к ", ".", Dative.prepositions, BOTH, None,
       en: "We are going to ___.", fr: "Nous allons vers ___.", de: "Wir gehen zu ___."),
    t!("dat.miss", "Я скуча́ю по ", ".", Dative.prepositions, BOTH, None,
       en: "I miss ___.", fr: "___ me manque.", de: "Ich vermisse ___."),
    t!("dat.thanks", "Всё получи́лось благодаря́ ", ".", Dative.prepositions, BOTH, None,
       en: "It all worked out thanks to ___.", fr: "Tout a réussi grâce à ___.",
       de: "Alles hat dank ___ geklappt."),
    t!("dat.call", "Я позвони́л ", ".", Dative.verbs, BOTH, ANIMATE,
       en: "I called ___.", fr: "J’ai téléphoné à ___.", de: "Ich habe ___ angerufen."),
    t!("dat.help", "Я помога́ю ", ".", Dative.verbs, BOTH, ANIMATE,
       en: "I help ___.", fr: "J’aide ___.", de: "Ich helfe ___."),
    t!("dat.give", "Я дал кни́гу ", ".", Dative.recipient, BOTH, ANIMATE,
       en: "I gave the book to ___.", fr: "J’ai donné le livre à ___.",
       de: "Ich habe ___ das Buch gegeben."),
    t!("dat.letter", "Она́ пи́шет письмо́ ", ".", Dative.recipient, BOTH, ANIMATE,
       en: "She is writing a letter to ___.", fr: "Elle écrit une lettre à ___.",
       de: "Sie schreibt ___ einen Brief."),
    // accusative
    t!("acc.see", "Я ви́жу ", ".", Accusative.object, BOTH, None,
       en: "I see ___.", fr: "Je vois ___.", de: "Ich sehe ___."),
    t!("acc.love", "Я люблю́ ", ".", Accusative.object, BOTH, None,
       en: "I love ___.", fr: "J’aime ___.", de: "Ich liebe ___."),
    t!("acc.wait", "Мы ждём ", ".", Accusative.object, BOTH, None,
       en: "We are waiting for ___.", fr: "Nous attendons ___.", de: "Wir warten auf ___."),
    t!("acc.draw", "Он рису́ет ", ".", Accusative.object, BOTH, None,
       en: "He is drawing ___.", fr: "Il dessine ___.", de: "Er zeichnet ___."),
    t!("acc.tell", "Расскажи́ мне про ", ".", Accusative.prepositions, BOTH, None,
       en: "Tell me about ___.", fr: "Parle-moi de ___.", de: "Erzähl mir von ___."),
    t!("acc.thanks", "Спаси́бо за ", ".", Accusative.prepositions, BOTH, None,
       en: "Thank you for ___.", fr: "Merci pour ___.", de: "Danke für ___."),
    // instrumental
    t!("ins.proud", "Я горжу́сь ", ".", Instrumental.verbs, BOTH, None,
       en: "I am proud of ___.", fr: "Je suis fier de ___.", de: "Ich bin stolz auf ___."),
    t!("ins.interested", "Он интересу́ется ", ".", Instrumental.verbs, BOTH, None,
       en: "He is interested in ___.", fr: "Il s’intéresse à ___.",
       de: "Er interessiert sich für ___."),
    t!("ins.admires", "Она́ любу́ется ", ".", Instrumental.verbs, BOTH, None,
       en: "She is admiring ___.", fr: "Elle admire ___.", de: "Sie bewundert ___."),
    t!("ins.talk", "Я говорю́ с ", ".", Instrumental.with, BOTH, ANIMATE,
       en: "I am talking with ___.", fr: "Je parle avec ___.", de: "Ich spreche mit ___."),
    t!("ins.met", "Я познако́мился с ", ".", Instrumental.with, BOTH, ANIMATE,
       en: "I got to know ___.", fr: "J’ai fait la connaissance de ___.",
       de: "Ich habe ___ kennengelernt."),
    t!("ins.become", "Он хо́чет стать ", ".", Instrumental.being, BOTH, ANIMATE,
       en: "He wants to become ___.", fr: "Il veut devenir ___.", de: "Er will ___ werden."),
    t!("ins.under", "Ко́шка спит под ", ".", Instrumental.location, BOTH, INANIMATE,
       en: "The cat is sleeping under ___.", fr: "Le chat dort sous ___.",
       de: "Die Katze schläft unter ___."),
    t!("ins.behind", "Мы стои́м за ", ".", Instrumental.location, BOTH, None,
       en: "We are standing behind ___.", fr: "Nous sommes derrière ___.",
       de: "Wir stehen hinter ___."),
    // prepositional (о becomes об before a vowel: Template::before_for)
    t!("prep.think", "Я ду́маю о ", ".", Prepositional.about, BOTH, None,
       en: "I am thinking about ___.", fr: "Je pense à ___.", de: "Ich denke an ___."),
    t!("prep.talk", "Мы говори́м о ", ".", Prepositional.about, BOTH, None,
       en: "We are talking about ___.", fr: "Nous parlons de ___.",
       de: "Wir sprechen über ___."),
    t!("prep.dream", "Она́ мечта́ет о ", ".", Prepositional.about, BOTH, None,
       en: "She dreams of ___.", fr: "Elle rêve de ___.", de: "Sie träumt von ___."),
    t!("prep.book", "Э́то кни́га о ", ".", Prepositional.about, BOTH, None,
       en: "This is a book about ___.", fr: "C’est un livre sur ___.",
       de: "Das ist ein Buch über ___."),
    t!("prep.forgot", "Я забы́л о ", ".", Prepositional.about, BOTH, None,
       en: "I forgot about ___.", fr: "J’ai oublié ___.", de: "Ich habe ___ vergessen."),
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
            for (_, text) in t.translations {
                assert_eq!(text.matches("___").count(), 1, "{}: {text}", t.id);
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
    fn o_becomes_ob_before_a_vowel() {
        let think = TEMPLATES.iter().find(|t| t.id == "prep.think").unwrap();
        assert_eq!(think.before_for("окне́"), "Я ду́маю об ");
        assert_eq!(think.before_for("кни́ге"), "Я ду́маю о ");
        assert_eq!(think.before_for("ёжике"), "Я ду́маю о "); // о ёжике
        let see = TEMPLATES.iter().find(|t| t.id == "acc.see").unwrap();
        assert_eq!(see.before_for("окно́"), "Я ви́жу ");
    }
}
