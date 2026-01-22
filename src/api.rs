use crate::models::{Template, RecipientData, Signature, LinkingData};
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone)]
pub struct GasClient {
    client: Client,
    post_client: Client, // Separate client for POST with no redirect
    url: String,
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
        // POST client with no automatic redirects (GAS redirects lose POST body)
        let post_client = Client::builder()
            .redirect(Policy::none())
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Client::new(),
            post_client,
            url,
        }
    }

    // Helper method for POST requests that handles GAS redirects
    fn post_with_redirect(&self, url: &str, payload: &serde_json::Value) -> Result<String, String> {
        // First request - will get 302 redirect
        let response = self.post_client.post(url)
            .json(payload)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        // Check if it's a redirect
        if response.status().is_redirection() {
            if let Some(location) = response.headers().get("location") {
                let redirect_url = location.to_str()
                    .map_err(|_| "Invalid redirect URL".to_string())?;

                // Follow redirect with GET (GAS returns result via GET after POST redirect)
                let final_response = self.client.get(redirect_url)
                    .send()
                    .map_err(|e| format!("Redirect request failed: {}", e))?;

                return final_response.text()
                    .map_err(|e| format!("Failed to read response: {}", e));
            }
        }

        response.text()
            .map_err(|e| format!("Failed to read response: {}", e))
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

    pub fn get_recipients(&self) -> Result<Vec<RecipientData>, String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let response = self.client.get(&base_url)
            .query(&[("action", "getRecipients")])
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: GetRecipientsResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;

        // The original code had error handling here, but the instruction implies removing the error field from GetRecipientsResponse.
        // If the API can still return an error, this logic needs to be re-evaluated.
        // For now, assuming the new GetRecipientsResponse structure means no direct error field.
        Ok(parsed.recipients)
    }

    pub fn get_signatures(&self) -> Result<Vec<Signature>, String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let url = format!("{}?action=getSignatures", base_url);
        let response = self.client.get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: GetSignaturesResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(parsed.signatures)
    }

    pub fn get_linkings(&self) -> Result<Vec<LinkingData>, String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let url = format!("{}?action=getLinkings", base_url);
        let response = self.client.get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: GetLinkingsResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(parsed.linkings)
    }

    pub fn get_settings(&self) -> Result<std::collections::HashMap<String, String>, String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let url = format!("{}?action=getSettings", base_url);
        let response = self.client.get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: GetSettingsResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(parsed.settings)
    }

    pub fn save_settings(&self, settings: &std::collections::HashMap<String, String>) -> Result<(), String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let payload = json!({
            "action": "saveSettings",
            "settings": settings,
        });

        let text = self.post_with_redirect(&base_url, &payload)?;
        let parsed: PostResponse = serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))?;

        if !parsed.success {
            return Err(parsed.error.unwrap_or("Unknown error".to_string()));
        }
        Ok(())
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

        let text = self.post_with_redirect(&base_url, &payload)?;
        let parsed: PostResponse = serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))?;

        if !parsed.success {
            return Err(parsed.error.unwrap_or("Unknown error".to_string()));
        }
        Ok(())
    }

    pub fn send_batch_mail(&self, items: Vec<(&str, &str, &str)>, attachments: &[crate::models::Attachment]) -> Result<(), String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let emails: Vec<serde_json::Value> = items.into_iter()
            .map(|(to, sub, body)| {
                let mut email_obj = serde_json::json!({
                    "to": to,
                    "subject": sub,
                    "body": body,
                });
                
                // Add attachments if present
                if !attachments.is_empty() {
                    let attachments_json: Vec<serde_json::Value> = attachments.iter()
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

        let text = self.post_with_redirect(&base_url, &payload)?;
        let parsed: PostResponse = serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))?;

        if !parsed.success {
            return Err(parsed.error.unwrap_or("Unknown error".to_string()));
        }
        Ok(())
    }

    pub fn get_history(&self) -> Result<Vec<crate::models::HistoryItem>, String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let url = format!("{}?action=getLogs", base_url);
        let response = self.client.get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let parsed: GetLogsResponse = response.json().map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(parsed.logs)
    }

    pub fn save_template(&self, template: &crate::models::Template) -> Result<(), String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let payload = json!({
            "action": "saveTemplate",
            "template": template,
        });

        let text = self.post_with_redirect(&base_url, &payload)?;

        // Debug
        eprintln!("GAS save_template Response: {}", text);

        let parsed: PostResponse = serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {} | Raw: {}", e, text))?;

        if !parsed.success {
            return Err(parsed.error.unwrap_or("Unknown error".to_string()));
        }
        Ok(())
    }

    pub fn delete_template(&self, name: &str) -> Result<(), String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let payload = json!({
            "action": "deleteTemplate",
            "name": name,
        });

        let text = self.post_with_redirect(&base_url, &payload)?;
        let parsed: PostResponse = serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))?;

        if !parsed.success {
            return Err(parsed.error.unwrap_or("Unknown error".to_string()));
        }
        Ok(())
    }

    pub fn save_recipient(&self, recipient: &crate::models::RecipientData) -> Result<(), String> {
        let mut base_url = self.url.trim().to_string();
        if base_url.is_empty() {
            return Err("GAS URL is not set".to_string());
        }

        if let Some(pos) = base_url.find('?') {
            base_url.truncate(pos);
        }

        let payload = json!({
            "action": "saveRecipient",
            "recipient": recipient,
        });

        let text = self.post_with_redirect(&base_url, &payload)?;
        let parsed: PostResponse = serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))?;

        if !parsed.success {
            return Err(parsed.error.unwrap_or("Unknown error".to_string()));
        }
        Ok(())
    }
}
