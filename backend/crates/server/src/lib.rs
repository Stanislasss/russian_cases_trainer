//! The desktop app's backend, shared by `server serve-local` and the Tauri app:
//! the SQLite word list and progress, one local user, the downloadable speech
//! pack, and the built frontend, all on one local address.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use api::{Accounts, AppState};
use application::game::EndingsGame;
use application::sentences::SentenceGame;
use application::speech::SpeechService;
use chrono::DateTime;
use domain::grammar::Locale;
use domain::user::{DisplayName, Email, User, UserId};
use infrastructure::speech::FsAudioCache;
use infrastructure::speech_pack::{DesktopSpeech, Manifest};
use infrastructure::sqlite::games::{SqliteAttemptRepository, SqliteExerciseRepository};
use infrastructure::sqlite::preferences::SqlitePreferenceRepository;

pub struct LocalConfig {
    /// `lexicon.sqlite`, shipped with the app.
    pub lexicon: PathBuf,
    /// The user's data: progress, speech pack, audio cache.
    pub data_dir: PathBuf,
    /// The built frontend (`frontend/dist`).
    pub static_dir: Option<PathBuf>,
    /// Where to listen; the app uses 127.0.0.1 with any free port.
    pub bind: SocketAddr,
    /// The address the app is opened at, if not `http://<bind>` (e.g. behind
    /// Docker's port mapping). Requests for other hosts are refused.
    pub origin: Option<String>,
}

pub struct LocalServer {
    /// Where the app is, e.g. `http://127.0.0.1:52341`.
    pub url: String,
    pub task: tokio::task::JoinHandle<std::io::Result<()>>,
}

/// Where the speech pack is downloaded from: `backend/speech-pack.json`.
pub fn speech_pack_manifest() -> Manifest {
    serde_json::from_str(include_str!("../../../speech-pack.json"))
        .expect("speech-pack.json is valid")
}

/// The desktop app's one player.
pub fn local_user() -> User {
    User {
        id: UserId(uuid::Uuid::nil()),
        email: Email::parse("me@localhost.local").expect("valid"),
        display_name: DisplayName::parse("Me").expect("valid"),
        ui_locale: Locale::En,
        created_at: DateTime::UNIX_EPOCH,
    }
}

/// Opens the databases and serves the app in the background.
pub async fn start_local(config: LocalConfig) -> anyhow::Result<LocalServer> {
    tokio::fs::create_dir_all(&config.data_dir)
        .await
        .with_context(|| format!("creating {}", config.data_dir.display()))?;
    let pool =
        infrastructure::sqlite::open(&config.lexicon, &config.data_dir.join("progress.sqlite"))
            .await
            .context("opening the word list and progress")?;
    let speech = Arc::new(DesktopSpeech::new(
        config.data_dir.join("speech-pack"),
        speech_pack_manifest(),
    ));
    let preferences = Arc::new(SqlitePreferenceRepository::new(pool.clone()));
    let state = AppState {
        accounts: Accounts::Local(local_user()),
        endings: EndingsGame::new(
            Arc::new(SqliteExerciseRepository::new(pool.clone())),
            Arc::new(SqliteAttemptRepository::new(pool.clone())),
            preferences.clone(),
        ),
        sentences: SentenceGame::new(
            Arc::new(SqliteExerciseRepository::new(pool.clone())),
            Arc::new(SqliteAttemptRepository::new(pool)),
            preferences.clone(),
        ),
        preferences,
        speech: SpeechService::new(
            speech.clone(),
            Arc::new(FsAudioCache::new(config.data_dir.join("speech-cache"))),
        ),
        speech_pack: Some(speech),
    };

    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    let url = config
        .origin
        .unwrap_or_else(|| format!("http://{}", listener.local_addr().expect("bound")));
    let app = api::local::only_from(api::router(state, config.static_dir.as_deref()), &url);
    tracing::info!(%url, "serving the app");
    let task = tokio::spawn(async move { axum::serve(listener, app).await });
    Ok(LocalServer { url, task })
}
