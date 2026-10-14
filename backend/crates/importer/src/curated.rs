//! Words written out by hand because the OpenRussian dumps lack them: personal
//! pronouns (listed without forms), наш and ваш (missing), and surnames.

use domain::grammar::{Case, Gender, Locale, Number, PartOfSpeech};
use domain::lexicon::{NewForm, NewLexeme, NewTranslation, Slot};
use domain::surname;

use crate::declension::adjective_like_forms;

pub const SOURCE: &str = "curated";

const CASES: [Case; 6] = [
    Case::Nominative,
    Case::Genitive,
    Case::Dative,
    Case::Accusative,
    Case::Instrumental,
    Case::Prepositional,
];

struct PersonalPronoun {
    lemma: &'static str,
    number: Number,
    gender: Option<Gender>,
    /// Forms per case (nominative … prepositional); the first is the main form,
    /// the others alternatives such as the н- forms after prepositions (у него́).
    forms: [&'static [&'static str]; 6],
    /// English, German, French.
    translations: [&'static str; 3],
}

const PERSONAL_PRONOUNS: &[PersonalPronoun] = &[
    PersonalPronoun {
        lemma: "я",
        number: Number::Singular,
        gender: None,
        forms: [
            &["я"],
            &["меня\u{301}"],
            &["мне"],
            &["меня\u{301}"],
            &["мной", "мно\u{301}ю"],
            &["мне"],
        ],
        translations: ["I", "ich", "je"],
    },
    PersonalPronoun {
        lemma: "ты",
        number: Number::Singular,
        gender: None,
        forms: [
            &["ты"],
            &["тебя\u{301}"],
            &["тебе\u{301}"],
            &["тебя\u{301}"],
            &["тобо\u{301}й", "тобо\u{301}ю"],
            &["тебе\u{301}"],
        ],
        translations: ["you", "du", "tu"],
    },
    PersonalPronoun {
        lemma: "он",
        number: Number::Singular,
        gender: Some(Gender::Masculine),
        forms: [
            &["он"],
            &["его\u{301}", "него\u{301}"],
            &["ему\u{301}", "нему\u{301}"],
            &["его\u{301}", "него\u{301}"],
            &["им", "ним"],
            &["нём"],
        ],
        translations: ["he, it", "er", "il"],
    },
    PersonalPronoun {
        lemma: "она",
        number: Number::Singular,
        gender: Some(Gender::Feminine),
        forms: [
            &["она\u{301}"],
            &["её", "неё"],
            &["ей", "ней"],
            &["её", "неё"],
            &["ей", "е\u{301}ю", "ней", "не\u{301}ю"],
            &["ней"],
        ],
        translations: ["she, it", "sie", "elle"],
    },
    PersonalPronoun {
        lemma: "оно",
        number: Number::Singular,
        gender: Some(Gender::Neuter),
        forms: [
            &["оно\u{301}"],
            &["его\u{301}", "него\u{301}"],
            &["ему\u{301}", "нему\u{301}"],
            &["его\u{301}", "него\u{301}"],
            &["им", "ним"],
            &["нём"],
        ],
        translations: ["it", "es", "il, elle (neutre)"],
    },
    PersonalPronoun {
        lemma: "мы",
        number: Number::Plural,
        gender: None,
        forms: [
            &["мы"],
            &["нас"],
            &["нам"],
            &["нас"],
            &["на\u{301}ми"],
            &["нас"],
        ],
        translations: ["we", "wir", "nous"],
    },
    PersonalPronoun {
        lemma: "вы",
        number: Number::Plural,
        gender: None,
        forms: [
            &["вы"],
            &["вас"],
            &["вам"],
            &["вас"],
            &["ва\u{301}ми"],
            &["вас"],
        ],
        translations: ["you (plural, formal)", "ihr, Sie", "vous"],
    },
    PersonalPronoun {
        lemma: "они",
        number: Number::Plural,
        gender: None,
        forms: [
            &["они\u{301}"],
            &["их", "них"],
            &["им", "ним"],
            &["их", "них"],
            &["и\u{301}ми", "ни\u{301}ми"],
            &["них"],
        ],
        translations: ["they", "sie", "ils, elles"],
    },
    PersonalPronoun {
        lemma: "себя",
        number: Number::Singular,
        gender: None,
        // No nominative: себя́ always refers back to the subject.
        forms: [
            &[],
            &["себя\u{301}"],
            &["себе\u{301}"],
            &["себя\u{301}"],
            &["собо\u{301}й", "собо\u{301}ю"],
            &["себе\u{301}"],
        ],
        translations: ["oneself", "sich", "soi"],
    },
    PersonalPronoun {
        lemma: "кто",
        number: Number::Singular,
        gender: None,
        forms: [
            &["кто"],
            &["кого\u{301}"],
            &["кому\u{301}"],
            &["кого\u{301}"],
            &["кем"],
            &["ком"],
        ],
        translations: ["who", "wer", "qui"],
    },
    PersonalPronoun {
        lemma: "что",
        number: Number::Singular,
        gender: None,
        forms: [
            &["что"],
            &["чего\u{301}"],
            &["чему\u{301}"],
            &["что"],
            &["чем"],
            &["чём"],
        ],
        translations: ["what", "was", "que, quoi"],
    },
];

