//! Parser for the OpenRussian.org dumps (tab-separated, CC BY-SA 4.0):
//! https://github.com/Badestrand/russian-dictionary
//!
//! Quirks handled here:
//! - stress is an apostrophe after the vowel (кни'га);
//! - a cell may hold several forms separated by `,` or `//` (стороно'й, стороно'ю);
//! - a leading `*` marks a rare or doubtful form, which is skipped;
//! - a few lemmas contain Latin look-alike letters (рoга), and those rows are skipped.

use std::collections::HashMap;

use domain::grammar::{Animacy, Case, Gender, Locale, Number, PartOfSpeech};
use domain::lexicon::{NewForm, NewLexeme, NewTranslation, Slot};
use domain::stress;

use crate::declension::{self, adjective_like_forms};

pub const SOURCE: &str = "openrussian";

/// Pinned so row numbers, which serve as source ids, stay stable across imports.
pub const COMMIT: &str = "50e210c4803237779cb562bc1abcea529066031c";

pub fn url(file: &str) -> String {
    format!("https://raw.githubusercontent.com/Badestrand/russian-dictionary/{COMMIT}/{file}")
}

/// Words that OpenRussian lists as adjectives but grammar calls pronouns. Their
/// stem comes from their forms (мо|его), not from the lemma.
pub const PRONOUNS: &[&str] = &[
    "мой",
    "твой",
    "свой",
    "этот",
    "тот",
    "весь",
    "сам",
    "чей",
    "ничей",
    "сей",
    "какой",
    "такой",
    "который",
    "каждый",
    "никакой",
    "некоторый",
    "другой",
    "любой",
    "самый",
    "всякий",
    "иной",
];

const NOUN_FORM_COLUMNS: [(&str, Case, Number); 12] = [
    ("sg_nom", Case::Nominative, Number::Singular),
    ("sg_gen", Case::Genitive, Number::Singular),
    ("sg_dat", Case::Dative, Number::Singular),
    ("sg_acc", Case::Accusative, Number::Singular),
    ("sg_inst", Case::Instrumental, Number::Singular),
    ("sg_prep", Case::Prepositional, Number::Singular),
    ("pl_nom", Case::Nominative, Number::Plural),
    ("pl_gen", Case::Genitive, Number::Plural),
    ("pl_dat", Case::Dative, Number::Plural),
    ("pl_acc", Case::Accusative, Number::Plural),
    ("pl_inst", Case::Instrumental, Number::Plural),
    ("pl_prep", Case::Prepositional, Number::Plural),
];

#[derive(Debug, Default)]
pub struct Parsed {
    pub lexemes: Vec<NewLexeme>,
    /// Rows whose lemma isn't purely Cyrillic.
    pub skipped_non_cyrillic: usize,
    /// Rows without a single usable form.
    pub skipped_without_forms: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("the file is empty")]
    Empty,
    #[error("missing column `{0}`")]
    MissingColumn(String),
}

/// A TSV file with named columns.
struct Table<'a> {
    header: HashMap<&'a str, usize>,
    lines: std::str::Lines<'a>,
}

impl<'a> Table<'a> {
    fn new(tsv: &'a str) -> Result<Self, ParseError> {
        let mut lines = tsv.lines();
        let header = lines
            .next()
            .ok_or(ParseError::Empty)?
            .trim_start_matches('\u{feff}')
            .split('\t')
            .enumerate()
            .map(|(i, name)| (name.trim(), i))
            .collect();
        Ok(Table { header, lines })
    }

    fn column(&self, name: &str) -> Result<usize, ParseError> {
        self.header
            .get(name)
            .copied()
            .ok_or_else(|| ParseError::MissingColumn(name.to_owned()))
    }

    /// Data rows, numbered from 1 (blank lines keep their number but are skipped).
    fn rows(self) -> impl Iterator<Item = Row<'a>> {
        (1..)
            .zip(self.lines)
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(number, line)| Row {
                number,
                cells: line.split('\t').collect(),
            })
    }
}

struct Row<'a> {
    number: i32,
    cells: Vec<&'a str>,
}

impl<'a> Row<'a> {
    fn cell(&self, column: usize) -> &'a str {
        self.cells.get(column).map_or("", |c| c.trim())
    }
}

/// Columns every OpenRussian file has.
struct Basics {
    bare: usize,
    accented: usize,
    translations_en: usize,
    translations_de: usize,
}

