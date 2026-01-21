use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub subject: String,
    pub body: String,
}

#[derive(Clone, Debug, Default)]
pub struct MailDraft {
    pub recipients: Vec<RecipientInfo>,
    pub subject: String,
    pub attachments: Vec<Attachment>,
    pub signature: String,
}

#[derive(Clone, Debug, Default)]
pub struct RecipientInfo {
    pub email: String,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub file_path: String,
    pub file_name: String,
    pub enabled: bool,
    #[serde(skip)]
    pub data: Option<String>, // Base64 encoded content
    #[serde(skip)]
    pub mime_type: Option<String>,
}

#[derive(PartialEq)]
pub enum Tab {
    Main,
    Settings,
}

pub struct AppState {
    pub templates: Vec<Template>,
    pub selected_template_index: Option<usize>,
    pub mail_draft: MailDraft,
    pub tab: Tab,
    pub gas_url: String,
    pub status_message: String,
    pub is_loading: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            templates: Vec::new(),
            selected_template_index: None,
            mail_draft: MailDraft::default(),
            tab: Tab::Main,
            gas_url: String::new(),
            status_message: String::new(),
            is_loading: false,
        }
    }
}
