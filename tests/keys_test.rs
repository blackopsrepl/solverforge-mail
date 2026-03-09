use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pretty_assertions::assert_eq;
use solverforge_mail::keys::{resolve, Action, View};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

// ── Global keybindings ──────────────────────────────────────────────

#[test]
fn ctrl_c_quits_from_any_view() {
    assert_eq!(
        resolve(View::EnvelopeList, ctrl(KeyCode::Char('c'))),
        Action::Quit
    );
    assert_eq!(
        resolve(View::MessageView, ctrl(KeyCode::Char('c'))),
        Action::Quit
    );
    assert_eq!(
        resolve(View::FolderList, ctrl(KeyCode::Char('c'))),
        Action::Quit
    );
}

#[test]
fn ctrl_a_switches_account() {
    assert_eq!(
        resolve(View::EnvelopeList, ctrl(KeyCode::Char('a'))),
        Action::SwitchAccount
    );
}

#[test]
fn ctrl_r_refreshes() {
    assert_eq!(
        resolve(View::EnvelopeList, ctrl(KeyCode::Char('r'))),
        Action::Refresh
    );
}

// ── Envelope list ───────────────────────────────────────────────────

#[test]
fn envelope_list_navigation() {
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('j'))),
        Action::MoveDown
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Down)),
        Action::MoveDown
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('k'))),
        Action::MoveUp
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Up)),
        Action::MoveUp
    );
}

#[test]
fn envelope_list_actions() {
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Enter)),
        Action::OpenMessage
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('c'))),
        Action::Compose
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('d'))),
        Action::Delete
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('/'))),
        Action::Search
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('!'))),
        Action::ToggleFlag
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('m'))),
        Action::MoveMessage
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Tab)),
        Action::FocusFolders
    );
}

#[test]
fn envelope_list_jumps() {
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('g'))),
        Action::JumpTop
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('G'))),
        Action::JumpBottom
    );
}

#[test]
fn envelope_list_paging() {
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('n'))),
        Action::PageDown
    );
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('p'))),
        Action::PageUp
    );
}

// ── Message view ────────────────────────────────────────────────────

#[test]
fn message_view_navigation() {
    assert_eq!(
        resolve(View::MessageView, key(KeyCode::Char('j'))),
        Action::ScrollDown
    );
    assert_eq!(
        resolve(View::MessageView, key(KeyCode::Char('k'))),
        Action::ScrollUp
    );
    assert_eq!(
        resolve(View::MessageView, key(KeyCode::Char('q'))),
        Action::Back
    );
    assert_eq!(resolve(View::MessageView, key(KeyCode::Esc)), Action::Back);
}

#[test]
fn message_view_actions() {
    assert_eq!(
        resolve(View::MessageView, key(KeyCode::Char('r'))),
        Action::Reply
    );
    assert_eq!(
        resolve(View::MessageView, key(KeyCode::Char('R'))),
        Action::ReplyAll
    );
    assert_eq!(
        resolve(View::MessageView, key(KeyCode::Char('f'))),
        Action::Forward
    );
    assert_eq!(
        resolve(View::MessageView, key(KeyCode::Char('a'))),
        Action::DownloadAttachments
    );
    assert_eq!(
        resolve(View::MessageView, key(KeyCode::Char('d'))),
        Action::Delete
    );
}

// ── Folder list ─────────────────────────────────────────────────────

#[test]
fn folder_list_navigation() {
    assert_eq!(
        resolve(View::FolderList, key(KeyCode::Char('j'))),
        Action::MoveDown
    );
    assert_eq!(
        resolve(View::FolderList, key(KeyCode::Enter)),
        Action::Select
    );
    assert_eq!(
        resolve(View::FolderList, key(KeyCode::Tab)),
        Action::FocusEnvelopes
    );
}

// ── Search ──────────────────────────────────────────────────────────

#[test]
fn search_input() {
    assert_eq!(
        resolve(View::Search, key(KeyCode::Char('a'))),
        Action::SearchInput('a')
    );
    assert_eq!(
        resolve(View::Search, key(KeyCode::Backspace)),
        Action::SearchBackspace
    );
    assert_eq!(
        resolve(View::Search, key(KeyCode::Enter)),
        Action::SearchSubmit
    );
    assert_eq!(
        resolve(View::Search, key(KeyCode::Esc)),
        Action::SearchCancel
    );
}

