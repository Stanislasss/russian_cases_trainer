//! Adapters for the application ports: Postgres and SQLite repositories, password hashing, etc.

pub mod games;
pub mod lexicon;
pub mod security;
pub mod speech;
pub mod speech_pack;
pub mod sqlite;
pub mod users;

#[cfg(test)]
mod contract;

use sqlx::postgres::{PgPool, PgPoolOptions};

/// Migrations embedded at compile time from `backend/migrations`.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}
