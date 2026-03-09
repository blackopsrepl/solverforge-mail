use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// The current view determines which keybindings are active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum View {
    EnvelopeList,
    MessageView,
    FolderList,
    AccountList,
    Search,
    Help,
    MovePrompt,
    /// Native compose / reply / forward editor.
    Compose,
    /// Address book browser.
    Contacts,
    /// Contact search input mode (within the address book).
    ContactSearch,
    /// Contact add/edit form.
    ContactEdit,
    /// Identity list for the current account.
    IdentityList,
    /// Identity add/edit form.
    IdentityEdit,
}

/// Modal editing mode for compose / identity-edit / contact-edit forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    /// Navigation: j/k/Tab move between fields; Enter enters Insert on text fields.
    Nav,
    /// Insert: characters typed freely into the focused field; Esc → Nav.
    Insert,
}

/// Actions the app can take in response to a key press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    Back,
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    JumpTop,
    JumpBottom,
    Select,
    OpenMessage,
    Compose,
    Reply,
    ReplyAll,
    Forward,
    Delete,
    ToggleFlag,
    DownloadAttachments,
    ToggleThread,
    Search,
    SearchSubmit,
    SearchCancel,
    SearchInput(char),
    SearchBackspace,
    Refresh,
    SwitchAccount,
    ToggleHelp,
    FocusFolders,
    FocusEnvelopes,
    MoveMessage,
    MoveInput(char),
    MoveBackspace,
    MoveSubmit,
    MoveCancel,
    ScrollUp,
    ScrollDown,
    // ── Compose editor ────────────────────────────────────────────────
    ComposeFieldNext,
    ComposeFieldPrev,
    ComposeSend,
    ComposeDiscard,
    ComposeConfirmDiscard,
    ComposeCancelDiscard,
    ComposeInput(char),
    ComposeBackspace,
    /// Enter Insert mode on the current field (Nav → Insert).
    ComposeEnterInsert,
    /// Exit Insert mode back to Nav (Insert → Nav).
    ComposeExitToNav,
    // ── Contacts browser ──────────────────────────────────────────────
    OpenContacts,
    ContactNew,
    ContactDelete,
    ContactEdit,
    ContactSearch,
    ContactSearchInput(char),
    ContactSearchBackspace,
    ContactSearchCancel,
    // ── Contact edit form ─────────────────────────────────────────────
    ContactEditFieldNext,
    ContactEditFieldPrev,
    ContactEditInput(char),
    ContactEditBackspace,
    ContactEditSave,
    ContactEditCancel,
    /// Enter key on contact edit: activates focused action-button or advances field.
    ContactEditActivate,
    // ── Identity list ─────────────────────────────────────────────────
    OpenIdentities,
    IdentityNew,
    IdentityEditSelected,
    IdentityDelete,
    IdentitySetDefault,
    IdentityListUp,
    IdentityListDown,
    IdentityListClose,
    // ── Identity edit form ────────────────────────────────────────────
    IdentityEditFieldNext,
    IdentityEditFieldPrev,
    IdentityEditInput(char),
    IdentityEditBackspace,
    IdentityEditToggle,
    IdentityEditSave,
    IdentityEditCancel,
    // ── Passthrough for edtui (compose body editor) ───────────────────
    /// Raw key event forwarded to edtui.
    EditorKey(crossterm::event::KeyEvent),
    None,
}

/// Resolve a key event into an action given the current view context.
pub fn resolve(view: View, key: KeyEvent) -> Action {
    match view {
        View::Compose => return resolve_compose(key),
        View::Contacts => return resolve_contacts(key),
        View::ContactSearch => return resolve_contact_search(key),
        View::ContactEdit => return resolve_contact_edit(key),
        View::IdentityList => return resolve_identity_list(key),
        View::IdentityEdit => return resolve_identity_edit(key),
        _ => {}
    }

    // Global keybindings (handled first)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('a') => Action::SwitchAccount,
            KeyCode::Char('r') => Action::Refresh,
            KeyCode::Char('b') => Action::OpenContacts,
            _ => Action::None,
        };
    }

    match view {
        View::EnvelopeList => resolve_envelope_list(key),
        View::MessageView => resolve_message_view(key),
        View::FolderList => resolve_folder_list(key),
        View::AccountList => resolve_account_list(key),
        View::Search => resolve_search(key),
        View::Help => resolve_help(key),
        View::MovePrompt => resolve_move_prompt(key),
        View::ContactSearch => resolve_contact_search(key),
        View::ContactEdit => resolve_contact_edit(key),
        // Already handled above
        View::Compose | View::Contacts | View::IdentityList | View::IdentityEdit => Action::None,
    }
}

