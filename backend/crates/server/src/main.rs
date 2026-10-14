//! Composition root: reads config, wires adapters into use cases, runs the HTTP server.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use application::auth::AuthService;
use clap::{Parser, Subcommand};
use infrastructure::security::{Argon2Hasher, OsTokenGenerator, SystemClock};
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
        } => {
            let pool = infrastructure::connect(&database_url)
                .await
                .context("connecting to the database")?;
            let state = api::AppState {
                auth: AuthService::new(
                    Arc::new(PgUserRepository::new(pool.clone())),
                    Arc::new(PgSessionRepository::new(pool)),
                    Arc::new(Argon2Hasher),
                    Arc::new(SystemClock),
                    Arc::new(OsTokenGenerator),
                ),
                cookie_secure,
            };
            let listener = tokio::net::TcpListener::bind(bind_addr).await?;
            tracing::info!(%bind_addr, "listening");
            axum::serve(listener, api::router(state))
                .with_graceful_shutdown(shutdown_signal())
                .await?;
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
