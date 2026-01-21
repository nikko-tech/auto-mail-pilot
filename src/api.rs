use crate::models::Template;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone)]
pub struct GasClient {
    client: Client,
    url: String,
}

#[derive(Deserialize)]
struct GetTemplatesResponse {
    templates: Option<Vec<Template>>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct PostResponse {
    success: bool,
    error: Option<String>,
}

impl GasClient {
    pub fn new(url: String) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }

    pub fn get_templates(&self) -> Result<Vec<Template>, String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let response = self.client.get(&base_url)
            .query(&[("action", "getTemplates")])
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: GetTemplatesResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;

        if let Some(error) = parsed.error {
            return Err(error);
        }

        Ok(parsed.templates.unwrap_or_default())
    }

    pub fn send_mail(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
             return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let payload = json!({
            "action": "sendMail",
            "to": to,
            "subject": subject,
            "body": body,
        });

        let response = self.client.post(&base_url)
            .json(&payload)
            .send()
             .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: PostResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;

        if !parsed.success {
            return Err(parsed.error.unwrap_or("Unknown error".to_string()));
        }
        Ok(())
    }
}
