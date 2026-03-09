/* Contact add/edit form state. */

use crate::contacts::Contact;

/// Which field in the contact edit form currently has focus.
///
/// Tab cycle: Name → Email → Phone → Org → Notes → Tags → Save → Cancel → Name
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactField {
    Name,
    Email,
    Phone,
    Org,
    Notes,
    Tags,
    // ── Action bar buttons ─────────────────────────────────────────
    Save,
    Cancel,
}

impl ContactField {
    pub fn next(self) -> Self {
        match self {
            ContactField::Name => ContactField::Email,
            ContactField::Email => ContactField::Phone,
            ContactField::Phone => ContactField::Org,
            ContactField::Org => ContactField::Notes,
            ContactField::Notes => ContactField::Tags,
            ContactField::Tags => ContactField::Save,
            ContactField::Save => ContactField::Cancel,
            ContactField::Cancel => ContactField::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ContactField::Name => ContactField::Cancel,
            ContactField::Email => ContactField::Name,
            ContactField::Phone => ContactField::Email,
            ContactField::Org => ContactField::Phone,
            ContactField::Notes => ContactField::Org,
            ContactField::Tags => ContactField::Notes,
            ContactField::Save => ContactField::Tags,
            ContactField::Cancel => ContactField::Save,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ContactField::Name => "Name",
            ContactField::Email => "Email",
            ContactField::Phone => "Phone",
            ContactField::Org => "Org",
            ContactField::Notes => "Notes",
            ContactField::Tags => "Tags",
            ContactField::Save => "Save",
            ContactField::Cancel => "Cancel",
        }
    }

    /// True if this is an action-bar button.
    pub fn is_action_button(self) -> bool {
        matches!(self, ContactField::Save | ContactField::Cancel)
    }
}

/// State for the contact add/edit form.
pub struct ContactEditState {
    /// ID of the contact being edited. `None` for new contacts.
    pub contact_id: Option<i64>,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub org: String,
    pub notes: String,
    /// Comma-separated tag list as edited by the user.
    pub tags: String,
    /// Currently focused field.
    pub focused: ContactField,
    /// Validation error message (e.g. empty email).
    pub error: Option<String>,
}

impl ContactEditState {
    /// Create a blank form for a new contact.
    pub fn new() -> Self {
        Self {
            contact_id: None,
            name: String::new(),
            email: String::new(),
            phone: String::new(),
            org: String::new(),
            notes: String::new(),
            tags: String::new(),
            focused: ContactField::Name,
            error: None,
        }
    }

    /// Pre-fill the form from an existing contact.
    pub fn from_contact(c: &Contact) -> Self {
        Self {
            contact_id: Some(c.id),
            name: c.name.clone().unwrap_or_default(),
            email: c.email.clone(),
            phone: c.phone.clone().unwrap_or_default(),
            org: c.org.clone().unwrap_or_default(),
            notes: c.notes.clone().unwrap_or_default(),
            tags: c.tags.join(", "),
            focused: ContactField::Name,
            error: None,
        }
    }

    /// Mutable reference to the currently focused field's text.
    /// Returns `None` for action-bar buttons (Save/Cancel have no text).
    pub fn focused_field_mut(&mut self) -> Option<&mut String> {
        match self.focused {
            ContactField::Name => Some(&mut self.name),
            ContactField::Email => Some(&mut self.email),
            ContactField::Phone => Some(&mut self.phone),
            ContactField::Org => Some(&mut self.org),
            ContactField::Notes => Some(&mut self.notes),
            ContactField::Tags => Some(&mut self.tags),
            ContactField::Save | ContactField::Cancel => None,
        }
    }

    /// Parse the tags string into a sorted, deduplicated list of tag strings.
    pub fn parsed_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .tags
            .split(',')
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Convert form state into a `Contact` for saving.
    /// Returns `Err` if validation fails.
    pub fn to_contact(&self) -> Result<Contact, String> {
        let email = self.email.trim().to_string();
        if email.is_empty() || !email.contains('@') {
            return Err("Email is required and must contain '@'.".to_string());
        }
        Ok(Contact {
            id: self.contact_id.unwrap_or(0),
            name: {
                let n = self.name.trim().to_string();
                if n.is_empty() {
                    None
                } else {
                    Some(n)
                }
            },
            email,
            phone: {
                let p = self.phone.trim().to_string();
                if p.is_empty() {
                    None
                } else {
                    Some(p)
                }
            },
            org: {
                let o = self.org.trim().to_string();
                if o.is_empty() {
                    None
                } else {
                    Some(o)
                }
            },
            notes: {
                let n = self.notes.trim().to_string();
                if n.is_empty() {
                    None
                } else {
                    Some(n)
                }
            },
            harvested: false,
            tags: self.parsed_tags(),
        })
    }
}
