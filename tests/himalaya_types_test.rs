use pretty_assertions::assert_eq;
use solverforge_mail::himalaya::types::*;

#[test]
fn deserialize_accounts() {
    let json = include_str!("fixtures/accounts.json");
    let accounts: Vec<Account> = serde_json::from_str(json).unwrap();

    assert_eq!(accounts.len(), 3);
    assert_eq!(accounts[0].name, "personal");
    assert_eq!(accounts[0].backend, "imap");
    assert!(accounts[0].default);
    assert_eq!(accounts[1].name, "work");
    assert!(!accounts[1].default);
    assert_eq!(accounts[2].backend, "maildir");
}

#[test]
fn deserialize_folders() {
    let json = include_str!("fixtures/folders.json");
    let folders: Vec<Folder> = serde_json::from_str(json).unwrap();

    assert_eq!(folders.len(), 5);
    assert_eq!(folders[0].name, "INBOX");
    assert_eq!(folders[0].desc, Some("Main inbox".to_string()));
    assert_eq!(folders[4].name, "Archive");
    assert_eq!(folders[4].desc, None);
}

#[test]
fn deserialize_envelopes() {
    let json = include_str!("fixtures/envelopes.json");
    let envelopes: Vec<Envelope> = serde_json::from_str(json).unwrap();

    assert_eq!(envelopes.len(), 5);

    // Envelope 1: Seen + Answered
    let e1 = &envelopes[0];
    assert_eq!(e1.id, "1");
    assert!(e1.is_seen());
    assert!(e1.is_answered());
    assert!(!e1.is_flagged());
    assert_eq!(e1.flag_icon(), "\u{21a9}"); // ↩

    // Envelope 2: Flagged (unseen)
    let e2 = &envelopes[1];
    assert!(e2.is_flagged());
    assert!(!e2.is_seen());
    assert_eq!(e2.flag_icon(), "!");

    // Envelope 3: No flags (unseen)
    let e3 = &envelopes[2];
    assert!(!e3.is_seen());
    assert!(!e3.is_flagged());
    assert_eq!(e3.flag_icon(), "\u{25cf}"); // ●

    // Envelope 4: Seen only
    let e4 = &envelopes[3];
    assert!(e4.is_seen());
    assert_eq!(e4.flag_icon(), " ");

    // Envelope 5: Seen + Flagged
    let e5 = &envelopes[4];
    assert!(e5.is_seen());
    assert!(e5.is_flagged());
    assert_eq!(e5.flag_icon(), "!");
}

#[test]
fn deserialize_sender_plain_string() {
    let json =
        r#"{"id":"1","flags":[],"subject":"test","from":"alice@example.com","date":"2026-01-01"}"#;
    let env: Envelope = serde_json::from_str(json).unwrap();
    assert_eq!(env.sender_display(), "alice@example.com");
}

#[test]
fn deserialize_sender_structured() {
    let json = r#"{"id":"1","flags":[],"subject":"test","from":{"name":"Alice","addr":"alice@example.com"},"date":"2026-01-01"}"#;
    let env: Envelope = serde_json::from_str(json).unwrap();
    assert_eq!(env.sender_display(), "Alice");
}

#[test]
fn deserialize_sender_structured_no_name() {
    let json = r#"{"id":"1","flags":[],"subject":"test","from":{"name":"","addr":"alice@example.com"},"date":"2026-01-01"}"#;
    let env: Envelope = serde_json::from_str(json).unwrap();
    assert_eq!(env.sender_display(), "alice@example.com");
}
