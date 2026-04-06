/* SQLite database layer.
The database is stored at `~/.local/share/solverforge/mail.db`.
Schema is current-state only. Older local DBs are reset instead of migrated. */

use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::Connection;

/// Current schema version. Changing this resets local DB state.
const SCHEMA_VERSION: u32 = 1;

/// Return the path to the database file.
pub fn db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("solverforge")
        .join("mail.db")
}

/// Open (or create) the database. Creates parent directories if needed.
pub fn open() -> Result<Connection> {
    let path = db_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create data directory: {}", parent.display()))?;
    }

    let conn = Connection::open(&path)
        .with_context(|| format!("cannot open database: {}", path.display()))?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;",
    )
    .context("cannot configure pragmas")?;

    init_for_test(&conn).context("schema initialization failed")?;

    Ok(conn)
}

/// Initialize the current schema. Exposed for tests.
pub fn init_for_test(conn: &Connection) -> Result<()> {
    ensure_current_schema(conn)
}

fn ensure_current_schema(conn: &Connection) -> Result<()> {
    let current_version = stored_schema_version(conn)?;
    if current_version != Some(SCHEMA_VERSION) {
        reset_schema(conn)?;
        create_schema(conn)?;
        set_schema_version(conn, SCHEMA_VERSION)?;
    }

    crate::mail::account_store::seed_defaults(conn)?;
    Ok(())
}

fn stored_schema_version(conn: &Connection) -> Result<Option<u32>> {
    if !table_exists(conn, "meta")? {
        return Ok(None);
    }

    let version = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|value| value.parse().ok());
    Ok(version)
}

fn set_schema_version(conn: &Connection, version: u32) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
        [version.to_string()],
    )?;
    Ok(())
}

fn reset_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "DROP TABLE IF EXISTS contact_tags;
         DROP TABLE IF EXISTS contacts;
         DROP TABLE IF EXISTS identities;
         DROP TABLE IF EXISTS accounts;
         DROP TABLE IF EXISTS credentials;
         DROP TABLE IF EXISTS legacy_credentials_backup;
         DROP TABLE IF EXISTS meta;",
    )?;
    Ok(())
}

fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE meta (
             key   TEXT PRIMARY KEY,
             value TEXT NOT NULL
         );

         CREATE TABLE contacts (
             id         INTEGER PRIMARY KEY AUTOINCREMENT,
             name       TEXT,
             email      TEXT    NOT NULL,
             phone      TEXT,
             org        TEXT,
             notes      TEXT,
             harvested  INTEGER NOT NULL DEFAULT 0,
             created_at TEXT    NOT NULL DEFAULT (datetime('now')),
             updated_at TEXT    NOT NULL DEFAULT (datetime('now'))
         );
         CREATE UNIQUE INDEX idx_contacts_email
             ON contacts(email);
         CREATE INDEX idx_contacts_name
             ON contacts(name);

         CREATE TABLE contact_tags (
             contact_id INTEGER NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
             tag        TEXT    NOT NULL,
             PRIMARY KEY (contact_id, tag)
         );

         CREATE TABLE identities (
             id           INTEGER PRIMARY KEY AUTOINCREMENT,
             account      TEXT    NOT NULL,
             name         TEXT,
             display_name TEXT,
             email        TEXT    NOT NULL,
             is_default   INTEGER NOT NULL DEFAULT 0,
             created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
             UNIQUE(account, email)
         );
         CREATE INDEX idx_identities_account
             ON identities(account);

         CREATE TABLE accounts (
             id            INTEGER PRIMARY KEY AUTOINCREMENT,
             name          TEXT    NOT NULL UNIQUE,
             backend_kind  TEXT    NOT NULL,
             provider_kind TEXT    NOT NULL DEFAULT 'custom',
             enabled       INTEGER NOT NULL DEFAULT 1,
             is_default    INTEGER NOT NULL DEFAULT 0,
             maildir_path  TEXT,
             created_at    TEXT    NOT NULL DEFAULT (datetime('now')),
             updated_at    TEXT    NOT NULL DEFAULT (datetime('now'))
         );
         CREATE INDEX idx_accounts_enabled
             ON accounts(enabled);
         CREATE INDEX idx_accounts_default
             ON accounts(is_default);",
    )?;
    Ok(())
}

fn table_exists(conn: &Connection, table_name: &str) -> Result<bool> {
    let exists = conn.query_row(
        "SELECT EXISTS(
             SELECT 1
             FROM sqlite_master
             WHERE type = 'table' AND name = ?1
         )",
        [table_name],
        |row| row.get::<_, i64>(0),
    )? != 0;
    Ok(exists)
}

/// Schema version for diagnostics / tests.
pub fn schema_version(conn: &Connection) -> u32 {
    stored_schema_version(conn).ok().flatten().unwrap_or(0)
}

/// The expected schema version constant (exposed for tests).
pub const CURRENT_SCHEMA_VERSION: u32 = SCHEMA_VERSION; // = 1

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{init_for_test, schema_version, table_exists, CURRENT_SCHEMA_VERSION};

    #[test]
    fn init_creates_current_schema() {
        let conn = Connection::open_in_memory().unwrap();

        init_for_test(&conn).unwrap();

        assert_eq!(schema_version(&conn), CURRENT_SCHEMA_VERSION);
        assert!(table_exists(&conn, "meta").unwrap());
        assert!(table_exists(&conn, "contacts").unwrap());
        assert!(table_exists(&conn, "contact_tags").unwrap());
        assert!(table_exists(&conn, "identities").unwrap());
        assert!(table_exists(&conn, "accounts").unwrap());
    }

    #[test]
    fn init_resets_old_schema_to_current_layout() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE meta (
                 key   TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );
             INSERT INTO meta (key, value) VALUES ('schema_version', '999');
             CREATE TABLE credentials (
                 id INTEGER PRIMARY KEY AUTOINCREMENT
             );",
        )
        .unwrap();

        init_for_test(&conn).unwrap();

        assert_eq!(schema_version(&conn), CURRENT_SCHEMA_VERSION);
        assert!(!table_exists(&conn, "credentials").unwrap());
        assert!(table_exists(&conn, "accounts").unwrap());
    }
}