fn resolve_envelope_list(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('g') => Action::JumpTop,
        KeyCode::Char('G') => Action::JumpBottom,
        KeyCode::Enter => Action::OpenMessage,
        KeyCode::Char('c') => Action::Compose,
        KeyCode::Char('d') => Action::Delete,
        KeyCode::Char('m') => Action::MoveMessage,
        KeyCode::Char('!') => Action::ToggleFlag,
        KeyCode::Char('/') => Action::Search,
        KeyCode::Char('n') => Action::PageDown,
        KeyCode::Char('p') => Action::PageUp,
        KeyCode::Char('t') => Action::ToggleThread,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('I') => Action::OpenIdentities,
        KeyCode::Tab => Action::FocusFolders,
        KeyCode::Esc => Action::Quit,
        _ => Action::None,
    }
}

fn resolve_message_view(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Back,
        KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,
        KeyCode::Char(' ') => Action::PageDown,
        KeyCode::Char('r') => Action::Reply,
        KeyCode::Char('R') => Action::ReplyAll,
        KeyCode::Char('f') => Action::Forward,
        KeyCode::Char('d') => Action::Delete,
        KeyCode::Char('a') => Action::DownloadAttachments,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('g') => Action::JumpTop,
        KeyCode::Char('G') => Action::JumpBottom,
        _ => Action::None,
    }
}

fn resolve_folder_list(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::FocusEnvelopes,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Enter => Action::Select,
        KeyCode::Tab => Action::FocusEnvelopes,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('I') => Action::OpenIdentities,
        _ => Action::None,
    }
}

fn resolve_account_list(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Back,
        KeyCode::Char('q') => Action::Back,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Enter => Action::Select,
        _ => Action::None,
    }
}

fn resolve_search(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Enter => Action::SearchSubmit,
        KeyCode::Esc => Action::SearchCancel,
        KeyCode::Backspace => Action::SearchBackspace,
        KeyCode::Char(c) => Action::SearchInput(c),
        _ => Action::None,
    }
}

fn resolve_help(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,
        _ => Action::None,
    }
}

fn resolve_move_prompt(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Enter => Action::MoveSubmit,
        KeyCode::Esc => Action::MoveCancel,
        KeyCode::Backspace => Action::MoveBackspace,
        KeyCode::Char(c) => Action::MoveInput(c),
        _ => Action::None,
    }
}

/// Resolve compose keys.  The caller (app.rs) is responsible for knowing
/// whether we are in Nav or Insert mode and dispatching accordingly.
/// We encode the mode intent into the Action so app.rs can act on it cleanly.
///
/// Modal scheme:
///   Nav mode:
///     j / Down        → ComposeFieldNext
///     k / Up          → ComposeFieldPrev
///     Tab             → ComposeFieldNext
///     BackTab         → ComposeFieldPrev
///     Enter           → ComposeEnterInsert  (enters Insert on text fields)
///     Esc             → ComposeExitToNav    (on action-bar buttons → back to Body)
///   Insert mode (text header fields):
///     Esc             → ComposeExitToNav
///     Backspace       → ComposeBackspace
///     any printable   → ComposeInput(c)
///   Body (edtui handles its own modal editing via EditorKey passthrough):
///     Enter in Nav    → ComposeEnterInsert  (let edtui enter insert)
///     Esc in Insert   → ComposeExitToNav    (exit edtui insert → Nav)
///     all other keys  → EditorKey passthrough
fn resolve_compose(key: KeyEvent) -> Action {
    // Allow Ctrl+C / Ctrl+Q globally in compose as quit-discard
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('q') => Action::ComposeDiscard,
            _ => Action::EditorKey(key),
        };
    }

    match key.code {
        KeyCode::Tab => Action::ComposeFieldNext,
        KeyCode::BackTab => Action::ComposeFieldPrev,
        KeyCode::Down | KeyCode::Char('j') => Action::ComposeFieldNext,
        KeyCode::Up | KeyCode::Char('k') => Action::ComposeFieldPrev,
        KeyCode::Enter => Action::ComposeEnterInsert,
        KeyCode::Esc => Action::ComposeExitToNav,
        KeyCode::Backspace => Action::ComposeBackspace,
        KeyCode::Char(c) => Action::ComposeInput(c),
        _ => Action::EditorKey(key),
    }
}

fn resolve_contact_edit(key: KeyEvent) -> Action {
    // Ctrl+C / Ctrl+Q cancel without saving
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('q') => Action::ContactEditCancel,
            _ => Action::None,
        };
    }
    match key.code {
        KeyCode::Esc => Action::ContactEditCancel,
        KeyCode::Tab => Action::ContactEditFieldNext,
        KeyCode::BackTab => Action::ContactEditFieldPrev,
        KeyCode::Enter => Action::ContactEditActivate,
        KeyCode::Backspace => Action::ContactEditBackspace,
        KeyCode::Char(c) => Action::ContactEditInput(c),
        _ => Action::None,
    }
}

