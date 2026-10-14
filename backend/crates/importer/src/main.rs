//! Loads the OpenRussian.org word lists (CC BY-SA) into Postgres.
//! Safe to re-run: words are matched on their source id and updated in place.

mod openrussian;

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(about = "Import the lexicon")]
struct Cli {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
    /// Where downloaded files are cached.
    #[arg(long, env = "IMPORT_DATA_DIR", default_value = "data")]
    data_dir: PathBuf,
    /// Use this nouns file instead of downloading it.
    #[arg(long)]
    nouns_file: Option<PathBuf>,
    #[arg(long, default_value_t = 2000)]
    batch_size: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let cli = Cli::parse();

    let nouns_file = match cli.nouns_file {
        Some(path) => path,
        None => download_nouns(&cli.data_dir).await?,
    };
    let tsv = tokio::fs::read_to_string(&nouns_file)
        .await
        .with_context(|| format!("reading {}", nouns_file.display()))?;
    let parsed = openrussian::parse_nouns(&tsv)?;
    let forms = parsed.lexemes.iter().map(|l| l.forms.len()).sum::<usize>();
    let irregular = parsed
        .lexemes
        .iter()
        .flat_map(|l| &l.forms)
        .filter(|f| f.irregular)
        .count();
    tracing::info!(
        nouns = parsed.lexemes.len(),
        forms,
        irregular_forms = irregular,
        skipped_non_cyrillic = parsed.skipped_non_cyrillic,
        skipped_without_forms = parsed.skipped_without_forms,
        "parsed nouns"
    );

    let pool = infrastructure::connect(&cli.database_url).await?;
    let mut tx = pool.begin().await?;
    for (i, batch) in parsed.lexemes.chunks(cli.batch_size.max(1)).enumerate() {
        infrastructure::lexicon::upsert_lexemes(&mut tx, batch).await?;
        tracing::info!(
            done = (i * cli.batch_size + batch.len()),
            total = parsed.lexemes.len(),
            "imported"
        );
    }
    tx.commit().await?;
    tracing::info!("import finished");
    Ok(())
}

/// Downloads the pinned nouns file into `data_dir` unless it's already there.
async fn download_nouns(data_dir: &Path) -> anyhow::Result<PathBuf> {
    let path = data_dir.join(format!("nouns-{}.csv", &openrussian::COMMIT[..12]));
    if tokio::fs::try_exists(&path).await? {
        tracing::info!(path = %path.display(), "using cached download");
        return Ok(path);
    }

    let url = openrussian::nouns_url();
    tracing::info!(%url, "downloading");
    let body = reqwest::get(&url)
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    tokio::fs::create_dir_all(data_dir).await?;
    // Write to a temp file first so an interrupted download isn't mistaken for a cached one.
    let partial = path.with_extension("part");
    tokio::fs::write(&partial, &body).await?;
    tokio::fs::rename(&partial, &path).await?;
    Ok(path)
}
