use crossterm::event::KeyEvent;
use ratatui::widgets::TableState;

use crate::himalaya::client;
use crate::himalaya::types::*;
use crate::keys::{self, Action, View};

/// Page size for envelope listing.
const PAGE_SIZE: usize = 50;

/// Top-level application state (TEA model).
pub struct App {
    pub running: bool,

    // ── View state ──────────────────────────────────────────────────
    pub view: View,
    pub previous_view: Option<View>,

    // ── Account state ───────────────────────────────────────────────
    pub accounts: Vec<Account>,
    pub account_index: usize,
    pub account_name: Option<String>,

    // ── Folder state ────────────────────────────────────────────────
    pub folders: Vec<Folder>,
    pub folder_index: usize,
    pub current_folder: String,

    // ── Envelope state ──────────────────────────────────────────────
    pub envelopes: Vec<Envelope>,
    pub envelope_state: TableState,
    pub page: usize,

    // ── Message view state ──────────────────────────────────────────
    pub message_body: String,
    pub message_scroll: u16,

    // ── Search state ────────────────────────────────────────────────
    pub search_query: String,
    pub active_query: Option<String>,

    // ── Move prompt state ───────────────────────────────────────────
    pub move_target: String,

    // ── Help scroll ─────────────────────────────────────────────────
    pub help_scroll: u16,

    // ── Status / loading ────────────────────────────────────────────
    pub status_message: String,
    pub loading: bool,
    pub tick_count: u64,

    // ── Shell-out command ───────────────────────────────────────────
    pub pending_shell: Option<String>,
}

impl App {
    pub fn new(initial_account: Option<String>) -> Self {
        Self {
            running: true,
            view: View::EnvelopeList,
            previous_view: None,
            accounts: Vec::new(),
            account_index: 0,
            account_name: initial_account,
            folders: Vec::new(),
            folder_index: 0,
            current_folder: "INBOX".to_string(),
            envelopes: Vec::new(),
            envelope_state: TableState::default(),
            page: 1,
            message_body: String::new(),
            message_scroll: 0,
            search_query: String::new(),
            active_query: None,
            move_target: String::new(),
            help_scroll: 0,
            status_message: String::new(),
            loading: false,
            tick_count: 0,
            pending_shell: None,
        }
    }

    /// Initial data load on startup.
    pub fn init(&mut self) {
        self.load_accounts();
        self.load_folders();
        self.load_envelopes();
    }

