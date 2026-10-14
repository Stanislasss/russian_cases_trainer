//! Loads the OpenRussian.org word lists (CC BY-SA) and the grammar rules into Postgres.

use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(about = "Import the lexicon and grammar rules")]
struct Cli {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
    /// Where the downloaded CSV files are cached.
    #[arg(long, env = "IMPORT_DATA_DIR", default_value = "data")]
    data_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();
    let _pool = infrastructure::connect(&cli.database_url).await?;
    tracing::warn!(data_dir = %cli.data_dir.display(), "import not implemented yet");
    Ok(())
}
