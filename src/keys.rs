use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// The current view determines which keybindings are active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    EnvelopeList,
    MessageView,
    FolderList,
    AccountList,
    Search,
    Help,
    MovePrompt,
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
    None,
}

/// Resolve a key event into an action given the current view context.
pub fn resolve(view: View, key: KeyEvent) -> Action {
    // Global keybindings (handled first)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('a') => Action::SwitchAccount,
            KeyCode::Char('r') => Action::Refresh,
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
    }
}

fn resolve_envelope_list(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('g') => Action::JumpTop, // simplified: single g = top
        KeyCode::Char('G') => Action::JumpBottom,
        KeyCode::Enter => Action::OpenMessage,
        KeyCode::Char('c') => Action::Compose,
        KeyCode::Char('d') => Action::Delete,
        KeyCode::Char('m') => Action::MoveMessage,
        KeyCode::Char('!') => Action::ToggleFlag,
        KeyCode::Char('/') => Action::Search,
        KeyCode::Char('n') => Action::PageDown,
        KeyCode::Char('p') => Action::PageUp,
        KeyCode::Char('?') => Action::ToggleHelp,
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
            ("/", "search"),
            ("Tab", "folders"),
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
    }
}
