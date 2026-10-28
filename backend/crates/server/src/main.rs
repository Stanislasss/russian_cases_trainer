//! Composition root: reads config, wires adapters into use cases, runs the HTTP server.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use application::auth::AuthService;
use application::game::EndingsGame;
use application::sentences::SentenceGame;
use application::speech::SpeechService;
use clap::{Parser, Subcommand};
use infrastructure::games::{PgAttemptRepository, PgExerciseRepository};
use infrastructure::preferences::PgPreferenceRepository;
use infrastructure::security::{Argon2Hasher, OsTokenGenerator, SystemClock};
use infrastructure::speech::{FsAudioCache, PiperSynthesizer};
use infrastructure::users::{PgSessionRepository, PgUserRepository};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(about = "Russian Cases Trainer API server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the HTTP server (default).
    Serve {
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
        #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0:8080")]
        bind_addr: SocketAddr,
        /// Mark cookies `Secure` (only sent over HTTPS).
        #[arg(long, env = "COOKIE_SECURE", default_value_t = false, action = clap::ArgAction::Set)]
        cookie_secure: bool,
        /// Piper's HTTP server (see tts/Dockerfile).
        #[arg(long, env = "TTS_URL", default_value = "http://tts:5000")]
        tts_url: String,
        #[arg(long, env = "TTS_VOICE", default_value = "ru_RU-irina-medium")]
        tts_voice: String,
        /// Where synthesized audio is kept.
        #[arg(long, env = "SPEECH_CACHE_DIR", default_value = "speech-cache")]
        speech_cache_dir: std::path::PathBuf,
    },
    /// Run the desktop app's backend without its window: SQLite, one local user,
    /// the speech pack and the built frontend (for trying it out in a browser).
    ServeLocal {
        /// The word list written by `importer --sqlite`.
        #[arg(long)]
        lexicon: std::path::PathBuf,
        /// Progress, speech pack and audio cache go here.
        #[arg(long)]
        data_dir: std::path::PathBuf,
        /// The built frontend (`frontend/dist`).
        #[arg(long = "static")]
        static_dir: Option<std::path::PathBuf>,
        #[arg(long, default_value = "127.0.0.1:8080")]
        bind: SocketAddr,
        /// The address the browser uses, when not `http://<bind>` (Docker port mapping).
        #[arg(long)]
        origin: Option<String>,
    },
    /// Apply pending database migrations and exit.
    Migrate {
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
    },
    /// Print the OpenAPI document as JSON and exit.
    PrintOpenapi,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let command = match Cli::parse().command {
        Some(command) => command,
        // `server` with no subcommand behaves like `server serve`.
        None => Cli::parse_from(["server", "serve"]).command.expect("serve"),
    };

    match command {
        Command::Serve {
            database_url,
            bind_addr,
            cookie_secure,
            tts_url,
            tts_voice,
            speech_cache_dir,
        } => {
            let pool = infrastructure::connect(&database_url)
                .await
                .context("connecting to the database")?;
            let preferences = Arc::new(PgPreferenceRepository::new(pool.clone()));
            let state = api::AppState {
                accounts: api::Accounts::Sessions {
                    auth: AuthService::new(
                        Arc::new(PgUserRepository::new(pool.clone())),
                        Arc::new(PgSessionRepository::new(pool.clone())),
                        Arc::new(Argon2Hasher),
                        Arc::new(SystemClock),
                        Arc::new(OsTokenGenerator),
                    ),
                    cookie_secure,
                },
                endings: EndingsGame::new(
                    Arc::new(PgExerciseRepository::new(pool.clone())),
                    Arc::new(PgAttemptRepository::new(pool.clone())),
                    preferences.clone(),
                ),
                sentences: SentenceGame::new(
                    Arc::new(PgExerciseRepository::new(pool.clone())),
                    Arc::new(PgAttemptRepository::new(pool)),
                    preferences.clone(),
                ),
                preferences,
                speech: SpeechService::new(
                    Arc::new(PiperSynthesizer::new(&tts_url, &tts_voice)),
                    Arc::new(FsAudioCache::new(speech_cache_dir)),
                ),
                speech_pack: None,
            };
            let listener = tokio::net::TcpListener::bind(bind_addr).await?;
            tracing::info!(%bind_addr, "listening");
            axum::serve(listener, api::router(state, None))
                .with_graceful_shutdown(shutdown_signal())
                .await?;
        }
        Command::ServeLocal {
            lexicon,
            data_dir,
            static_dir,
            bind,
            origin,
        } => {
            let local = server::start_local(server::LocalConfig {
                lexicon,
                data_dir,
                static_dir,
                bind,
                origin,
            })
            .await?;
            tokio::select! {
                result = local.task => result??,
                () = shutdown_signal() => {}
            }
        }
        Command::Migrate { database_url } => {
            let pool = infrastructure::connect(&database_url)
                .await
                .context("connecting to the database")?;
            infrastructure::MIGRATOR.run(&pool).await?;
            tracing::info!("migrations applied");
        }
        Command::PrintOpenapi => {
            println!("{}", serde_json::to_string_pretty(&api::openapi())?);
        }
    }
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
