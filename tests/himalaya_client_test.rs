use pretty_assertions::assert_eq;
use solverforge_mail::himalaya::client;

#[test]
fn build_envelope_args_basic() {
    let args = client::build_envelope_args(None, "INBOX", 1, 50, None);
    assert_eq!(
        args,
        vec!["-o", "json", "envelope", "list", "-f", "INBOX", "-p", "1", "-s", "50"]
    );
}

#[test]
fn build_envelope_args_with_account() {
    let args = client::build_envelope_args(Some("work"), "Sent", 2, 25, None);
    assert_eq!(
        args,
        vec!["-o", "json", "-a", "work", "envelope", "list", "-f", "Sent", "-p", "2", "-s", "25"]
    );
}

#[test]
fn build_envelope_args_with_query() {
    let args = client::build_envelope_args(None, "INBOX", 1, 50, Some("subject foo and from bar"));
    assert_eq!(
        args,
        vec![
            "-o", "json", "envelope", "list", "-f", "INBOX", "-p", "1", "-s", "50", "subject",
            "foo", "and", "from", "bar"
        ]
    );
}

#[test]
fn build_read_args_basic() {
    let args = client::build_read_args(None, "INBOX", "42");
    assert_eq!(
        args,
        vec!["-o", "json", "message", "read", "-f", "INBOX", "42"]
    );
}

#[test]
fn build_read_args_with_account() {
    let args = client::build_read_args(Some("personal"), "Drafts", "7");
    assert_eq!(
        args,
        vec!["-o", "json", "-a", "personal", "message", "read", "-f", "Drafts", "7"]
    );
}