fn translations(texts: [&str; 3]) -> Vec<NewTranslation> {
    [Locale::En, Locale::De, Locale::Fr]
        .into_iter()
        .zip(texts)
        .map(|(locale, text)| NewTranslation {
            locale,
            position: 0,
            text: text.to_owned(),
        })
        .collect()
}

fn lexeme(kind: &str, lemma: &str, stressed: &str, pos: PartOfSpeech, rank: i32) -> NewLexeme {
    NewLexeme {
        source: SOURCE.to_owned(),
        source_id: format!("{kind}:{lemma}"),
        lemma: lemma.to_owned(),
        stressed_lemma: Some(stressed.to_owned()),
        pos,
        gender: None,
        animacy: None,
        indeclinable: false,
        frequency_rank: Some(rank),
        forms: Vec::new(),
        translations: Vec::new(),
    }
}

pub fn personal_pronouns() -> Vec<NewLexeme> {
    (1..)
        .zip(PERSONAL_PRONOUNS)
        .map(|(rank, p)| {
            let forms = CASES
                .iter()
                .zip(p.forms)
                .flat_map(|(&case, variants)| {
                    let slot = Slot::new(case, p.number);
                    (0..)
                        .zip(variants.iter())
                        .map(move |(variant, form)| NewForm::whole(form, slot, variant))
                })
                .collect();
            let nominative = p.forms[0].first().copied().unwrap_or(p.lemma);
            NewLexeme {
                gender: p.gender,
                forms,
                translations: translations(p.translations),
                ..lexeme(
                    "pronoun",
                    p.lemma,
                    nominative,
                    PartOfSpeech::PersonalPronoun,
                    rank,
                )
            }
        })
        .collect()
}

/// наш and ваш decline alike; `ваш` is `наш` with в.
fn our(initial: char) -> impl Fn(Number, Option<Gender>, Case) -> Vec<String> {
    move |number, gender, case| {
        let ending = match (number, gender, case) {
            (Number::Plural, _, Case::Nominative) => "и",
            (Number::Plural, _, Case::Dative) => "им",
            (Number::Plural, _, Case::Instrumental) => "ими",
            (Number::Plural, _, _) => "их",
            (_, Some(Gender::Feminine), Case::Nominative) => "а",
            (_, Some(Gender::Feminine), Case::Accusative) => "у",
            (_, Some(Gender::Feminine), _) => "ей",
            (_, Some(Gender::Neuter), Case::Nominative | Case::Accusative) => "е",
            (_, _, Case::Nominative | Case::Accusative) => "",
            (_, _, Case::Genitive) => "его",
            (_, _, Case::Dative) => "ему",
            (_, _, Case::Instrumental) => "им",
            (_, _, Case::Prepositional) => "ем",
        };
        // Stressed on the stem: на́шего; the bare наш needs no mark.
        let stem = if ending.is_empty() {
            format!("{initial}аш")
        } else {
            format!("{initial}а\u{301}ш")
        };
        let mut forms = vec![format!("{stem}{ending}")];
        if gender == Some(Gender::Feminine) && case == Case::Instrumental {
            forms.push(format!("{stem}ею"));
        }
        forms
    }
}

