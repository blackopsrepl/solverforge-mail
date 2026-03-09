use solverforge_mail::keys::View;

// Note: App depends on himalaya CLI being available, so we test
// the state machine logic that doesn't require subprocess calls.
// We construct an App and manipulate its state directly.

use solverforge_mail::app::App;

#[test]
fn app_initial_state() {
    let app = App::new(None);
    assert!(app.running);
    assert_eq!(app.view, View::EnvelopeList);
    assert_eq!(app.current_folder, "INBOX");
    assert_eq!(app.page, 1);
    assert!(app.envelopes.is_empty());
    assert!(app.folders.is_empty());
    assert!(app.accounts.is_empty());
    assert!(app.pending_shell.is_none());
}

#[test]
fn app_with_account_flag() {
    let app = App::new(Some("work".to_string()));
    assert_eq!(app.account_name, Some("work".to_string()));
}

#[test]
fn app_set_status() {
    let mut app = App::new(None);
    app.set_status("test message");
    assert_eq!(app.status_message, "test message");
}

#[test]
fn app_tick_increments_counter() {
    let mut app = App::new(None);
    assert_eq!(app.tick_count, 0);
    app.tick();
    assert_eq!(app.tick_count, 1);
    app.tick();
    assert_eq!(app.tick_count, 2);
}

#[test]
fn app_search_query_manipulation() {
    let mut app = App::new(None);
    app.search_query.push('f');
    app.search_query.push('o');
    app.search_query.push('o');
    assert_eq!(app.search_query, "foo");
    app.search_query.pop();
    assert_eq!(app.search_query, "fo");
}

#[test]
fn app_move_target_manipulation() {
    let mut app = App::new(None);
    app.move_target.push('S');
    app.move_target.push('e');
    app.move_target.push('n');
    app.move_target.push('t');
    assert_eq!(app.move_target, "Sent");
}

#[test]
fn app_selected_envelope_empty() {
    let app = App::new(None);
    assert!(app.selected_envelope_id().is_none());
    assert!(app.selected_envelope().is_none());
}

#[test]
fn app_help_scroll() {
    let mut app = App::new(None);
    assert_eq!(app.help_scroll, 0);
    app.help_scroll = 5;
    assert_eq!(app.help_scroll, 5);
}

#[test]
fn tab_to_save_saves_identity_edit_form() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use rusqlite::Connection;
    use solverforge_mail::identity_edit::{IdentityEditState, IdentityField};
    use solverforge_mail::keys::View;

    // Set up an in-memory DB with the full schema.
    let conn = Connection::open_in_memory().unwrap();
    solverforge_mail::db::migrate_for_test(&conn).unwrap();

    let mut app = App::new(Some("testaccount".to_string()));
    app.db = Some(conn);
    app.view = View::IdentityEdit;

    // Pre-fill a valid form.
    let mut state = IdentityEditState::new("testaccount");
    state.name = "Work".to_string();
    state.email = "work@example.com".to_string();
    app.identity_edit_state = Some(state);

    // Tab through fields to reach the Save button:
    // Name → SenderName → Email → IsDefault → Save  (4 Tabs)
    let tab = KeyEvent {
        code: KeyCode::Tab,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    for _ in 0..4 {
        app.handle_key(tab);
    }

    // Verify focus is now on Save.
    assert_eq!(
        app.identity_edit_state.as_ref().unwrap().focused,
        IdentityField::Save,
        "focus should be on Save after 4 Tabs"
    );

    // Press Enter to activate Save.
    let enter = KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };
    app.handle_key(enter);

    // The modal should be gone and we should be back on IdentityList.
    assert!(
        app.identity_edit_state.is_none(),
        "modal should close after save"
    );
    assert_eq!(app.view, View::IdentityList);
    assert_eq!(app.identities.len(), 1);
    assert_eq!(app.identities[0].email, "work@example.com");
}
