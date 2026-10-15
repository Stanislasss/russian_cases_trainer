//! Writes `lexicon.sqlite`, the word list shipped with the desktop app.

use std::collections::HashMap;
use std::path::Path;

use application::ports::BoxError;
use domain::game::askable;
use domain::lexicon::NewLexeme;
use sha2::{Digest, Sha256};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{ConnectOptions, Connection};

use crate::lexicon::ANY;

const SCHEMA: &str = include_str!("../../../../migrations-sqlite/lexicon.sql");

/// Only these many meanings per language are shown, so only these are kept.
const MEANINGS: i16 = 3;

/// A word's id, derived from its source key so that it's the same in every build
/// of the word list. 52 bits: JavaScript numbers hold it exactly.
pub fn lexeme_id(source: &str, source_id: &str) -> i64 {
    let hash = Sha256::new()
        .chain_update(source)
        .chain_update([0])
        .chain_update(source_id)
        .finalize();
    let bits = u64::from_be_bytes(hash[..8].try_into().expect("8 bytes")) >> 12;
    i64::try_from(bits).expect("52 bits fit") + 1
}

/// (Re)creates the word list at `path` from the words some level can ask. Later
/// entries with the same source key replace earlier ones, as in Postgres.
/// Returns how many words were written.
pub async fn write_lexicon(path: &Path, lexemes: &[NewLexeme]) -> Result<usize, BoxError> {
    let mut words: HashMap<i64, &NewLexeme> = HashMap::new();
    for word in lexemes.iter().filter(|l| askable(l.pos, l.frequency_rank)) {
        let id = lexeme_id(&word.source, &word.source_id);
        if let Some(other) = words.insert(id, word)
            && (other.source != word.source || other.source_id != word.source_id)
        {
            return Err(format!(
                "id collision between {}:{} and {}:{}",
                other.source, other.source_id, word.source, word.source_id
            )
            .into());
        }
    }

    match tokio::fs::remove_file(path).await {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
        _ => {}
    }
    if let Some(dir) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        tokio::fs::create_dir_all(dir).await?;
    }
    let mut conn = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        // A single file, so it can be opened read-only (no -wal next to it).
        .journal_mode(SqliteJournalMode::Delete)
        .connect()
        .await?;
    sqlx::raw_sql(SCHEMA).execute(&mut conn).await?;

    let mut tx = conn.begin().await?;
    for (&id, word) in &words {
        let rank = word.frequency_rank.expect("askable words have a rank");
        sqlx::query(
            "INSERT INTO lexemes
                 (id, lemma, stressed_lemma, pos, gender, indeclinable, frequency_rank, proper)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(id)
        .bind(&word.lemma)
        .bind(&word.stressed_lemma)
        .bind(word.pos.as_str())
        .bind(word.gender.map(|g| g.as_str()))
        .bind(word.indeclinable)
        .bind(rank)
        .bind(word.lemma != word.lemma.to_lowercase())
        .execute(&mut *tx)
        .await?;

        let lemma = word.lemma.to_lowercase();
        for form in &word.forms {
            sqlx::query(
                "INSERT INTO forms
                     (lexeme_id, grammatical_case, number, gender, animacy, variant,
                      form, stressed_form, ending, irregular, is_lemma)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            )
            .bind(id)
            .bind(form.slot.case.as_str())
            .bind(form.slot.number.as_str())
            .bind(form.slot.gender.map_or(ANY, |g| g.as_str()))
            .bind(form.slot.animacy.map_or(ANY, |a| a.as_str()))
            .bind(form.variant)
            .bind(&form.form)
            .bind(&form.stressed_form)
            .bind(&form.ending)
            .bind(form.irregular)
            .bind(form.form.to_lowercase() == lemma)
            .execute(&mut *tx)
            .await?;
        }

        for translation in word.translations.iter().filter(|t| t.position < MEANINGS) {
            sqlx::query(
                "INSERT INTO translations (lexeme_id, locale, position, text)
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .bind(id)
            .bind(translation.locale.as_str())
            .bind(translation.position)
            .bind(&translation.text)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;

    sqlx::raw_sql("ANALYZE; VACUUM;").execute(&mut conn).await?;
    conn.close().await?;
    Ok(words.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable_and_safe_for_javascript() {
        let id = lexeme_id("openrussian", "123");
        assert_eq!(id, lexeme_id("openrussian", "123"));
        assert_ne!(id, lexeme_id("openrussian", "124"));
        assert_ne!(lexeme_id("a", "bc"), lexeme_id("ab", "c"));
        assert!((1..=1 << 52).contains(&id));
    }

    #[tokio::test]
    async fn missing_folders_are_created() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("build/new/lexicon.sqlite");
        assert_eq!(write_lexicon(&path, &[]).await.unwrap(), 0);
        assert!(path.is_file());
    }
}