// ── Help ────────────────────────────────────────────────────────────

#[test]
fn help_toggle() {
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::Char('?'))),
        Action::ToggleHelp
    );
    assert_eq!(
        resolve(View::Help, key(KeyCode::Char('?'))),
        Action::ToggleHelp
    );
    assert_eq!(resolve(View::Help, key(KeyCode::Esc)), Action::ToggleHelp);
}

// ── Move prompt ─────────────────────────────────────────────────────

#[test]
fn move_prompt_input() {
    assert_eq!(
        resolve(View::MovePrompt, key(KeyCode::Char('S'))),
        Action::MoveInput('S')
    );
    assert_eq!(
        resolve(View::MovePrompt, key(KeyCode::Backspace)),
        Action::MoveBackspace
    );
    assert_eq!(
        resolve(View::MovePrompt, key(KeyCode::Enter)),
        Action::MoveSubmit
    );
    assert_eq!(
        resolve(View::MovePrompt, key(KeyCode::Esc)),
        Action::MoveCancel
    );
}

// ── Unrecognized keys ───────────────────────────────────────────────

#[test]
fn unrecognized_key_returns_none() {
    assert_eq!(
        resolve(View::EnvelopeList, key(KeyCode::F(12))),
        Action::None
    );
}

// ── New modal scheme tests (replaces old Ctrl+p/s tests) ─────────────────────

#[test]
fn tab_cycles_identity_edit_fields() {
    // Tab advances through fields: Name → SenderName → Email → IsDefault → Save → Cancel → Name
    assert_eq!(
        resolve(View::IdentityEdit, key(KeyCode::Tab)),
        Action::IdentityEditFieldNext
    );
    assert_eq!(
        resolve(View::IdentityEdit, key(KeyCode::BackTab)),
        Action::IdentityEditFieldPrev
    );
}

#[test]
fn enter_activates_identity_edit_focused_item() {
    // Enter on any field = IdentityEditToggle (which app.rs routes to save/cancel/next)
    assert_eq!(
        resolve(View::IdentityEdit, key(KeyCode::Enter)),
        Action::IdentityEditToggle
    );
}

#[test]
fn tab_cycles_contact_edit_fields() {
    assert_eq!(
        resolve(View::ContactEdit, key(KeyCode::Tab)),
        Action::ContactEditFieldNext
    );
    assert_eq!(
        resolve(View::ContactEdit, key(KeyCode::BackTab)),
        Action::ContactEditFieldPrev
    );
}

#[test]
fn enter_activates_contact_edit_focused_item() {
    // Enter = ContactEditActivate (app.rs checks focused field and dispatches)
    assert_eq!(
        resolve(View::ContactEdit, key(KeyCode::Enter)),
        Action::ContactEditActivate
    );
}

#[test]
fn compose_enter_triggers_enter_insert() {
    assert_eq!(
        resolve(View::Compose, key(KeyCode::Enter)),
        Action::ComposeEnterInsert
    );
}

#[test]
fn compose_esc_triggers_exit_to_nav() {
    assert_eq!(
        resolve(View::Compose, key(KeyCode::Esc)),
        Action::ComposeExitToNav
    );
}

#[test]
fn compose_tab_advances_field() {
    assert_eq!(
        resolve(View::Compose, key(KeyCode::Tab)),
        Action::ComposeFieldNext
    );
}

#[test]
fn compose_j_advances_field() {
    assert_eq!(
        resolve(View::Compose, key(KeyCode::Char('j'))),
        Action::ComposeFieldNext
    );
}

#[test]
fn compose_k_retreats_field() {
    assert_eq!(
        resolve(View::Compose, key(KeyCode::Char('k'))),
        Action::ComposeFieldPrev
    );
}

#[test]
fn ctrl_c_still_discards_compose() {
    assert_eq!(
        resolve(View::Compose, ctrl(KeyCode::Char('c'))),
        Action::ComposeDiscard
    );
}

#[test]
fn ctrl_c_still_cancels_identity_edit() {
    assert_eq!(
        resolve(View::IdentityEdit, ctrl(KeyCode::Char('c'))),
        Action::IdentityEditCancel
    );
}

#[test]
fn ctrl_c_still_cancels_contact_edit() {
    assert_eq!(
        resolve(View::ContactEdit, ctrl(KeyCode::Char('c'))),
        Action::ContactEditCancel
    );
}