pub fn possessive_pronouns() -> Vec<NewLexeme> {
    [
        ("наш", 'н', ["our, ours", "unser", "notre, nôtre"]),
        ("ваш", 'в', ["your, yours", "euer, Ihr", "votre, vôtre"]),
    ]
    .into_iter()
    .map(|(lemma, initial, texts)| NewLexeme {
        forms: adjective_like_forms(lemma, PartOfSpeech::Pronoun, our(initial)),
        translations: translations(texts),
        // After the OpenRussian pronouns in rank; pronoun levels take all ranks anyway.
        ..lexeme("pronoun", lemma, lemma, PartOfSpeech::Pronoun, 1)
    })
    .collect()
}

/// The most common Russian surnames (after Balanovska et al., as listed on
/// Wikipedia), plus some well-known -ский/-ой ones. All decline by rule.
pub const SURNAMES: &[&str] = &[
    "Смирнов",
    "Иванов",
    "Кузнецов",
    "Соколов",
    "Попов",
    "Лебедев",
    "Козлов",
    "Новиков",
    "Морозов",
    "Петров",
    "Волков",
    "Соловьёв",
    "Васильев",
    "Зайцев",
    "Павлов",
    "Семёнов",
    "Голубев",
    "Виноградов",
    "Богданов",
    "Воробьёв",
    "Фёдоров",
    "Михайлов",
    "Беляев",
    "Тарасов",
    "Белов",
    "Комаров",
    "Орлов",
    "Киселёв",
    "Макаров",
    "Андреев",
    "Ковалёв",
    "Ильин",
    "Гусев",
    "Титов",
    "Кузьмин",
    "Кудрявцев",
    "Баранов",
    "Куликов",
    "Алексеев",
    "Степанов",
    "Яковлев",
    "Сорокин",
    "Сергеев",
    "Романов",
    "Захаров",
    "Борисов",
    "Королёв",
    "Герасимов",
    "Пономарёв",
    "Григорьев",
    "Лазарев",
    "Медведев",
    "Ершов",
    "Никитин",
    "Соболев",
    "Рябов",
    "Поляков",
    "Цветков",
    "Данилов",
    "Жуков",
    "Фролов",
    "Журавлёв",
    "Николаев",
    "Крылов",
    "Максимов",
    "Сидоров",
    "Осипов",
    "Белоусов",
    "Федотов",
    "Дорофеев",
    "Егоров",
    "Матвеев",
    "Бобров",
    "Дмитриев",
    "Калинин",
    "Анисимов",
    "Петухов",
    "Антонов",
    "Тимофеев",
    "Никифоров",
    "Веселов",
    "Филиппов",
    "Марков",
    "Большаков",
    "Суханов",
    "Миронов",
    "Ширяев",
    "Александров",
    "Коновалов",
    "Шестаков",
    "Казаков",
    "Ефимов",
    "Денисов",
    "Громов",
    "Фомин",
    "Давыдов",
    "Мельников",
    "Щербаков",
    "Блинов",
    "Колесников",
    "Карпов",
    "Афанасьев",
    "Власов",
    "Маслов",
    "Исаков",
    "Тихонов",
    "Аксёнов",
    "Гаврилов",
    "Родионов",
    "Котов",
    "Горбунов",
    "Кудряшов",
    "Быков",
    "Зуев",
    "Третьяков",
    "Савельев",
    "Панов",
    "Рыбаков",
    "Суворов",
    "Абрамов",
    "Воронов",
    "Мухин",
    "Архипов",
    "Трофимов",
    "Мартынов",
    "Емельянов",
    "Горшков",
    "Чернов",
    "Овчинников",
    "Селезнёв",
    "Панфилов",
    "Копылов",
    "Михеев",
    "Галкин",
    "Назаров",
    "Лобанов",
    "Лукин",
    "Беляков",
    "Потапов",
    "Некрасов",
    "Хохлов",
    "Жданов",
    "Наумов",
    "Шилов",
    "Воронцов",
    "Ермаков",
    "Дроздов",
    "Игнатьев",
    "Савин",
    "Логинов",
    "Сафонов",
    "Капустин",
    "Кириллов",
    "Моисеев",
    "Елисеев",
    "Кошелев",
    "Костин",
    "Горбачёв",
    "Пушкин",
    "Чехов",
    "Толстой",
    "Достоевский",
    "Чайковский",
    "Маяковский",
    "Успенский",
    "Вишневский",
    "Покровский",
    "Островский",
    "Высоцкий",
    "Троцкий",
    "Вяземский",
    "Донской",
];

