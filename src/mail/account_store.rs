use std::path::PathBuf;

use anyhow::Result;
use rusqlite::{params, Connection};

use super::maildir;
use super::types::Account;

pub const TEST_ACCOUNT_NAME: &str = "test";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountRecord {
    pub name: String,
    pub backend_kind: String,
    pub provider_kind: String,
    pub enabled: bool,
    pub is_default: bool,
    pub maildir_path: Option<PathBuf>,
}

impl AccountRecord {
    pub fn to_account(&self) -> Account {
        Account {
            name: self.name.clone(),
            backend: self.backend_kind.clone(),
            default: self.is_default,
        }
    }
}

pub fn seed_defaults(conn: &Connection) -> Result<()> {
    let maildir_path = maildir::default_test_maildir_path();

    conn.execute(
        "INSERT INTO accounts (
             name, backend_kind, provider_kind, enabled, is_default, maildir_path
        ) VALUES (?1, 'maildir', 'custom', 1, ?2, ?3)
         ON CONFLICT(name) DO UPDATE SET
             backend_kind = excluded.backend_kind,
             enabled = excluded.enabled,
             maildir_path = excluded.maildir_path,
             updated_at = datetime('now')",
        params![
            TEST_ACCOUNT_NAME,
            0,
            maildir_path.to_string_lossy().to_string()
        ],
    )?;

    Ok(())
}

pub fn list_accounts(conn: &Connection) -> Result<Vec<AccountRecord>> {
    let mut stmt = conn.prepare(
        "SELECT name, backend_kind, provider_kind, enabled, is_default, maildir_path
         FROM accounts
         WHERE enabled = 1
         ORDER BY is_default DESC, name ASC",
    )?;

    let rows = stmt.query_map([], row_to_account_record)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_account(conn: &Connection, name: &str) -> Result<Option<AccountRecord>> {
    let mut stmt = conn.prepare(
        "SELECT name, backend_kind, provider_kind, enabled, is_default, maildir_path
         FROM accounts
         WHERE name = ?1
         LIMIT 1",
    )?;

    let mut rows = stmt.query([name])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_account_record(row)?))
    } else {
        Ok(None)
    }
}

pub fn preferred_account(records: &[AccountRecord]) -> Option<&AccountRecord> {
    records
        .iter()
        .find(|account| account.is_default)
        .or_else(|| {
            records
                .iter()
                .find(|account| !account.backend_kind.eq_ignore_ascii_case("maildir"))
        })
        .or_else(|| records.first())
}

pub fn upsert_legacy_account(conn: &Connection, account: &Account) -> Result<()> {
    if account.default {
        conn.execute("UPDATE accounts SET is_default = 0", [])?;
    }

    conn.execute(
        "INSERT INTO accounts (
             name, backend_kind, provider_kind, enabled, is_default
         ) VALUES (?1, ?2, 'legacy', 1, ?3)
         ON CONFLICT(name) DO UPDATE SET
             backend_kind = excluded.backend_kind,
             enabled = excluded.enabled,
             is_default = CASE
                 WHEN excluded.is_default = 1 THEN 1
                 ELSE accounts.is_default
             END,
             updated_at = datetime('now')",
        params![
            account.name,
            account.backend,
            if account.default { 1 } else { 0 }
        ],
    )?;

    Ok(())
}

fn row_to_account_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<AccountRecord> {
    let path = row.get::<_, Option<String>>(5)?;
    Ok(AccountRecord {
        name: row.get(0)?,
        backend_kind: row.get(1)?,
        provider_kind: row.get(2)?,
        enabled: row.get::<_, i64>(3)? != 0,
        is_default: row.get::<_, i64>(4)? != 0,
        maildir_path: path.map(PathBuf::from),
    })
}
