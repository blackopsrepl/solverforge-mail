//! Credential CRUD operations on the SQLite database.
//!
//! Credentials replace the old `secret-tool` / `~/.authinfo.gpg` / OAuth
//! token files. All secrets live in `mail.db`.

use anyhow::{Context, Result};
use rusqlite::Connection;

/// A stored credential record.
#[derive(Debug, Clone)]
pub struct Credential {
    pub id: i64,
    pub account: String,
    pub cred_type: String,
    pub service: String,
    pub value: String,
}

/// Known credential types.
pub mod cred_type {
    pub const PASSWORD: &str = "password";
    pub const APP_PASSWORD: &str = "app_password";
    pub const OAUTH_TOKEN: &str = "oauth_token";
    pub const OAUTH_REFRESH: &str = "oauth_refresh";
}

/// Known service names.
pub mod service {
    pub const IMAP: &str = "imap";
    pub const SMTP: &str = "smtp";
    pub const OAUTH: &str = "oauth";
}

/// Store or update a credential. Uses upsert semantics (INSERT OR REPLACE).
pub fn store(
    conn: &Connection,
    account: &str,
    cred_type: &str,
    service: &str,
    value: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO credentials (account, cred_type, service, value, updated_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))
         ON CONFLICT(account, cred_type, service)
         DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        [account, cred_type, service, value],
    )
    .with_context(|| format!("cannot store credential for {account}/{service}"))?;
    Ok(())
}

/// Retrieve a credential value, or `None` if not found.
pub fn get(
    conn: &Connection,
    account: &str,
    cred_type: &str,
    service: &str,
) -> Result<Option<String>> {
    let result = conn.query_row(
        "SELECT value FROM credentials WHERE account = ?1 AND cred_type = ?2 AND service = ?3",
        [account, cred_type, service],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e).with_context(|| format!("cannot read credential for {account}/{service}")),
    }
}

/// Delete a specific credential.
pub fn delete(conn: &Connection, account: &str, cred_type: &str, service: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM credentials WHERE account = ?1 AND cred_type = ?2 AND service = ?3",
        [account, cred_type, service],
    )
    .with_context(|| format!("cannot delete credential for {account}/{service}"))?;
    Ok(())
}

/// List all credentials for an account (values are redacted for display).
pub fn list_for_account(conn: &Connection, account: &str) -> Result<Vec<Credential>> {
    let mut stmt = conn.prepare(
        "SELECT id, account, cred_type, service, value
         FROM credentials WHERE account = ?1
         ORDER BY service, cred_type",
    )?;
    let rows = stmt.query_map([account], |row| {
        Ok(Credential {
            id: row.get(0)?,
            account: row.get(1)?,
            cred_type: row.get(2)?,
            service: row.get(3)?,
            value: row.get(4)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .context("cannot list credentials")
}

/// List all stored accounts that have at least one credential.
pub fn list_accounts(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT account FROM credentials ORDER BY account")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .context("cannot list credential accounts")
}
