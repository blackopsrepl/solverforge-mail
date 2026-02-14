use serde::Deserialize;

/// An email account as returned by `himalaya -o json account list`.
#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    pub name: String,
    pub backend: String,
    pub default: bool,
}

/// A mail folder (mailbox) as returned by `himalaya -o json folder list`.
#[derive(Debug, Clone, Deserialize)]
pub struct Folder {
    pub name: String,
    #[serde(default)]
    pub desc: Option<String>,
}

/// An envelope (message summary) as returned by `himalaya -o json envelope list`.
#[derive(Debug, Clone, Deserialize)]
pub struct Envelope {
    pub id: String,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub subject: String,
    #[serde(default, alias = "from")]
    pub sender: Sender,
    #[serde(default)]
    pub date: String,
}

/// The sender field can be a string or a structured object depending
/// on himalaya version.  We normalize to a display string.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(untagged)]
pub enum Sender {
    Plain(String),
    Structured {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        addr: Option<String>,
    },
    #[default]
    Unknown,
}

impl Sender {
    pub fn display(&self) -> String {
        match self {
            Sender::Plain(s) => s.clone(),
            Sender::Structured { name, addr } => {
                if let Some(n) = name {
                    if !n.is_empty() {
                        return n.clone();
                    }
                }
                addr.clone().unwrap_or_default()
            }
            Sender::Unknown => String::new(),
        }
    }
}

impl std::fmt::Display for Sender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl Envelope {
    /// Whether this envelope has been read.
    pub fn is_seen(&self) -> bool {
        self.flags.iter().any(|f| f.eq_ignore_ascii_case("seen"))
    }

    /// Whether this envelope is flagged/starred.
    pub fn is_flagged(&self) -> bool {
        self.flags.iter().any(|f| f.eq_ignore_ascii_case("flagged"))
    }

    /// Whether this envelope has been answered.
    pub fn is_answered(&self) -> bool {
        self.flags
            .iter()
            .any(|f| f.eq_ignore_ascii_case("answered"))
    }

    /// Single-character flag indicator for the list view.
    pub fn flag_icon(&self) -> &'static str {
        if self.is_flagged() {
            "!"
        } else if !self.is_seen() {
            "\u{25cf}" // ●
        } else if self.is_answered() {
            "\u{21a9}" // ↩
        } else {
            " "
        }
    }

    /// Sender display string.
    pub fn sender_display(&self) -> String {
        self.sender.display()
    }
}

/// Parsed response types from himalaya CLI.
#[derive(Debug)]
pub enum HimalayaResponse {
    Accounts(Vec<Account>),
    Folders(Vec<Folder>),
    Envelopes(Vec<Envelope>),
    MessageBody(String),
    Success(String),
    Error(String),
}
