use crate::models::{Template, RecipientData, Signature, LinkingData};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use thiserror::Error;

/// API エラーの種類を表す列挙型
#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("GAS URLが設定されていません")]
    UrlNotSet,

    #[error("ネットワークエラー: {0}")]
    NetworkError(String),

    #[error("タイムアウト: サーバーからの応答がありません")]
    Timeout,

    #[error("サーバーエラー (HTTP {status}): {message}")]
    ServerError { status: u16, message: String },

    #[error("レスポンス解析エラー: {0}")]
    ParseError(String),

    #[error("API エラー: {0}")]
    ApiResponseError(String),

    #[error("リトライ失敗 ({attempts}回試行): {last_error}")]
    RetryExhausted { attempts: u32, last_error: String },
}

/// リトライ設定
#[derive(Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 500,
            max_delay_ms: 5000,
        }
    }
}

#[derive(Clone)]
pub struct GasClient {
    client: Client,
    url: String,
    retry_config: RetryConfig,
    timeout: Duration,
}

#[derive(Deserialize)]
struct GetTemplatesResponse {
    templates: Option<Vec<Template>>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct GetRecipientsResponse {
    pub recipients: Vec<RecipientData>,
    // error: Option<String>, // Removed error field as per instruction's implied change
}

#[derive(Deserialize)]
pub struct GetSignaturesResponse {
    pub signatures: Vec<Signature>,
}

#[derive(Deserialize)]
pub struct GetLinkingsResponse {
    pub linkings: Vec<LinkingData>,
}

#[derive(Deserialize)]
pub struct GetSettingsResponse {
    pub settings: std::collections::HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct GetLogsResponse {
    pub logs: Vec<crate::models::HistoryItem>,
}

#[derive(Deserialize)]
struct PostResponse {
    success: bool,
    error: Option<String>,
}

#[derive(Serialize)]
struct BatchMailItem<'a> {
    to: &'a str,
    subject: &'a str,
    body: &'a str,
}

impl GasClient {
    pub fn new(url: String) -> Self {
        let timeout = Duration::from_secs(30);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            url,
            retry_config: RetryConfig::default(),
            timeout,
        }
    }

