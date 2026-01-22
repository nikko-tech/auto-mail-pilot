use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub subject: String,
    pub body: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct RecipientData {
    pub id: String,
    pub company: String,
    pub name: String,
    pub email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signature {
    pub name: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinkingData {
    pub recipient_id: String,
    pub template_id: String,
}

#[derive(Clone, Debug)]
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
    pub data: String,        // Base64 encoded content
    pub mime_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryItem {
    pub date: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub status: String,
}

impl Default for Attachment {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            file_name: String::new(),
            enabled: true,
            data: String::new(),
            mime_type: String::new(),
        }
    }
}

#[derive(PartialEq)]
pub enum Tab {
    Main,
    History,
    Settings,
}

pub struct AppState {
    pub templates: Vec<Template>,
    pub selected_template_index: Option<usize>,
    pub template_search: String,
    pub recipients_master: Vec<RecipientData>,
    pub selected_recipient_index: Option<usize>,
    pub recipient_search: String,
    pub active_recipient_index: usize, // 0, 1, or 2
    pub signatures: Vec<Signature>,
    pub selected_signature_index: Option<usize>,
    pub linkings_master: Vec<LinkingData>,
    pub mail_draft: MailDraft,
    pub history: Vec<HistoryItem>,
    pub tab: Tab,
    pub gas_url: String,
    pub status_message: String,
    pub is_loading: bool,
}

impl Default for MailDraft {
    fn default() -> Self {
        Self {
            recipients: vec![RecipientInfo::default(); 3],
            subject: String::new(),
            attachments: Vec::new(),
            signature: String::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            templates: Vec::new(),
            selected_template_index: None,
            template_search: String::new(),
            recipients_master: Vec::new(),
            selected_recipient_index: None,
            recipient_search: String::new(),
            active_recipient_index: 0,
            signatures: Vec::new(),
            selected_signature_index: None,
            linkings_master: Vec::new(),
            mail_draft: MailDraft::default(),
            history: Vec::new(),
            tab: Tab::Main,
            gas_url: "https://script.google.com/macros/s/AKfycbwUAgPH2nh3Mn7JYbsRUWadfXHlCPkPKMm1OOqzbFg1mjjDvVS76ZKuM8sNB1NwP2wE/exec".to_string(),
            status_message: "準備完了".to_string(),
            is_loading: false,
        }
    }
}
