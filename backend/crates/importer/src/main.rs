//! Loads the lexicon into Postgres: nouns, adjectives and pronouns from
//! OpenRussian.org (CC BY-SA), hand-written pronouns and surnames, French
//! translations and Russian definitions from Wiktionary (CC BY-SA).
//! Safe to re-run: words are matched on their source id and updated in place.

mod curated;
mod declension;
mod openrussian;
mod wiktionary;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;
use domain::lexicon::NewLexeme;
use tokio::io::AsyncWriteExt;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(about = "Import the lexicon")]
struct Cli {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
    /// Where downloaded files are cached.
    #[arg(long, env = "IMPORT_DATA_DIR", default_value = "data")]
    data_dir: PathBuf,
    /// Skip the French Wiktionary (a 176 MB download); French then falls back to English.
    #[arg(long)]
    no_french: bool,
    /// Skip the Russian Wiktionary (a 1.7 GB download); Russian then falls back to English.
    #[arg(long)]
    no_russian: bool,
    #[arg(long, default_value_t = 2000)]
    batch_size: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let cli = Cli::parse();
    let short_commit = &openrussian::COMMIT[..12];

    type Parser = fn(&str) -> Result<openrussian::Parsed, openrussian::ParseError>;
    let files: [(&str, Parser); 2] = [
        ("nouns", openrussian::parse_nouns),
        ("adjectives", openrussian::parse_adjectives),
    ];
    let mut lexemes: Vec<NewLexeme> = Vec::new();
    for (file, parse) in files {
        let path = download(
            &openrussian::url(&format!("{file}.csv")),
            &cli.data_dir.join(format!("{file}-{short_commit}.csv")),
        )
        .await?;
        let tsv = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("reading {}", path.display()))?;
        let parsed = parse(&tsv).with_context(|| format!("parsing {file}"))?;
        tracing::info!(
            file,
            words = parsed.lexemes.len(),
            skipped_non_cyrillic = parsed.skipped_non_cyrillic,
            skipped_without_forms = parsed.skipped_without_forms,
            "parsed"
        );
        lexemes.extend(parsed.lexemes);
    }
    lexemes.extend(curated::personal_pronouns());
    lexemes.extend(curated::possessive_pronouns());
    lexemes.extend(curated::surnames());

    for (edition, skip) in [
        (&wiktionary::FRENCH, cli.no_french),
        (&wiktionary::RUSSIAN, cli.no_russian),
    ] {
        let locale = edition.locale.as_str();
        if skip {
            tracing::info!(locale, "skipping Wiktionary");
            continue;
        }
        match load_wiktionary(edition, &cli.data_dir).await {
            Ok(dictionary) => {
                let translated = dictionary.translate(&mut lexemes);
                tracing::info!(locale, entries = dictionary.len(), translated, "Wiktionary");
            }
            // kaikki.org marks these downloads as deprecated; don't fail the whole import.
            Err(e) => {
                tracing::warn!(locale, error = %e, "no Wiktionary data; English is used instead")
            }
        }
    }

    log_summary(&lexemes);
    let pool = infrastructure::connect(&cli.database_url).await?;
    let mut tx = pool.begin().await?;
    for (i, batch) in lexemes.chunks(cli.batch_size.max(1)).enumerate() {
        infrastructure::lexicon::upsert_lexemes(&mut tx, batch).await?;
        tracing::info!(
            done = (i * cli.batch_size + batch.len()),
            total = lexemes.len(),
            "imported"
        );
    }
    tx.commit().await?;
    tracing::info!("import finished");
    Ok(())
}

async fn load_wiktionary(
    edition: &'static wiktionary::Edition,
    data_dir: &Path,
) -> anyhow::Result<wiktionary::Dictionary> {
    let path = download(edition.url, &data_dir.join(edition.cache_file)).await?;
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&path)?;
        Ok(wiktionary::Dictionary::parse(
            edition,
            std::io::BufReader::new(file),
        )?)
    })
    .await?
}

#[derive(Default)]
struct Summary {
    words: usize,
    forms: usize,
    irregular: usize,
    /// Words with at least one translation, per locale.
    translated: BTreeMap<&'static str, usize>,
}

/// Words, forms and translations per part of speech.
fn log_summary(lexemes: &[NewLexeme]) {
    let mut per_pos: BTreeMap<&str, Summary> = BTreeMap::new();
    for l in lexemes {
        let summary = per_pos.entry(l.pos.as_str()).or_default();
        summary.words += 1;
        summary.forms += l.forms.len();
        summary.irregular += l.forms.iter().filter(|f| f.irregular).count();
        // Translations are grouped by locale, so dedup leaves one entry per locale.
        let mut locales: Vec<&'static str> =
            l.translations.iter().map(|t| t.locale.as_str()).collect();
        locales.dedup();
        for locale in locales {
            *summary.translated.entry(locale).or_default() += 1;
        }
    }
    for (pos, s) in per_pos {
        tracing::info!(
            pos,
            words = s.words,
            forms = s.forms,
            irregular = s.irregular,
            translated = ?s.translated,
            "ready to import"
        );
    }
}

/// Downloads `url` to `path` unless it's already there; returns `path`.
async fn download(url: &str, path: &Path) -> anyhow::Result<PathBuf> {
    if tokio::fs::try_exists(path).await? {
        tracing::info!(path = %path.display(), "using cached download");
        return Ok(path.to_owned());
    }
    tracing::info!(%url, "downloading");
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    // Write to a temp file first so an interrupted download isn't mistaken for a cached one.
    let partial = path.with_extension("part");
    let mut response = reqwest::get(url).await?.error_for_status()?;
    let mut file = tokio::fs::File::create(&partial).await?;
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    tokio::fs::rename(&partial, path).await?;
    Ok(path.to_owned())
}
