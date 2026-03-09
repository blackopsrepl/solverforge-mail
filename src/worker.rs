use std::sync::mpsc;
use std::thread;

use crate::himalaya::client;
use crate::himalaya::types::*;

/// Messages sent from background threads back to the main App.
#[derive(Debug)]
pub enum WorkerResult {
    Accounts(Result<Vec<Account>, String>),
    Folders(Result<Vec<Folder>, String>),
    Envelopes(Result<Vec<Envelope>, String>),
    Message(Result<String, String>),
    ActionDone(Result<String, String>),
    /// Unread count for a specific folder: (folder_name, count).
    FolderUnread(String, Result<usize, String>),
    /// A fetched compose/reply/forward template.
    Template(Result<String, String>),
    /// Result of sending a template.
    SendDone(Result<String, String>),
}

/// Lightweight handle for dispatching work to background threads.
/// Results are collected via try_recv() in the main loop.
pub struct Worker {
    tx: mpsc::Sender<WorkerResult>,
    rx: mpsc::Receiver<WorkerResult>,
}

impl Default for Worker {
    fn default() -> Self {
        Self::new()
    }
}

impl Worker {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self { tx, rx }
    }

    /// Non-blocking: returns any completed results.
    pub fn try_recv(&self) -> Option<WorkerResult> {
        self.rx.try_recv().ok()
    }

    /// Drain all pending results.
    pub fn drain(&self) -> Vec<WorkerResult> {
        let mut results = Vec::new();
        while let Some(r) = self.try_recv() {
            results.push(r);
        }
        results
    }

    // ── Dispatch methods ────────────────────────────────────────────

    pub fn fetch_accounts(&self) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::list_accounts().map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::Accounts(result));
        });
    }

    pub fn fetch_folders(&self, account: Option<String>) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::list_folders(account.as_deref()).map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::Folders(result));
        });
    }

    pub fn fetch_envelopes(
        &self,
        account: Option<String>,
        folder: String,
        page: usize,
        page_size: usize,
        query: Option<String>,
    ) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::list_envelopes(
                account.as_deref(),
                &folder,
                page,
                page_size,
                query.as_deref(),
            )
            .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::Envelopes(result));
        });
    }

    pub fn fetch_envelopes_threaded(
        &self,
        account: Option<String>,
        folder: String,
        query: Option<String>,
    ) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result =
                client::list_envelopes_threaded(account.as_deref(), &folder, query.as_deref())
                    .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::Envelopes(result));
        });
    }

    /// Fetch unread count for a specific folder by querying for unseen envelopes.
    pub fn fetch_folder_unread(&self, account: Option<String>, folder: String) {
        let tx = self.tx.clone();
        let folder_name = folder.clone();
        thread::spawn(move || {
            // Query for unseen envelopes with page-size 1 to get a count.
            // We use the envelope list with "not flag seen" filter.
            let result = client::list_envelopes(
                account.as_deref(),
                &folder,
                1,
                200, // fetch up to 200 to count
                Some("not flag seen"),
            )
            .map(|envs| envs.len())
            .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::FolderUnread(folder_name, result));
        });
    }

    pub fn fetch_message(&self, account: Option<String>, folder: String, id: String) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result =
                client::read_message(account.as_deref(), &folder, &id).map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::Message(result));
        });
    }

    pub fn delete_message(&self, account: Option<String>, folder: String, id: String) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::delete_message(account.as_deref(), &folder, &id)
                .map(|()| "Message deleted.".to_string())
                .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::ActionDone(result));
        });
    }

    pub fn flag_add(&self, account: Option<String>, folder: String, id: String, flag: String) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::flag_add(account.as_deref(), &folder, &id, &flag)
                .map(|()| format!("Flag '{flag}' added."))
                .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::ActionDone(result));
        });
    }

    pub fn flag_remove(&self, account: Option<String>, folder: String, id: String, flag: String) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::flag_remove(account.as_deref(), &folder, &id, &flag)
                .map(|()| format!("Flag '{flag}' removed."))
                .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::ActionDone(result));
        });
    }

    pub fn move_message(
        &self,
        account: Option<String>,
        folder: String,
        target: String,
        id: String,
    ) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::move_message(account.as_deref(), &folder, &target, &id)
                .map(|()| format!("Moved to {target}."))
                .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::ActionDone(result));
        });
    }

    pub fn download_attachments(&self, account: Option<String>, folder: String, id: String) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::download_attachments(account.as_deref(), &folder, &id)
                .map(|s| format!("Attachments: {}", s.trim()))
                .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::ActionDone(result));
        });
    }

    /// Fetch a compose template (new message).
    pub fn fetch_template_write(&self, account: Option<String>) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::template_write(account.as_deref()).map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::Template(result));
        });
    }

    /// Fetch a reply template.
    pub fn fetch_template_reply(
        &self,
        account: Option<String>,
        folder: String,
        id: String,
        all: bool,
    ) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::template_reply(account.as_deref(), &folder, &id, all)
                .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::Template(result));
        });
    }

    /// Fetch a forward template.
    pub fn fetch_template_forward(&self, account: Option<String>, folder: String, id: String) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::template_forward(account.as_deref(), &folder, &id)
                .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::Template(result));
        });
    }

    /// Send a compiled template.
    pub fn send_template(&self, account: Option<String>, template: String) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let result = client::template_send(account.as_deref(), &template)
                .map(|s| {
                    let s = s.trim();
                    if s.is_empty() {
                        "Message sent.".to_string()
                    } else {
                        s.to_string()
                    }
                })
                .map_err(|e| format!("{e}"));
            let _ = tx.send(WorkerResult::SendDone(result));
        });
    }
}