pub fn surnames() -> Vec<NewLexeme> {
    (1..)
        .zip(SURNAMES)
        .filter_map(|(rank, &name)| {
            Some(NewLexeme {
                forms: surname::decline(name)?,
                ..lexeme("surname", name, name, PartOfSpeech::Surname, rank)
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forms_of(l: &NewLexeme, slot: Slot) -> Vec<&str> {
        l.forms
            .iter()
            .filter(|f| f.slot == slot)
            .map(|f| f.form.as_str())
            .collect()
    }

    #[test]
    fn personal_pronouns_have_whole_forms_and_n_variants() {
        let pronouns = personal_pronouns();
        assert_eq!(pronouns.len(), PERSONAL_PRONOUNS.len());
        let on = pronouns.iter().find(|p| p.lemma == "он").unwrap();
        assert_eq!(on.pos, PartOfSpeech::PersonalPronoun);
        assert_eq!(
            forms_of(on, Slot::new(Case::Genitive, Number::Singular)),
            ["его", "него"]
        );
        assert!(on.forms.iter().all(|f| f.irregular && f.stem.is_none()));

        let sebya = pronouns.iter().find(|p| p.lemma == "себя").unwrap();
        assert!(forms_of(sebya, Slot::new(Case::Nominative, Number::Singular)).is_empty());
        assert_eq!(sebya.translations.len(), 3);
    }

    #[test]
    fn nash_and_vash_decline_like_pronouns() {
        let pronouns = possessive_pronouns();
        let vash = pronouns.iter().find(|p| p.lemma == "ваш").unwrap();
        let f_ins = Slot::new(Case::Instrumental, Number::Singular).gender(Gender::Feminine);
        assert_eq!(forms_of(vash, f_ins), ["вашей", "вашею"]);
        let m_gen = Slot::new(Case::Genitive, Number::Singular).gender(Gender::Masculine);
        let form = vash.forms.iter().find(|f| f.slot == m_gen).unwrap();
        assert_eq!(form.stressed_form, "ва\u{301}шего");
        assert_eq!(form.ending.as_deref(), Some("его"));
        assert!(vash.forms.iter().all(|f| !f.irregular));
        // 24 slots, the masculine and plural accusative doubled by animacy, plus -ею.
        assert_eq!(vash.forms.len(), 24 + 2 + 1);
    }

    #[test]
    fn every_listed_surname_declines() {
        let lexemes = surnames();
        assert_eq!(lexemes.len(), SURNAMES.len());
        let mut unique: Vec<_> = SURNAMES.to_vec();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), SURNAMES.len(), "duplicates in the list");
    }
}
