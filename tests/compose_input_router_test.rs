use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pretty_assertions::assert_eq;
use solverforge_mail::keys::{
    resolve_compose_with_context, Action, ComposeFocus, ComposeKeyContext, EditMode,
};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

fn ctx(
    focus: ComposeFocus,
    edit_mode: EditMode,
    autocomplete_visible: bool,
    confirm_discard_visible: bool,
) -> ComposeKeyContext {
    ComposeKeyContext {
        focus,
        edit_mode,
        autocomplete_visible,
        confirm_discard_visible,
    }
}

#[test]
fn precedence_ctrl_shortcuts_override_everything() {
    let contexts = [
        ctx(ComposeFocus::Header, EditMode::Nav, false, false),
        ctx(ComposeFocus::Body, EditMode::Insert, true, false),
        ctx(ComposeFocus::ActionBar, EditMode::Nav, false, true),
    ];
    for c in contexts {
        assert_eq!(
            resolve_compose_with_context(ctrl(KeyCode::Char('c')), c),
            Action::ComposeDiscard
        );
    }
}

#[test]
fn precedence_discard_confirm_blocks_other_layers() {
    let c = ctx(ComposeFocus::Body, EditMode::Insert, true, true);
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::Char('y')), c),
        Action::ComposeConfirmDiscard
    );
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::Esc), c),
        Action::ComposeCancelDiscard
    );
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::Char('j')), c),
        Action::None
    );
}

#[test]
fn precedence_autocomplete_intercepts_navigation_acceptance_keys() {
    let c = ctx(ComposeFocus::Header, EditMode::Insert, true, false);
    for code in [
        KeyCode::Down,
        KeyCode::Up,
        KeyCode::Enter,
        KeyCode::Tab,
        KeyCode::Esc,
    ] {
        assert_eq!(
            resolve_compose_with_context(key(code), c),
            Action::EditorKey(key(code))
        );
    }
}

#[test]
fn body_focus_passthrough_except_tab_cycle() {
    let c = ctx(ComposeFocus::Body, EditMode::Nav, false, false);
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::Char('j')), c),
        Action::EditorKey(key(KeyCode::Char('j')))
    );
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::Backspace), c),
        Action::EditorKey(key(KeyCode::Backspace))
    );
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::Tab), c),
        Action::ComposeFieldNext
    );
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::BackTab), c),
        Action::ComposeFieldPrev
    );
}

#[test]
fn header_nav_mode_uses_jk_for_field_navigation() {
    let c = ctx(ComposeFocus::Header, EditMode::Nav, false, false);
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::Char('j')), c),
        Action::ComposeFieldNext
    );
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::Char('k')), c),
        Action::ComposeFieldPrev
    );
}

#[test]
fn header_insert_mode_uses_jk_as_text_input() {
    let c = ctx(ComposeFocus::Header, EditMode::Insert, false, false);
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::Char('j')), c),
        Action::ComposeInput('j')
    );
    assert_eq!(
        resolve_compose_with_context(key(KeyCode::Char('k')), c),
        Action::ComposeInput('k')
    );
}