impl Basics {
    fn new(table: &Table) -> Result<Self, ParseError> {
        Ok(Basics {
            bare: table.column("bare")?,
            accented: table.column("accented")?,
            translations_en: table.column("translations_en")?,
            translations_de: table.column("translations_de")?,
        })
    }

    /// The lexeme of a row in `file`, without forms or grammatical details yet.
    fn lexeme(&self, file: &str, row: &Row, pos: PartOfSpeech) -> NewLexeme {
        NewLexeme {
            source: SOURCE.to_owned(),
            source_id: format!("{file}:{}", row.number),
            lemma: row.cell(self.bare).to_owned(),
            stressed_lemma: Some(row.cell(self.accented))
                .filter(|s| !s.is_empty())
                .map(stress::from_apostrophes),
            pos,
            gender: None,
            animacy: None,
            indeclinable: false,
            frequency_rank: Some(row.number),
            forms: Vec::new(),
            translations: translations(&[
                (Locale::En, row.cell(self.translations_en)),
                (Locale::De, row.cell(self.translations_de)),
            ]),
        }
    }
}

/// Translation cells hold meanings separated by `;`.
fn translations(cells: &[(Locale, &str)]) -> Vec<NewTranslation> {
    cells
        .iter()
        .flat_map(|&(locale, cell)| {
            (0..)
                .zip(cell.split(';').map(str::trim).filter(|t| !t.is_empty()))
                .map(move |(position, text)| NewTranslation {
                    locale,
                    position,
                    text: text.to_owned(),
                })
        })
        .collect()
}

pub fn parse_nouns(tsv: &str) -> Result<Parsed, ParseError> {
    let table = Table::new(tsv)?;
    let basics = Basics::new(&table)?;
    let gender = table.column("gender")?;
    let animate = table.column("animate")?;
    let indeclinable = table.column("indeclinable")?;
    let form_columns = NOUN_FORM_COLUMNS
        .iter()
        .map(|&(name, case, number)| Ok((table.column(name)?, Slot::new(case, number))))
        .collect::<Result<Vec<_>, ParseError>>()?;

    let mut parsed = Parsed::default();
    for row in table.rows() {
        let lemma = row.cell(basics.bare);
        if !stress::is_russian(lemma) {
            parsed.skipped_non_cyrillic += 1;
            continue;
        }
        let is_indeclinable = row.cell(indeclinable) == "1";

        let forms: Vec<NewForm> = form_columns
            .iter()
            .flat_map(|&(column, slot)| {
                (0..)
                    .zip(variants(row.cell(column)))
                    .map(move |(variant, stressed)| {
                        NewForm::noun(lemma, &stressed, slot, variant, is_indeclinable)
                    })
            })
            .collect();
        if forms.is_empty() {
            parsed.skipped_without_forms += 1;
            continue;
        }

        parsed.lexemes.push(NewLexeme {
            gender: match row.cell(gender) {
                "m" => Some(Gender::Masculine),
                "f" => Some(Gender::Feminine),
                "n" => Some(Gender::Neuter),
                _ => None, // "both" (сирота), "pl", or unknown
            },
            animacy: match row.cell(animate) {
                "1" => Some(Animacy::Animate),
                "0" => Some(Animacy::Inanimate),
                _ => None,
            },
            indeclinable: is_indeclinable,
            forms,
            ..basics.lexeme("nouns", &row, PartOfSpeech::Noun)
        });
    }
    Ok(parsed)
}

/// Adjectives, and the pronouns in [`PRONOUNS`] that OpenRussian files with them.
pub fn parse_adjectives(tsv: &str) -> Result<Parsed, ParseError> {
    let table = Table::new(tsv)?;
    let basics = Basics::new(&table)?;
    let mut columns: HashMap<(Number, Option<Gender>, Case), usize> = HashMap::new();
    for (number, gender, column) in declension::COLUMNS {
        for (case, case_name) in declension::CASES {
            let index = table.column(&format!("decl_{column}_{case_name}"))?;
            columns.insert((number, gender, case), index);
        }
    }

    let mut parsed = Parsed::default();
    for row in table.rows() {
        let lemma = row.cell(basics.bare);
        if !stress::is_russian(lemma) {
            parsed.skipped_non_cyrillic += 1;
            continue;
        }
        let pos = if PRONOUNS.contains(&lemma) {
            PartOfSpeech::Pronoun
        } else {
            PartOfSpeech::Adjective
        };
        let forms = adjective_like_forms(lemma, pos, |number, gender, case| {
            variants(row.cell(columns[&(number, gender, case)]))
        });
        if forms.is_empty() {
            parsed.skipped_without_forms += 1;
            continue;
        }
        parsed.lexemes.push(NewLexeme {
            forms,
            ..basics.lexeme("adjectives", &row, pos)
        });
    }
    Ok(parsed)
}

