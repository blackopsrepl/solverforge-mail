use std::path::PathBuf;

use super::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    BackendMissing,
    ConfigMissing,
    KeyringUnavailable,
    GpgFailure,
    OauthReconfigure,
    AuthRejected,
    LocalBackendFailure,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub kind: FailureKind,
    pub detail: String,
}

impl Failure {
    pub fn message(&self) -> String {
        match self.kind {
            FailureKind::BackendMissing => format!("Himalaya backend unavailable: {}", self.detail),
            FailureKind::ConfigMissing => format!(
                "Himalaya config missing or unreadable. Set HIMALAYA_CONFIG or create {}: {}",
                config_hint().display(),
                self.detail
            ),
            FailureKind::KeyringUnavailable => format!(
                "Keyring secret missing or inaccessible. Check your desktop secret service and stored Himalaya secrets: {}",
                self.detail
            ),
            FailureKind::GpgFailure => format!(
                "GPG-backed auth failed. Check ~/.authinfo.gpg and your gpg-agent session: {}",
                self.detail
            ),
            FailureKind::OauthReconfigure => format!(
                "OAuth credentials need reconfiguration. Re-run `himalaya account configure <account>`: {}",
                self.detail
            ),
            FailureKind::AuthRejected => {
                format!("Server rejected authentication: {}", self.detail)
            }
            FailureKind::LocalBackendFailure => format!(
                "Local maildir backend failed. This is not an authentication error: {}",
                self.detail
            ),
            FailureKind::Other => self.detail.clone(),
        }
    }
}

pub fn explain(account_backend: Option<&str>, raw: &str) -> String {
    classify(account_backend, raw).message()
}

pub fn classify(account_backend: Option<&str>, raw: &str) -> Failure {
    let detail = clean_error(raw);
    let lower = strip_ansi(raw).to_ascii_lowercase();

    let kind = if lower.contains("backend not found") || lower.contains("failed to execute") {
        FailureKind::BackendMissing
    } else if lower.contains("config not found") || lower.contains("config.toml") {
        FailureKind::ConfigMissing
    } else if lower.contains("org.freedesktop.secrets")
        || lower.contains("secret service")
        || lower.contains("secret-service")
        || lower.contains("libsecret")
        || lower.contains("keyring")
    {
        FailureKind::KeyringUnavailable
    } else if lower.contains(".authinfo.gpg") || lower.contains("gpg") {
        FailureKind::GpgFailure
    } else if lower.contains("invalid_grant")
        || lower.contains("invalid_client")
        || lower.contains("invalid_token")
        || (lower.contains("oauth") && lower.contains("token"))
    {
        FailureKind::OauthReconfigure
    } else if matches!(account_backend, Some("maildir")) {
        FailureKind::LocalBackendFailure
    } else if lower.contains("cannot authenticate")
        || lower.contains("authentication failed")
        || lower.contains("auth failed")
        || lower.contains("login failed")
    {
        FailureKind::AuthRejected
    } else {
        FailureKind::Other
    };

    Failure { kind, detail }
}

pub fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            while let Some(&next) = chars.peek() {
                chars.next();
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

pub fn clean_error(raw: &str) -> String {
    let stripped = strip_ansi(raw);
    let body = stripped
        .strip_prefix("himalaya error: ")
        .unwrap_or(&stripped);

    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("0: ") {
            return rest.trim().to_string();
        }
        if let Some(rest) = trimmed.strip_prefix("Error: ") {
            return rest.trim().to_string();
        }
    }

    body.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("unknown error")
        .to_string()
}

fn config_hint() -> PathBuf {
    config::configured_config_path()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_escape_codes() {
        let input = "\u{1b}[31mcannot build IMAP client\u{1b}[0m";
        assert_eq!(strip_ansi(input), "cannot build IMAP client");
    }

    #[test]
    fn strip_ansi_preserves_plain_text() {
        assert_eq!(strip_ansi("hello world"), "hello world");
    }

    #[test]
    fn clean_error_extracts_first_error() {
        let raw = "himalaya error: 0: cannot build IMAP client\n1: boom";
        assert_eq!(clean_error(raw), "cannot build IMAP client");
    }

    #[test]
    fn clean_error_handles_simple_message() {
        assert_eq!(
            clean_error("himalaya error: something went wrong"),
            "something went wrong"
        );
    }

    #[test]
    fn explain_flags_maildir_as_non_auth() {
        let raw = "himalaya error: 0: cannot open maildir";
        let explained = explain(Some("maildir"), raw);
        assert!(explained.contains("not an authentication error"));
    }

    #[test]
    fn explain_flags_keyring_failures() {
        let raw = "himalaya error: secret service unavailable";
        let explained = explain(Some("imap"), raw);
        assert!(explained.contains("Keyring secret missing or inaccessible"));
    }

    #[test]
    fn explain_flags_oauth_failures() {
        let raw = "himalaya error: invalid_grant";
        let explained = explain(Some("imap"), raw);
        assert!(explained.contains("OAuth credentials need reconfiguration"));
    }
}
