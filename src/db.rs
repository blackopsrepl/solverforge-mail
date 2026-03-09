//! SQLite database layer.
//!
//! The database is stored at `~/.local/share/solverforge/mail.db`.
//! Schema is forward-only: each version adds tables/columns, never removes them.

use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::Connection;

/// Current schema version. Increment when adding new migrations.
const SCHEMA_VERSION: u32 = 3;

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

    // Ensure parent directory exists.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create data directory: {}", parent.display()))?;
    }

    let conn = Connection::open(&path)
        .with_context(|| format!("cannot open database: {}", path.display()))?;

    // Configure for performance + safety.
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;",
    )
    .context("cannot configure pragmas")?;

    // Run migrations.
    migrate(&conn).context("schema migration failed")?;

    Ok(conn)
}

/// Run all schema migrations in order, skipping those already applied.
/// Exposed for integration tests.
pub fn migrate_for_test(conn: &Connection) -> Result<()> {
    migrate(conn)
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )?;

    let current_version: u32 = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    if current_version < 1 {
        migrate_v1(conn)?;
        set_version(conn, 1)?;
    }
    if current_version < 2 {
        migrate_v2(conn)?;
        set_version(conn, 2)?;
    }
    if current_version < 3 {
        migrate_v3(conn)?;
        set_version(conn, 3)?;
    }

    Ok(())
}

fn set_version(conn: &Connection, version: u32) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
        [version.to_string()],
    )?;
    Ok(())
}

/// v1: credentials + contacts tables.
fn migrate_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "-- Credentials store.
         CREATE TABLE IF NOT EXISTS credentials (
             id         INTEGER PRIMARY KEY AUTOINCREMENT,
             account    TEXT    NOT NULL,
             cred_type  TEXT    NOT NULL,
             service    TEXT    NOT NULL,
             value      TEXT    NOT NULL,
             created_at TEXT    NOT NULL DEFAULT (datetime('now')),
             updated_at TEXT    NOT NULL DEFAULT (datetime('now')),
             UNIQUE(account, cred_type, service)
         );

         -- Contacts: rich address book.
         CREATE TABLE IF NOT EXISTS contacts (
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

         CREATE UNIQUE INDEX IF NOT EXISTS idx_contacts_email
             ON contacts(email);
         CREATE INDEX IF NOT EXISTS idx_contacts_name
             ON contacts(name);

         -- Tags for contacts.
         CREATE TABLE IF NOT EXISTS contact_tags (
             contact_id INTEGER NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
             tag        TEXT    NOT NULL,
             PRIMARY KEY (contact_id, tag)
         );",
    )?;
    Ok(())
}

/// v2: sender identities (From addresses per account).
fn migrate_v2(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS identities (
             id           INTEGER PRIMARY KEY AUTOINCREMENT,
             account      TEXT    NOT NULL,
             display_name TEXT,
             email        TEXT    NOT NULL,
             is_default   INTEGER NOT NULL DEFAULT 0,
             created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
             UNIQUE(account, email)
         );
         CREATE INDEX IF NOT EXISTS idx_identities_account
             ON identities(account);",
    )?;
    Ok(())
}

/// v3: add `name` column to identities (separate identity label from sender display name).
fn migrate_v3(conn: &Connection) -> Result<()> {
    conn.execute_batch("ALTER TABLE identities ADD COLUMN name TEXT;")?;
    Ok(())
}

/// Schema version for diagnostics / tests.
pub fn schema_version(conn: &Connection) -> u32 {
    conn.query_row(
        "SELECT value FROM meta WHERE key = 'schema_version'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(0)
}

/// The expected schema version constant (exposed for tests).
pub const CURRENT_SCHEMA_VERSION: u32 = SCHEMA_VERSION; // = 3