fn resolve_contact_search(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::ContactSearchCancel,
        KeyCode::Enter => Action::ContactSearchCancel,
        KeyCode::Backspace => Action::ContactSearchBackspace,
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::ContactSearchInput(c)
        }
        _ => Action::None,
    }
}

fn resolve_identity_list(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Action::IdentityListClose,
        KeyCode::Char('j') | KeyCode::Down => Action::IdentityListDown,
        KeyCode::Char('k') | KeyCode::Up => Action::IdentityListUp,
        KeyCode::Char('n') => Action::IdentityNew,
        KeyCode::Char('e') | KeyCode::Enter => Action::IdentityEditSelected,
        KeyCode::Char('d') => Action::IdentityDelete,
        KeyCode::Char('s') => Action::IdentitySetDefault,
        _ => Action::None,
    }
}

fn resolve_identity_edit(key: KeyEvent) -> Action {
    // Ctrl+C / Ctrl+Q cancel without saving
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('q') => Action::IdentityEditCancel,
            _ => Action::None,
        };
    }
    match key.code {
        KeyCode::Esc => Action::IdentityEditCancel,
        KeyCode::Tab => Action::IdentityEditFieldNext,
        KeyCode::BackTab => Action::IdentityEditFieldPrev,
        KeyCode::Enter => Action::IdentityEditToggle,
        KeyCode::Backspace => Action::IdentityEditBackspace,
        KeyCode::Char(c) => Action::IdentityEditInput(c),
        _ => Action::None,
    }
}

fn resolve_contacts(key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('b') => Action::Back,
            KeyCode::Char('c') | KeyCode::Char('q') => Action::Quit,
            _ => Action::None,
        };
    }
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Action::Back,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('g') => Action::JumpTop,
        KeyCode::Char('G') => Action::JumpBottom,
        KeyCode::Char('n') => Action::ContactNew,
        KeyCode::Char('d') => Action::ContactDelete,
        KeyCode::Char('e') => Action::ContactEdit,
        KeyCode::Char('/') => Action::ContactSearch,
        KeyCode::Char('c') => Action::Compose,
        _ => Action::None,
    }
}

/// Keybinding hint strings for the status bar, per view.
pub fn hints(view: View) -> Vec<(&'static str, &'static str)> {
    match view {
        View::EnvelopeList => vec![
            ("j/k", "nav"),
            ("Enter", "read"),
            ("c", "compose"),
            ("d", "del"),
            ("m", "move"),
            ("!", "flag"),
            ("t", "thread"),
            ("/", "search"),
            ("Tab", "folders"),
            ("Ctrl+b", "contacts"),
            ("?", "help"),
        ],
        View::MessageView => vec![
            ("j/k", "scroll"),
            ("r", "reply"),
            ("R", "all"),
            ("f", "fwd"),
            ("d", "del"),
            ("a", "attach"),
            ("q", "back"),
            ("?", "help"),
        ],
        View::FolderList => vec![
            ("j/k", "nav"),
            ("Enter", "select"),
            ("Tab", "emails"),
            ("?", "help"),
        ],
        View::AccountList => vec![("j/k", "nav"), ("Enter", "select"), ("Esc", "cancel")],
        View::Search => vec![("Enter", "search"), ("Esc", "cancel")],
        View::Help => vec![("j/k", "scroll"), ("q/?/Esc", "close")],
        View::MovePrompt => vec![("Enter", "move"), ("Esc", "cancel")],
        View::Compose => vec![
            ("Tab/j/k", "nav"),
            ("Enter", "insert"),
            ("Esc", "nav/discard"),
        ],
        View::Contacts => vec![
            ("j/k", "nav"),
            ("n", "new"),
            ("e", "edit"),
            ("d", "del"),
            ("/", "search"),
            ("q", "close"),
        ],
        View::ContactSearch => vec![("Enter", "confirm"), ("Esc", "cancel")],
        View::ContactEdit => vec![
            ("Tab/Enter", "next field"),
            ("Tab→Save→Enter", "save"),
            ("Esc", "cancel"),
        ],
        View::IdentityList => vec![
            ("j/k", "nav"),
            ("n", "new"),
            ("e", "edit"),
            ("d", "delete"),
            ("s", "set default"),
            ("q/Esc", "close"),
        ],
        View::IdentityEdit => vec![
            ("Tab/Enter", "next field"),
            ("Space", "toggle"),
            ("Tab→Save→Enter", "save"),
            ("Esc", "cancel"),
        ],
    }
}