/// The usable forms in a cell, with stress converted to U+0301.
pub fn variants(cell: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in cell.split("//").flat_map(|part| part.split(',')) {
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('*') {
            continue;
        }
        let stressed = stress::from_apostrophes(raw);
        if stress::is_russian(&stressed) && !out.contains(&stressed) {
            out.push(stressed);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOUN_HEADER: &str = "bare\taccented\ttranslations_en\ttranslations_de\tgender\tpartner\tanimate\tindeclinable\tsg_only\tpl_only\tsg_nom\tsg_gen\tsg_dat\tsg_acc\tsg_inst\tsg_prep\tpl_nom\tpl_gen\tpl_dat\tpl_acc\tpl_inst\tpl_prep";

    fn nouns(rows: &[&str]) -> Parsed {
        let tsv = std::iter::once(NOUN_HEADER)
            .chain(rows.iter().copied())
            .collect::<Vec<_>>()
            .join("\n");
        parse_nouns(&tsv).unwrap()
    }

    fn form(l: &NewLexeme, slot: Slot, variant: i16) -> &NewForm {
        l.forms
            .iter()
            .find(|f| f.slot == slot && f.variant == variant)
            .unwrap()
    }

    fn texts(l: &NewLexeme, locale: Locale) -> Vec<&str> {
        l.translations
            .iter()
            .filter(|t| t.locale == locale)
            .map(|t| t.text.as_str())
            .collect()
    }

    const KNIGA: &str = "книга\tкни'га\tbook; volume\tBuch\tf\t\t0\t0\t0\t0\tкни'га\tкни'ги\tкни'ге\tкни'гу\tкни'гой, кни'гою\tкни'ге\tкни'ги\tкни'г\tкни'гам\tкни'ги\tкни'гами\tкни'гах";

    #[test]
    fn parses_a_regular_noun() {
        let parsed = nouns(&[KNIGA]);
        let kniga = &parsed.lexemes[0];

        assert_eq!(kniga.source_id, "nouns:1");
        assert_eq!(kniga.frequency_rank, Some(1));
        assert_eq!(kniga.stressed_lemma.as_deref(), Some("кни\u{301}га"));
        assert_eq!(kniga.gender, Some(Gender::Feminine));
        assert_eq!(kniga.animacy, Some(Animacy::Inanimate));
        assert_eq!(kniga.forms.len(), 13);

        let ins = Slot::new(Case::Instrumental, Number::Singular);
        assert_eq!(form(kniga, ins, 0).form, "книгой");
        assert_eq!(form(kniga, ins, 0).ending.as_deref(), Some("ой"));
        assert_eq!(form(kniga, ins, 1).form, "книгою");

        assert_eq!(texts(kniga, Locale::En), ["book", "volume"]);
        assert_eq!(texts(kniga, Locale::De), ["Buch"]);
    }

    #[test]
    fn skips_starred_and_odd_variants() {
        assert_eq!(variants("*све'ты"), Vec::<String>::new());
        assert_eq!(
            variants("зверя'ми//зверьми'"),
            ["зверя\u{301}ми", "зверьми\u{301}"]
        );
        assert_eq!(
            variants("ного'й,     ного'ю"),
            ["ного\u{301}й", "ного\u{301}ю"]
        );
        assert_eq!(variants("MP3"), Vec::<String>::new());
    }

    #[test]
    fn skips_bad_rows() {
        let latin_o = "рoга\tрo'га\thorn\t\tf\t\t0\t0\t0\t0\tрo'га\t\t\t\t\t\t\t\t\t\t\t";
        let no_forms = "ммм\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t";
        let parsed = nouns(&[latin_o, no_forms, KNIGA]);

        assert_eq!(parsed.skipped_non_cyrillic, 1);
        assert_eq!(parsed.skipped_without_forms, 1);
        assert_eq!(parsed.lexemes.len(), 1);
        assert_eq!(parsed.lexemes[0].source_id, "nouns:3");
    }

    #[test]
    fn common_gender_and_indeclinable() {
        let sirota = "сирота\tсирота'\torphan\t\tboth\t\t1\t0\t0\t0\tсирота'\t\t\t\t\t\t\t\t\t\t\t";
        let kofe = "кофе\tко'фе\tcoffee\t\tm\t\t0\t1\t1\t0\tко'фе\tко'фе\tко'фе\tко'фе\tко'фе\tко'фе\t\t\t\t\t\t";
        let parsed = nouns(&[sirota, kofe]);

        assert_eq!(parsed.lexemes[0].gender, None);
        assert_eq!(parsed.lexemes[0].animacy, Some(Animacy::Animate));
        assert!(parsed.lexemes[1].indeclinable);
        assert!(parsed.lexemes[1].forms.iter().all(|f| f.irregular));
    }

    #[test]
    fn missing_column_is_an_error() {
        assert!(matches!(
            parse_nouns("bare\taccented\n"),
            Err(ParseError::MissingColumn(c)) if c == "translations_en"
        ));
    }

    const ADJECTIVE_HEADER: &str = "bare\taccented\ttranslations_en\ttranslations_de\tcomparative\tsuperlative\tshort_m\tshort_f\tshort_n\tshort_pl\tdecl_m_nom\tdecl_m_gen\tdecl_m_dat\tdecl_m_acc\tdecl_m_inst\tdecl_m_prep\tdecl_f_nom\tdecl_f_gen\tdecl_f_dat\tdecl_f_acc\tdecl_f_inst\tdecl_f_prep\tdecl_n_nom\tdecl_n_gen\tdecl_n_dat\tdecl_n_acc\tdecl_n_inst\tdecl_n_prep\tdecl_pl_nom\tdecl_pl_gen\tdecl_pl_dat\tdecl_pl_acc\tdecl_pl_inst\tdecl_pl_prep";

    fn adjectives(rows: &[&str]) -> Parsed {
        let tsv = std::iter::once(ADJECTIVE_HEADER)
            .chain(rows.iter().copied())
            .collect::<Vec<_>>()
            .join("\n");
        parse_adjectives(&tsv).unwrap()
    }

    // Real rows from adjectives.csv (note русский lists the animate accusative first).
    const NOVY: &str = "новый\tно'вый\tnew\tneu\tнове'е\t\tнов\tнова'\tно'во\tно'вы\tно'вый\tно'вого\tно'вому\tно'вый, но'вого\tно'вым\tно'вом\tно'вая\tно'вой\tно'вой\tно'вую\tно'вой, но'вою\tно'вой\tно'вое\tно'вого\tно'вому\tно'вое\tно'вым\tно'вом\tно'вые\tно'вых\tно'вым\tно'вые, но'вых\tно'выми\tно'вых";
    const MOY: &str = "мой\tмо'й\tmy, mine\tmein\t\t\t\t\t\t\tмо'й\tмоего'\tмоему'\tмо'й, моего'\tмои'м\tмоём\tмоя'\tмое'й\tмое'й\tмою'\tмое'й\tмое'й\tмоё\tмоего'\tмоему'\tмоё\tмои'м\tмоём\tмои'\tмои'х\tмои'м\tмои', мои'х\tмои'ми\tмои'х";

    #[test]
    fn parses_adjectives_with_gender_and_animacy() {
        let parsed = adjectives(&[
            NOVY,
            "как\tкак\thow\twie\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t",
        ]);
        assert_eq!(parsed.skipped_without_forms, 1);
        let novy = &parsed.lexemes[0];
        assert_eq!(novy.pos, PartOfSpeech::Adjective);
        assert_eq!(novy.source_id, "adjectives:1");
        assert_eq!(texts(novy, Locale::De), ["neu"]);

        let f_ins = Slot::new(Case::Instrumental, Number::Singular).gender(Gender::Feminine);
        assert_eq!(form(novy, f_ins, 0).ending.as_deref(), Some("ой"));
        assert_eq!(form(novy, f_ins, 1).ending.as_deref(), Some("ою"));
        let m_acc_anim = Slot::new(Case::Accusative, Number::Singular)
            .gender(Gender::Masculine)
            .animacy(Animacy::Animate);
        assert_eq!(form(novy, m_acc_anim, 0).form, "нового");
        assert!(
            novy.forms.iter().all(|f| !f.irregular),
            "{:?}",
            novy.forms.iter().find(|f| f.irregular)
        );
    }

    #[test]
    fn pronominal_adjectives_become_pronouns() {
        let moy = &adjectives(&[MOY]).lexemes[0];
        assert_eq!(moy.pos, PartOfSpeech::Pronoun);
        let m_gen = Slot::new(Case::Genitive, Number::Singular).gender(Gender::Masculine);
        let f = form(moy, m_gen, 0);
        assert_eq!(
            (f.stem.as_deref(), f.ending.as_deref()),
            (Some("мо"), Some("его"))
        );
    }
}
