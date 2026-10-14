//! Parser for the OpenRussian.org noun dump (tab-separated, CC BY-SA 4.0):
//! https://github.com/Badestrand/russian-dictionary
//!
//! Quirks handled here:
//! - stress is an apostrophe after the vowel (кни'га);
//! - a cell may hold several forms separated by `,` or `//` (стороно'й, стороно'ю);
//! - a leading `*` marks a rare or doubtful form, which is skipped;
//! - a few lemmas contain Latin look-alike letters (рoга), and those rows are skipped.

use std::collections::HashMap;

use domain::grammar::{Animacy, Case, Gender, Locale, Number, PartOfSpeech};
use domain::lexicon::{NewForm, NewLexeme, NewTranslation};
use domain::stress;

pub const SOURCE: &str = "openrussian";

/// Pinned so row numbers, which serve as source ids, stay stable across imports.
pub const COMMIT: &str = "50e210c4803237779cb562bc1abcea529066031c";

pub fn nouns_url() -> String {
    format!("https://raw.githubusercontent.com/Badestrand/russian-dictionary/{COMMIT}/nouns.csv")
}

const FORM_COLUMNS: [(&str, Case, Number); 12] = [
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
pub struct ParsedNouns {
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
    MissingColumn(&'static str),
}

pub fn parse_nouns(tsv: &str) -> Result<ParsedNouns, ParseError> {
    let mut lines = tsv.lines();
    let header: HashMap<&str, usize> = lines
        .next()
        .ok_or(ParseError::Empty)?
        .trim_start_matches('\u{feff}')
        .split('\t')
        .enumerate()
        .map(|(i, name)| (name.trim(), i))
        .collect();
    let column = |name: &'static str| {
        header
            .get(name)
            .copied()
            .ok_or(ParseError::MissingColumn(name))
    };

    let bare = column("bare")?;
    let accented = column("accented")?;
    let translations_en = column("translations_en")?;
    let gender = column("gender")?;
    let animate = column("animate")?;
    let indeclinable = column("indeclinable")?;
    let form_columns = FORM_COLUMNS
        .iter()
        .map(|&(name, case, number)| Ok((column(name)?, case, number)))
        .collect::<Result<Vec<_>, ParseError>>()?;

    let mut parsed = ParsedNouns::default();
    for (row, line) in (1..).zip(lines) {
        if line.trim().is_empty() {
            continue;
        }
        let cells: Vec<&str> = line.split('\t').collect();
        let cell = |i: usize| cells.get(i).map_or("", |c| c.trim());

        let lemma = cell(bare);
        if !is_russian_word(lemma) {
            parsed.skipped_non_cyrillic += 1;
            continue;
        }
        let is_indeclinable = cell(indeclinable) == "1";

        let mut forms = Vec::new();
        for &(i, case, number) in &form_columns {
            for (variant, stressed) in (0..).zip(variants(cell(i))) {
                forms.push(NewForm::noun(
                    lemma,
                    &stressed,
                    case,
                    number,
                    variant,
                    is_indeclinable,
                ));
            }
        }
        if forms.is_empty() {
            parsed.skipped_without_forms += 1;
            continue;
        }

        parsed.lexemes.push(NewLexeme {
            source: SOURCE.to_owned(),
            source_id: format!("nouns:{row}"),
            lemma: lemma.to_owned(),
            stressed_lemma: Some(cell(accented))
                .filter(|s| !s.is_empty())
                .map(stress::from_apostrophes),
            pos: PartOfSpeech::Noun,
            gender: match cell(gender) {
                "m" => Some(Gender::Masculine),
                "f" => Some(Gender::Feminine),
                "n" => Some(Gender::Neuter),
                _ => None, // "both" (сирота), "pl", or unknown
            },
            animacy: match cell(animate) {
                "1" => Some(Animacy::Animate),
                "0" => Some(Animacy::Inanimate),
                _ => None,
            },
            indeclinable: is_indeclinable,
            frequency_rank: Some(row),
            forms,
            translations: (0..)
                .zip(
                    cell(translations_en)
                        .split(';')
                        .map(str::trim)
                        .filter(|t| !t.is_empty()),
                )
                .map(|(position, text)| NewTranslation {
                    locale: Locale::En,
                    position,
                    text: text.to_owned(),
                })
                .collect(),
        });
    }
    Ok(parsed)
}

/// The usable forms in a cell, with stress converted to U+0301.
fn variants(cell: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in cell.split("//").flat_map(|part| part.split(',')) {
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('*') {
            continue;
        }
        let stressed = stress::from_apostrophes(raw);
        if is_russian_word(&stress::strip(&stressed)) && !out.contains(&stressed) {
            out.push(stressed);
        }
    }
    out
}

/// Cyrillic letters, optionally joined by hyphens or spaces (пресс-конференция).
fn is_russian_word(word: &str) -> bool {
    word.chars().any(|c| c.is_alphabetic())
        && word
            .chars()
            .all(|c| matches!(c, 'а'..='я' | 'А'..='Я' | 'ё' | 'Ё' | '-' | ' '))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "bare\taccented\ttranslations_en\ttranslations_de\tgender\tpartner\tanimate\tindeclinable\tsg_only\tpl_only\tsg_nom\tsg_gen\tsg_dat\tsg_acc\tsg_inst\tsg_prep\tpl_nom\tpl_gen\tpl_dat\tpl_acc\tpl_inst\tpl_prep";

    fn parse(rows: &[&str]) -> ParsedNouns {
        let tsv = std::iter::once(HEADER)
            .chain(rows.iter().copied())
            .collect::<Vec<_>>()
            .join("\n");
        parse_nouns(&tsv).unwrap()
    }

    fn form(l: &NewLexeme, case: Case, number: Number, variant: i16) -> &NewForm {
        l.forms
            .iter()
            .find(|f| f.case == case && f.number == number && f.variant == variant)
            .unwrap()
    }

    const KNIGA: &str = "книга\tкни'га\tbook; volume\tBuch\tf\t\t0\t0\t0\t0\tкни'га\tкни'ги\tкни'ге\tкни'гу\tкни'гой, кни'гою\tкни'ге\tкни'ги\tкни'г\tкни'гам\tкни'ги\tкни'гами\tкни'гах";

    #[test]
    fn parses_a_regular_noun() {
        let parsed = parse(&[KNIGA]);
        let kniga = &parsed.lexemes[0];

        assert_eq!(kniga.source_id, "nouns:1");
        assert_eq!(kniga.frequency_rank, Some(1));
        assert_eq!(kniga.stressed_lemma.as_deref(), Some("кни\u{301}га"));
        assert_eq!(kniga.gender, Some(Gender::Feminine));
        assert_eq!(kniga.animacy, Some(Animacy::Inanimate));
        assert_eq!(kniga.forms.len(), 13);

        let ins = form(kniga, Case::Instrumental, Number::Singular, 0);
        assert_eq!(ins.form, "книгой");
        assert_eq!(ins.stem.as_deref(), Some("книг"));
        assert_eq!(ins.ending.as_deref(), Some("ой"));
        assert_eq!(
            form(kniga, Case::Instrumental, Number::Singular, 1).form,
            "книгою"
        );

        let texts: Vec<_> = kniga.translations.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, ["book", "volume"]);
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
        let parsed = parse(&[latin_o, no_forms, KNIGA]);

        assert_eq!(parsed.skipped_non_cyrillic, 1);
        assert_eq!(parsed.skipped_without_forms, 1);
        assert_eq!(parsed.lexemes.len(), 1);
        assert_eq!(parsed.lexemes[0].source_id, "nouns:3");
    }

    #[test]
    fn common_gender_and_indeclinable() {
        let sirota = "сирота\tсирота'\torphan\t\tboth\t\t1\t0\t0\t0\tсирота'\t\t\t\t\t\t\t\t\t\t\t";
        let kofe = "кофе\tко'фе\tcoffee\t\tm\t\t0\t1\t1\t0\tко'фе\tко'фе\tко'фе\tко'фе\tко'фе\tко'фе\t\t\t\t\t\t";
        let parsed = parse(&[sirota, kofe]);

        assert_eq!(parsed.lexemes[0].gender, None);
        assert_eq!(parsed.lexemes[0].animacy, Some(Animacy::Animate));
        assert!(parsed.lexemes[1].indeclinable);
        assert!(parsed.lexemes[1].forms.iter().all(|f| f.irregular));
    }

    #[test]
    fn missing_column_is_an_error() {
        assert!(matches!(
            parse_nouns("bare\taccented\n"),
            Err(ParseError::MissingColumn("translations_en"))
        ));
    }
}
