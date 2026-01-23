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
    pub locked_recipient_id: Option<String>,  // 紐付けられた宛先マスターのID
    pub locked_company: Option<String>,       // ロック時の会社名（照合用）
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub file_path: String,
    pub file_name: String,
    pub enabled: bool,
    pub data: String,        // Base64 encoded content
    pub mime_type: String,
    #[serde(default)]
    pub linked_company: Option<String>,      // ファイル名から抽出した会社名
    #[serde(default)]
    pub linked_recipient_index: Option<usize>, // 紐付けられた宛先インデックス
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
            linked_company: None,
            linked_recipient_index: None,
        }
    }
}

#[derive(PartialEq)]
pub enum Tab {
    Main,
    History,
    Settings,
}

/// アプリの起動フェーズ
#[derive(PartialEq, Clone)]
pub enum StartupPhase {
    Splash,      // スプラッシュ画面表示中
    Loading,     // データロード中
    Ready,       // 準備完了
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
    // 起動フェーズ
    pub startup_phase: StartupPhase,
    pub loading_message: String,
    // 送信前確認用
    pub show_send_confirmation: bool,
    pub confirmation_company_input: String,
    pub confirmation_checked: bool,
    pub validation_errors: Vec<String>,
    pub pending_send_data: Option<PendingSendData>,
    // Basic認証
    pub is_authenticated: bool,
    pub auth_username: String,
    pub auth_password: String,
    pub auth_error: Option<String>,
    pub expected_username: String,  // 正しいユーザー名（設定で変更可能）
    pub expected_password: String,  // 正しいパスワード（設定で変更可能）
    // カラム幅（リサイズ可能）
    pub col_recipients_width: f32,
    pub col_templates_width: f32,
    pub col_signatures_width: f32,
    // 本文エディタの高さ（リサイズ可能）
    pub body_editor_height: f32,
}

#[derive(Clone, Debug, Default)]
pub struct PendingSendData {
    pub recipients: Vec<PendingRecipient>,
    pub subject: String,
}

#[derive(Clone, Debug, Default)]
pub struct PendingRecipient {
    pub email: String,
    pub company: String,
    pub name: String,
    pub body: String,
    pub attachments: Vec<String>,  // ファイル名のリスト
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
            startup_phase: StartupPhase::Splash,
            loading_message: "起動中...".to_string(),
            show_send_confirmation: false,
            confirmation_company_input: String::new(),
            confirmation_checked: false,
            validation_errors: Vec::new(),
            pending_send_data: None,
            // Basic認証（デフォルト: admin/password）
            is_authenticated: false,
            auth_username: String::new(),
            auth_password: String::new(),
            auth_error: None,
            expected_username: "nikko".to_string(),
            expected_password: "nikko".to_string(),
            // カラム幅のデフォルト値
            col_recipients_width: 220.0,
            col_templates_width: 220.0,
            col_signatures_width: 150.0,
            // 本文エディタの高さ
            body_editor_height: 100.0,
        }
    }
}
