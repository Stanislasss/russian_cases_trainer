//! Bulk writes into the lexicon tables, used by the importer.

use std::collections::HashMap;

use domain::lexicon::NewLexeme;
use sqlx::{Postgres, Transaction};

/// Inserts or updates `lexemes` (matched on `source` + `source_id`) and replaces
/// their forms and translations. Each table is written with one statement per
/// call, using arrays + UNNEST, so callers should pass batches of a few thousand.
pub async fn upsert_lexemes(
    tx: &mut Transaction<'_, Postgres>,
    lexemes: &[NewLexeme],
) -> Result<(), sqlx::Error> {
    if lexemes.is_empty() {
        return Ok(());
    }

    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "INSERT INTO lexemes
             (source, source_id, lemma, stressed_lemma, pos, gender, animacy, indeclinable, frequency_rank)
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::text[],
                              $6::text[], $7::text[], $8::bool[], $9::int4[])
         ON CONFLICT (source, source_id) DO UPDATE SET
             lemma = EXCLUDED.lemma,
             stressed_lemma = EXCLUDED.stressed_lemma,
             pos = EXCLUDED.pos,
             gender = EXCLUDED.gender,
             animacy = EXCLUDED.animacy,
             indeclinable = EXCLUDED.indeclinable,
             frequency_rank = EXCLUDED.frequency_rank
         RETURNING id, source, source_id",
    )
    .bind(lexemes.iter().map(|l| l.source.clone()).collect::<Vec<_>>())
    .bind(lexemes.iter().map(|l| l.source_id.clone()).collect::<Vec<_>>())
    .bind(lexemes.iter().map(|l| l.lemma.clone()).collect::<Vec<_>>())
    .bind(lexemes.iter().map(|l| l.stressed_lemma.clone()).collect::<Vec<_>>())
    .bind(lexemes.iter().map(|l| l.pos.as_str()).collect::<Vec<_>>())
    .bind(lexemes.iter().map(|l| l.gender.map(|g| g.as_str())).collect::<Vec<_>>())
    .bind(lexemes.iter().map(|l| l.animacy.map(|a| a.as_str())).collect::<Vec<_>>())
    .bind(lexemes.iter().map(|l| l.indeclinable).collect::<Vec<_>>())
    .bind(lexemes.iter().map(|l| l.frequency_rank).collect::<Vec<_>>())
    .fetch_all(&mut **tx)
    .await?;

    // RETURNING order isn't guaranteed to follow the input, so match on the key.
    let ids: HashMap<(&str, &str), i64> = rows
        .iter()
        .map(|(id, source, source_id)| ((source.as_str(), source_id.as_str()), *id))
        .collect();
    let id_of = |l: &NewLexeme| ids[&(l.source.as_str(), l.source_id.as_str())];
    let all_ids: Vec<i64> = lexemes.iter().map(id_of).collect();

    sqlx::query("DELETE FROM forms WHERE lexeme_id = ANY($1)")
        .bind(&all_ids)
        .execute(&mut **tx)
        .await?;
    sqlx::query("DELETE FROM translations WHERE lexeme_id = ANY($1)")
        .bind(&all_ids)
        .execute(&mut **tx)
        .await?;

    let forms: Vec<_> = lexemes
        .iter()
        .flat_map(|l| l.forms.iter().map(move |f| (id_of(l), f)))
        .collect();
    sqlx::query(
        "INSERT INTO forms
             (lexeme_id, grammatical_case, number, variant, form, stressed_form, stem, ending, irregular)
         SELECT * FROM UNNEST($1::int8[], $2::text[], $3::text[], $4::int2[], $5::text[],
                              $6::text[], $7::text[], $8::text[], $9::bool[])",
    )
    .bind(forms.iter().map(|(id, _)| *id).collect::<Vec<_>>())
    .bind(forms.iter().map(|(_, f)| f.case.as_str()).collect::<Vec<_>>())
    .bind(forms.iter().map(|(_, f)| f.number.as_str()).collect::<Vec<_>>())
    .bind(forms.iter().map(|(_, f)| f.variant).collect::<Vec<_>>())
    .bind(forms.iter().map(|(_, f)| f.form.clone()).collect::<Vec<_>>())
    .bind(forms.iter().map(|(_, f)| f.stressed_form.clone()).collect::<Vec<_>>())
    .bind(forms.iter().map(|(_, f)| f.stem.clone()).collect::<Vec<_>>())
    .bind(forms.iter().map(|(_, f)| f.ending.clone()).collect::<Vec<_>>())
    .bind(forms.iter().map(|(_, f)| f.irregular).collect::<Vec<_>>())
    .execute(&mut **tx)
    .await?;

    let translations: Vec<_> = lexemes
        .iter()
        .flat_map(|l| l.translations.iter().map(move |t| (id_of(l), t)))
        .collect();
    sqlx::query(
        "INSERT INTO translations (lexeme_id, locale, position, text)
         SELECT * FROM UNNEST($1::int8[], $2::text[], $3::int2[], $4::text[])",
    )
    .bind(translations.iter().map(|(id, _)| *id).collect::<Vec<_>>())
    .bind(
        translations
            .iter()
            .map(|(_, t)| t.locale.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        translations
            .iter()
            .map(|(_, t)| t.position)
            .collect::<Vec<_>>(),
    )
    .bind(
        translations
            .iter()
            .map(|(_, t)| t.text.clone())
            .collect::<Vec<_>>(),
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