    /// Handle a tick event (loading spinner animation).
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    /// Set a transient status message.
    pub fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
    }

    /// Currently selected account name for himalaya commands.
    fn acct(&self) -> Option<&str> {
        self.account_name.as_deref()
    }

    /// Currently selected envelope ID, if any.
    pub fn selected_envelope_id(&self) -> Option<&str> {
        let idx = self.envelope_state.selected()?;
        self.envelopes.get(idx).map(|e| e.id.as_str())
    }

    /// Currently selected envelope, if any.
    pub fn selected_envelope(&self) -> Option<&Envelope> {
        let idx = self.envelope_state.selected()?;
        self.envelopes.get(idx)
    }

    // ── Data loading ────────────────────────────────────────────────

    fn load_accounts(&mut self) {
        match client::list_accounts() {
            Ok(accounts) => {
                // If no account was specified on the CLI, use the default.
                if self.account_name.is_none() {
                    if let Some(default) = accounts.iter().find(|a| a.default) {
                        self.account_name = Some(default.name.clone());
                    } else if let Some(first) = accounts.first() {
                        self.account_name = Some(first.name.clone());
                    }
                }
                // Set account_index to match account_name
                if let Some(name) = &self.account_name {
                    self.account_index = accounts.iter().position(|a| &a.name == name).unwrap_or(0);
                }
                self.accounts = accounts;
            }
            Err(e) => {
                self.set_status(&format!("Failed to load accounts: {e}"));
            }
        }
    }

    fn load_folders(&mut self) {
        match client::list_folders(self.acct()) {
            Ok(folders) => {
                self.folders = folders;
                // Reset folder index to match current_folder
                self.folder_index = self
                    .folders
                    .iter()
                    .position(|f| f.name == self.current_folder)
                    .unwrap_or(0);
            }
            Err(e) => {
                self.set_status(&format!("Failed to load folders: {e}"));
            }
        }
    }

    fn load_envelopes(&mut self) {
        self.loading = true;
        match client::list_envelopes(
            self.acct(),
            &self.current_folder,
            self.page,
            PAGE_SIZE,
            self.active_query.as_deref(),
        ) {
            Ok(envelopes) => {
                self.envelopes = envelopes;
                if !self.envelopes.is_empty() {
                    self.envelope_state.select(Some(0));
                } else {
                    self.envelope_state.select(None);
                }
                self.loading = false;
                if self.envelopes.is_empty() {
                    self.set_status("No messages.");
                } else {
                    self.status_message.clear();
                }
            }
            Err(e) => {
                self.loading = false;
                self.set_status(&format!("Failed to load envelopes: {e}"));
            }
        }
    }

    pub fn refresh_envelopes(&mut self) {
        self.load_envelopes();
    }

    fn load_message(&mut self) {
        if let Some(id) = self.selected_envelope_id().map(|s| s.to_string()) {
            self.loading = true;
            match client::read_message(self.acct(), &self.current_folder, &id) {
                Ok(body) => {
                    self.message_body = body;
                    self.message_scroll = 0;
                    self.loading = false;
                    self.view = View::MessageView;
                    // Mark as seen in local state
                    if let Some(idx) = self.envelope_state.selected() {
                        if let Some(env) = self.envelopes.get_mut(idx) {
                            if !env.is_seen() {
                                env.flags.push("Seen".to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    self.loading = false;
                    self.set_status(&format!("Failed to read message: {e}"));
                }
            }
        }
    }

    // ── Key handling ────────────────────────────────────────────────

    pub fn handle_key(&mut self, key: KeyEvent) {
        let action = keys::resolve(self.view, key);
        self.status_message.clear();

        match action {
            Action::Quit => self.running = false,
            Action::Back => self.go_back(),
            Action::MoveUp => self.move_selection(-1),
            Action::MoveDown => self.move_selection(1),
            Action::PageUp => self.page_up(),
            Action::PageDown => self.page_down(),
            Action::JumpTop => self.jump_top(),
            Action::JumpBottom => self.jump_bottom(),
            Action::Select => self.select_item(),
            Action::OpenMessage => self.load_message(),
            Action::Compose => self.compose(),
            Action::Reply => self.reply(false),
            Action::ReplyAll => self.reply(true),
            Action::Forward => self.forward(),
            Action::Delete => self.delete(),
            Action::ToggleFlag => self.toggle_flag(),
            Action::DownloadAttachments => self.download_attachments(),
            Action::Search => self.enter_search(),
            Action::SearchSubmit => self.submit_search(),
            Action::SearchCancel => self.cancel_search(),
            Action::SearchInput(c) => self.search_query.push(c),
            Action::SearchBackspace => {
                self.search_query.pop();
            }
            Action::Refresh => self.refresh(),
            Action::SwitchAccount => self.enter_account_picker(),
            Action::ToggleHelp => self.toggle_help(),
            Action::FocusFolders => self.view = View::FolderList,
            Action::FocusEnvelopes => self.view = View::EnvelopeList,
            Action::ScrollUp => self.scroll(-1),
            Action::ScrollDown => self.scroll(1),
            Action::MoveMessage => self.enter_move_prompt(),
            Action::MoveInput(c) => self.move_target.push(c),
            Action::MoveBackspace => {
                self.move_target.pop();
            }
            Action::MoveSubmit => self.submit_move(),
            Action::MoveCancel => self.cancel_move(),
            Action::None => {}
        }
    }

    // ── Action handlers ─────────────────────────────────────────────

    fn go_back(&mut self) {
        match self.view {
            View::MessageView => {
                self.view = View::EnvelopeList;
                self.message_body.clear();
            }
            View::AccountList => {
                self.view = View::EnvelopeList;
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: i32) {
        match self.view {
            View::EnvelopeList => {
                let len = self.envelopes.len();
                if len == 0 {
                    return;
                }
                let current = self.envelope_state.selected().unwrap_or(0);
                let next = if delta > 0 {
                    (current + 1).min(len - 1)
                } else {
                    current.saturating_sub(1)
                };
                self.envelope_state.select(Some(next));
            }
            View::FolderList => {
                let len = self.folders.len();
                if len == 0 {
                    return;
                }
                if delta > 0 {
                    self.folder_index = (self.folder_index + 1).min(len - 1);
                } else {
                    self.folder_index = self.folder_index.saturating_sub(1);
                }
            }
            View::AccountList => {
                let len = self.accounts.len();
                if len == 0 {
                    return;
                }
                if delta > 0 {
                    self.account_index = (self.account_index + 1).min(len - 1);
                } else {
                    self.account_index = self.account_index.saturating_sub(1);
                }
            }
            _ => {}
        }
    }

    fn jump_top(&mut self) {
        match self.view {
            View::EnvelopeList => {
                if !self.envelopes.is_empty() {
                    self.envelope_state.select(Some(0));
                }
            }
            View::MessageView | View::Help => {
                if self.view == View::Help {
                    self.help_scroll = 0;
                } else {
                    self.message_scroll = 0;
                }
            }
            _ => {}
        }
    }

    fn jump_bottom(&mut self) {
        match self.view {
            View::EnvelopeList => {
                if !self.envelopes.is_empty() {
                    self.envelope_state.select(Some(self.envelopes.len() - 1));
                }
            }
            View::MessageView => {
                let lines = self.message_body.lines().count() as u16;
                self.message_scroll = lines.saturating_sub(5);
            }
            View::Help => {
                self.help_scroll = 100; // will be clamped in render
            }
            _ => {}
        }
    }

    fn page_up(&mut self) {
        if self.view == View::EnvelopeList && self.page > 1 {
            self.page -= 1;
            self.load_envelopes();
        }
    }

    fn page_down(&mut self) {
        if self.view == View::EnvelopeList && self.envelopes.len() >= PAGE_SIZE {
            self.page += 1;
            self.load_envelopes();
        }
    }

    fn select_item(&mut self) {
        match self.view {
            View::FolderList => {
                if let Some(folder) = self.folders.get(self.folder_index) {
                    self.current_folder = folder.name.clone();
                    self.page = 1;
                    self.active_query = None;
                    self.view = View::EnvelopeList;
                    self.load_envelopes();
                }
            }
            View::AccountList => {
                if let Some(account) = self.accounts.get(self.account_index) {
                    self.account_name = Some(account.name.clone());
                    self.current_folder = "INBOX".to_string();
                    self.page = 1;
                    self.active_query = None;
                    self.view = View::EnvelopeList;
                    self.load_folders();
                    self.load_envelopes();
                }
            }
            _ => {}
        }
    }

    fn scroll(&mut self, delta: i32) {
        match self.view {
            View::MessageView => {
                if delta > 0 {
                    self.message_scroll = self.message_scroll.saturating_add(1);
                } else {
                    self.message_scroll = self.message_scroll.saturating_sub(1);
                }
            }
            View::Help => {
                if delta > 0 {
                    self.help_scroll = self.help_scroll.saturating_add(1);
                } else {
                    self.help_scroll = self.help_scroll.saturating_sub(1);
                }
            }
            _ => {}
        }
    }

    fn compose(&mut self) {
        self.pending_shell = Some(client::compose_command(self.acct()));
    }

    fn reply(&mut self, all: bool) {
        if let Some(id) = self.selected_envelope_id().map(|s| s.to_string()) {
            self.pending_shell = Some(client::reply_command(
                self.acct(),
                &self.current_folder,
                &id,
                all,
            ));
        }
    }

    fn forward(&mut self) {
        if let Some(id) = self.selected_envelope_id().map(|s| s.to_string()) {
            self.pending_shell = Some(client::forward_command(
                self.acct(),
                &self.current_folder,
                &id,
            ));
        }
    }

    fn delete(&mut self) {
        if let Some(id) = self.selected_envelope_id().map(|s| s.to_string()) {
            match client::delete_message(self.acct(), &self.current_folder, &id) {
                Ok(()) => {
                    self.set_status("Message deleted.");
                    if self.view == View::MessageView {
                        self.view = View::EnvelopeList;
                    }
                    self.load_envelopes();
                }
                Err(e) => self.set_status(&format!("Delete failed: {e}")),
            }
        }
    }

    fn toggle_flag(&mut self) {
        if let Some(id) = self.selected_envelope_id().map(|s| s.to_string()) {
            let is_flagged = self
                .selected_envelope()
                .map(|e| e.is_flagged())
                .unwrap_or(false);
            let result = if is_flagged {
                client::flag_remove(self.acct(), &self.current_folder, &id, "flagged")
            } else {
                client::flag_add(self.acct(), &self.current_folder, &id, "flagged")
            };
            match result {
                Ok(()) => self.load_envelopes(),
                Err(e) => self.set_status(&format!("Flag toggle failed: {e}")),
            }
        }
    }

    fn download_attachments(&mut self) {
        if let Some(id) = self.selected_envelope_id().map(|s| s.to_string()) {
            match client::download_attachments(self.acct(), &self.current_folder, &id) {
                Ok(output) => self.set_status(&format!("Attachments: {}", output.trim())),
                Err(e) => self.set_status(&format!("Download failed: {e}")),
            }
        }
    }

    fn enter_search(&mut self) {
        self.search_query.clear();
        self.view = View::Search;
    }

    fn submit_search(&mut self) {
        let query = self.search_query.clone();
        self.active_query = if query.is_empty() { None } else { Some(query) };
        self.page = 1;
        self.view = View::EnvelopeList;
        self.load_envelopes();
    }

    fn cancel_search(&mut self) {
        self.view = View::EnvelopeList;
    }

    fn refresh(&mut self) {
        self.load_folders();
        self.load_envelopes();
        self.set_status("Refreshed.");
    }

    fn enter_account_picker(&mut self) {
        self.load_accounts();
        self.view = View::AccountList;
    }

    fn toggle_help(&mut self) {
        if self.view == View::Help {
            self.view = self.previous_view.unwrap_or(View::EnvelopeList);
            self.previous_view = None;
        } else {
            self.previous_view = Some(self.view);
            self.help_scroll = 0;
            self.view = View::Help;
        }
    }

    fn enter_move_prompt(&mut self) {
        if self.selected_envelope_id().is_some() {
            self.move_target.clear();
            self.view = View::MovePrompt;
        }
    }

    fn submit_move(&mut self) {
        let target = self.move_target.clone();
        if target.is_empty() {
            self.set_status("No target folder specified.");
            self.view = View::EnvelopeList;
            return;
        }
        if let Some(id) = self.selected_envelope_id().map(|s| s.to_string()) {
            match client::move_message(self.acct(), &self.current_folder, &target, &id) {
                Ok(()) => {
                    self.set_status(&format!("Moved to {target}."));
                    self.view = View::EnvelopeList;
                    self.load_envelopes();
                }
                Err(e) => {
                    self.set_status(&format!("Move failed: {e}"));
                    self.view = View::EnvelopeList;
                }
            }
        }
    }

    fn cancel_move(&mut self) {
        self.view = View::EnvelopeList;
    }
}
