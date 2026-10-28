//! SQLite adapters for the desktop app: the word list shipped with the app
//! (`lexicon.sqlite`, read-only) attached to the user's own `progress.sqlite`.
//! Schemas: `backend/migrations-sqlite`.

pub mod games;
pub mod lexicon;
pub mod preferences;

use std::path::Path;

use sqlx::sqlite::{
    SqliteConnectOptions, SqliteConnection, SqliteJournalMode, SqlitePool, SqlitePoolOptions,
};
use sqlx::{ConnectOptions, Connection};

/// Migrations of `progress.sqlite`, embedded at compile time.
pub static PROGRESS_MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../migrations-sqlite/progress");

/// Opens `progress` (created and migrated when needed) with `lexicon` attached
/// read-only as `lex`. The lexicon's tables have names of their own, so queries
/// use them unqualified.
pub async fn open(lexicon: &Path, progress: &Path) -> Result<SqlitePool, sqlx::Error> {
    if !lexicon.is_file() {
        return Err(sqlx::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("no word list at {}", lexicon.display()),
        )));
    }
    let lexicon = read_only_uri(&std::path::absolute(lexicon)?);
    let options = SqliteConnectOptions::new()
        .filename(progress)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    // Once by hand first: the pool would retry a failing ATTACH until it times
    // out, and report only the timeout.
    let mut first = options.connect().await?;
    attach(&mut first, &lexicon).await?;
    first.close().await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .after_connect(move |conn, _| {
            let lexicon = lexicon.clone();
            Box::pin(async move { attach(conn, &lexicon).await })
        })
        .connect_with(options)
        .await?;
    PROGRESS_MIGRATOR
        .run(&pool)
        .await
        .map_err(|e| sqlx::Error::Migrate(Box::new(e)))?;
    Ok(pool)
}

/// Attaches the word list (a `read_only_uri`) to a connection as `lex`.
async fn attach(conn: &mut SqliteConnection, lexicon: &str) -> Result<(), sqlx::Error> {
    sqlx::query("ATTACH DATABASE ?1 AS lex")
        .bind(lexicon)
        .execute(&mut *conn)
        .await?;
    // SQLite opens it lazily; read it now so a bad file fails here.
    sqlx::query("SELECT count(*) FROM lex.sqlite_master")
        .execute(&mut *conn)
        .await?;
    // Read through the OS page cache, shared by all connections, instead of
    // each connection reading it again.
    sqlx::query("PRAGMA lex.mmap_size = 268435456")
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// A `file:` URI that opens `path` read-only and immutable: it lives in the app's
/// install folder, which may not be writable and never changes while running.
fn read_only_uri(path: &Path) -> String {
    let path = path.to_string_lossy();
    // Windows hands out extended-length paths (\\?\C:\…), which aren't URIs.
    let path = path
        .strip_prefix(r"\\?\")
        .unwrap_or(&path)
        .replace('\\', "/");
    // Windows paths (C:/…) need a leading slash: file:/C:/…
    let mut uri = String::from(if path.starts_with('/') {
        "file:"
    } else {
        "file:/"
    });
    for c in path.chars() {
        match c {
            '%' => uri.push_str("%25"),
            '?' => uri.push_str("%3F"),
            '#' => uri.push_str("%23"),
            ' ' => uri.push_str("%20"),
            _ => uri.push(c),
        }
    }
    uri.push_str("?mode=ro&immutable=1");
    uri
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uris_escape_paths() {
        assert_eq!(
            read_only_uri(Path::new("/opt/my app/lex#1.sqlite")),
            "file:/opt/my%20app/lex%231.sqlite?mode=ro&immutable=1"
        );
        assert_eq!(
            read_only_uri(Path::new(r"C:\Program Files\Trainer\lexicon.sqlite")),
            "file:/C:/Program%20Files/Trainer/lexicon.sqlite?mode=ro&immutable=1"
        );
        // What Tauri gives on Windows.
        assert_eq!(
            read_only_uri(Path::new(
                r"\\?\C:\Users\me\AppData\Local\Russian Cases Trainer\lexicon.sqlite"
            )),
            "file:/C:/Users/me/AppData/Local/Russian%20Cases%20Trainer/lexicon.sqlite?mode=ro&immutable=1"
        );
    }

    #[tokio::test]
    async fn a_word_list_that_cant_be_attached_says_why() {
        let dir = tempfile::tempdir().unwrap();
        let lexicon = dir.path().join("lexicon.sqlite");
        std::fs::write(&lexicon, b"not a database").unwrap();
        let err = open(&lexicon, &dir.path().join("progress.sqlite"))
            .await
            .unwrap_err();
        assert!(!matches!(err, sqlx::Error::PoolTimedOut), "{err}");
    }
}