    /// リトライ設定をカスタマイズ
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// タイムアウトをカスタマイズ
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| Client::new());
        self
    }

    /// リトライ付きでリクエストを実行
    fn execute_with_retry<F, T>(&self, operation: F) -> Result<T, ApiError>
    where
        F: Fn() -> Result<T, ApiError>,
    {
        let mut last_error = ApiError::NetworkError("不明なエラー".to_string());
        let mut delay_ms = self.retry_config.initial_delay_ms;

        for attempt in 1..=self.retry_config.max_attempts {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = e.clone();

                    // リトライ不可能なエラーは即座に返す
                    match &e {
                        ApiError::UrlNotSet | ApiError::ParseError(_) | ApiError::ApiResponseError(_) => {
                            return Err(e);
                        }
                        _ => {}
                    }

                    if attempt < self.retry_config.max_attempts {
                        eprintln!("リトライ {}/{}: {:?}", attempt, self.retry_config.max_attempts, e);
                        std::thread::sleep(Duration::from_millis(delay_ms));
                        delay_ms = (delay_ms * 2).min(self.retry_config.max_delay_ms);
                    }
                }
            }
        }

        Err(ApiError::RetryExhausted {
            attempts: self.retry_config.max_attempts,
            last_error: last_error.to_string(),
        })
    }

    /// reqwest エラーを ApiError に変換
    fn convert_reqwest_error(&self, e: reqwest::Error) -> ApiError {
        if e.is_timeout() {
            ApiError::Timeout
        } else if e.is_connect() {
            ApiError::NetworkError("接続できません。ネットワーク接続を確認してください。".to_string())
        } else {
            ApiError::NetworkError(e.to_string())
        }
    }

    /// ベースURL を取得
    fn get_base_url(&self) -> Result<String, ApiError> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err(ApiError::UrlNotSet);
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }
        Ok(base_url)
    }

    pub fn get_templates(&self) -> Result<Vec<Template>, ApiError> {
        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let response = self.client.get(&base_url)
                .query(&[("action", "getTemplates")])
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: format!("テンプレート取得に失敗しました"),
                });
            }

            let parsed: GetTemplatesResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;

            if let Some(error) = parsed.error {
                return Err(ApiError::ApiResponseError(error));
            }

            Ok(parsed.templates.unwrap_or_default())
        })
    }

    pub fn get_recipients(&self) -> Result<Vec<RecipientData>, ApiError> {
        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let response = self.client.get(&base_url)
                .query(&[("action", "getRecipients")])
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: "宛先リスト取得に失敗しました".to_string(),
                });
            }

            let parsed: GetRecipientsResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;

            Ok(parsed.recipients)
        })
    }

    pub fn get_signatures(&self) -> Result<Vec<Signature>, ApiError> {
        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let url = format!("{}?action=getSignatures", base_url);
            let response = self.client.get(&url)
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: "署名取得に失敗しました".to_string(),
                });
            }

            let parsed: GetSignaturesResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;
            Ok(parsed.signatures)
        })
    }

    pub fn get_linkings(&self) -> Result<Vec<LinkingData>, ApiError> {
        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let url = format!("{}?action=getLinkings", base_url);
            let response = self.client.get(&url)
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: "紐付けデータ取得に失敗しました".to_string(),
                });
            }

            let parsed: GetLinkingsResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;
            Ok(parsed.linkings)
        })
    }

    pub fn get_settings(&self) -> Result<std::collections::HashMap<String, String>, ApiError> {
        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let url = format!("{}?action=getSettings", base_url);
            let response = self.client.get(&url)
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: "設定取得に失敗しました".to_string(),
                });
            }

            let parsed: GetSettingsResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;
            Ok(parsed.settings)
        })
    }

    pub fn save_settings(&self, settings: &std::collections::HashMap<String, String>) -> Result<(), ApiError> {
        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let payload = json!({
                "action": "saveSettings",
                "settings": settings,
            });

            let response = self.client.post(&base_url)
                .json(&payload)
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: "設定保存に失敗しました".to_string(),
                });
            }

            let parsed: PostResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;

            if !parsed.success {
                return Err(ApiError::ApiResponseError(
                    parsed.error.unwrap_or_else(|| "不明なエラー".to_string())
                ));
            }
            Ok(())
        })
    }

    pub fn send_mail(&self, to: &str, subject: &str, body: &str) -> Result<(), ApiError> {
        let to = to.to_string();
        let subject = subject.to_string();
        let body = body.to_string();

        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let payload = json!({
                "action": "sendMail",
                "to": &to,
                "subject": &subject,
                "body": &body,
            });

            let response = self.client.post(&base_url)
                .json(&payload)
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: format!("メール送信に失敗しました (宛先: {})", &to),
                });
            }

            let parsed: PostResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;

            if !parsed.success {
                return Err(ApiError::ApiResponseError(
                    parsed.error.unwrap_or_else(|| "メール送信に失敗しました".to_string())
                ));
            }
            Ok(())
        })
    }

    pub fn send_batch_mail(&self, items: Vec<(&str, &str, &str)>, attachments: &[crate::models::Attachment]) -> Result<(), ApiError> {
        // Clone data for retry
        let items_owned: Vec<(String, String, String)> = items.into_iter()
            .map(|(to, sub, body)| (to.to_string(), sub.to_string(), body.to_string()))
            .collect();
        let attachments_owned: Vec<crate::models::Attachment> = attachments.to_vec();

        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let emails: Vec<serde_json::Value> = items_owned.iter()
                .map(|(to, sub, body)| {
                    let mut email_obj = serde_json::json!({
                        "to": to,
                        "subject": sub,
                        "body": body,
                    });

                    if !attachments_owned.is_empty() {
                        let attachments_json: Vec<serde_json::Value> = attachments_owned.iter()
                            .filter(|att| att.enabled)
                            .map(|att| serde_json::json!({
                                "fileName": att.file_name,
                                "mimeType": att.mime_type,
                                "data": att.data,
                            }))
                            .collect();
                        email_obj["attachments"] = serde_json::json!(attachments_json);
                    }

                    email_obj
                })
                .collect();

            let payload = json!({
                "action": "sendBatchMail",
                "emails": emails,
            });

            let response = self.client.post(&base_url)
                .json(&payload)
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: "一括メール送信に失敗しました".to_string(),
                });
            }

            let parsed: PostResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;

            if !parsed.success {
                return Err(ApiError::ApiResponseError(
                    parsed.error.unwrap_or_else(|| "一括メール送信に失敗しました".to_string())
                ));
            }
            Ok(())
        })
    }

    pub fn get_history(&self) -> Result<Vec<crate::models::HistoryItem>, ApiError> {
        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let url = format!("{}?action=getLogs", base_url);
            let response = self.client.get(&url)
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: "送信履歴取得に失敗しました".to_string(),
                });
            }

            let parsed: GetLogsResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;
            Ok(parsed.logs)
        })
    }

    pub fn save_template(&self, template: &crate::models::Template) -> Result<(), ApiError> {
        let template_owned = template.clone();

        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let payload = json!({
                "action": "saveTemplate",
                "template": &template_owned,
            });

            let response = self.client.post(&base_url)
                .json(&payload)
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: format!("テンプレート「{}」の保存に失敗しました", &template_owned.name),
                });
            }

            let text = response.text()
                .map_err(|e| ApiError::ParseError(format!("レスポンス読み取りエラー: {}", e)))?;

            let parsed: PostResponse = serde_json::from_str(&text)
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {} | レスポンス: {}", e, text)))?;

            if !parsed.success {
                return Err(ApiError::ApiResponseError(
                    parsed.error.unwrap_or_else(|| "テンプレート保存に失敗しました".to_string())
                ));
            }
            Ok(())
        })
    }

    pub fn delete_template(&self, name: &str) -> Result<(), ApiError> {
        let name_owned = name.to_string();

        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let payload = json!({
                "action": "deleteTemplate",
                "name": &name_owned,
            });

            let response = self.client.post(&base_url)
                .json(&payload)
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: format!("テンプレート「{}」の削除に失敗しました", &name_owned),
                });
            }

            let parsed: PostResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;

            if !parsed.success {
                return Err(ApiError::ApiResponseError(
                    parsed.error.unwrap_or_else(|| "テンプレート削除に失敗しました".to_string())
                ));
            }
            Ok(())
        })
    }

    pub fn save_recipient(&self, recipient: &crate::models::RecipientData) -> Result<(), ApiError> {
        let recipient_owned = recipient.clone();

        self.execute_with_retry(|| {
            let base_url = self.get_base_url()?;

            let payload = json!({
                "action": "saveRecipient",
                "recipient": &recipient_owned,
            });

            let response = self.client.post(&base_url)
                .json(&payload)
                .send()
                .map_err(|e| self.convert_reqwest_error(e))?;

            let status = response.status();
            if !status.is_success() {
                return Err(ApiError::ServerError {
                    status: status.as_u16(),
                    message: format!("宛先「{}」の保存に失敗しました", &recipient_owned.name),
                });
            }

            let parsed: PostResponse = response.json()
                .map_err(|e| ApiError::ParseError(format!("JSON解析エラー: {}", e)))?;

            if !parsed.success {
                return Err(ApiError::ApiResponseError(
                    parsed.error.unwrap_or_else(|| "宛先保存に失敗しました".to_string())
                ));
            }
            Ok(())
        })
    }
}
